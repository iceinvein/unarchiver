//! Security and safety checks for archive extraction.
//!
//! This module provides functions to validate archive entry paths and enforce
//! security policies to prevent attacks like zip-slip (path traversal).

use crate::error::SecurityError;
use crate::types::ExtractOptions;
use std::path::{Component, Path, PathBuf};

/// Entry type for filtering special file types.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EntryType {
    /// Regular file
    File,
    /// Directory
    Directory,
    /// Symbolic link
    Symlink,
    /// Hard link
    Hardlink,
    /// Other special file types (device, socket, etc.)
    Other,
}

/// Validates and normalizes an archive entry path to prevent security vulnerabilities.
///
/// This function performs the following checks:
/// - Rejects absolute paths
/// - Rejects paths containing ".." components (path traversal)
/// - Normalizes the path to remove redundant separators and "." components
/// - Validates UTF-8 encoding
///
/// # Arguments
///
/// * `path` - The entry path from the archive
///
/// # Returns
///
/// Returns a normalized `PathBuf` if the path is safe, or a `SecurityError` if validation fails.
///
/// # Examples
///
/// ```
/// use std::path::Path;
/// use extractor::safety::validate_entry_path;
///
/// // Valid relative path
/// let safe_path = validate_entry_path(Path::new("dir/file.txt")).unwrap();
/// assert_eq!(safe_path, Path::new("dir/file.txt"));
///
/// // Path traversal attempt - rejected
/// let result = validate_entry_path(Path::new("../../etc/passwd"));
/// assert!(result.is_err());
///
/// // Absolute path - rejected
/// let result = validate_entry_path(Path::new("/etc/passwd"));
/// assert!(result.is_err());
/// ```
pub fn validate_entry_path(path: &Path) -> Result<PathBuf, SecurityError> {
    // Check if path is absolute
    if path.is_absolute() {
        return Err(SecurityError::AbsolutePath(path.display().to_string()));
    }

    // Validate UTF-8 encoding
    let path_str = path.to_str().ok_or_else(|| {
        SecurityError::PathTraversal("Path contains invalid UTF-8 characters".to_string())
    })?;

    // Normalize and validate path components
    let mut normalized = PathBuf::new();

    for component in path.components() {
        match component {
            Component::Normal(part) => {
                // Check for ".." in the component itself (some archives may encode it differently)
                let part_str = part.to_str().ok_or_else(|| {
                    SecurityError::PathTraversal(
                        "Path component contains invalid UTF-8".to_string(),
                    )
                })?;

                if part_str == ".." {
                    return Err(SecurityError::PathTraversal(format!(
                        "Path contains '..' component: {}",
                        path_str
                    )));
                }

                normalized.push(part);
            }
            Component::CurDir => {
                // Skip "." components
                continue;
            }
            Component::ParentDir => {
                // Reject ".." components
                return Err(SecurityError::PathTraversal(format!(
                    "Path contains '..' component: {}",
                    path_str
                )));
            }
            Component::RootDir => {
                // Should not happen since we already checked for absolute paths
                return Err(SecurityError::AbsolutePath(path.display().to_string()));
            }
            Component::Prefix(_) => {
                // Windows-specific prefix (e.g., "C:")
                return Err(SecurityError::AbsolutePath(path.display().to_string()));
            }
        }
    }

    // Ensure the normalized path is not empty
    if normalized.as_os_str().is_empty() {
        return Err(SecurityError::PathTraversal(
            "Path normalizes to empty".to_string(),
        ));
    }

    Ok(normalized)
}

/// Checks if the current extracted size exceeds the configured limit.
///
/// # Arguments
///
/// * `current_bytes` - Total bytes extracted so far
/// * `limit` - Optional size limit in bytes (None means no limit)
///
/// # Returns
///
/// Returns `Ok(())` if within limits, or an error if the limit is exceeded.
///
/// # Examples
///
/// ```
/// use extractor::safety::check_size_limits;
///
/// // Within limit
/// assert!(check_size_limits(1000, Some(2000)).is_ok());
///
/// // Exceeds limit
/// assert!(check_size_limits(3000, Some(2000)).is_err());
///
/// // No limit
/// assert!(check_size_limits(999_999_999, None).is_ok());
/// ```
pub fn check_size_limits(current_bytes: u64, limit: Option<u64>) -> Result<(), SecurityError> {
    if let Some(max_bytes) = limit {
        if current_bytes > max_bytes {
            return Err(SecurityError::PathTraversal(format!(
                "Size limit exceeded: {} bytes > {} bytes",
                current_bytes, max_bytes
            )));
        }
    }
    Ok(())
}

/// Determines if an entry type is safe to extract based on the extraction options.
///
/// By default, symlinks and hardlinks are blocked for security reasons.
/// Other special file types (devices, sockets, etc.) are always blocked.
///
/// # Arguments
///
/// * `entry_type` - The type of the archive entry
/// * `options` - Extraction options that control which entry types are allowed
///
/// # Returns
///
/// Returns `true` if the entry type is safe to extract, `false` otherwise.
///
/// # Examples
///
/// ```
/// use extractor::safety::{is_safe_entry_type, EntryType};
/// use extractor::ExtractOptions;
///
/// let options = ExtractOptions::default();
///
/// // Regular files and directories are always safe
/// assert!(is_safe_entry_type(EntryType::File, &options));
/// assert!(is_safe_entry_type(EntryType::Directory, &options));
///
/// // Symlinks blocked by default
/// assert!(!is_safe_entry_type(EntryType::Symlink, &options));
///
/// // Allow symlinks with option
/// let mut options_with_symlinks = ExtractOptions::default();
/// options_with_symlinks.allow_symlinks = true;
/// assert!(is_safe_entry_type(EntryType::Symlink, &options_with_symlinks));
/// ```
pub fn is_safe_entry_type(entry_type: EntryType, options: &ExtractOptions) -> bool {
    match entry_type {
        EntryType::File | EntryType::Directory => true,
        EntryType::Symlink => options.allow_symlinks,
        EntryType::Hardlink => options.allow_hardlinks,
        EntryType::Other => false, // Always block special files
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validate_entry_path_valid() {
        // Simple relative path
        let result = validate_entry_path(Path::new("file.txt"));
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), Path::new("file.txt"));

        // Nested path
        let result = validate_entry_path(Path::new("dir/subdir/file.txt"));
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), Path::new("dir/subdir/file.txt"));

        // Path with current directory component
        let result = validate_entry_path(Path::new("./dir/file.txt"));
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), Path::new("dir/file.txt"));
    }

    #[test]
    fn test_validate_entry_path_absolute() {
        // Unix absolute path
        let result = validate_entry_path(Path::new("/etc/passwd"));
        assert!(result.is_err());
        assert!(matches!(
            result.unwrap_err(),
            SecurityError::AbsolutePath(_)
        ));

        // Another absolute path
        let result = validate_entry_path(Path::new("/tmp/file.txt"));
        assert!(result.is_err());
    }

    #[test]
    fn test_validate_entry_path_traversal() {
        // Parent directory component
        let result = validate_entry_path(Path::new("../etc/passwd"));
        assert!(result.is_err());
        assert!(matches!(
            result.unwrap_err(),
            SecurityError::PathTraversal(_)
        ));

        // Multiple parent components
        let result = validate_entry_path(Path::new("../../etc/passwd"));
        assert!(result.is_err());

        // Parent in middle of path
        let result = validate_entry_path(Path::new("dir/../etc/passwd"));
        assert!(result.is_err());

        // Parent at end
        let result = validate_entry_path(Path::new("dir/.."));
        assert!(result.is_err());
    }

    #[test]
    fn test_check_size_limits_within() {
        // Within limit
        assert!(check_size_limits(1000, Some(2000)).is_ok());
        assert!(check_size_limits(0, Some(1000)).is_ok());
        assert!(check_size_limits(999, Some(1000)).is_ok());
    }

    #[test]
    fn test_check_size_limits_exceeded() {
        // Exceeds limit
        let result = check_size_limits(2001, Some(2000));
        assert!(result.is_err());

        let result = check_size_limits(1_000_000, Some(999_999));
        assert!(result.is_err());
    }

    #[test]
    fn test_check_size_limits_no_limit() {
        // No limit set
        assert!(check_size_limits(0, None).is_ok());
        assert!(check_size_limits(999_999_999, None).is_ok());
        assert!(check_size_limits(u64::MAX, None).is_ok());
    }

    #[test]
    fn test_is_safe_entry_type_defaults() {
        let options = ExtractOptions::default();

        // Files and directories always safe
        assert!(is_safe_entry_type(EntryType::File, &options));
        assert!(is_safe_entry_type(EntryType::Directory, &options));

        // Symlinks and hardlinks blocked by default
        assert!(!is_safe_entry_type(EntryType::Symlink, &options));
        assert!(!is_safe_entry_type(EntryType::Hardlink, &options));

        // Other types always blocked
        assert!(!is_safe_entry_type(EntryType::Other, &options));
    }

    #[test]
    fn test_is_safe_entry_type_with_symlinks() {
        let mut options = ExtractOptions::default();
        options.allow_symlinks = true;

        assert!(is_safe_entry_type(EntryType::Symlink, &options));
        assert!(!is_safe_entry_type(EntryType::Hardlink, &options));
    }

    #[test]
    fn test_is_safe_entry_type_with_hardlinks() {
        let mut options = ExtractOptions::default();
        options.allow_hardlinks = true;

        assert!(!is_safe_entry_type(EntryType::Symlink, &options));
        assert!(is_safe_entry_type(EntryType::Hardlink, &options));
    }

    #[test]
    fn test_is_safe_entry_type_with_both() {
        let mut options = ExtractOptions::default();
        options.allow_symlinks = true;
        options.allow_hardlinks = true;

        assert!(is_safe_entry_type(EntryType::Symlink, &options));
        assert!(is_safe_entry_type(EntryType::Hardlink, &options));

        // Other types still blocked
        assert!(!is_safe_entry_type(EntryType::Other, &options));
    }

    #[test]
    fn test_validate_entry_path_unicode() {
        // Japanese characters
        let result = validate_entry_path(Path::new("日本語/ファイル.txt"));
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), Path::new("日本語/ファイル.txt"));

        // Chinese characters
        let result = validate_entry_path(Path::new("中文/文件.txt"));
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), Path::new("中文/文件.txt"));

        // Arabic characters
        let result = validate_entry_path(Path::new("عربي/ملف.txt"));
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), Path::new("عربي/ملف.txt"));

        // Emoji
        let result = validate_entry_path(Path::new("📁/📄.txt"));
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), Path::new("📁/📄.txt"));

        // Mixed unicode and ASCII
        let result = validate_entry_path(Path::new("folder/файл-file-文件.txt"));
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), Path::new("folder/файл-file-文件.txt"));

        // Unicode normalization - combining characters
        let result = validate_entry_path(Path::new("café/file.txt")); // é as single character
        assert!(result.is_ok());

        let result = validate_entry_path(Path::new("café/file.txt")); // é as e + combining accent
        assert!(result.is_ok());
    }

    #[test]
    fn test_validate_entry_path_unicode_traversal() {
        // Unicode path traversal attempts should still be blocked
        let result = validate_entry_path(Path::new("日本語/../etc/passwd"));
        assert!(result.is_err());
        assert!(matches!(
            result.unwrap_err(),
            SecurityError::PathTraversal(_)
        ));

        // Unicode with parent directory
        let result = validate_entry_path(Path::new("../中文/file.txt"));
        assert!(result.is_err());
    }

    #[test]
    fn test_validate_entry_path_edge_cases() {
        // Empty path components should be handled
        let result = validate_entry_path(Path::new("dir//file.txt"));
        assert!(result.is_ok());

        // Multiple current directory components
        let result = validate_entry_path(Path::new("./././file.txt"));
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), Path::new("file.txt"));

        // Path with only current directory
        let result = validate_entry_path(Path::new("."));
        assert!(result.is_err()); // Should normalize to empty and be rejected

        // Path with trailing slash (directory)
        let result = validate_entry_path(Path::new("dir/subdir/"));
        assert!(result.is_ok());
    }

    #[test]
    fn test_validate_entry_path_zip_slip_variants() {
        // Classic zip-slip
        let result = validate_entry_path(Path::new("../../etc/passwd"));
        assert!(result.is_err());

        // Zip-slip with more levels
        let result = validate_entry_path(Path::new("../../../../../../../etc/passwd"));
        assert!(result.is_err());

        // Zip-slip in middle of path
        let result = validate_entry_path(Path::new("safe/../../etc/passwd"));
        assert!(result.is_err());

        // Zip-slip with current directory obfuscation
        let result = validate_entry_path(Path::new("./../../etc/passwd"));
        assert!(result.is_err());

        // Zip-slip targeting home directory
        let result = validate_entry_path(Path::new("../../home/user/.ssh/id_rsa"));
        assert!(result.is_err());
    }

    #[test]
    fn test_check_size_limits_boundary() {
        // Exact limit should pass
        assert!(check_size_limits(1000, Some(1000)).is_ok());

        // One byte over should fail
        let result = check_size_limits(1001, Some(1000));
        assert!(result.is_err());

        // Large values
        let gb_20 = 20 * 1024 * 1024 * 1024u64;
        assert!(check_size_limits(gb_20, Some(gb_20)).is_ok());
        assert!(check_size_limits(gb_20 + 1, Some(gb_20)).is_err());
    }
}
