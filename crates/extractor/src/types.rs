//! Type definitions for archive extraction.

use serde::{Deserialize, Serialize};
use std::time::Duration;
use ts_rs::TS;

/// Individual entry within an archive.
#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export, export_to = "../../../src/lib/bindings/")]
pub struct ArchiveEntry {
    /// Path of the entry within the archive
    pub path: String,

    /// Whether this entry is a directory
    pub is_directory: bool,

    /// Uncompressed size in bytes
    #[ts(type = "number")]
    pub size: u64,

    /// Compressed size in bytes (if available)
    #[ts(optional, type = "number")]
    pub compressed_size: Option<u64>,
}

/// Metadata information about an archive.
#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export, export_to = "../../../src/lib/bindings/")]
pub struct ArchiveInfo {
    /// Archive format (e.g., "ZIP", "TAR", "7Z")
    pub format: String,

    /// Number of entries in the archive
    #[ts(type = "number")]
    pub entries: u64,

    /// Compressed size in bytes (if available)
    #[ts(optional, type = "number")]
    pub compressed_bytes: Option<u64>,

    /// Estimated uncompressed size in bytes (if available)
    #[ts(optional, type = "number")]
    pub uncompressed_estimate: Option<u64>,

    /// Whether the archive is password-protected
    pub encrypted: bool,

    /// List of all entries in the archive
    pub entry_list: Vec<ArchiveEntry>,
}

/// Options for extracting an archive.
#[derive(Debug, Clone)]
pub struct ExtractOptions {
    /// How to handle file conflicts during extraction
    pub overwrite: OverwriteMode,

    /// Maximum total extracted size in bytes (default: 20 GB)
    pub size_limit_bytes: Option<u64>,

    /// Number of leading path components to strip from extracted files
    pub strip_components: u32,

    /// Whether to allow extraction of symbolic links
    pub allow_symlinks: bool,

    /// Whether to allow extraction of hard links
    pub allow_hardlinks: bool,

    /// Password for encrypted archives
    pub password: Option<String>,
}

impl Default for ExtractOptions {
    fn default() -> Self {
        Self {
            overwrite: OverwriteMode::Rename,
            size_limit_bytes: Some(20 * 1024 * 1024 * 1024), // 20 GB
            strip_components: 0,
            allow_symlinks: false,
            allow_hardlinks: false,
            password: None,
        }
    }
}

/// How to handle file conflicts during extraction.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum OverwriteMode {
    /// Replace existing files
    Replace,

    /// Skip files that already exist
    Skip,

    /// Rename new files by appending (1), (2), etc.
    Rename,
}

/// Statistics about a completed extraction operation.
#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export, export_to = "../../../src/lib/bindings/")]
pub struct ExtractStats {
    /// Number of files successfully extracted
    #[ts(type = "number")]
    pub files_extracted: u64,

    /// Total bytes written to disk
    #[ts(type = "number")]
    pub bytes_written: u64,

    /// Duration of the extraction operation (in seconds)
    #[serde(with = "duration_serde")]
    #[ts(type = "number")]
    pub duration: Duration,

    /// Whether the extraction was cancelled
    pub cancelled: bool,
}

impl Default for ExtractStats {
    fn default() -> Self {
        Self {
            files_extracted: 0,
            bytes_written: 0,
            duration: Duration::from_secs(0),
            cancelled: false,
        }
    }
}

// Helper module for Duration serialization
mod duration_serde {
    use serde::{Deserialize, Deserializer, Serialize, Serializer};
    use std::time::Duration;

    pub fn serialize<S>(duration: &Duration, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        duration.as_secs().serialize(serializer)
    }

    pub fn deserialize<'de, D>(deserializer: D) -> Result<Duration, D::Error>
    where
        D: Deserializer<'de>,
    {
        let secs = u64::deserialize(deserializer)?;
        Ok(Duration::from_secs(secs))
    }
}
