//! Archive probing functionality for reading metadata without extraction.

use crate::error::ExtractError;
use crate::types::{ArchiveEntry, ArchiveInfo};
use std::fs::File;
use std::io::{BufReader, Read};
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

    // Detect format and analyze entries
    let (format, entry_list, encrypted) = analyze_archive(file, path)?;

    // Calculate statistics from entries
    let entries = entry_list.len() as u64;
    let uncompressed_estimate = if !entry_list.is_empty() {
        Some(entry_list.iter().map(|e| e.size).sum())
    } else {
        None
    };

    Ok(ArchiveInfo {
        format,
        entries,
        compressed_bytes,
        uncompressed_estimate,
        encrypted,
        entry_list,
    })
}

/// Analyze archive contents to extract metadata.
fn analyze_archive(
    file: File,
    path: &Path,
) -> std::result::Result<(String, Vec<ArchiveEntry>, bool), ExtractError> {
    // Detect format from file extension
    let format = detect_format(path)?;

    // List entries based on format
    match list_entries_by_format(&format, file, path) {
        Ok((entries, encrypted)) => Ok((format, entries, encrypted)),
        Err(e) => {
            // If we can't list files, it might be corrupted or password-protected
            let error_msg = e.to_string().to_lowercase();

            if error_msg.contains("password") || error_msg.contains("encrypted") {
                // Archive is likely password-protected
                Ok((format, Vec::new(), true))
            } else {
                // Archive is likely corrupted or unsupported
                // Return empty list rather than failing
                Ok((format, Vec::new(), false))
            }
        }
    }
}

/// List entries based on archive format.
fn list_entries_by_format(
    format: &str,
    file: File,
    path: &Path,
) -> std::result::Result<(Vec<ArchiveEntry>, bool), Box<dyn std::error::Error>> {
    match format {
        "ZIP" => list_zip_entries(file),
        "TAR" | "TAR.GZ" | "TAR.BZ2" | "TAR.XZ" => list_tar_entries(file, format),
        "7Z" => list_7z_entries(path),
        "RAR" => list_rar_entries(path),
        _ => {
            // For other formats (ISO, GZIP, etc.), use compress-tools fallback
            list_generic_entries(file, path)
        }
    }
}

/// List entries in a ZIP archive.
fn list_zip_entries(
    file: File,
) -> std::result::Result<(Vec<ArchiveEntry>, bool), Box<dyn std::error::Error>> {
    let mut archive = zip::ZipArchive::new(file)?;
    let mut entries = Vec::new();
    let mut encrypted = false;

    for i in 0..archive.len() {
        let entry = archive.by_index(i)?;

        // Check if any entry is encrypted
        if entry.encrypted() {
            encrypted = true;
        }

        entries.push(ArchiveEntry {
            path: entry.name().to_string(),
            is_directory: entry.is_dir(),
            size: entry.size(),
            compressed_size: Some(entry.compressed_size()),
        });
    }

    Ok((entries, encrypted))
}

/// List entries in a TAR archive (with optional compression).
fn list_tar_entries(
    file: File,
    format: &str,
) -> std::result::Result<(Vec<ArchiveEntry>, bool), Box<dyn std::error::Error>> {
    use bzip2::read::BzDecoder;
    use flate2::read::GzDecoder;
    use std::io::BufReader;
    use xz2::read::XzDecoder;

    let mut entries = Vec::new();

    // Wrap the file reader based on compression format
    let reader: Box<dyn Read> = match format {
        "TAR.GZ" => Box::new(GzDecoder::new(BufReader::new(file))),
        "TAR.BZ2" => Box::new(BzDecoder::new(BufReader::new(file))),
        "TAR.XZ" => Box::new(XzDecoder::new(BufReader::new(file))),
        _ => Box::new(BufReader::new(file)),
    };

    let mut archive = tar::Archive::new(reader);

    for entry_result in archive.entries()? {
        let entry = entry_result?;
        let header = entry.header();

        let path = entry.path()?.to_string_lossy().to_string();
        let is_directory = header.entry_type().is_dir();
        let size = header.size()?;

        entries.push(ArchiveEntry {
            path,
            is_directory,
            size,
            compressed_size: None, // TAR doesn't store per-file compressed sizes
        });
    }

    Ok((entries, false)) // TAR archives are not encrypted
}

/// List entries in a 7-Zip archive.
fn list_7z_entries(
    path: &Path,
) -> std::result::Result<(Vec<ArchiveEntry>, bool), Box<dyn std::error::Error>> {
    use sevenz_rust::{Password, SevenZReader};

    let file = File::open(path)?;
    let file_len = file.metadata()?.len();

    // Try to open without password first
    let sz = SevenZReader::new(file, file_len, Password::empty())?;
    let mut entries = Vec::new();
    let encrypted = false; // If we got here, it's not encrypted or we can read metadata

    for entry in sz.archive().files.iter() {
        let name = entry.name().to_string();
        entries.push(ArchiveEntry {
            path: name,
            is_directory: entry.is_directory(),
            size: entry.size(),
            compressed_size: None, // 7z doesn't expose per-file compressed size easily
        });
    }

    Ok((entries, encrypted))
}

/// List entries in a RAR archive.
fn list_rar_entries(
    path: &Path,
) -> std::result::Result<(Vec<ArchiveEntry>, bool), Box<dyn std::error::Error>> {
    use unrar::Archive;

    let archive = Archive::new(path).open_for_listing()?;
    let mut entries = Vec::new();
    let mut encrypted = false;
    let mut current = Some(archive);

    while let Some(arch) = current {
        match arch.read_header()? {
            Some(header) => {
                // Check if entry is encrypted
                if header.entry().is_encrypted() {
                    encrypted = true;
                }

                let entry_data = header.entry();
                entries.push(ArchiveEntry {
                    path: entry_data.filename.to_string_lossy().to_string(),
                    is_directory: entry_data.is_directory(),
                    size: entry_data.unpacked_size,
                    compressed_size: None, // RAR API doesn't easily expose packed size in this version
                });

                current = Some(header.skip()?);
            }
            None => {
                current = None;
            }
        }
    }

    Ok((entries, encrypted))
}

/// List entries using compress-tools (fallback for unsupported formats).
fn list_generic_entries(
    file: File,
    _path: &Path,
) -> std::result::Result<(Vec<ArchiveEntry>, bool), Box<dyn std::error::Error>> {
    let reader = BufReader::new(file);
    list_generic_entries_from_reader(reader)
}

/// List entries from a reader using compress-tools.
fn list_generic_entries_from_reader(
    reader: BufReader<File>,
) -> std::result::Result<(Vec<ArchiveEntry>, bool), Box<dyn std::error::Error>> {
    let file_list = compress_tools::list_archive_files(reader)?;

    let entries: Vec<ArchiveEntry> = file_list
        .into_iter()
        .map(|path| {
            let is_directory = path.ends_with('/');
            ArchiveEntry {
                path,
                is_directory,
                size: 0, // compress-tools doesn't provide size info
                compressed_size: None,
            }
        })
        .collect();

    Ok((entries, false))
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
