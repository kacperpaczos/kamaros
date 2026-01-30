//! # Kamaros Python Bindings
//!
//! PyO3 bindings for kamaros-corelib, exposing JCF operations to Python.

use pyo3::prelude::*;
use pyo3::types::{PyDict, PyBytes};
use pythonize::{pythonize, depythonize};
use serde::Serialize;
use kamaros_corelib::domain::manifest::{Manifest, ProjectMetadata};
use kamaros_corelib::ports::{StoragePort, PortResult, PortError, EncryptionPort};
use kamaros_corelib::application::save_checkpoint::{SaveCheckpointUseCase, SaveCheckpointInput};
use kamaros_corelib::application::restore_version::{RestoreVersionUseCase, RestoreVersionInput};
use kamaros_corelib::application::garbage_collect::GcUseCase;
use kamaros_corelib::infrastructure::{SimpleDiff, Sha256Hasher, AesGcmEncryptor};
use std::collections::HashMap;
use async_trait::async_trait;


/// Bridge between Python StorageAdapter and Rust StoragePort
pub struct PyStorageWrapper {
    adapter: PyObject,
}

impl PyStorageWrapper {
    pub fn new(adapter: PyObject) -> Self {
        Self { adapter }
    }
}

#[async_trait]
impl StoragePort for PyStorageWrapper {
    async fn read(&self, path: &str) -> PortResult<Vec<u8>> {
        Python::with_gil(|py| {
            let res = self.adapter.call_method1(py, "read", (path,))
                .map_err(|e| PortError::Io(std::io::Error::new(std::io::ErrorKind::Other, format!("Python error: {}", e))))?;
            let data: Vec<u8> = res.extract(py)
                .map_err(|e| PortError::Io(std::io::Error::new(std::io::ErrorKind::Other, format!("Extract error: {}", e))))?;
            Ok(data)
        })
    }

    async fn write(&self, path: &str, data: &[u8]) -> PortResult<()> {
        Python::with_gil(|py| {
            let bytes = PyBytes::new_bound(py, data);
            self.adapter.call_method1(py, "write", (path, bytes))
                .map_err(|e| PortError::Io(std::io::Error::new(std::io::ErrorKind::Other, format!("Python error: {}", e))))?;
            Ok(())
        })
    }

    async fn delete(&self, path: &str) -> PortResult<()> {
        Python::with_gil(|py| {
            self.adapter.call_method1(py, "delete", (path,))
                .map_err(|e| PortError::Io(std::io::Error::new(std::io::ErrorKind::Other, format!("Python error: {}", e))))?;
            Ok(())
        })
    }

    async fn exists(&self, path: &str) -> PortResult<bool> {
        Python::with_gil(|py| {
            let res = self.adapter.call_method1(py, "exists", (path,))
                .map_err(|e| PortError::Io(std::io::Error::new(std::io::ErrorKind::Other, format!("Python error: {}", e))))?;
            let exists: bool = res.extract(py)
                .map_err(|e| PortError::Io(std::io::Error::new(std::io::ErrorKind::Other, format!("Extract error: {}", e))))?;
            Ok(exists)
        })
    }

    async fn list(&self, dir: &str) -> PortResult<Vec<String>> {
        Python::with_gil(|py| {
            let res = self.adapter.call_method1(py, "list", (dir,))
                .map_err(|e| PortError::Io(std::io::Error::new(std::io::ErrorKind::Other, format!("Python error: {}", e))))?;
            let list: Vec<String> = res.extract(py)
                .map_err(|e| PortError::Io(std::io::Error::new(std::io::ErrorKind::Other, format!("Extract error: {}", e))))?;
            Ok(list)
        })
    }

    async fn size(&self, path: &str) -> PortResult<usize> {
        Python::with_gil(|py| {
            // If adapter doesn't have size(), we fallback to read().len()
            if let Ok(res) = self.adapter.call_method1(py, "size", (path,)) {
                let size: usize = res.extract(py)
                    .map_err(|e| PortError::Io(std::io::Error::new(std::io::ErrorKind::Other, format!("Extract error: {}", e))))?;
                Ok(size)
            } else {
                let res = self.adapter.call_method1(py, "read", (path,))
                    .map_err(|e| PortError::Io(std::io::Error::new(std::io::ErrorKind::Other, format!("Python error: {}", e))))?;
                let data: Vec<u8> = res.extract(py)
                    .map_err(|e| PortError::Io(std::io::Error::new(std::io::ErrorKind::Other, format!("Extract error: {}", e))))?;
                Ok(data.len())
            }
        })
    }
}

// In PyO3, we must mark our Wrapper as Send/Sync since it will be used in async traits
unsafe impl Send for PyStorageWrapper {}
unsafe impl Sync for PyStorageWrapper {}


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

// Helper: get current time in RFC3339
fn chrono_now() -> String {
    chrono::Utc::now().to_rfc3339()
}

/// Save checkpoint - create a new version
#[pyfunction]
#[pyo3(signature = (manifest, adapter, working_dir, message, author, encryption_key=None))]
fn save_checkpoint(
    py: Python<'_>,
    manifest: &Bound<'_, PyDict>,
    adapter: PyObject,
    working_dir: &Bound<'_, PyDict>,
    message: &str,
    author: &str,
    encryption_key: Option<Vec<u8>>,
) -> PyResult<PyObject> {
    let mut rust_manifest: Manifest = depythonize(manifest)
        .map_err(|e| PyErr::new::<pyo3::exceptions::PyValueError, _>(format!("Parse manifest error: {}", e)))?;
    
    let storage = PyStorageWrapper::new(adapter.clone_ref(py));
    
    // Sync working_dir to storage first (under content/)
    // This allows the use case to identify changes correctly.
    for (key, val) in working_dir {
        let path: String = key.extract()?;
        let data: Vec<u8> = val.extract()?;
        futures::executor::block_on(storage.write(&format!("content/{}", path), &data))
            .map_err(|e| PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(format!("Storage write error: {}", e)))?;
    }

    let input = SaveCheckpointInput {
        message: message.to_string(),
        author: author.to_string(),
        encryption_key,
    };

    let use_case = SaveCheckpointUseCase::new(
        PyStorageWrapper::new(adapter.clone_ref(py)),
        SimpleDiff::new(),
        Sha256Hasher::new(),
        AesGcmEncryptor::new(),
    );

    // Run async use case in blocking fashion
    let result = futures::executor::block_on(use_case.execute(&mut rust_manifest, input))
        .map_err(|e| PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(format!("SaveCheckpoint error: {}", e)))?;

    let py_result = pythonize(py, &result)?;
    let py_manifest = pythonize(py, &rust_manifest)?;
    
    let dict = PyDict::new_bound(py);
    dict.set_item("manifest", py_manifest)?;
    dict.set_item("version_id", py_result.get_item("version_id")?)?;
    dict.set_item("files_changed", py_result.get_item("files_changed")?)?;
    dict.set_item("files_added", py_result.get_item("files_added")?)?;
    dict.set_item("files_deleted", py_result.get_item("files_deleted")?)?;
    
    Ok(dict.into())
}





/// Restore version - checkout specific version
#[pyfunction]
#[pyo3(signature = (manifest, adapter, version_id, encryption_key=None))]
fn restore_version(
    py: Python<'_>,
    manifest: &Bound<'_, PyDict>,
    adapter: PyObject,
    version_id: &str,
    encryption_key: Option<Vec<u8>>,
) -> PyResult<PyObject> {
    let mut rust_manifest: Manifest = depythonize(manifest)
        .map_err(|e| PyErr::new::<pyo3::exceptions::PyValueError, _>(format!("Parse manifest error: {}", e)))?;
    
    let input = RestoreVersionInput {
        target_version_id: version_id.to_string(),
        force: true,
        encryption_key,
    };

    let storage = PyStorageWrapper::new(adapter.clone_ref(py));
    let use_case = RestoreVersionUseCase::new(
        storage,
        SimpleDiff::new(),
        AesGcmEncryptor::new(),
    );

    let result = futures::executor::block_on(use_case.execute(&mut rust_manifest, input))
        .map_err(|e| PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(format!("RestoreVersion error: {}", e)))?;

    let py_result = pythonize(py, &result)?;
    let py_manifest = pythonize(py, &rust_manifest)?;
    
    let dict = PyDict::new_bound(py);
    dict.set_item("manifest", py_manifest)?;
    dict.set_item("restored_version_id", py_result.get_item("restored_version_id")?)?;
    
    Ok(dict.into())
}

/// Run garbage collection
#[pyfunction]
fn gc(
    py: Python<'_>,
    manifest: &Bound<'_, PyDict>,
    adapter: PyObject,
) -> PyResult<PyObject> {
    let mut rust_manifest: Manifest = depythonize(manifest)
        .map_err(|e| PyErr::new::<pyo3::exceptions::PyValueError, _>(format!("Parse manifest error: {}", e)))?;
    
    let storage = PyStorageWrapper::new(adapter.clone_ref(py));
    let use_case = GcUseCase::new(storage);

    let result = futures::executor::block_on(use_case.run(&mut rust_manifest))
        .map_err(|e| PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(format!("GC error: {}", e)))?;

    let py_obj = pythonize(py, &result)?;
    Ok(py_obj.into())
}

/// Derive key from passphrase
#[pyfunction]
fn derive_key(passphrase: &str, salt: &[u8]) -> PyResult<Vec<u8>> {
    let encryptor = AesGcmEncryptor::new();
    let key = encryptor.derive_key(passphrase, salt)
        .map_err(|e| PyErr::new::<pyo3::exceptions::PyValueError, _>(format!("Derive key error: {}", e)))?;
    Ok(key)
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
    m.add_function(wrap_pyfunction!(gc, m)?)?;
    m.add_function(wrap_pyfunction!(derive_key, m)?)?;
    Ok(())
}
