//! Archive extraction implementation with security features.

use crate::error::ExtractError;
use crate::safety::validate_entry_path;
use crate::types::{ExtractOptions, ExtractStats, OverwriteMode};
use crate::ProgressCallback;
use bzip2::read::BzDecoder;
use flate2::read::GzDecoder;
use lzma_rs::xz_decompress;
use std::fs::{self, File};
use std::io::{self, Read};
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::time::Instant;

/// Extract an archive to the specified output directory.
///
/// This function performs secure extraction with the following features:
/// - Path validation to prevent zip-slip attacks
/// - Size limit enforcement
/// - Progress tracking with cancellation support
/// - Configurable overwrite modes
/// - Strip leading path components
/// - Password support for encrypted archives
///
/// # Arguments
///
/// * `archive_path` - Path to the archive file
/// * `output_dir` - Directory where files will be extracted
/// * `options` - Extraction options
/// * `progress_cb` - Callback for progress updates
/// * `cancel_flag` - Atomic flag to signal cancellation
///
/// # Returns
///
/// Returns `ExtractStats` with extraction statistics on success.
pub fn extract_archive(
    archive_path: &Path,
    output_dir: &Path,
    options: &ExtractOptions,
    progress_cb: &ProgressCallback,
    cancel_flag: Arc<AtomicBool>,
) -> Result<ExtractStats, ExtractError> {
    let start_time = Instant::now();

    // Verify archive exists
    if !archive_path.exists() {
        return Err(ExtractError::NotFound(archive_path.to_path_buf()));
    }

    // For RAR multi-part archives, we need to use the first part
    let actual_archive_path = if is_rar_archive(archive_path) {
        use unrar::Archive;
        let temp_archive = Archive::new(archive_path);
        // as_first_part() returns an Archive pointing to the first part
        // We need to get the path from it using the filename() method
        let first_part_archive = temp_archive.as_first_part();
        PathBuf::from(first_part_archive.filename())
    } else {
        archive_path.to_path_buf()
    };

    // Create output directory if it doesn't exist
    fs::create_dir_all(output_dir)?;

    // Track extraction statistics
    let mut stats = ExtractStats {
        files_extracted: 0,
        bytes_written: 0,
        duration: std::time::Duration::from_secs(0),
        cancelled: false,
    };

    // Check for unsupported multi-part archives
    if is_multipart_archive(archive_path) && !is_rar_archive(archive_path) {
        return Err(ExtractError::UnsupportedFormat(
            "Multi-part 7-Zip and ZIP archives are not currently supported. Please use the first part (.001) or combine the parts using an external tool.".to_string()
        ));
    }

    // Detect format
    let format = crate::probe::detect_format(&actual_archive_path)?;
    
    // Use appropriate extraction method based on archive type
    let result = match format.as_str() {
        "ZIP" => extract_zip_archive(
            &actual_archive_path,
            output_dir,
            options,
            progress_cb,
            cancel_flag.clone(),
            &mut stats,
        ),
        "TAR" | "TAR.GZ" | "TAR.BZ2" | "TAR.XZ" => extract_tar_archive(
            &actual_archive_path,
            output_dir,
            options,
            progress_cb,
            cancel_flag.clone(),
            &mut stats,
            &format,
        ),
        "GZIP" | "BZIP2" | "XZ" => extract_compressed_file(
            &actual_archive_path,
            output_dir,
            options,
            progress_cb,
            cancel_flag.clone(),
            &mut stats,
            &format,
        ),
        "7Z" => extract_7z_archive(
            &actual_archive_path,
            output_dir,
            options,
            progress_cb,
            cancel_flag.clone(),
            &mut stats,
        ),
        "RAR" => extract_rar_archive(
            &actual_archive_path,
            output_dir,
            options,
            progress_cb,
            cancel_flag.clone(),
            &mut stats,
        ),
        _ => Err(ExtractError::UnsupportedFormat(format)),
    };

    // Check if cancelled
    if cancel_flag.load(Ordering::Relaxed) {
        stats.cancelled = true;
        stats.duration = start_time.elapsed();
        return Err(ExtractError::Cancelled);
    }

    // Handle extraction result
    result?;

    stats.duration = start_time.elapsed();
    Ok(stats)
}

/// Extract ZIP archive using zip crate.
fn extract_zip_archive(
    archive_path: &Path,
    output_dir: &Path,
    options: &ExtractOptions,
    progress_cb: &ProgressCallback,
    cancel_flag: Arc<AtomicBool>,
    stats: &mut ExtractStats,
) -> Result<(), ExtractError> {
    let file = File::open(archive_path)?;
    let mut archive = zip::ZipArchive::new(file).map_err(|e| {
        if e.to_string().contains("password") || e.to_string().contains("encrypted") {
            if options.password.is_some() {
                ExtractError::InvalidPassword
            } else {
                ExtractError::PasswordRequired
            }
        } else {
            ExtractError::Corrupted(e.to_string())
        }
    })?;

    for i in 0..archive.len() {
        // Check cancellation
        if cancel_flag.load(Ordering::Relaxed) {
            return Err(ExtractError::Cancelled);
        }

        let mut file = match archive.by_index(i) {
            Ok(f) => f,
            Err(e) => {
                let err_str = e.to_string();
                if err_str.contains("password") || err_str.contains("encrypted") {
                    if options.password.is_some() {
                        return Err(ExtractError::InvalidPassword);
                    } else {
                        return Err(ExtractError::PasswordRequired);
                    }
                }
                return Err(ExtractError::Corrupted(err_str));
            }
        };

        let entry_path = file.enclosed_name().ok_or_else(|| {
            ExtractError::Security(crate::error::SecurityError::PathTraversal(file.name().to_string()))
        })?;

        // Validate and strip path components
        let validated_path = validate_entry_path(&entry_path)?;
        let final_path = strip_path_components(&validated_path, options.strip_components);

        if final_path.as_os_str().is_empty() {
            continue;
        }

        let output_path = output_dir.join(&final_path);

        if file.is_dir() {
            fs::create_dir_all(&output_path)?;
        } else {
            // Create parent directories
            if let Some(parent) = output_path.parent() {
                fs::create_dir_all(parent)?;
            }

            // Check size limits
            let file_size = file.size();
            let new_total = stats.bytes_written + file_size;
            if let Some(limit) = options.size_limit_bytes {
                if new_total > limit {
                    return Err(ExtractError::SizeLimitExceeded {
                        current: new_total,
                        limit,
                    });
                }
            }

            // Handle overwrite mode
            let actual_output_path = handle_overwrite_mode(&output_path, options.overwrite)?;

            if options.overwrite == OverwriteMode::Skip && actual_output_path.exists() {
                continue;
            }

            // Extract file
            let mut outfile = File::create(&actual_output_path)?;
            io::copy(&mut file, &mut outfile)?;

            // Update stats
            stats.bytes_written += file_size;
            stats.files_extracted += 1;

            // Progress callback
            let continue_extraction = progress_cb(
                &final_path.to_string_lossy(),
                stats.bytes_written,
                Some(file_size),
            );

            if !continue_extraction {
                return Err(ExtractError::Cancelled);
            }
        }
    }

    Ok(())
}

/// Extract a single compressed file (gz, bz2, xz) - not a tar archive.
fn extract_compressed_file(
    archive_path: &Path,
    output_dir: &Path,
    options: &ExtractOptions,
    progress_cb: &ProgressCallback,
    cancel_flag: Arc<AtomicBool>,
    stats: &mut ExtractStats,
    format: &str,
) -> Result<(), ExtractError> {
    // Check cancellation
    if cancel_flag.load(Ordering::Relaxed) {
        return Err(ExtractError::Cancelled);
    }

    let file = File::open(archive_path)?;
    
    // Determine output filename by removing the compression extension
    let output_filename = archive_path
        .file_stem()
        .ok_or_else(|| ExtractError::Corrupted("Invalid filename".to_string()))?;
    
    let output_path = output_dir.join(output_filename);
    
    // Handle overwrite mode
    let actual_output_path = handle_overwrite_mode(&output_path, options.overwrite)?;
    
    if options.overwrite == OverwriteMode::Skip && actual_output_path.exists() {
        return Ok(());
    }
    
    // Create parent directories
    if let Some(parent) = actual_output_path.parent() {
        fs::create_dir_all(parent)?;
    }
    
    // Decompress based on format
    let mut reader: Box<dyn Read> = match format {
        "GZIP" => Box::new(GzDecoder::new(file)),
        "BZIP2" => Box::new(BzDecoder::new(file)),
        "XZ" => {
            // lzma-rs requires decompressing to memory first
            let mut compressed = Vec::new();
            let mut file = file;
            file.read_to_end(&mut compressed)?;
            let mut decompressed = Vec::new();
            xz_decompress(&mut compressed.as_slice(), &mut decompressed)
                .map_err(|e| ExtractError::Corrupted(format!("XZ decompression failed: {}", e)))?;
            Box::new(std::io::Cursor::new(decompressed))
        }
        _ => return Err(ExtractError::UnsupportedFormat(format.to_string())),
    };
    
    // Write decompressed data to output file
    let mut outfile = File::create(&actual_output_path)?;
    let bytes_written = io::copy(&mut reader, &mut outfile)?;
    
    // Check size limits
    if let Some(limit) = options.size_limit_bytes {
        if bytes_written > limit {
            // Clean up the file we just created
            let _ = fs::remove_file(&actual_output_path);
            return Err(ExtractError::SizeLimitExceeded {
                current: bytes_written,
                limit,
            });
        }
    }
    
    // Update stats
    stats.bytes_written = bytes_written;
    stats.files_extracted = 1;
    
    // Progress callback
    let continue_extraction = progress_cb(
        &output_filename.to_string_lossy(),
        bytes_written,
        Some(bytes_written),
    );
    
    if !continue_extraction {
        return Err(ExtractError::Cancelled);
    }
    
    Ok(())
}

/// Extract TAR archive (with optional compression) using tar crate.
fn extract_tar_archive(
    archive_path: &Path,
    output_dir: &Path,
    options: &ExtractOptions,
    progress_cb: &ProgressCallback,
    cancel_flag: Arc<AtomicBool>,
    stats: &mut ExtractStats,
    format: &str,
) -> Result<(), ExtractError> {
    let file = File::open(archive_path)?;

    // Create appropriate decompressor based on format
    let reader: Box<dyn Read> = match format {
        "TAR.GZ" => Box::new(GzDecoder::new(file)),
        "TAR.BZ2" => Box::new(BzDecoder::new(file)),
        "TAR.XZ" => {
            // lzma-rs requires decompressing to memory first
            let mut compressed = Vec::new();
            let mut file = file;
            file.read_to_end(&mut compressed)?;
            let mut decompressed = Vec::new();
            xz_decompress(&mut compressed.as_slice(), &mut decompressed)
                .map_err(|e| ExtractError::Corrupted(format!("XZ decompression failed: {}", e)))?;
            Box::new(std::io::Cursor::new(decompressed))
        }
        _ => Box::new(file),
    };

    let mut archive = tar::Archive::new(reader);

    for entry_result in archive.entries()? {
        // Check cancellation
        if cancel_flag.load(Ordering::Relaxed) {
            return Err(ExtractError::Cancelled);
        }

        let mut entry = entry_result?;
        let entry_path = entry.path()?.to_path_buf();

        // Validate and strip path components
        let validated_path = validate_entry_path(&entry_path)?;
        let final_path = strip_path_components(&validated_path, options.strip_components);

        if final_path.as_os_str().is_empty() {
            continue;
        }

        let output_path = output_dir.join(&final_path);

        if entry.header().entry_type().is_dir() {
            fs::create_dir_all(&output_path)?;
        } else {
            // Create parent directories
            if let Some(parent) = output_path.parent() {
                fs::create_dir_all(parent)?;
            }

            // Check size limits
            let file_size = entry.header().size()?;
            let new_total = stats.bytes_written + file_size;
            if let Some(limit) = options.size_limit_bytes {
                if new_total > limit {
                    return Err(ExtractError::SizeLimitExceeded {
                        current: new_total,
                        limit,
                    });
                }
            }

            // Handle overwrite mode
            let actual_output_path = handle_overwrite_mode(&output_path, options.overwrite)?;

            if options.overwrite == OverwriteMode::Skip && actual_output_path.exists() {
                continue;
            }

            // Extract file
            let mut outfile = File::create(&actual_output_path)?;
            io::copy(&mut entry, &mut outfile)?;

            // Update stats
            stats.bytes_written += file_size;
            stats.files_extracted += 1;

            // Progress callback
            let continue_extraction = progress_cb(
                &final_path.to_string_lossy(),
                stats.bytes_written,
                Some(file_size),
            );

            if !continue_extraction {
                return Err(ExtractError::Cancelled);
            }
        }
    }

    Ok(())
}

/// Extract 7Z archive using sevenz-rust2 crate.
fn extract_7z_archive(
    archive_path: &Path,
    output_dir: &Path,
    options: &ExtractOptions,
    progress_cb: &ProgressCallback,
    cancel_flag: Arc<AtomicBool>,
    stats: &mut ExtractStats,
) -> Result<(), ExtractError> {
    // sevenz-rust2 extracts directly to output directory
    // We need to validate paths after extraction
    let temp_dir = tempfile::tempdir()?;
    
    // Extract to temp directory first
    sevenz_rust2::decompress_file(archive_path, temp_dir.path())
        .map_err(|e| {
            let err_msg = e.to_string();
            if err_msg.contains("password") || err_msg.contains("encrypted") {
                if options.password.is_some() {
                    ExtractError::InvalidPassword
                } else {
                    ExtractError::PasswordRequired
                }
            } else {
                ExtractError::Corrupted(err_msg)
            }
        })?;

    // Walk through extracted files and move them with validation
    for entry in walkdir::WalkDir::new(temp_dir.path()) {
        // Check cancellation
        if cancel_flag.load(Ordering::Relaxed) {
            return Err(ExtractError::Cancelled);
        }

        let entry = entry.map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e))?;
        let temp_path = entry.path();
        
        // Get relative path from temp dir
        let relative_path = temp_path.strip_prefix(temp_dir.path())
            .map_err(|_| ExtractError::Security(crate::error::SecurityError::PathTraversal(temp_path.display().to_string())))?;

        if relative_path.as_os_str().is_empty() {
            continue;
        }

        // Validate and strip path components
        let validated_path = validate_entry_path(relative_path)?;
        let final_path = strip_path_components(&validated_path, options.strip_components);

        if final_path.as_os_str().is_empty() {
            continue;
        }

        let output_path = output_dir.join(&final_path);

        if entry.file_type().is_dir() {
            fs::create_dir_all(&output_path)?;
        } else {
            // Create parent directories
            if let Some(parent) = output_path.parent() {
                fs::create_dir_all(parent)?;
            }

            // Check size limits
            let file_size = entry.metadata().map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e))?.len();
            let new_total = stats.bytes_written + file_size;
            if let Some(limit) = options.size_limit_bytes {
                if new_total > limit {
                    return Err(ExtractError::SizeLimitExceeded {
                        current: new_total,
                        limit,
                    });
                }
            }

            // Handle overwrite mode
            let actual_output_path = handle_overwrite_mode(&output_path, options.overwrite)?;

            if options.overwrite == OverwriteMode::Skip && actual_output_path.exists() {
                continue;
            }

            // Copy file
            fs::copy(temp_path, &actual_output_path)?;

            // Update stats
            stats.bytes_written += file_size;
            stats.files_extracted += 1;

            // Progress callback
            let continue_extraction = progress_cb(
                &final_path.to_string_lossy(),
                stats.bytes_written,
                Some(file_size),
            );

            if !continue_extraction {
                return Err(ExtractError::Cancelled);
            }
        }
    }

    Ok(())
}

/// Extract RAR archive using unrar library (supports multi-part archives).
fn extract_rar_archive(
    archive_path: &Path,
    output_dir: &Path,
    options: &ExtractOptions,
    progress_cb: &ProgressCallback,
    cancel_flag: Arc<AtomicBool>,
    stats: &mut ExtractStats,
) -> Result<(), ExtractError> {
    use unrar::Archive;

    // Create archive instance (will automatically handle multi-part)
    let mut archive = if let Some(password) = &options.password {
        Archive::with_password(archive_path, password.as_bytes())
    } else {
        Archive::new(archive_path)
    };

    // Use as_first_part to ensure we start from the first part
    archive = archive.as_first_part();

    // Open for processing
    let open_archive = archive.open_for_processing().map_err(|e| {
        let err_msg = e.to_string().to_lowercase();
        if err_msg.contains("password") || err_msg.contains("encrypted") {
            if options.password.is_none() {
                ExtractError::PasswordRequired
            } else {
                ExtractError::InvalidPassword
            }
        } else if err_msg.contains("corrupt") || err_msg.contains("bad") {
            ExtractError::Corrupted(e.to_string())
        } else {
            ExtractError::Io(std::io::Error::new(std::io::ErrorKind::Other, e))
        }
    })?;

    let mut current = Some(open_archive);

    while let Some(arch) = current {
        // Check cancellation
        if cancel_flag.load(Ordering::Relaxed) {
            return Err(ExtractError::Cancelled);
        }

        match arch.read_header() {
            Ok(Some(header)) => {
                let entry = header.entry();
                let entry_filename = entry.filename.to_string_lossy().to_string();
                let entry_path = Path::new(&entry_filename);
                let is_directory = entry.is_directory();
                let unpacked_size = entry.unpacked_size;

                // Validate the entry path
                let validated_path = match validate_entry_path(entry_path) {
                    Ok(p) => p,
                    Err(_) => {
                        // Skip invalid paths
                        current = Some(header.skip().map_err(|e| {
                            ExtractError::Io(std::io::Error::new(std::io::ErrorKind::Other, e))
                        })?);
                        continue;
                    }
                };

                // Apply strip_components
                let final_path = strip_path_components(&validated_path, options.strip_components);

                // Skip if path becomes empty after stripping
                if final_path.as_os_str().is_empty() {
                    current = Some(header.skip().map_err(|e| {
                        ExtractError::Io(std::io::Error::new(std::io::ErrorKind::Other, e))
                    })?);
                    continue;
                }

                let output_path = output_dir.join(&final_path);

                // Handle overwrite mode
                let actual_output_path = handle_overwrite_mode(&output_path, options.overwrite)?;

                // Skip if file exists and mode is Skip
                if options.overwrite == OverwriteMode::Skip && actual_output_path.exists() {
                    current = Some(header.skip().map_err(|e| {
                        ExtractError::Io(std::io::Error::new(std::io::ErrorKind::Other, e))
                    })?);
                    continue;
                }

                // Create parent directories
                if let Some(parent) = actual_output_path.parent() {
                    fs::create_dir_all(parent)?;
                }

                // Extract the entry
                if is_directory {
                    fs::create_dir_all(&actual_output_path)?;
                    current = Some(header.skip().map_err(|e| {
                        ExtractError::Io(std::io::Error::new(std::io::ErrorKind::Other, e))
                    })?);
                } else {
                    // Extract file
                    current = Some(header.extract_to(&actual_output_path).map_err(|e| {
                        ExtractError::Io(std::io::Error::new(std::io::ErrorKind::Other, e))
                    })?);

                    // Update stats
                    stats.files_extracted += 1;
                    stats.bytes_written += unpacked_size;

                    // Call progress callback
                    let continue_extraction =
                        progress_cb(&final_path.to_string_lossy(), stats.bytes_written, None);

                    if !continue_extraction {
                        return Err(ExtractError::Cancelled);
                    }
                }
            }
            Ok(None) => {
                // End of archive
                current = None;
            }
            Err(e) => {
                return Err(ExtractError::Io(std::io::Error::new(
                    std::io::ErrorKind::Other,
                    e,
                )));
            }
        }
    }

    Ok(())
}

/// Check if a file is a multi-part archive (any format).
fn is_multipart_archive(path: &Path) -> bool {
    let extension = path
        .extension()
        .and_then(|e| e.to_str())
        .unwrap_or("")
        .to_lowercase();

    let filename = path
        .file_name()
        .and_then(|f| f.to_str())
        .unwrap_or("")
        .to_lowercase();

    // Check for .partXX.rar or .partXX.zip pattern
    if filename.contains(".part") && (filename.ends_with(".rar") || filename.ends_with(".zip")) {
        return true;
    }

    // Check for .7z.XXX or .zip.XXX pattern
    if (filename.contains(".7z.") || filename.contains(".zip."))
        && extension.chars().all(|c| c.is_ascii_digit())
    {
        return true;
    }

    // Check for .rXX extensions (RAR)
    if extension.starts_with('r') && extension.len() >= 2 {
        if extension[1..].chars().all(|c| c.is_ascii_digit()) {
            return true;
        }
    }

    false
}

/// Check if a file is a RAR archive based on extension.
fn is_rar_archive(path: &Path) -> bool {
    let extension = path
        .extension()
        .and_then(|e| e.to_str())
        .unwrap_or("")
        .to_lowercase();

    let filename = path
        .file_name()
        .and_then(|f| f.to_str())
        .unwrap_or("")
        .to_lowercase();

    // Check for .rar extension
    if extension == "rar" {
        return true;
    }

    // Check for multi-part RAR (.part1.rar, .r00, etc.)
    if filename.contains(".part") && filename.ends_with(".rar") {
        return true;
    }

    // Check for .rXX extensions
    if extension.starts_with('r') && extension.len() >= 2 {
        if extension[1..].chars().all(|c| c.is_ascii_digit()) {
            return true;
        }
    }

    false
}

/// Strip leading path components from a path.
fn strip_path_components(path: &Path, count: u32) -> PathBuf {
    if count == 0 {
        return path.to_path_buf();
    }

    let components: Vec<_> = path.components().collect();
    let skip = count as usize;

    if skip >= components.len() {
        return PathBuf::new();
    }

    components[skip..].iter().collect()
}

/// Handle file overwrite based on the configured mode.
fn handle_overwrite_mode(path: &Path, mode: OverwriteMode) -> Result<PathBuf, ExtractError> {
    match mode {
        OverwriteMode::Replace => {
            // Always use the original path, will overwrite
            Ok(path.to_path_buf())
        }
        OverwriteMode::Skip => {
            // If file exists, return error to skip
            if path.exists() {
                // We'll handle this by returning the same path but checking later
                Ok(path.to_path_buf())
            } else {
                Ok(path.to_path_buf())
            }
        }
        OverwriteMode::Rename => {
            // If file exists, find a unique name
            if !path.exists() {
                return Ok(path.to_path_buf());
            }

            let parent = path.parent().unwrap_or(Path::new(""));
            let file_stem = path.file_stem().and_then(|s| s.to_str()).unwrap_or("file");
            let extension = path.extension().and_then(|s| s.to_str());

            // Try appending (1), (2), etc.
            for i in 1..1000 {
                let new_name = if let Some(ext) = extension {
                    format!("{} ({}).{}", file_stem, i, ext)
                } else {
                    format!("{} ({})", file_stem, i)
                };

                let new_path = parent.join(new_name);
                if !new_path.exists() {
                    return Ok(new_path);
                }
            }

            // If we couldn't find a unique name after 1000 tries, error out
            Err(ExtractError::Io(io::Error::new(
                io::ErrorKind::AlreadyExists,
                "Could not find unique filename",
            )))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_strip_path_components() {
        // No stripping
        let path = Path::new("a/b/c/file.txt");
        assert_eq!(strip_path_components(path, 0), path);

        // Strip 1 component
        assert_eq!(strip_path_components(path, 1), Path::new("b/c/file.txt"));

        // Strip 2 components
        assert_eq!(strip_path_components(path, 2), Path::new("c/file.txt"));

        // Strip 3 components
        assert_eq!(strip_path_components(path, 3), Path::new("file.txt"));

        // Strip more than available
        assert_eq!(strip_path_components(path, 4), PathBuf::new());
        assert_eq!(strip_path_components(path, 10), PathBuf::new());
    }

    #[test]
    fn test_handle_overwrite_mode_replace() {
        let path = Path::new("/tmp/test_file.txt");
        let result = handle_overwrite_mode(path, OverwriteMode::Replace);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), path);
    }

    #[test]
    fn test_handle_overwrite_mode_rename() {
        use std::fs;
        use tempfile::TempDir;

        let temp_dir = TempDir::new().unwrap();
        let file_path = temp_dir.path().join("test.txt");

        // Create the original file
        fs::write(&file_path, "content").unwrap();

        // First rename should give us "test (1).txt"
        let result = handle_overwrite_mode(&file_path, OverwriteMode::Rename);
        assert!(result.is_ok());
        let renamed = result.unwrap();
        assert_eq!(renamed, temp_dir.path().join("test (1).txt"));

        // Create that file too
        fs::write(&renamed, "content").unwrap();

        // Second rename should give us "test (2).txt"
        let result = handle_overwrite_mode(&file_path, OverwriteMode::Rename);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), temp_dir.path().join("test (2).txt"));
    }
}
