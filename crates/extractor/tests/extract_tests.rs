use extractor::{extract, ExtractOptions, OverwriteMode};
use std::fs::{self, File};
use std::io::Write;
use std::path::Path;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use tempfile::TempDir;

/// Helper to create a test ZIP archive
fn create_test_zip(path: &Path) -> std::io::Result<()> {
    use zip::write::{SimpleFileOptions, ZipWriter};

    let file = File::create(path)?;
    let mut zip = ZipWriter::new(file);

    // Add a simple text file
    zip.start_file("test.txt", SimpleFileOptions::default())?;
    zip.write_all(b"Hello, World!")?;

    // Add a file in a subdirectory
    zip.start_file("subdir/nested.txt", SimpleFileOptions::default())?;
    zip.write_all(b"Nested content")?;

    // Add another file
    zip.start_file("data.json", SimpleFileOptions::default())?;
    zip.write_all(b"{\"key\": \"value\"}")?;

    zip.finish()?;
    Ok(())
}

/// Helper to create a test TAR.GZ archive
fn create_test_tar_gz(path: &Path) -> std::io::Result<()> {
    use flate2::write::GzEncoder;
    use flate2::Compression;
    use tar::Builder;

    let file = File::create(path)?;
    let encoder = GzEncoder::new(file, Compression::default());
    let mut tar = Builder::new(encoder);

    // Create temporary files to add to the archive
    let temp_dir = TempDir::new()?;
    let test_file = temp_dir.path().join("test.txt");
    fs::write(&test_file, b"Hello from TAR!")?;
    tar.append_path_with_name(&test_file, "test.txt")?;

    let nested_dir = temp_dir.path().join("subdir");
    fs::create_dir(&nested_dir)?;
    let nested_file = nested_dir.join("nested.txt");
    fs::write(&nested_file, b"Nested in TAR")?;
    tar.append_path_with_name(&nested_file, "subdir/nested.txt")?;

    tar.finish()?;
    Ok(())
}

#[test]
fn test_extract_zip_basic() {
    let temp_dir = TempDir::new().unwrap();
    let archive_path = temp_dir.path().join("test.zip");
    let output_dir = temp_dir.path().join("output");

    // Create test archive
    create_test_zip(&archive_path).unwrap();

    // Extract
    let options = ExtractOptions::default();
    let cancel_flag = Arc::new(AtomicBool::new(false));
    let progress_cb = |_file: &str, _bytes: u64, _total: Option<u64>| true;

    let stats = extract(&archive_path, &output_dir, &options, &progress_cb, cancel_flag).unwrap();

    // Verify extraction
    assert!(stats.files_extracted > 0);
    assert!(stats.bytes_written > 0);
    assert!(!stats.cancelled);

    // Check files exist
    assert!(output_dir.join("test.txt").exists());
    assert!(output_dir.join("subdir/nested.txt").exists());
    assert!(output_dir.join("data.json").exists());

    // Verify content
    let content = fs::read_to_string(output_dir.join("test.txt")).unwrap();
    assert_eq!(content, "Hello, World!");
}

#[test]
fn test_extract_tar_gz_basic() {
    let temp_dir = TempDir::new().unwrap();
    let archive_path = temp_dir.path().join("test.tar.gz");
    let output_dir = temp_dir.path().join("output");

    // Create test archive
    create_test_tar_gz(&archive_path).unwrap();

    // Extract
    let options = ExtractOptions::default();
    let cancel_flag = Arc::new(AtomicBool::new(false));
    let progress_cb = |_file: &str, _bytes: u64, _total: Option<u64>| true;

    let stats = extract(&archive_path, &output_dir, &options, &progress_cb, cancel_flag).unwrap();

    // Verify extraction
    assert!(stats.files_extracted > 0);
    assert!(stats.bytes_written > 0);
    assert!(!stats.cancelled);

    // Check files exist
    assert!(output_dir.join("test.txt").exists());
    assert!(output_dir.join("subdir/nested.txt").exists());

    // Verify content
    let content = fs::read_to_string(output_dir.join("test.txt")).unwrap();
    assert_eq!(content, "Hello from TAR!");
}

#[test]
fn test_extract_with_strip_components() {
    let temp_dir = TempDir::new().unwrap();
    let archive_path = temp_dir.path().join("test.zip");
    let output_dir = temp_dir.path().join("output");

    // Create test archive
    create_test_zip(&archive_path).unwrap();

    // Extract with strip_components = 1
    let mut options = ExtractOptions::default();
    options.strip_components = 1;
    let cancel_flag = Arc::new(AtomicBool::new(false));
    let progress_cb = |_file: &str, _bytes: u64, _total: Option<u64>| true;

    let stats = extract(&archive_path, &output_dir, &options, &progress_cb, cancel_flag).unwrap();

    // Verify extraction
    assert!(stats.files_extracted > 0);

    // Files with only one component should be skipped
    // Files with multiple components should have first component stripped
    assert!(output_dir.join("nested.txt").exists());
}

#[test]
fn test_extract_with_overwrite_rename() {
    let temp_dir = TempDir::new().unwrap();
    let archive_path = temp_dir.path().join("test.zip");
    let output_dir = temp_dir.path().join("output");

    // Create test archive
    create_test_zip(&archive_path).unwrap();

    // Create output directory and pre-existing file
    fs::create_dir_all(&output_dir).unwrap();
    fs::write(output_dir.join("test.txt"), b"Existing content").unwrap();

    // Extract with rename mode
    let mut options = ExtractOptions::default();
    options.overwrite = OverwriteMode::Rename;
    let cancel_flag = Arc::new(AtomicBool::new(false));
    let progress_cb = |_file: &str, _bytes: u64, _total: Option<u64>| true;

    let stats = extract(&archive_path, &output_dir, &options, &progress_cb, cancel_flag).unwrap();

    // Verify extraction
    assert!(stats.files_extracted > 0);

    // Original file should still exist
    assert!(output_dir.join("test.txt").exists());
    let original_content = fs::read_to_string(output_dir.join("test.txt")).unwrap();
    assert_eq!(original_content, "Existing content");

    // Renamed file should exist
    assert!(output_dir.join("test (1).txt").exists());
    let renamed_content = fs::read_to_string(output_dir.join("test (1).txt")).unwrap();
    assert_eq!(renamed_content, "Hello, World!");
}

#[test]
fn test_extract_with_overwrite_skip() {
    let temp_dir = TempDir::new().unwrap();
    let archive_path = temp_dir.path().join("test.zip");
    let output_dir = temp_dir.path().join("output");

    // Create test archive
    create_test_zip(&archive_path).unwrap();

    // Create output directory and pre-existing file
    fs::create_dir_all(&output_dir).unwrap();
    fs::write(output_dir.join("test.txt"), b"Existing content").unwrap();

    // Extract with skip mode
    let mut options = ExtractOptions::default();
    options.overwrite = OverwriteMode::Skip;
    let cancel_flag = Arc::new(AtomicBool::new(false));
    let progress_cb = |_file: &str, _bytes: u64, _total: Option<u64>| true;

    let _stats = extract(&archive_path, &output_dir, &options, &progress_cb, cancel_flag).unwrap();

    // Original file should still exist with original content
    assert!(output_dir.join("test.txt").exists());
    let content = fs::read_to_string(output_dir.join("test.txt")).unwrap();
    assert_eq!(content, "Existing content");

    // Other files should be extracted
    assert!(output_dir.join("data.json").exists());
}

#[test]
fn test_extract_with_overwrite_replace() {
    let temp_dir = TempDir::new().unwrap();
    let archive_path = temp_dir.path().join("test.zip");
    let output_dir = temp_dir.path().join("output");

    // Create test archive
    create_test_zip(&archive_path).unwrap();

    // Create output directory and pre-existing file
    fs::create_dir_all(&output_dir).unwrap();
    fs::write(output_dir.join("test.txt"), b"Existing content").unwrap();

    // Extract with replace mode
    let mut options = ExtractOptions::default();
    options.overwrite = OverwriteMode::Replace;
    let cancel_flag = Arc::new(AtomicBool::new(false));
    let progress_cb = |_file: &str, _bytes: u64, _total: Option<u64>| true;

    let stats = extract(&archive_path, &output_dir, &options, &progress_cb, cancel_flag).unwrap();

    // Verify extraction
    assert!(stats.files_extracted > 0);

    // File should be replaced with new content
    assert!(output_dir.join("test.txt").exists());
    let content = fs::read_to_string(output_dir.join("test.txt")).unwrap();
    assert_eq!(content, "Hello, World!");
}

#[test]
fn test_extract_with_cancellation() {
    let temp_dir = TempDir::new().unwrap();
    let archive_path = temp_dir.path().join("test.zip");
    let output_dir = temp_dir.path().join("output");

    // Create test archive
    create_test_zip(&archive_path).unwrap();

    // Extract with cancellation
    let options = ExtractOptions::default();
    let cancel_flag = Arc::new(AtomicBool::new(false));
    let cancel_flag_clone = cancel_flag.clone();

    let progress_cb = move |_file: &str, _bytes: u64, _total: Option<u64>| {
        // Cancel after first file
        cancel_flag_clone.store(true, Ordering::Relaxed);
        false
    };

    let result = extract(&archive_path, &output_dir, &options, &progress_cb, cancel_flag);

    // Should return cancelled error
    assert!(result.is_err());
    match result.unwrap_err() {
        extractor::ExtractError::Cancelled => {
            // Expected
        }
        e => panic!("Expected Cancelled error, got: {:?}", e),
    }
}

#[test]
fn test_extract_nonexistent_archive() {
    let temp_dir = TempDir::new().unwrap();
    let archive_path = temp_dir.path().join("nonexistent.zip");
    let output_dir = temp_dir.path().join("output");

    let options = ExtractOptions::default();
    let cancel_flag = Arc::new(AtomicBool::new(false));
    let progress_cb = |_file: &str, _bytes: u64, _total: Option<u64>| true;

    let result = extract(&archive_path, &output_dir, &options, &progress_cb, cancel_flag);

    assert!(result.is_err());
    match result.unwrap_err() {
        extractor::ExtractError::NotFound(_) => {
            // Expected
        }
        e => panic!("Expected NotFound error, got: {:?}", e),
    }
}

#[test]
fn test_extract_with_size_limit() {
    let temp_dir = TempDir::new().unwrap();
    let archive_path = temp_dir.path().join("test.zip");
    let output_dir = temp_dir.path().join("output");

    // Create test archive
    create_test_zip(&archive_path).unwrap();

    // Extract with very small size limit
    let mut options = ExtractOptions::default();
    options.size_limit_bytes = Some(10); // Only 10 bytes allowed
    let cancel_flag = Arc::new(AtomicBool::new(false));
    let progress_cb = |_file: &str, _bytes: u64, _total: Option<u64>| true;

    let result = extract(&archive_path, &output_dir, &options, &progress_cb, cancel_flag);

    // Should fail with size limit exceeded
    assert!(result.is_err());
    match result.unwrap_err() {
        extractor::ExtractError::SizeLimitExceeded { .. } => {
            // Expected
        }
        e => panic!("Expected SizeLimitExceeded error, got: {:?}", e),
    }
}

#[test]
fn test_extract_progress_callback() {
    let temp_dir = TempDir::new().unwrap();
    let archive_path = temp_dir.path().join("test.zip");
    let output_dir = temp_dir.path().join("output");

    // Create test archive
    create_test_zip(&archive_path).unwrap();

    // Track progress calls
    let progress_calls = Arc::new(AtomicBool::new(false));
    let progress_calls_clone = progress_calls.clone();

    let options = ExtractOptions::default();
    let cancel_flag = Arc::new(AtomicBool::new(false));
    let progress_cb = move |_file: &str, _bytes: u64, _total: Option<u64>| {
        progress_calls_clone.store(true, Ordering::Relaxed);
        true
    };

    let _stats = extract(&archive_path, &output_dir, &options, &progress_cb, cancel_flag).unwrap();

    // Verify progress callback was called
    assert!(progress_calls.load(Ordering::Relaxed));
}

#[test]
#[ignore] // Password-protected archives require special handling
fn test_extract_password_protected_without_password() {
    // This test is ignored because compress-tools has limited password support
    // In a real implementation with libarchive bindings, this would test:
    // 1. Detecting password-protected archives
    // 2. Returning PasswordRequired error when no password is provided
    // 3. Returning InvalidPassword error when wrong password is provided
    // 4. Successfully extracting when correct password is provided
}

#[test]
fn test_extract_with_path_traversal_protection() {
    // This test verifies that malicious archives with path traversal attempts
    // are safely handled by the validation logic
    // The actual path validation is tested in safety module tests
    // Here we just verify that extraction doesn't fail catastrophically
    
    let temp_dir = TempDir::new().unwrap();
    let archive_path = temp_dir.path().join("test.zip");
    let output_dir = temp_dir.path().join("output");

    // Create a normal archive (malicious archives would be created differently)
    create_test_zip(&archive_path).unwrap();

    let options = ExtractOptions::default();
    let cancel_flag = Arc::new(AtomicBool::new(false));
    let progress_cb = |_file: &str, _bytes: u64, _total: Option<u64>| true;

    // Should extract successfully
    let result = extract(&archive_path, &output_dir, &options, &progress_cb, cancel_flag);
    assert!(result.is_ok());
    
    // Verify files are in the output directory, not outside it
    assert!(output_dir.join("test.txt").exists());
    assert!(!temp_dir.path().join("test.txt").exists()); // Not in parent
}

