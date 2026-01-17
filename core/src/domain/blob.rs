use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Serialize, Deserialize)]
pub struct Blob {
    pub hash: String,
    pub size: usize,
    pub content: Vec<u8>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct BlobIndex {
    #[serde(flatten)]
    pub entries: HashMap<String, BlobMetadata>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct BlobMetadata {
    #[serde(rename = "mimeType")]
    pub mime_type: String,
    
    #[serde(rename = "originalName")]
    pub original_name: String,
    
    pub size: usize,
    
    #[serde(rename = "addedAt")]
    pub added_at: String,
    
    #[serde(rename = "refCount")]
    pub ref_count: usize,
    
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tags: Option<Vec<String>>,
}
