//! Archive extraction implementation with security features.

use crate::error::ExtractError;
use crate::safety::{check_size_limits, validate_entry_path};
use crate::types::{ExtractOptions, ExtractStats, OverwriteMode};
use crate::ProgressCallback;
use std::fs::{self, File};
use std::io;
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

    // Use appropriate extraction method based on archive type
    let result = if is_rar_archive(archive_path) {
        // Use unrar library for RAR archives (supports multi-part)
        extract_rar_archive(
            &actual_archive_path,
            output_dir,
            options,
            progress_cb,
            cancel_flag.clone(),
            &mut stats,
        )
    } else {
        // Use compress-tools for other formats
        extract_with_compress_tools(
            &actual_archive_path,
            output_dir,
            options,
            progress_cb,
            cancel_flag.clone(),
            &mut stats,
        )
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

/// Extract using compress-tools library with custom validation.
///
/// Note: Password support is limited by the compress-tools library.
/// The function will detect password-protected archives and return appropriate errors,
/// but actual password-based decryption may not work for all formats.
fn extract_with_compress_tools(
    archive_path: &Path,
    output_dir: &Path,
    options: &ExtractOptions,
    progress_cb: &ProgressCallback,
    cancel_flag: Arc<AtomicBool>,
    stats: &mut ExtractStats,
) -> Result<(), ExtractError> {
    // First, list all entries to validate them
    let archive_file = File::open(archive_path)?;
    let entries = compress_tools::list_archive_files(archive_file)
        .map_err(|e| map_compress_tools_error(e, options))?;

    // Validate all paths before extraction
    let mut validated_entries = Vec::new();
    for entry_path_str in entries {
        // Check cancellation
        if cancel_flag.load(Ordering::Relaxed) {
            return Err(ExtractError::Cancelled);
        }

        let entry_path = Path::new(&entry_path_str);

        // Validate the entry path
        let validated_path = match validate_entry_path(entry_path) {
            Ok(p) => p,
            Err(_) => {
                // Skip invalid paths but continue extraction
                continue;
            }
        };

        // Apply strip_components
        let final_path = strip_path_components(&validated_path, options.strip_components);

        // Skip if path becomes empty after stripping
        if final_path.as_os_str().is_empty() {
            continue;
        }

        validated_entries.push((entry_path_str, final_path));
    }

    // Now extract to a temporary location and move files with validation
    let temp_dir = tempfile::tempdir().map_err(|e| ExtractError::Io(e))?;

    // Extract entire archive to temp directory
    // Note: compress-tools doesn't support password parameter directly
    // Password handling would require using libarchive bindings directly
    let archive_file = File::open(archive_path)?;
    compress_tools::uncompress_archive(
        archive_file,
        temp_dir.path(),
        compress_tools::Ownership::Preserve,
    )
    .map_err(|e| map_compress_tools_error(e, options))?;

    // Move validated files from temp to output directory
    for (original_path, final_path) in validated_entries {
        // Check cancellation
        if cancel_flag.load(Ordering::Relaxed) {
            return Err(ExtractError::Cancelled);
        }

        let temp_file_path = temp_dir.path().join(&original_path);

        // Skip if file doesn't exist (might be a directory)
        if !temp_file_path.exists() {
            continue;
        }

        // Skip directories
        if temp_file_path.is_dir() {
            // Create the directory in output
            let output_path = output_dir.join(&final_path);
            fs::create_dir_all(&output_path)?;
            continue;
        }

        // Get file size
        let file_size = temp_file_path.metadata()?.len();

        // Check size limits
        let new_total = stats.bytes_written + file_size;
        check_size_limits(new_total, options.size_limit_bytes).map_err(|_| {
            ExtractError::SizeLimitExceeded {
                current: new_total,
                limit: options.size_limit_bytes.unwrap_or(0),
            }
        })?;

        // Determine output path with overwrite handling
        let output_path = output_dir.join(&final_path);

        // Create parent directories
        if let Some(parent) = output_path.parent() {
            fs::create_dir_all(parent)?;
        }

        let actual_output_path = handle_overwrite_mode(&output_path, options.overwrite)?;

        // Skip if file exists and mode is Skip
        if options.overwrite == OverwriteMode::Skip && actual_output_path.exists() {
            continue;
        }

        // Copy file from temp to output
        fs::copy(&temp_file_path, &actual_output_path)?;

        // Update statistics
        stats.bytes_written += file_size;
        stats.files_extracted += 1;

        // Call progress callback
        let continue_extraction = progress_cb(
            &final_path.to_string_lossy(),
            stats.bytes_written,
            Some(file_size),
        );

        if !continue_extraction {
            return Err(ExtractError::Cancelled);
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

/// Map compress-tools errors to ExtractError with password detection.
fn map_compress_tools_error(e: compress_tools::Error, options: &ExtractOptions) -> ExtractError {
    let err_msg = e.to_string().to_lowercase();

    // Check for password-related errors
    if err_msg.contains("password")
        || err_msg.contains("encrypted")
        || err_msg.contains("passphrase")
        || err_msg.contains("decrypt")
    {
        if options.password.is_none() {
            return ExtractError::PasswordRequired;
        } else {
            return ExtractError::InvalidPassword;
        }
    }

    // Check for corruption indicators
    if err_msg.contains("corrupt")
        || err_msg.contains("invalid")
        || err_msg.contains("malformed")
        || err_msg.contains("damaged")
    {
        return ExtractError::Corrupted(e.to_string());
    }

    // Check for unsupported format
    if err_msg.contains("unsupported")
        || err_msg.contains("unknown format")
        || err_msg.contains("not recognized")
    {
        return ExtractError::UnsupportedFormat(e.to_string());
    }

    // Default to corrupted for other errors
    ExtractError::Corrupted(e.to_string())
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
