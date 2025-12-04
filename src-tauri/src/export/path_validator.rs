use crate::error::{CortexError, Result};
use std::path::{Component, Path, PathBuf};

/// Validates export output paths to prevent path traversal attacks
pub struct PathValidator;

impl PathValidator {
    /// Validate an output path for export operations
    ///
    /// Security checks:
    /// 1. Rejects paths with ".." (parent directory) components
    /// 2. For absolute paths, ensures they're within safe directories
    /// 3. Canonicalizes paths to resolve symlinks and relative components
    ///
    /// # Arguments
    /// * `path` - The user-provided path to validate
    ///
    /// # Returns
    /// * `Ok(PathBuf)` - Validated and canonicalized path
    /// * `Err(CortexError::InvalidPath)` - Path failed validation
    ///
    /// # Examples
    /// ```
    /// // Valid paths
    /// PathValidator::validate_export_path(".cortex-export").unwrap();
    /// PathValidator::validate_export_path("./my-export").unwrap();
    ///
    /// // Invalid paths (path traversal attempts)
    /// PathValidator::validate_export_path("../../etc/passwd").unwrap_err();
    /// PathValidator::validate_export_path("/etc/passwd").unwrap_err();
    /// ```
    pub fn validate_export_path(path: &str) -> Result<PathBuf> {
        if path.trim().is_empty() {
            return Err(CortexError::InvalidPath {
                path: path.to_string(),
                reason: "Export path cannot be empty".to_string(),
            });
        }

        let path_buf = PathBuf::from(path);

        // Check for path traversal attempts (..)
        for component in path_buf.components() {
            if component == Component::ParentDir {
                return Err(CortexError::InvalidPath {
                    path: path.to_string(),
                    reason: "Path traversal is not allowed (..)".to_string(),
                });
            }
        }

        // For absolute paths, validate they're in safe locations
        let is_absolute = path_buf.is_absolute();
        if is_absolute {
            Self::validate_absolute_path(&path_buf)?;
        }

        // Get current working directory for relative paths
        let base_dir = std::env::current_dir().map_err(|e| CortexError::Internal {
            message: format!("Failed to get current directory: {}", e),
        })?;

        // Resolve relative path against current directory
        let full_path = if is_absolute {
            path_buf
        } else {
            base_dir.join(&path_buf)
        };

        // Canonicalize if the parent directory exists
        // (we can't canonicalize non-existent paths, but we can validate parent)
        if let Some(parent) = full_path.parent() {
            if parent.exists() {
                let canonical = parent
                    .canonicalize()
                    .map_err(|e| CortexError::Internal {
                        message: format!("Failed to canonicalize path: {}", e),
                    })?;

                // Verify the canonical path doesn't escape the base directory
                if !is_absolute && !canonical.starts_with(&base_dir) {
                    return Err(CortexError::InvalidPath {
                        path: path.to_string(),
                        reason: "Path resolves outside of working directory".to_string(),
                    });
                }

                // Return the validated path (with original filename)
                let filename = full_path.file_name().ok_or_else(|| CortexError::InvalidPath {
                    path: path.to_string(),
                    reason: "Path must include a filename".to_string(),
                })?;

                return Ok(canonical.join(filename));
            }
        }

        // Parent doesn't exist yet - that's OK for new exports
        // Just return the full path (already validated)
        Ok(full_path)
    }

    /// Validate that an absolute path is within safe directories
    fn validate_absolute_path(path: &Path) -> Result<()> {
        // Get user's home directory
        let home_dir = dirs::home_dir().ok_or_else(|| CortexError::Internal {
            message: "Could not determine home directory".to_string(),
        })?;

        // Get system temp directory
        let temp_dir = std::env::temp_dir();

        // Check if path is within home directory or temp directory
        let is_safe = path.starts_with(&home_dir) || path.starts_with(&temp_dir);

        if !is_safe {
            return Err(CortexError::InvalidPath {
                path: path.display().to_string(),
                reason: format!(
                    "Absolute paths must be within home directory ({}) or temp directory ({})",
                    home_dir.display(),
                    temp_dir.display()
                ),
            });
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_reject_empty_path() {
        let result = PathValidator::validate_export_path("");
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), CortexError::InvalidPath { .. }));
    }

    #[test]
    fn test_reject_parent_dir_traversal() {
        let result = PathValidator::validate_export_path("../../etc/passwd");
        assert!(result.is_err());
        match result.unwrap_err() {
            CortexError::InvalidPath { reason, .. } => {
                assert!(reason.contains("traversal"));
            }
            _ => panic!("Expected InvalidPath error"),
        }
    }

    #[test]
    fn test_reject_hidden_parent_traversal() {
        let result = PathValidator::validate_export_path("./foo/../../etc/passwd");
        assert!(result.is_err());
    }

    #[test]
    fn test_accept_valid_relative_path() {
        let result = PathValidator::validate_export_path(".cortex-export");
        assert!(result.is_ok());
    }

    #[test]
    fn test_accept_subdirectory() {
        let result = PathValidator::validate_export_path("exports/my-export");
        assert!(result.is_ok());
    }

    #[test]
    fn test_reject_absolute_system_path() {
        let result = PathValidator::validate_export_path("/etc/passwd");
        assert!(result.is_err());
        match result.unwrap_err() {
            CortexError::InvalidPath { reason, .. } => {
                assert!(reason.contains("home directory") || reason.contains("temp directory"));
            }
            _ => panic!("Expected InvalidPath error"),
        }
    }

    #[test]
    fn test_accept_absolute_path_in_home() {
        // This will only work if the test is run with a valid home directory
        if let Some(home) = dirs::home_dir() {
            let test_path = home.join("test-export");
            let result = PathValidator::validate_export_path(test_path.to_str().unwrap());
            assert!(result.is_ok());
        }
    }
}
