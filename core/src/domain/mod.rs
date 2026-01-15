pub mod version;
pub mod manifest;
pub mod blob;
// pub mod patch;

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    #[test]
    fn test_manifest_serialization() {
        let manifest = manifest::Manifest {
            format_version: "1.0.0".to_string(),
            metadata: manifest::ProjectMetadata {
                name: "Test Project".to_string(),
                description: None,
                created: "2024-01-01T00:00:00Z".to_string(),
                last_modified: "2024-01-01T00:00:00Z".to_string(),
                author: Some("Tester".to_string()),
            },
            file_map: HashMap::new(),
            version_history: vec![],
            refs: HashMap::from([("head".to_string(), "v1".to_string())]),
            rename_log: vec![],
        };

        let json = serde_json::to_string(&manifest).unwrap();
        assert!(json.contains("\"formatVersion\":\"1.0.0\""));
        assert!(json.contains("\"fileMap\":{}"));
    }
}
