use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Serialize, Deserialize)]
pub struct Version {
    pub id: String,
    
    #[serde(rename = "parentId")]
    pub parent_id: Option<String>,
    
    pub timestamp: String,
    pub message: String,
    pub author: String,
    
    #[serde(rename = "fileStates")]
    pub file_states: HashMap<String, FileState>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct FileState {
    #[serde(rename = "inodeId")]
    pub inode_id: String,
    
    pub hash: Option<String>,
    
    #[serde(rename = "contentRef")]
    pub content_ref: Option<String>,
    
    #[serde(skip_serializing_if = "Option::is_none")]
    pub deleted: Option<bool>,
}
