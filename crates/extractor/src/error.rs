//! Error types for archive extraction operations.

use std::path::PathBuf;
use thiserror::Error;

/// Main error type for extraction operations.
#[derive(Debug, Error)]
pub enum ExtractError {
    /// Archive file not found at the specified path.
    #[error("Archive not found: {0}")]
    NotFound(PathBuf),

    /// The archive format is not supported.
    #[error("Unsupported format: {0}")]
    UnsupportedFormat(String),

    /// The archive requires a password but none was provided.
    #[error("Password required")]
    PasswordRequired,

    /// The provided password is incorrect.
    #[error("Invalid password")]
    InvalidPassword,

    /// A security violation was detected during extraction.
    #[error("Security violation: {0}")]
    Security(#[from] SecurityError),

    /// The extraction size limit was exceeded.
    #[error("Size limit exceeded: {current} bytes > {limit} bytes")]
    SizeLimitExceeded {
        /// Current extracted size in bytes
        current: u64,
        /// Configured size limit in bytes
        limit: u64,
    },

    /// The archive is corrupted or malformed.
    #[error("Corrupted archive: {0}")]
    Corrupted(String),

    /// An I/O error occurred during extraction.
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    /// The extraction was cancelled by the user.
    #[error("Cancelled by user")]
    Cancelled,
}

/// Security-related errors during extraction.
#[derive(Debug, Error)]
pub enum SecurityError {
    /// Path traversal attempt detected (e.g., "../../../etc/passwd").
    #[error("Path traversal attempt: {0}")]
    PathTraversal(String),

    /// Absolute path not allowed in archive entries.
    #[error("Absolute path not allowed: {0}")]
    AbsolutePath(String),

    /// Unsafe entry type detected (e.g., symlink when not allowed).
    #[error("Unsafe entry type: {0}")]
    UnsafeEntryType(String),
}
