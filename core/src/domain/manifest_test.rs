//! Tests for Manifest domain entity
//! 
//! Each test focuses on a single aspect of Manifest functionality.

#[cfg(test)]
mod manifest_tests {
    use crate::domain::manifest::{Manifest, ProjectMetadata, FileEntry, FileType, RenameEntry};
    use std::collections::HashMap;

    /// Helper: Create a minimal valid manifest
    fn create_test_manifest(name: &str) -> Manifest {
        Manifest {
            format_version: "1.0.0".to_string(),
            metadata: ProjectMetadata {
                name: name.to_string(),
                description: None,
                created: "2024-01-01T00:00:00Z".to_string(),
                last_modified: "2024-01-01T00:00:00Z".to_string(),
                author: None,
            },
            file_map: HashMap::new(),
            version_history: vec![],
            refs: HashMap::from([("head".to_string(), "".to_string())]),
            rename_log: vec![],
        }
    }

    // =========================================================================
    // HAPPY PATH TESTS
    // =========================================================================

    /// Test: Manifest can be serialized to JSON
    #[test]
    fn test_manifest_serializes_to_json() {
        let manifest = create_test_manifest("TestProject");
        
        let json = serde_json::to_string(&manifest);
        
        assert!(json.is_ok(), "Manifest should serialize to JSON");
        let json_str = json.unwrap();
        assert!(json_str.contains("TestProject"), "JSON should contain project name");
    }

    /// Test: Manifest can be deserialized from JSON
    #[test]
    fn test_manifest_deserializes_from_json() {
        let json = r#"{
            "formatVersion": "1.0.0",
            "metadata": {
                "name": "TestProject",
                "created": "2024-01-01T00:00:00Z",
                "lastModified": "2024-01-01T00:00:00Z"
            },
            "fileMap": {},
            "versionHistory": [],
            "refs": {"head": ""},
            "renameLog": []
        }"#;
        
        let manifest: Result<Manifest, _> = serde_json::from_str(json);
        
        assert!(manifest.is_ok(), "Should deserialize valid JSON");
        assert_eq!(manifest.unwrap().metadata.name, "TestProject");
    }

    /// Test: Manifest serialization round-trip preserves data
    #[test]
    fn test_manifest_roundtrip_preserves_data() {
        let original = create_test_manifest("RoundtripTest");
        
        let json = serde_json::to_string(&original).unwrap();
        let restored: Manifest = serde_json::from_str(&json).unwrap();
        
        assert_eq!(original.metadata.name, restored.metadata.name);
        assert_eq!(original.format_version, restored.format_version);
    }

    /// Test: FileEntry serializes with correct serde rename
    #[test]
    fn test_file_entry_serde_rename() {
        let entry = FileEntry {
            inode_id: "abc123".to_string(),
            file_type: FileType::Text,
            current_hash: Some("sha256hash".to_string()),
            created: "2024-01-01T00:00:00Z".to_string(),
            modified: "2024-01-01T00:00:00Z".to_string(),
        };
        
        let json = serde_json::to_string(&entry).unwrap();
        
        // Check serde renames work correctly
        assert!(json.contains("\"inodeId\""), "Should use camelCase inodeId");
        assert!(json.contains("\"type\""), "Should rename file_type to type");
        assert!(json.contains("\"currentHash\""), "Should use camelCase currentHash");
    }

    /// Test: FileType serializes to lowercase
    #[test]
    fn test_file_type_serializes_lowercase() {
        let text = FileType::Text;
        let binary = FileType::Binary;
        
        let text_json = serde_json::to_string(&text).unwrap();
        let binary_json = serde_json::to_string(&binary).unwrap();
        
        assert_eq!(text_json, "\"text\"", "FileType::Text should serialize to 'text'");
        assert_eq!(binary_json, "\"binary\"", "FileType::Binary should serialize to 'binary'");
    }

    // =========================================================================
    // EDGE CASE TESTS
    // =========================================================================

    /// Edge case: Manifest with optional fields as None
    #[test]
    fn test_manifest_with_optional_none() {
        let manifest = Manifest {
            format_version: "1.0.0".to_string(),
            metadata: ProjectMetadata {
                name: "MinimalProject".to_string(),
                description: None,  // Optional
                created: "2024-01-01T00:00:00Z".to_string(),
                last_modified: "2024-01-01T00:00:00Z".to_string(),
                author: None,  // Optional
            },
            file_map: HashMap::new(),
            version_history: vec![],
            refs: HashMap::new(),
            rename_log: vec![],
        };
        
        let json = serde_json::to_string(&manifest);
        
        assert!(json.is_ok(), "Manifest with None optionals should serialize");
    }

    /// Edge case: Manifest with empty name
    #[test]
    fn test_manifest_with_empty_name() {
        let manifest = create_test_manifest("");
        
        let json = serde_json::to_string(&manifest);
        
        assert!(json.is_ok(), "Empty name should still serialize (business validation is separate)");
    }

    /// Edge case: Manifest with Unicode characters
    #[test]
    fn test_manifest_with_unicode() {
        let manifest = create_test_manifest("Проект 日本語 🚀");
        
        let json = serde_json::to_string(&manifest).unwrap();
        let restored: Manifest = serde_json::from_str(&json).unwrap();
        
        assert_eq!(restored.metadata.name, "Проект 日本語 🚀");
    }

    /// Edge case: Manifest with special JSON characters
    #[test]
    fn test_manifest_with_special_chars() {
        let manifest = create_test_manifest(r#"Name "with" quotes \ and newline
        "#);
        
        let json = serde_json::to_string(&manifest);
        
        assert!(json.is_ok(), "Special characters should be escaped properly");
    }

    /// Edge case: Deserialize JSON missing optional fields
    #[test]
    fn test_deserialize_missing_optional_fields() {
        let json = r#"{
            "formatVersion": "1.0.0",
            "metadata": {
                "name": "TestProject",
                "created": "2024-01-01T00:00:00Z",
                "lastModified": "2024-01-01T00:00:00Z"
            },
            "fileMap": {},
            "versionHistory": [],
            "refs": {},
            "renameLog": []
        }"#;
        
        let manifest: Result<Manifest, _> = serde_json::from_str(json);
        
        assert!(manifest.is_ok(), "Should accept JSON without optional fields");
        let m = manifest.unwrap();
        assert!(m.metadata.description.is_none());
        assert!(m.metadata.author.is_none());
    }

    /// Edge case: RenameEntry serialization
    #[test]
    fn test_rename_entry_serialization() {
        let entry = RenameEntry {
            from: "old/path.txt".to_string(),
            to: "new/path.txt".to_string(),
            timestamp: "2024-01-01T00:00:00Z".to_string(),
            version_id: "v123".to_string(),
        };
        
        let json = serde_json::to_string(&entry).unwrap();
        
        assert!(json.contains("\"versionId\""), "Should use camelCase versionId");
    }
}
