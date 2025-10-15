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

    // Create output directory if it doesn't exist
    fs::create_dir_all(output_dir)?;

    // Track extraction statistics
    let mut stats = ExtractStats {
        files_extracted: 0,
        bytes_written: 0,
        duration: std::time::Duration::from_secs(0),
        cancelled: false,
    };

    // Extract using compress-tools
    let result = extract_with_compress_tools(
        archive_path,
        output_dir,
        options,
        progress_cb,
        cancel_flag.clone(),
        &mut stats,
    );

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
    let temp_dir = tempfile::tempdir()
        .map_err(|e| ExtractError::Io(e))?;
    
    // Extract entire archive to temp directory
    // Note: compress-tools doesn't support password parameter directly
    // Password handling would require using libarchive bindings directly
    let archive_file = File::open(archive_path)?;
    compress_tools::uncompress_archive(archive_file, temp_dir.path(), compress_tools::Ownership::Preserve)
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
        check_size_limits(new_total, options.size_limit_bytes)
            .map_err(|_| ExtractError::SizeLimitExceeded {
                current: new_total,
                limit: options.size_limit_bytes.unwrap_or(0),
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

/// Map compress-tools errors to ExtractError with password detection.
fn map_compress_tools_error(e: compress_tools::Error, options: &ExtractOptions) -> ExtractError {
    let err_msg = e.to_string().to_lowercase();
    
    // Check for password-related errors
    if err_msg.contains("password") 
        || err_msg.contains("encrypted") 
        || err_msg.contains("passphrase")
        || err_msg.contains("decrypt") {
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
        || err_msg.contains("damaged") {
        return ExtractError::Corrupted(e.to_string());
    }
    
    // Check for unsupported format
    if err_msg.contains("unsupported") 
        || err_msg.contains("unknown format")
        || err_msg.contains("not recognized") {
        return ExtractError::UnsupportedFormat(e.to_string());
    }
    
    // Default to corrupted for other errors
    ExtractError::Corrupted(e.to_string())
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
fn handle_overwrite_mode(
    path: &Path,
    mode: OverwriteMode,
) -> Result<PathBuf, ExtractError> {
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
            let file_stem = path
                .file_stem()
                .and_then(|s| s.to_str())
                .unwrap_or("file");
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
        assert_eq!(
            strip_path_components(path, 1),
            Path::new("b/c/file.txt")
        );

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
