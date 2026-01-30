//! # Simple Diff Adapter
//!
//! Implementation of DiffPort using the `similar` crate.

use crate::ports::{DiffPort, PortResult};
use similar::TextDiff;

/// Simple diff implementation using `similar` crate
pub struct SimpleDiff;

impl SimpleDiff {
    pub fn new() -> Self {
        Self
    }
}

impl Default for SimpleDiff {
    fn default() -> Self {
        Self::new()
    }
}

impl DiffPort for SimpleDiff {
    /// Compute unified diff between old and new text
    fn compute_diff(&self, old: &str, new: &str) -> String {
        let diff = TextDiff::from_lines(old, new);
        
        diff.unified_diff()
            .context_radius(3)
            .to_string()
    }

    /// Apply unified diff patch to text
    fn apply_patch(&self, text: &str, patch: &str) -> PortResult<String> {
        if patch.is_empty() {
            return Ok(text.to_string());
        }

        let p = diffy::Patch::from_str(patch)
            .map_err(|e| crate::ports::PortError::PatchFailed(format!("Parse patch error: {}", e)))?;
            
        diffy::apply(text, &p)
            .map_err(|e| crate::ports::PortError::PatchFailed(format!("Apply patch error: {}", e)))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_compute_diff() {
        let diff = SimpleDiff::new();
        
        let old = "line1\nline2\nline3";
        let new = "line1\nmodified\nline3";
        
        let patch = diff.compute_diff(old, new);
        
        assert!(patch.contains("-line2"));
        assert!(patch.contains("+modified"));
    }

    #[test]
    fn test_compute_diff_identical() {
        let diff = SimpleDiff::new();
        
        let text = "same\ntext";
        let patch = diff.compute_diff(text, text);
        
        // Empty or minimal patch for identical files
        assert!(!patch.contains("-") || !patch.contains("+"));
    }

    #[test]
    fn test_diff_empty_to_content() {
        let diff = SimpleDiff::new();
        let old = "";
        let new = "Hello ZIP";
        let patch = diff.compute_diff(old, new);
        
        assert!(patch.contains("+Hello ZIP"));
        
        // Verify application
        let applied = diff.apply_patch(old, &patch).expect("Apply failed");
        assert_eq!(applied, new);
    }

    #[test]
    fn test_apply_patch_complex() {
        let diff = SimpleDiff::new();
        let old = "line1\nline2\nline3";
        let new = "line1\nmodified\nline3";
        let patch = diff.compute_diff(old, new);
        
        let applied = diff.apply_patch(old, &patch).expect("Apply failed");
        assert_eq!(applied, new);
    }
}
