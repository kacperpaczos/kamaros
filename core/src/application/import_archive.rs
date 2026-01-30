//! # Import Archive Use Case
//!
//! Imports a project from a ZIP archive.
//! 
//! - Validates valid JCF structure (must have .store/manifest.json)
//! - Overwrites existing storage content

use crate::domain::manifest::Manifest;
use crate::ports::{PortResult, StoragePort, PortError};
use std::io::Cursor;
use zip::ZipArchive;
use std::io::Read;

/// Input for ImportArchive
#[derive(Debug)]
pub struct ImportArchiveInput {
    pub archive_data: Vec<u8>,
}

/// Output for ImportArchive
#[derive(Debug, serde::Serialize)]
pub struct ImportArchiveOutput {
    pub project_name: String,
    pub files_imported: usize,
    pub total_size: usize,
}

/// Use case for importing ZIP
pub struct ImportArchiveUseCase<S: StoragePort> {
    storage: S,
}

impl<S: StoragePort> ImportArchiveUseCase<S> {
    pub fn new(storage: S) -> Self {
        Self { storage }
    }

    /// Execute import from ZIP bytes
    pub async fn execute(&self, input: ImportArchiveInput) -> PortResult<ImportArchiveOutput> {
        let cursor = Cursor::new(input.archive_data);
        let mut zip = ZipArchive::new(cursor).map_err(|e| PortError::Io(std::io::Error::new(std::io::ErrorKind::Other, e.to_string())))?;

        // 1. Validate Manifest exists
        let manifest_path = ".store/manifest.json";
        let project_name = {
            let mut manifest_file = zip.by_name(manifest_path).map_err(|_| PortError::Io(std::io::Error::new(std::io::ErrorKind::NotFound, "Invalid JCF Archive: .store/manifest.json not found")))?;
            
            let mut content = String::new();
            manifest_file.read_to_string(&mut content).map_err(PortError::Io)?;
            
            let manifest: Manifest = serde_json::from_str(&content)
                .map_err(|e| PortError::Io(std::io::Error::new(std::io::ErrorKind::InvalidData, format!("Invalid manifest JSON: {}", e))))?;
            
            manifest.metadata.name
        };

        // 2. Extract all files
        let mut files_imported = 0;
        let mut total_size = 0;

        for i in 0..zip.len() {
            let mut file = zip.by_index(i).map_err(|e| PortError::Io(std::io::Error::new(std::io::ErrorKind::Other, e.to_string())))?;
            
            // Skip directories (StoragePort.write creates parents automatically usually, or we don't track empty dirs)
            if file.is_dir() {
                continue;
            }

            let path = file.name().to_string();
            
            // Security check: prevent zip slip (writing outside storage root)
            // Although simple path traversal check is good practice.
            if path.contains("..") {
                 return Err(PortError::Io(std::io::Error::new(std::io::ErrorKind::PermissionDenied, format!("Zip slip detected: {}", path))));
            }

            let mut content = Vec::new();
            file.read_to_end(&mut content).map_err(PortError::Io)?;
            
            total_size += content.len();
            
            // Write to storage
            self.storage.write(&path, &content).await?;
            files_imported += 1;
        }

        Ok(ImportArchiveOutput {
            project_name,
            files_imported,
            total_size,
        })
    }
}
