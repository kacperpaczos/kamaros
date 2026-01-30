//! # Export Archive Use Case
//!
//! Exports the current project state (including history) as a ZIP archive.
//! The archive structure mirrors the JCF layout:
//! - .store/
//!   - manifest.json
//!   - blobs/
//!   - deltas/
//! - content/
//!   - [working directory files]

use crate::ports::{PortResult, StoragePort, PortError};
use std::io::{Write, Cursor};
use zip::write::FileOptions;

/// Input for file ExportArchive
#[derive(Debug)]
pub struct ExportArchiveInput {
    // Potential options: compression level, exclusion patterns, etc.
}

/// Use case for exporting project to ZIP
pub struct ExportArchiveUseCase<S: StoragePort> {
    storage: S,
}

impl<S: StoragePort> ExportArchiveUseCase<S> {
    pub fn new(storage: S) -> Self {
        Self { storage }
    }

    /// Execute export to in-memory ZIP buffer
    pub async fn execute(&self) -> PortResult<Vec<u8>> {
        let mut buffer = Vec::new();
        let mut zip = zip::ZipWriter::new(Cursor::new(&mut buffer));
        
        // Define directories to include
        let dirs_to_scan = vec![".store", "content"];
        
        let options = FileOptions::<()>::default()
            .compression_method(zip::CompressionMethod::Deflated)
            .unix_permissions(0o644);

        for root_dir in dirs_to_scan {
            // List all files recursively
            // We don't check exists(root_dir) because some adapters don't track directories explicitly
            let files = self.list_recursive(root_dir).await?;

            for file_path in files {
                let content = self.storage.read(&file_path).await?;
                zip.start_file(&file_path, options).map_err(|e| PortError::Io(std::io::Error::new(std::io::ErrorKind::Other, e.to_string())))?;
                zip.write_all(&content).map_err(PortError::Io)?;
            }
        }

        zip.finish().map_err(|e| PortError::Io(std::io::Error::new(std::io::ErrorKind::Other, e.to_string())))?;
        
        Ok(buffer)
    }

    /// Helper to Recursively list files
    async fn list_recursive(&self, dir: &str) -> PortResult<Vec<String>> {
        let mut results = Vec::new();
        let mut queue = vec![dir.to_string()];

        while let Some(current_dir) = queue.pop() {
            let entries = self.storage.list(&current_dir).await?;
            
            for entry in entries {
                let full_path = format!("{}/{}", current_dir, entry);
                
                match self.storage.list(&full_path).await {
                    Ok(children) if !children.is_empty() => {
                        queue.push(full_path);
                    },
                    _ => {
                        if self.storage.size(&full_path).await.is_ok() {
                            results.push(full_path);
                        }
                    }
                }
            }
        }
        
        Ok(results)
    }
}
