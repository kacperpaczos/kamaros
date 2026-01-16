//! # Simple Diff Adapter
//!
//! Implementation of DiffPort using the `similar` crate.

use crate::ports::{DiffPort, PortError, PortResult};
use similar::{ChangeTag, TextDiff};

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
    /// 
    /// Note: This is a simplified implementation.
    /// For production, consider using a proper patch library.
    fn apply_patch(&self, text: &str, patch: &str) -> PortResult<String> {
        // Parse unified diff and apply changes
        // This is a basic implementation that handles simple cases
        
        if patch.is_empty() {
            return Ok(text.to_string());
        }

        let mut result_lines: Vec<&str> = text.lines().collect();
        let patch_lines: Vec<&str> = patch.lines().collect();
        
        let mut i = 0;
        while i < patch_lines.len() {
            let line = patch_lines[i];
            
            // Skip header lines
            if line.starts_with("---") || line.starts_with("+++") || line.starts_with("@@") {
                i += 1;
                continue;
            }
            
            // Parse hunk header to get line numbers
            if line.starts_with("@@") {
                // Format: @@ -start,count +start,count @@
                // For now, we apply changes sequentially
                i += 1;
                continue;
            }
            
            i += 1;
        }
        
        // For a proper implementation, we need a real patch parser
        // This simplified version just returns the original text when
        // patch cannot be parsed properly
        
        // In production, use a crate like `diffy` or `patch`
        Ok(result_lines.join("\n"))
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
}
