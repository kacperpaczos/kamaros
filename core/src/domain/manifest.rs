use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Serialize, Deserialize)]
pub struct Manifest {
    #[serde(rename = "formatVersion")]
    pub format_version: String,
    
    pub metadata: ProjectMetadata,
    
    #[serde(rename = "fileMap")]
    pub file_map: HashMap<String, FileEntry>,
    
    #[serde(rename = "versionHistory")]
    pub version_history: Vec<crate::domain::version::Version>,
    
    pub refs: HashMap<String, String>,
    
    #[serde(rename = "renameLog")]
    pub rename_log: Vec<RenameEntry>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ProjectMetadata {
    pub name: String,
    pub description: Option<String>,
    pub created: String,
    #[serde(rename = "lastModified")]
    pub last_modified: String,
    pub author: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct FileEntry {
    #[serde(rename = "inodeId")]
    pub inode_id: String,
    
    #[serde(rename = "type")]
    pub file_type: FileType,
    
    #[serde(rename = "currentHash")]
    pub current_hash: Option<String>,
    
    pub created: String,
    pub modified: String,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum FileType {
    Text,
    Binary,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct RenameEntry {
    pub from: String,
    pub to: String,
    pub timestamp: String,
    #[serde(rename = "versionId")]
    pub version_id: String,
}
