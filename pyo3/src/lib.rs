//! # Kamaros Python Bindings
//!
//! PyO3 bindings for kamaros-corelib, exposing JCF operations to Python.

use pyo3::prelude::*;
use pyo3::types::{PyDict, PyBytes};
use pythonize::{pythonize, depythonize};
use serde::{Serialize, Deserialize};
use kamaros_corelib::domain::manifest::{Manifest, ProjectMetadata, FileEntry, FileType};
use kamaros_corelib::domain::version::{Version, FileState};
use std::collections::HashMap;
use sha2::{Sha256, Digest};

/// Get library version
#[pyfunction]
fn version() -> String {
    env!("CARGO_PKG_VERSION").to_string()
}

/// Say hello from Kamaros
#[pyfunction]
fn greet(name: &str) -> String {
    format!("Hello from Kamaros Python, {}!", name)
}

/// Create an empty manifest - returns a Python dict
#[pyfunction]
fn create_empty_manifest(py: Python<'_>, project_name: &str) -> PyResult<PyObject> {
    let manifest = Manifest {
        format_version: "1.0.0".to_string(),
        metadata: ProjectMetadata {
            name: project_name.to_string(),
            description: None,
            created: chrono_now(),
            last_modified: chrono_now(),
            author: None,
        },
        file_map: HashMap::new(),
        version_history: vec![],
        refs: HashMap::from([("head".to_string(), "".to_string())]),
        rename_log: vec![],
    };
    
    // Convert Rust struct to Python object using pythonize
    let py_obj = pythonize(py, &manifest)?;
    Ok(py_obj.into())
}

/// Simple info struct for serialization
#[derive(Serialize)]
struct ManifestInfo {
    name: String,
    version_count: usize,
    file_count: usize,
}

/// Get manifest info from dict
#[pyfunction]
fn get_manifest_info(manifest: &Bound<'_, PyAny>) -> PyResult<PyObject> {
    let py = manifest.py();
    
    let metadata = manifest.get_item("metadata")?;
    let name: String = metadata.get_item("name")?.extract()?;
    
    let version_history = manifest.get_item("version_history")?;
    let version_count: usize = version_history.len()?;
    
    let file_map = manifest.get_item("file_map")?;
    let file_count: usize = file_map.len()?;
    
    let info = ManifestInfo {
        name,
        version_count,
        file_count,
    };
    
    let py_obj = pythonize(py, &info)?;
    Ok(py_obj.into())
}

/// Result of save_checkpoint operation
#[derive(Serialize)]
struct SaveCheckpointResult {
    manifest: serde_json::Value,
    version_id: String,
    files_added: usize,
    files_modified: usize,
    files_deleted: usize,
    blobs: Vec<(String, Vec<u8>)>, // Path -> Content
}

/// Save checkpoint - create a new version
/// 
/// Args:
///     manifest: Current manifest dict
///     working_dir: Dict mapping file paths to bytes content
///     message: Commit message
///     author: Author name
/// 
/// Returns:
///     Dict with updated manifest, version_id, change counts, and blobs to write
#[pyfunction]
fn save_checkpoint(
    py: Python<'_>,
    manifest: &Bound<'_, PyDict>,
    working_dir: &Bound<'_, PyDict>,
    message: &str,
    author: &str,
) -> PyResult<PyObject> {
    // Parse manifest from Python dict
    let mut rust_manifest: Manifest = depythonize(manifest)
        .map_err(|e| PyErr::new::<pyo3::exceptions::PyValueError, _>(format!("Parse manifest error: {}", e)))?;
    
    // Generate new version ID
    let version_id = uuid::Uuid::new_v4().to_string();
    let parent_id = rust_manifest.refs.get("head").cloned().filter(|s| !s.is_empty());
    
    // Get current files from working_dir
    let mut file_states: HashMap<String, FileState> = HashMap::new();
    let mut blobs: Vec<(String, Vec<u8>)> = Vec::new();
    let mut files_added = 0usize;
    let mut files_modified = 0usize;
    let mut files_deleted = 0usize;
    
    // Check current files
    // Iterate directly over the dictionary (key, value) pairs
    for (key, content_obj) in working_dir {
        let file_path: String = key.extract()?;
        
        let content: Vec<u8> = content_obj.extract::<Vec<u8>>()
            .or_else(|_| {
                // Try extracting as bytes object
                content_obj.downcast::<PyBytes>()
                    .map(|b| b.as_bytes().to_vec())
                    .map_err(|e| PyErr::new::<pyo3::exceptions::PyTypeError, _>(format!("Cannot extract bytes: {}", e)))
            })?;
        
        let hash = compute_sha256(&content);
        
        let inode_id = if let Some(entry) = rust_manifest.file_map.get(&file_path) {
            // Check if modified
            if entry.current_hash.as_ref() != Some(&hash) {
                files_modified += 1;
                // Add new blob
                blobs.push((format!(".store/blobs/{}", hash), content.clone()));
            }
            entry.inode_id.clone()
        } else {
            // New file
            files_added += 1;
            // Add new blob
            blobs.push((format!(".store/blobs/{}", hash), content.clone()));
            uuid::Uuid::new_v4().to_string()
        };
        
        file_states.insert(file_path.clone(), FileState {
            inode_id: inode_id.clone(),
            hash: Some(hash.clone()),
            content_ref: Some(format!("blobs/{}", hash)),
            deleted: None,
        });
        
        // Update file_map
        let now = chrono_now();
        rust_manifest.file_map.entry(file_path.clone())
            .and_modify(|e| {
                e.current_hash = Some(hash.clone());
                e.modified = now.clone();
            })
            .or_insert_with(|| FileEntry {
                inode_id,
                file_type: infer_file_type(&file_path),
                created: now.clone(),
                modified: now,
                current_hash: Some(hash),
            });
    }
    
    // Check for deleted files
    // We need to collect keys from the iterator or reconstruction to check for deletions
    // Since we just iterated everything, let's collect the paths we found
    let found_paths: Vec<String> = file_states.keys().cloned().collect();
    let found_set: std::collections::HashSet<_> = found_paths.iter().collect();
    
    let deleted_paths: Vec<_> = rust_manifest.file_map.keys()
        .filter(|p| !found_set.contains(p))
        .cloned()
        .collect();
    
    for path in deleted_paths {
        files_deleted += 1;
        if let Some(entry) = rust_manifest.file_map.get(&path) {
            file_states.insert(path.clone(), FileState {
                inode_id: entry.inode_id.clone(),
                hash: None,
                content_ref: None,
                deleted: Some(true),
            });
        }
        rust_manifest.file_map.remove(&path);
    }
    
    // Create version
    let version = Version {
        id: version_id.clone(),
        parent_id,
        timestamp: chrono_now(),
        message: message.to_string(),
        author: author.to_string(),
        file_states,
    };
    
    // Update manifest
    rust_manifest.version_history.push(version);
    rust_manifest.refs.insert("head".to_string(), version_id.clone());
    rust_manifest.metadata.last_modified = chrono_now();
    
    // Convert manifest to JSON value for serialization
    let manifest_json = serde_json::to_value(&rust_manifest)
        .map_err(|e| PyErr::new::<pyo3::exceptions::PyValueError, _>(format!("Serialize error: {}", e)))?;
    
    let result = SaveCheckpointResult {
        manifest: manifest_json,
        version_id,
        files_added,
        files_modified,
        files_deleted,
        blobs,
    };
    
    let py_obj = pythonize(py, &result)?;
    Ok(py_obj.into())
}

// Helper: get current timestamp
fn chrono_now() -> String {
    chrono::Utc::now().to_rfc3339()
}

// Helper: compute SHA-256 hash
fn compute_sha256(data: &[u8]) -> String {
    let mut hasher = Sha256::new();
    hasher.update(data);
    format!("{:x}", hasher.finalize())
}

// Helper: infer file type from extension
fn infer_file_type(path: &str) -> FileType {
    let text_extensions = [".txt", ".md", ".json", ".js", ".ts", ".css", ".html", ".xml", ".yaml", ".yml", ".rs", ".py"];
    if text_extensions.iter().any(|ext| path.to_lowercase().ends_with(ext)) {
        FileType::Text
    } else {
        FileType::Binary
    }
}

/// Result of restore_version operation
#[derive(Serialize)]
struct RestoreVersionResult {
    manifest: serde_json::Value,
    restored_version_id: String,
    files_to_restore: HashMap<String, String>, // Path -> BlobRef/Path
    files_to_delete: Vec<String>,
    files_restored_count: usize,
    files_deleted_count: usize,
}

/// Restore version - checkout specific version
/// 
/// Args:
///     manifest: Current manifest dict
///     current_files: List of file paths currently in working directory
///     version_id: Target version ID
/// 
/// Returns:
///     Restoration plan (files to restore/delete) and updated manifest
#[pyfunction]
fn restore_version(
    py: Python<'_>,
    manifest: &Bound<'_, PyDict>,
    current_files: Vec<String>,
    version_id: &str,
) -> PyResult<PyObject> {
    let mut rust_manifest: Manifest = depythonize(manifest)
        .map_err(|e| PyErr::new::<pyo3::exceptions::PyValueError, _>(format!("Parse manifest error: {}", e)))?;
    
    // Find target version
    let target_version = rust_manifest.version_history.iter()
        .find(|v| v.id == version_id)
        .ok_or_else(|| PyErr::new::<pyo3::exceptions::PyValueError, _>(format!("Version {} not found", version_id)))?;
        
    let final_files = &target_version.file_states;
    let mut files_to_restore = HashMap::new();
    let mut files_to_delete = Vec::new();
    let mut files_restored_count = 0usize;
    let mut files_deleted_count = 0usize;
    
    // 1. Files to restore
    for (path, state) in final_files {
        if state.deleted.unwrap_or(false) {
            // Should be deleted
            if current_files.contains(path) {
                files_to_delete.push(path.clone());
                files_deleted_count += 1;
            }
            continue;
        }
        
        let blob_path = if let Some(ref r) = state.content_ref {
             format!(".store/{}", r)
        } else if let Some(ref h) = state.hash {
             format!(".store/blobs/{}", h)
        } else {
             continue; 
        };
        
        // Always restore matching files for now (overwrite)
        // Optimization: check hash of current file? We don't have current file content here.
        // Python side can optimize if needed, but for now we send instruction to restore.
        files_to_restore.insert(path.clone(), blob_path);
        files_restored_count += 1;
    }
    
    // 2. Files to delete (tracked files present in workspace but not in target version)
    for file in &current_files {
        if !final_files.contains_key(file) {
            // Only delete if it's a tracked file (in manifest.file_map)
            // Or maybe simpler: if it's not in target and we are checkout-ing, delete it?
            if rust_manifest.file_map.contains_key(file) {
                files_to_delete.push(file.clone());
                files_deleted_count += 1;
            }
        }
    }
    
    // Update refs
    rust_manifest.refs.insert("head".to_string(), version_id.to_string());
    
    let manifest_json = serde_json::to_value(&rust_manifest)
        .map_err(|e| PyErr::new::<pyo3::exceptions::PyValueError, _>(format!("Serialize error: {}", e)))?;
        
    let result = RestoreVersionResult {
        manifest: manifest_json,
        restored_version_id: version_id.to_string(),
        files_to_restore,
        files_to_delete,
        files_restored_count,
        files_deleted_count,
    };
    
    let py_obj = pythonize(py, &result)?;
    Ok(py_obj.into())
}

/// Kamaros Python module
#[pymodule]
fn _native(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_function(wrap_pyfunction!(version, m)?)?;
    m.add_function(wrap_pyfunction!(greet, m)?)?;
    m.add_function(wrap_pyfunction!(create_empty_manifest, m)?)?;
    m.add_function(wrap_pyfunction!(get_manifest_info, m)?)?;
    m.add_function(wrap_pyfunction!(save_checkpoint, m)?)?;
    m.add_function(wrap_pyfunction!(restore_version, m)?)?;
    Ok(())
}
