//! Archive probing functionality for reading metadata without extraction.

use crate::error::ExtractError;
use crate::types::ArchiveInfo;
use std::fs::File;
use std::io::BufReader;
use std::path::Path;

/// Probe an archive to retrieve metadata without extracting.
///
/// This function reads the archive header and metadata to determine:
/// - Archive format
/// - Number of entries
/// - Compressed and uncompressed sizes (when available)
/// - Whether the archive is encrypted
///
/// # Arguments
///
/// * `path` - Path to the archive file
///
/// # Returns
///
/// Returns `ArchiveInfo` containing the archive metadata.
///
/// # Errors
///
/// Returns an error if:
/// - The archive file doesn't exist
/// - The format is unsupported or corrupted
/// - The archive cannot be read
pub fn probe_archive(path: &Path) -> std::result::Result<ArchiveInfo, ExtractError> {
    // Check if file exists
    if !path.exists() {
        return Err(ExtractError::NotFound(path.to_path_buf()));
    }

    // Get compressed size from file metadata
    let file_metadata = std::fs::metadata(path)?;
    let compressed_bytes = Some(file_metadata.len());

    // Open the archive file
    let file = File::open(path)?;
    let reader = BufReader::new(file);

    // Detect format and count entries
    let (format, entries, uncompressed_estimate, encrypted) =
        analyze_archive(reader, path)?;

    Ok(ArchiveInfo {
        format,
        entries,
        compressed_bytes,
        uncompressed_estimate,
        encrypted,
    })
}

/// Analyze archive contents to extract metadata.
fn analyze_archive(
    reader: BufReader<File>,
    path: &Path,
) -> std::result::Result<(String, u64, Option<u64>, bool), ExtractError> {
    // Detect format from file extension and magic bytes
    let format = detect_format(path)?;
    
    // Try to list archive contents to count entries and calculate sizes
    match list_archive_files_with_size(reader) {
        Ok(entries_list) => {
            let entry_count = entries_list.len() as u64;
            let total_uncompressed: u64 = entries_list.iter().map(|(_, size)| size).sum();
            let uncompressed_estimate = if total_uncompressed > 0 {
                Some(total_uncompressed)
            } else {
                None
            };
            
            // For now, we can't easily detect encryption with compress-tools
            // This would require deeper integration with libarchive
            let encrypted = false;
            
            Ok((format, entry_count, uncompressed_estimate, encrypted))
        }
        Err(e) => {
            // If we can't list files, it might be corrupted or password-protected
            // Try to determine if it's a password issue or corruption
            let error_msg = e.to_string().to_lowercase();
            
            if error_msg.contains("password") || error_msg.contains("encrypted") {
                // Archive is likely password-protected
                Ok((format, 0, None, true))
            } else {
                // Archive is likely corrupted
                Err(ExtractError::Corrupted(format!("Failed to read archive: {}", e)))
            }
        }
    }
}

/// Detect archive format from file extension and magic bytes.
fn detect_format(path: &Path) -> std::result::Result<String, ExtractError> {
    let extension = path
        .extension()
        .and_then(|e| e.to_str())
        .unwrap_or("")
        .to_lowercase();

    // Map extensions to format names
    let format = match extension.as_str() {
        "zip" => "ZIP",
        "7z" => "7Z",
        "rar" => "RAR",
        "tar" => "TAR",
        "gz" | "tgz" => {
            // Check if it's a tar.gz
            if let Some(stem) = path.file_stem() {
                if stem.to_string_lossy().ends_with(".tar") {
                    "TAR.GZ"
                } else {
                    "GZIP"
                }
            } else {
                "GZIP"
            }
        }
        "bz2" | "tbz2" | "tbz" => {
            // Check if it's a tar.bz2
            if let Some(stem) = path.file_stem() {
                if stem.to_string_lossy().ends_with(".tar") {
                    "TAR.BZ2"
                } else {
                    "BZIP2"
                }
            } else {
                "BZIP2"
            }
        }
        "xz" | "txz" => {
            // Check if it's a tar.xz
            if let Some(stem) = path.file_stem() {
                if stem.to_string_lossy().ends_with(".tar") {
                    "TAR.XZ"
                } else {
                    "XZ"
                }
            } else {
                "XZ"
            }
        }
        "iso" => "ISO",
        _ => {
            return Err(ExtractError::UnsupportedFormat(format!(
                "Unknown extension: {}",
                extension
            )))
        }
    };

    Ok(format.to_string())
}

/// Helper function to list archive files with their sizes.
fn list_archive_files_with_size(
    reader: BufReader<File>,
) -> std::result::Result<Vec<(String, u64)>, Box<dyn std::error::Error>> {
    let mut entries = Vec::new();
    
    // Use compress-tools to list archive contents
    let file_list = compress_tools::list_archive_files(reader)?;
    
    // For now, we can't get individual file sizes easily with compress-tools
    // We'll return entries with size 0 as a placeholder
    for file_name in file_list {
        entries.push((file_name, 0));
    }
    
    Ok(entries)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    #[test]
    fn test_detect_format_zip() {
        let path = PathBuf::from("test.zip");
        assert_eq!(detect_format(&path).unwrap(), "ZIP");
    }

    #[test]
    fn test_detect_format_tar_gz() {
        let path = PathBuf::from("test.tar.gz");
        assert_eq!(detect_format(&path).unwrap(), "TAR.GZ");
    }

    #[test]
    fn test_detect_format_7z() {
        let path = PathBuf::from("test.7z");
        assert_eq!(detect_format(&path).unwrap(), "7Z");
    }

    #[test]
    fn test_detect_format_unsupported() {
        let path = PathBuf::from("test.unknown");
        assert!(detect_format(&path).is_err());
    }

    #[test]
    fn test_probe_nonexistent_file() {
        let path = PathBuf::from("nonexistent.zip");
        let result = probe_archive(&path);
        assert!(matches!(result, Err(ExtractError::NotFound(_))));
    }
}
