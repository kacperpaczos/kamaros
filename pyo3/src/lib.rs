//! # Kamaros Python Bindings
//!
//! PyO3 bindings for kamaros-corelib, exposing JCF operations to Python.

use pyo3::prelude::*;
use pythonize::pythonize;
use serde::Serialize;
use kamaros_corelib::domain::manifest::{Manifest, ProjectMetadata};
use std::collections::HashMap;

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

// Helper: get current timestamp
fn chrono_now() -> String {
    chrono::Utc::now().to_rfc3339()
}

/// Kamaros Python module
#[pymodule]
fn kamaros(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_function(wrap_pyfunction!(version, m)?)?;
    m.add_function(wrap_pyfunction!(greet, m)?)?;
    m.add_function(wrap_pyfunction!(create_empty_manifest, m)?)?;
    m.add_function(wrap_pyfunction!(get_manifest_info, m)?)?;
    Ok(())
}
