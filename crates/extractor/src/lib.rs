//! # Extractor
//!
//! A secure archive extraction library supporting multiple formats.
//!
//! This library provides safe extraction of various archive formats with built-in
//! security features to prevent path traversal attacks, size limit enforcement,
//! and configurable handling of special file types.
//!
//! ## Supported Formats
//!
//! - ZIP (including ZIP64)
//! - TAR (with gzip, bzip2, xz compression)
//! - 7-Zip
//! - RAR (read-only)
//! - ISO
//!
//! ## Example
//!
//! ```rust,no_run
//! use extractor::{probe, extract, ExtractOptions, OverwriteMode};
//! use std::path::Path;
//! use std::sync::Arc;
//! use std::sync::atomic::AtomicBool;
//!
//! # fn main() -> Result<(), Box<dyn std::error::Error>> {
//! // Probe archive metadata
//! let info = probe(Path::new("archive.zip"))?;
//! println!("Format: {}, Entries: {}", info.format, info.entries);
//!
//! // Extract with default options
//! let options = ExtractOptions::default();
//! let cancel_flag = Arc::new(AtomicBool::new(false));
//! let progress_cb = |file: &str, bytes: u64, total: Option<u64>| {
//!     println!("Extracting: {} ({} bytes)", file, bytes);
//!     true // Continue extraction
//! };
//!
//! let stats = extract(
//!     Path::new("archive.zip"),
//!     Path::new("output"),
//!     &options,
//!     &progress_cb,
//!     cancel_flag,
//! )?;
//!
//! println!("Extracted {} files ({} bytes)", stats.files_extracted, stats.bytes_written);
//! # Ok(())
//! # }
//! ```

pub mod error;
pub mod extract;
pub mod probe;
pub mod safety;
pub mod types;

// Re-export main types
pub use error::{ExtractError, SecurityError};
pub use safety::EntryType;
pub use types::{ArchiveEntry, ArchiveInfo, ExtractOptions, ExtractStats, OverwriteMode};

use std::path::Path;
use std::sync::atomic::AtomicBool;
use std::sync::Arc;

/// Type alias for progress callback functions.
///
/// The callback receives:
/// - `file`: The current file being extracted
/// - `bytes_written`: Number of bytes written so far
/// - `total_bytes`: Optional total size of the file
///
/// Returns `true` to continue extraction, `false` to cancel.
pub type ProgressCallback = dyn Fn(&str, u64, Option<u64>) -> bool + Send + Sync;

/// Probe an archive to retrieve metadata without extracting.
///
/// # Arguments
///
/// * `path` - Path to the archive file
///
/// # Returns
///
/// Returns `ArchiveInfo` containing format, entry count, sizes, and encryption status.
///
/// # Errors
///
/// Returns an error if:
/// - The archive file doesn't exist
/// - The format is unsupported or corrupted
/// - The archive cannot be read
pub fn probe(path: &Path) -> Result<ArchiveInfo, ExtractError> {
    probe::probe_archive(path)
}

/// Extract an archive to the specified output directory.
///
/// # Arguments
///
/// * `archive_path` - Path to the archive file
/// * `output_dir` - Directory where files will be extracted
/// * `options` - Extraction options (overwrite mode, size limits, etc.)
/// * `progress_cb` - Callback function for progress updates
/// * `cancel_flag` - Atomic flag to signal cancellation
///
/// # Returns
///
/// Returns `ExtractStats` with extraction statistics on success.
///
/// # Errors
///
/// Returns an error if:
/// - The archive file doesn't exist or is corrupted
/// - Security violations are detected (path traversal, size limits)
/// - Password is required or incorrect
/// - Extraction is cancelled
/// - I/O errors occur
pub fn extract(
    archive_path: &Path,
    output_dir: &Path,
    options: &ExtractOptions,
    progress_cb: &ProgressCallback,
    cancel_flag: Arc<AtomicBool>,
) -> Result<ExtractStats, ExtractError> {
    extract::extract_archive(archive_path, output_dir, options, progress_cb, cancel_flag)
}

#[cfg(test)]
mod tests {
    #[test]
    fn test_placeholder() {
        // Placeholder test - will be replaced with actual tests
        assert!(true);
    }
}
