//! Tests for Version domain entity
//! 
//! Each test focuses on a single aspect of Version functionality.

#[cfg(test)]
mod version_tests {
    use crate::domain::version::{Version, FileState};
    use std::collections::HashMap;

    /// Helper: Create a minimal valid version
    fn create_test_version(id: &str, message: &str) -> Version {
        Version {
            id: id.to_string(),
            parent_id: None,
            timestamp: "2024-01-01T00:00:00Z".to_string(),
            message: message.to_string(),
            author: "TestAuthor".to_string(),
            file_states: HashMap::new(),
        }
    }

    // =========================================================================
    // HAPPY PATH TESTS
    // =========================================================================

    /// Test: Version can be serialized to JSON
    #[test]
    fn test_version_serializes_to_json() {
        let version = create_test_version("v1", "Initial commit");
        
        let json = serde_json::to_string(&version);
        
        assert!(json.is_ok(), "Version should serialize to JSON");
        let json_str = json.unwrap();
        assert!(json_str.contains("Initial commit"), "JSON should contain message");
    }

    /// Test: Version can be deserialized from JSON
    #[test]
    fn test_version_deserializes_from_json() {
        let json = r#"{
            "id": "v1",
            "parentId": null,
            "timestamp": "2024-01-01T00:00:00Z",
            "message": "Test commit",
            "author": "Tester",
            "fileStates": {}
        }"#;
        
        let version: Result<Version, _> = serde_json::from_str(json);
        
        assert!(version.is_ok(), "Should deserialize valid JSON");
        assert_eq!(version.unwrap().message, "Test commit");
    }

    /// Test: Version with parent link
    #[test]
    fn test_version_with_parent_id() {
        let version = Version {
            id: "v2".to_string(),
            parent_id: Some("v1".to_string()),
            timestamp: "2024-01-01T00:00:00Z".to_string(),
            message: "Second commit".to_string(),
            author: "Author".to_string(),
            file_states: HashMap::new(),
        };
        
        let json = serde_json::to_string(&version).unwrap();
        
        assert!(json.contains("\"parentId\":\"v1\""), "Should serialize parent_id");
    }

    /// Test: Version roundtrip preserves data
    #[test]
    fn test_version_roundtrip_preserves_data() {
        let original = Version {
            id: "uuid-123".to_string(),
            parent_id: Some("uuid-122".to_string()),
            timestamp: "2024-01-15T10:30:00Z".to_string(),
            message: "Important change".to_string(),
            author: "developer@example.com".to_string(),
            file_states: HashMap::from([
                ("file.txt".to_string(), FileState {
                    inode_id: "inode1".to_string(),
                    hash: Some("sha256hash".to_string()),
                    content_ref: None,
                    deleted: None,
                }),
            ]),
        };
        
        let json = serde_json::to_string(&original).unwrap();
        let restored: Version = serde_json::from_str(&json).unwrap();
        
        assert_eq!(original.id, restored.id);
        assert_eq!(original.parent_id, restored.parent_id);
        assert_eq!(original.message, restored.message);
        assert_eq!(original.file_states.len(), restored.file_states.len());
    }

    /// Test: FileState serializes correctly
    #[test]
    fn test_file_state_serde_rename() {
        let state = FileState {
            inode_id: "abc123".to_string(),
            hash: Some("sha256".to_string()),
            content_ref: Some("delta/v1.patch".to_string()),
            deleted: None,
        };
        
        let json = serde_json::to_string(&state).unwrap();
        
        assert!(json.contains("\"inodeId\""), "Should use camelCase inodeId");
        assert!(json.contains("\"contentRef\""), "Should use camelCase contentRef");
        assert!(!json.contains("\"deleted\""), "Should skip None deleted field");
    }

    /// Test: FileState with deleted flag
    #[test]
    fn test_file_state_deleted_flag() {
        let state = FileState {
            inode_id: "abc123".to_string(),
            hash: None,
            content_ref: None,
            deleted: Some(true),
        };
        
        let json = serde_json::to_string(&state).unwrap();
        
        assert!(json.contains("\"deleted\":true"), "Should include deleted when Some");
    }

    // =========================================================================
    // EDGE CASE TESTS
    // =========================================================================

    /// Edge case: Version with empty message
    #[test]
    fn test_version_with_empty_message() {
        let version = create_test_version("v1", "");
        
        let json = serde_json::to_string(&version);
        
        assert!(json.is_ok(), "Empty message should be allowed");
    }

    /// Edge case: Version with many file states
    #[test]
    fn test_version_with_many_file_states() {
        let mut file_states = HashMap::new();
        for i in 0..100 {
            file_states.insert(
                format!("file_{}.txt", i),
                FileState {
                    inode_id: format!("inode_{}", i),
                    hash: Some(format!("hash_{}", i)),
                    content_ref: None,
                    deleted: None,
                },
            );
        }
        
        let version = Version {
            id: "v1".to_string(),
            parent_id: None,
            timestamp: "2024-01-01T00:00:00Z".to_string(),
            message: "Big commit".to_string(),
            author: "Author".to_string(),
            file_states,
        };
        
        let json = serde_json::to_string(&version).unwrap();
        let restored: Version = serde_json::from_str(&json).unwrap();
        
        assert_eq!(restored.file_states.len(), 100);
    }

    /// Edge case: Version with Unicode in message
    #[test]
    fn test_version_with_unicode_message() {
        let version = create_test_version("v1", "Изменения 変更 🎉");
        
        let json = serde_json::to_string(&version).unwrap();
        let restored: Version = serde_json::from_str(&json).unwrap();
        
        assert_eq!(restored.message, "Изменения 変更 🎉");
    }

    /// Edge case: Deserialize with null parentId
    #[test]
    fn test_deserialize_null_parent_id() {
        let json = r#"{
            "id": "v1",
            "parentId": null,
            "timestamp": "2024-01-01T00:00:00Z",
            "message": "First",
            "author": "Test",
            "fileStates": {}
        }"#;
        
        let version: Version = serde_json::from_str(json).unwrap();
        
        assert!(version.parent_id.is_none());
    }
}
