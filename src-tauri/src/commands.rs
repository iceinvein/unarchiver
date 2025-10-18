use crate::state::{AppState, JobHandle};
use extractor::{ExtractOptions, ExtractStats, OverwriteMode};
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};
use std::sync::atomic::AtomicBool;
use std::sync::Arc;
use tauri::{AppHandle, Emitter, Manager, State};
use ts_rs::TS;
use uuid::Uuid;

/// DTO for extraction options from frontend
#[derive(Debug, Deserialize, TS)]
#[ts(export, export_to = "../../src/lib/bindings/")]
#[serde(rename_all = "camelCase")]
pub struct ExtractOptionsDTO {
    pub overwrite: String,
    #[ts(optional, type = "number")]
    pub size_limit_bytes: Option<u64>,
    #[ts(type = "number")]
    pub strip_components: u32,
    pub allow_symlinks: bool,
    pub allow_hardlinks: bool,
    #[ts(optional)]
    pub password: Option<String>,
}

impl From<ExtractOptionsDTO> for ExtractOptions {
    fn from(dto: ExtractOptionsDTO) -> Self {
        let overwrite = match dto.overwrite.as_str() {
            "replace" => OverwriteMode::Replace,
            "skip" => OverwriteMode::Skip,
            _ => OverwriteMode::Rename,
        };

        ExtractOptions {
            overwrite,
            size_limit_bytes: dto.size_limit_bytes,
            strip_components: dto.strip_components,
            allow_symlinks: dto.allow_symlinks,
            allow_hardlinks: dto.allow_hardlinks,
            password: dto.password,
        }
    }
}

/// Progress event payload
#[derive(Debug, Clone, Serialize, TS)]
#[ts(export, export_to = "../../src/lib/bindings/")]
#[serde(rename_all = "camelCase")]
pub struct ProgressEvent {
    pub job_id: String,
    pub archive_path: String,
    pub current_file: String,
    #[ts(type = "number")]
    pub bytes_written: u64,
    #[ts(optional, type = "number")]
    pub total_bytes: Option<u64>,
}

/// Completion event payload
#[derive(Debug, Clone, Serialize, TS)]
#[ts(export, export_to = "../../src/lib/bindings/")]
#[serde(rename_all = "camelCase")]
pub struct CompletionEvent {
    pub job_id: String,
    pub archive_path: String,
    pub status: JobStatus,
    #[ts(optional)]
    pub stats: Option<ExtractStats>,
    #[ts(optional)]
    pub error: Option<String>,
}

#[derive(Debug, Clone, Serialize, TS)]
#[ts(export, export_to = "../../src/lib/bindings/")]
#[serde(rename_all = "lowercase")]
pub enum JobStatus {
    Success,
    Failed,
    Cancelled,
}

/// Password required event payload
#[derive(Debug, Clone, Serialize, TS)]
#[ts(export, export_to = "../../src/lib/bindings/")]
#[serde(rename_all = "camelCase")]
pub struct PasswordRequiredEvent {
    pub job_id: String,
    pub archive_path: String,
}

/// Extract one or more archives
#[tauri::command]
pub async fn extract(
    app: AppHandle,
    state: State<'_, AppState>,
    input_paths: Vec<String>,
    out_dir: String,
    options: ExtractOptionsDTO,
) -> Result<String, String> {
    // Generate unique job ID
    let job_id = Uuid::new_v4().to_string();

    // Convert options
    let mut extract_options: ExtractOptions = options.into();
    let output_dir = PathBuf::from(out_dir);

    // Create cancel flag
    let cancel_flag = Arc::new(AtomicBool::new(false));
    let cancel_flag_clone = cancel_flag.clone();

    // Create password channel (using mpsc for potential multiple retries)
    let (password_tx, mut password_rx) = tokio::sync::mpsc::channel::<String>(1);

    // Clone for the task
    let job_id_clone = job_id.clone();
    let app_clone = app.clone();

    // Spawn the extraction task
    let task = tokio::spawn(async move {
        let mut final_stats = None;

        for input_path in input_paths {
            let archive_path = PathBuf::from(&input_path);
            let archive_path_str = input_path.clone();

            // Try extraction with retry for password
            let mut retry_count = 0;
            let max_retries = 3;

            loop {
                // Clone for progress callback
                let job_id_for_progress = job_id_clone.clone();
                let app_for_progress = app_clone.clone();
                let archive_for_progress = archive_path_str.clone();

                // Create progress callback
                let progress_callback =
                    move |current_file: &str, bytes_written: u64, total_bytes: Option<u64>| {
                        let event = ProgressEvent {
                            job_id: job_id_for_progress.clone(),
                            archive_path: archive_for_progress.clone(),
                            current_file: current_file.to_string(),
                            bytes_written,
                            total_bytes,
                        };

                        let _ = app_for_progress.emit_to("main", "extract_progress", event);
                        true // Continue extraction
                    };

                // Run extraction in blocking context
                let archive_path_for_blocking = archive_path.clone();
                let output_dir_for_blocking = output_dir.clone();
                let options_for_blocking = extract_options.clone();
                let cancel_flag_for_blocking = cancel_flag_clone.clone();

                let result = tokio::task::spawn_blocking(move || {
                    extractor::extract(
                        &archive_path_for_blocking,
                        &output_dir_for_blocking,
                        &options_for_blocking,
                        &progress_callback,
                        cancel_flag_for_blocking,
                    )
                })
                .await;

                match result {
                    Ok(Ok(stats)) => {
                        final_stats = Some(stats);

                        // Emit completion event for this archive
                        let completion = CompletionEvent {
                            job_id: job_id_clone.clone(),
                            archive_path: archive_path_str,
                            status: JobStatus::Success,
                            stats: final_stats.clone(),
                            error: None,
                        };
                        let _ = app_clone.emit_to("main", "extract_done", completion);
                        break; // Success, move to next archive
                    }
                    Ok(Err(e)) => {
                        // Check if password is required
                        if matches!(
                            e,
                            extractor::ExtractError::PasswordRequired
                                | extractor::ExtractError::InvalidPassword
                        ) && retry_count < max_retries
                        {
                            retry_count += 1;

                            // Emit password_required event
                            let password_event = PasswordRequiredEvent {
                                job_id: job_id_clone.clone(),
                                archive_path: archive_path_str.clone(),
                            };
                            let _ = app_clone.emit_to("main", "password_required", password_event);

                            // Wait for password from frontend (with timeout)
                            match tokio::time::timeout(
                                tokio::time::Duration::from_secs(300), // 5 minute timeout
                                password_rx.recv(),
                            )
                            .await
                            {
                                Ok(Some(password)) => {
                                    // Update options with the provided password
                                    extract_options.password = Some(password);
                                    continue; // Retry extraction
                                }
                                Ok(None) | Err(_) => {
                                    // Channel closed or timeout - treat as cancellation
                                    let completion = CompletionEvent {
                                        job_id: job_id_clone.clone(),
                                        archive_path: archive_path_str.clone(),
                                        status: JobStatus::Cancelled,
                                        stats: None,
                                        error: Some(
                                            "Password prompt timed out or was cancelled"
                                                .to_string(),
                                        ),
                                    };
                                    let _ = app_clone.emit_to("main", "extract_done", completion);
                                    return Err(extractor::ExtractError::Cancelled);
                                }
                            }
                        }

                        let error_msg = e.to_string();

                        let status = if matches!(e, extractor::ExtractError::Cancelled) {
                            JobStatus::Cancelled
                        } else {
                            JobStatus::Failed
                        };

                        let completion = CompletionEvent {
                            job_id: job_id_clone.clone(),
                            archive_path: archive_path_str,
                            status,
                            stats: None,
                            error: Some(error_msg),
                        };
                        let _ = app_clone.emit_to("main", "extract_done", completion);

                        // Stop processing remaining archives on error
                        return Err(e);
                    }
                    Err(join_err) => {
                        let err = extractor::ExtractError::Io(std::io::Error::new(
                            std::io::ErrorKind::Other,
                            format!("Task join error: {}", join_err),
                        ));
                        let error_msg = err.to_string();

                        let completion = CompletionEvent {
                            job_id: job_id_clone.clone(),
                            archive_path: archive_path_str,
                            status: JobStatus::Failed,
                            stats: None,
                            error: Some(error_msg),
                        };
                        let _ = app_clone.emit_to("main", "extract_done", completion);

                        return Err(err);
                    }
                }
            }
        }

        Ok(final_stats.unwrap_or_default())
    });

    // Store job handle
    let job_handle = JobHandle {
        cancel_flag,
        task,
        password_sender: Some(password_tx),
    };

    state.jobs.lock().insert(job_id.clone(), job_handle);

    Ok(job_id)
}

/// Probe archive metadata without extracting
#[tauri::command]
pub async fn probe(path: String) -> Result<extractor::ArchiveInfo, String> {
    let archive_path = PathBuf::from(path);

    // Run probe in blocking context since it does I/O
    tokio::task::spawn_blocking(move || {
        extractor::probe(&archive_path)
    })
    .await
    .map_err(|e| format!("Task join error: {}", e))?
    .map_err(|e| {
        // Convert ExtractError to user-friendly message
        match e {
            extractor::ExtractError::NotFound(_) => {
                "Archive file not found. Please check the file path.".to_string()
            }
            extractor::ExtractError::UnsupportedFormat(fmt) => {
                format!("Unsupported archive format: {}. Supported formats include ZIP, TAR, 7Z, and RAR.", fmt)
            }
            extractor::ExtractError::Corrupted(msg) => {
                format!("Archive appears to be corrupted: {}", msg)
            }
            extractor::ExtractError::Io(io_err) => {
                format!("Failed to read archive: {}", io_err)
            }
            _ => e.to_string(),
        }
    })
}

/// Cancel a running extraction job
#[tauri::command]
pub async fn cancel_job(state: State<'_, AppState>, job_id: String) -> Result<(), String> {
    // Look up and remove the job handle
    let job_handle = {
        let mut jobs = state.jobs.lock();
        jobs.remove(&job_id)
    }; // Lock is dropped here

    if let Some(job_handle) = job_handle {
        // Set the cancel flag to signal cancellation
        job_handle
            .cancel_flag
            .store(true, std::sync::atomic::Ordering::Relaxed);

        // Wait for the task to complete (it should abort soon)
        let _ = job_handle.task.await;

        Ok(())
    } else {
        Err(format!("Job not found: {}", job_id))
    }
}

/// Provide password for a password-protected archive
#[tauri::command]
pub async fn provide_password(
    state: State<'_, AppState>,
    job_id: String,
    password: String,
) -> Result<(), String> {
    // Look up the job handle and clone the sender
    let password_sender = {
        let jobs = state.jobs.lock();
        if let Some(job_handle) = jobs.get(&job_id) {
            job_handle.password_sender.clone()
        } else {
            None
        }
    }; // Lock is dropped here

    if let Some(sender) = password_sender {
        // Send the password to the extraction task
        sender
            .send(password)
            .await
            .map_err(|_| "Failed to send password to extraction task".to_string())?;
        Ok(())
    } else {
        Err(format!("Job not found: {}", job_id))
    }
}

/// File system entry metadata
#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export, export_to = "../../src/lib/bindings/")]
#[serde(rename_all = "camelCase")]
pub struct FileSystemEntry {
    pub name: String,
    pub path: String,
    pub is_directory: bool,
    pub is_archive: bool,
    #[ts(optional, type = "number")]
    pub size: Option<u64>,
    #[ts(optional, type = "number")]
    pub modified_at: Option<u64>,
}

/// List directory contents with metadata
#[tauri::command]
pub async fn list_directory(path: String) -> Result<Vec<FileSystemEntry>, String> {
    let dir_path = PathBuf::from(&path);

    // Run in blocking context since it does I/O
    tokio::task::spawn_blocking(move || {
        let mut entries = Vec::new();

        // Check if path exists and is a directory
        if !dir_path.exists() {
            return Err(format!("PERMISSION_DENIED: Path does not exist or access denied: {}", path));
        }

        if !dir_path.is_dir() {
            return Err(format!("Path is not a directory: {}", path));
        }

        // Read directory entries
        let read_dir = std::fs::read_dir(&dir_path).map_err(|e| {
            // Check if it's a permission error
            if e.kind() == std::io::ErrorKind::PermissionDenied {
                format!("PERMISSION_DENIED: Access denied to directory: {}", path)
            } else {
                format!("Failed to read directory: {}", e)
            }
        })?;

        for entry_result in read_dir {
            let entry = match entry_result {
                Ok(e) => e,
                Err(e) => {
                    eprintln!("Skipping entry due to error: {}", e);
                    continue;
                }
            };

            let entry_path = entry.path();
            let metadata = match entry.metadata() {
                Ok(m) => m,
                Err(e) => {
                    eprintln!("Skipping {} due to metadata error: {}", entry_path.display(), e);
                    continue;
                }
            };

            let name = entry.file_name().to_string_lossy().to_string();

            let path_str = entry_path.to_string_lossy().to_string();
            let is_directory = metadata.is_dir();

            // Check if file is an archive based on extension
            let is_archive = if !is_directory {
                is_archive_file(&entry_path)
            } else {
                false
            };

            let size = if !is_directory {
                Some(metadata.len())
            } else {
                None
            };

            let modified_at = metadata.modified().ok().and_then(|time| {
                time.duration_since(std::time::UNIX_EPOCH)
                    .ok()
                    .map(|d| d.as_secs())
            });

            entries.push(FileSystemEntry {
                name,
                path: path_str,
                is_directory,
                is_archive,
                size,
                modified_at,
            });
        }

        // Sort: directories first, then by name
        entries.sort_by(|a, b| match (a.is_directory, b.is_directory) {
            (true, false) => std::cmp::Ordering::Less,
            (false, true) => std::cmp::Ordering::Greater,
            _ => a.name.to_lowercase().cmp(&b.name.to_lowercase()),
        });

        Ok(entries)
    })
    .await
    .map_err(|e| format!("Task join error: {}", e))?
}

/// Get user's home directory path
#[tauri::command]
pub async fn get_home_directory() -> Result<String, String> {
    dirs::home_dir()
        .ok_or_else(|| "Failed to get home directory".to_string())
        .map(|path| path.to_string_lossy().to_string())
}

/// Get accessible default directories (works in sandbox)
/// Note: In a sandboxed environment, these paths may not be accessible
/// until the user grants permission via the file picker
#[tauri::command]
pub async fn get_accessible_directories() -> Result<Vec<FileSystemEntry>, String> {
    let mut accessible = Vec::new();

    // Return directory paths without trying to access them
    // This avoids triggering permission prompts
    let dirs_to_check = vec![
        ("Downloads", dirs::download_dir()),
        ("Documents", dirs::document_dir()),
        ("Desktop", dirs::desktop_dir()),
    ];

    for (name, dir_option) in dirs_to_check {
        if let Some(dir_path) = dir_option {
            let path_str = dir_path.to_string_lossy().to_string();
            
            accessible.push(FileSystemEntry {
                name: name.to_string(),
                path: path_str,
                is_directory: true,
                is_archive: false,
                size: None,
                modified_at: None,
            });
        }
    }

    Ok(accessible)
}

/// Request folder access permission using native file picker
#[tauri::command]
pub async fn request_folder_access(app: AppHandle) -> Result<Option<String>, String> {
    use tauri_plugin_dialog::DialogExt;

    // Open folder picker directly - no need for explanation dialog
    // The system will handle permission prompts
    let folder = app.dialog().file().blocking_pick_folder();

    if let Some(folder_path) = folder {
        // Convert FilePath to string
        let path_str = folder_path.to_string();
        Ok(Some(path_str))
    } else {
        Ok(None)
    }
}

/// Check if a path exists
#[tauri::command]
pub async fn check_path_exists(path: String) -> Result<bool, String> {
    let path_buf = PathBuf::from(path);
    Ok(path_buf.exists())
}

/// Get a unique output path for extraction with conflict resolution
#[tauri::command]
pub async fn get_unique_output_path(archive_path: String) -> Result<String, String> {
    let archive = PathBuf::from(&archive_path);

    // Get the directory containing the archive
    let parent_dir = archive
        .parent()
        .ok_or_else(|| "Failed to get parent directory".to_string())?;

    // Get the archive filename without extension
    let base_name = archive
        .file_stem()
        .ok_or_else(|| "Failed to get archive filename".to_string())?
        .to_string_lossy()
        .to_string();

    // Start with the base name
    let mut output_path = parent_dir.join(&base_name);
    let mut counter = 1;

    // Keep incrementing until we find a unique name
    while output_path.exists() {
        let new_name = format!("{} ({})", base_name, counter);
        output_path = parent_dir.join(new_name);
        counter += 1;
    }

    Ok(output_path.to_string_lossy().to_string())
}

/// Settings structure for persistence
#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export, export_to = "../../src/lib/bindings/")]
#[serde(rename_all = "camelCase")]
pub struct SettingsData {
    pub overwrite_mode: String,
    #[ts(type = "number")]
    pub size_limit_gb: f64,
    #[ts(type = "number")]
    pub strip_components: u32,
    pub allow_symlinks: bool,
    pub allow_hardlinks: bool,
}

impl Default for SettingsData {
    fn default() -> Self {
        Self {
            overwrite_mode: "rename".to_string(),
            size_limit_gb: 20.0,
            strip_components: 0,
            allow_symlinks: false,
            allow_hardlinks: false,
        }
    }
}

/// Save settings to disk
#[tauri::command]
pub async fn save_settings(app: AppHandle, settings: SettingsData) -> Result<(), String> {
    // Get app data directory
    let app_data_dir = app
        .path()
        .app_data_dir()
        .map_err(|e| format!("Failed to get app data directory: {}", e))?;

    // Create directory if it doesn't exist
    tokio::fs::create_dir_all(&app_data_dir)
        .await
        .map_err(|e| format!("Failed to create app data directory: {}", e))?;

    // Settings file path
    let settings_path = app_data_dir.join("settings.json");

    // Serialize settings to JSON
    let json = serde_json::to_string_pretty(&settings)
        .map_err(|e| format!("Failed to serialize settings: {}", e))?;

    // Write to file
    tokio::fs::write(&settings_path, json)
        .await
        .map_err(|e| format!("Failed to write settings file: {}", e))?;

    Ok(())
}

/// Load settings from disk
#[tauri::command]
pub async fn load_settings(app: AppHandle) -> Result<SettingsData, String> {
    // Get app data directory
    let app_data_dir = app
        .path()
        .app_data_dir()
        .map_err(|e| format!("Failed to get app data directory: {}", e))?;

    // Settings file path
    let settings_path = app_data_dir.join("settings.json");

    // Check if file exists
    if !settings_path.exists() {
        // Return default settings if file doesn't exist
        return Ok(SettingsData::default());
    }

    // Read file
    let contents = tokio::fs::read_to_string(&settings_path)
        .await
        .map_err(|e| format!("Failed to read settings file: {}", e))?;

    // Parse JSON
    let settings: SettingsData = serde_json::from_str(&contents).unwrap_or_else(|e| {
        // If parsing fails (corrupted file), log error and return defaults
        eprintln!("Failed to parse settings file: {}. Using defaults.", e);
        SettingsData::default()
    });

    Ok(settings)
}

/// Helper function to check if a file is an archive based on extension
fn is_archive_file(path: &Path) -> bool {
    const ARCHIVE_EXTENSIONS: &[&str] = &[
        "zip", "7z", "rar", "tar", "gz", "bz2", "xz", "tgz", "tbz2", "txz",
    ];

    let filename = path
        .file_name()
        .and_then(|f| f.to_str())
        .unwrap_or("")
        .to_lowercase();

    // Check for multi-part archives
    // RAR: .part1.rar, .part01.rar, .r00, .r01, etc.
    if filename.contains(".part") && filename.ends_with(".rar") {
        return true;
    }

    // 7z: .7z.001, .7z.002, etc.
    if filename.contains(".7z.") {
        if let Some(ext) = path.extension() {
            if ext.to_string_lossy().chars().all(|c| c.is_ascii_digit()) {
                return true;
            }
        }
    }

    // ZIP: .zip.001, .zip.002, etc.
    if filename.contains(".zip.") {
        if let Some(ext) = path.extension() {
            if ext.to_string_lossy().chars().all(|c| c.is_ascii_digit()) {
                return true;
            }
        }
    }

    // Check standard extensions
    if let Some(ext) = path.extension() {
        let ext_lower = ext.to_string_lossy().to_lowercase();

        // Check for .rXX extensions (RAR multi-part)
        if ext_lower.starts_with('r') && ext_lower.len() >= 2 {
            if ext_lower[1..].chars().all(|c| c.is_ascii_digit()) {
                return true;
            }
        }

        ARCHIVE_EXTENSIONS.contains(&ext_lower.as_str())
    } else {
        false
    }
}
