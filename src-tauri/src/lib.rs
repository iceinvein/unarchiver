pub mod commands;
mod state;

use state::AppState;
use std::path::PathBuf;
use std::sync::{Arc, Mutex};
use tauri::{Emitter, Listener, Manager};

// Buffer for files opened before window is ready
#[derive(Default, Clone)]
struct PendingOpens(Arc<Mutex<Vec<PathBuf>>>);

impl PendingOpens {
    fn push_many(&self, items: Vec<PathBuf>) {
        self.0.lock().unwrap().extend(items);
    }

    fn take_all(&self) -> Vec<PathBuf> {
        let mut guard = self.0.lock().unwrap();
        std::mem::take(&mut *guard)
    }
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_fs::init())
        .plugin(tauri_plugin_deep_link::init())
        .plugin(tauri_plugin_single_instance::init(|app, args, _cwd| {
            eprintln!("Single instance callback - args: {:?}", args);

            // When a file is opened with the app, macOS launches a new instance
            // This plugin prevents that and instead sends the args to the existing instance
            let archive_extensions = [
                "zip", "7z", "rar", "tar", "gz", "bz2", "xz", "tgz", "tbz2", "txz",
            ];

            let archive_paths: Vec<String> = args
                .iter()
                .skip(1) // Skip the first arg (executable path)
                .filter(|arg| {
                    let path_obj = std::path::Path::new(arg);
                    if let Some(ext) = path_obj.extension().and_then(|e| e.to_str()) {
                        archive_extensions.contains(&ext.to_lowercase().as_str())
                    } else {
                        false
                    }
                })
                .cloned()
                .collect();

            if !archive_paths.is_empty() {
                eprintln!(
                    "Found archives in single-instance args: {:?}",
                    archive_paths
                );
                if let Some(window) = app.get_webview_window("main") {
                    // Bring window to front
                    let _ = window.set_focus();
                    // Emit the files_opened event
                    let _ = window.emit("files_opened", archive_paths);
                }
            }
        }))
        .manage(AppState::new())
        .manage(PendingOpens::default())
        .invoke_handler(tauri::generate_handler![
            commands::extract,
            commands::probe,
            commands::cancel_job,
            commands::provide_password,
            commands::list_directory,
            commands::get_home_directory,
            commands::get_accessible_directories,
            commands::request_folder_access,
            commands::check_path_exists,
            commands::get_unique_output_path,
            commands::save_settings,
            commands::load_settings,
        ])
        .setup(|app| {
            // Flush any pending file opens that were buffered before window was ready
            let pending_state = app.state::<PendingOpens>();
            let pending = pending_state.take_all();

            if !pending.is_empty() {
                eprintln!("Flushing {} pending file opens", pending.len());

                let archive_extensions = [
                    "zip", "7z", "rar", "tar", "gz", "bz2", "xz", "tgz", "tbz2", "txz",
                ];

                let archive_paths: Vec<String> = pending
                    .iter()
                    .filter(|path| {
                        if let Some(ext) = path.extension().and_then(|e| e.to_str()) {
                            archive_extensions.contains(&ext.to_lowercase().as_str())
                        } else {
                            false
                        }
                    })
                    .filter_map(|p| p.to_str().map(|s| s.to_string()))
                    .collect();

                if !archive_paths.is_empty() {
                    // Emit after a short delay to ensure window is fully ready
                    let app_handle_clone = app.handle().clone();
                    std::thread::spawn(move || {
                        std::thread::sleep(std::time::Duration::from_millis(500));
                        if let Some(window) = app_handle_clone.get_webview_window("main") {
                            eprintln!("Emitting files_opened event for pending files");
                            let _ = window.emit("files_opened", archive_paths);
                        }
                    });
                }
            }

            // Listen for deep-link events (file associations on macOS)
            let app_handle = app.handle().clone();

            // Try multiple event names that the deep-link plugin might use
            let event_names = vec![
                "deep-link://request",
                "deep-link://urls",
                "plugin:deep-link:urls",
            ];

            for event_name in event_names {
                let handle_clone = app_handle.clone();
                app.listen(event_name, move |event| {
                    eprintln!("Event '{}' received: {:?}", event_name, event);

                    if let Ok(urls) = serde_json::from_str::<Vec<String>>(event.payload()) {
                        eprintln!("URLs: {:?}", urls);

                        let archive_extensions = [
                            "zip", "7z", "rar", "tar", "gz", "bz2", "xz", "tgz", "tbz2", "txz",
                        ];

                        let archive_paths: Vec<String> = urls
                            .iter()
                            .filter_map(|url| {
                                // Handle file:// URLs
                                if url.starts_with("file://") {
                                    let path = url.strip_prefix("file://").unwrap_or(url);
                                    // URL decode the path
                                    if let Ok(decoded) = urlencoding::decode(path) {
                                        let path_str = decoded.to_string();

                                        // Check if it's an archive
                                        let path_obj = std::path::Path::new(&path_str);
                                        if let Some(ext) =
                                            path_obj.extension().and_then(|e| e.to_str())
                                        {
                                            if archive_extensions
                                                .contains(&ext.to_lowercase().as_str())
                                            {
                                                return Some(path_str);
                                            }
                                        }
                                    }
                                } else {
                                    // Maybe it's already a path, not a URL
                                    let path_obj = std::path::Path::new(url);
                                    if let Some(ext) = path_obj.extension().and_then(|e| e.to_str())
                                    {
                                        if archive_extensions.contains(&ext.to_lowercase().as_str())
                                        {
                                            return Some(url.clone());
                                        }
                                    }
                                }
                                None
                            })
                            .collect();

                        if !archive_paths.is_empty() {
                            eprintln!("Emitting files_opened event for: {:?}", archive_paths);
                            if let Some(window) = handle_clone.get_webview_window("main") {
                                let _ = window.emit("files_opened", archive_paths);
                            }
                        }
                    }
                });
            }

            Ok(())
        })
        .on_window_event(|window, event| {
            eprintln!("Window event received: {:?}", event);

            // Handle both drag-and-drop AND files opened from Finder
            if let tauri::WindowEvent::DragDrop(tauri::DragDropEvent::Drop { paths, .. }) = event {
                eprintln!("Files dropped or opened: {:?}", paths);

                // Filter for supported archive extensions
                let archive_extensions = [
                    "zip", "7z", "rar", "tar", "gz", "bz2", "xz", "tgz", "tbz2", "txz",
                ];

                let archive_paths: Vec<String> = paths
                    .iter()
                    .filter(|path| {
                        if let Some(ext) = path.extension().and_then(|e| e.to_str()) {
                            archive_extensions.contains(&ext.to_lowercase().as_str())
                        } else {
                            false
                        }
                    })
                    .map(|p| p.to_string_lossy().to_string())
                    .collect();

                if !archive_paths.is_empty() {
                    eprintln!("Emitting files_opened event for: {:?}", archive_paths);
                    let _ = window.emit("files_opened", archive_paths);
                }
            }
        })
        .build(tauri::generate_context!())
        .expect("error while building tauri application")
        .run(|app_handle, event| {
            match event {
                tauri::RunEvent::Opened { urls } => {
                    eprintln!("RunEvent::Opened received with URLs: {:?}", urls);

                    let archive_extensions = [
                        "zip", "7z", "rar", "tar", "gz", "bz2", "xz", "tgz", "tbz2", "txz",
                    ];

                    // Convert URLs to file paths
                    let paths: Vec<PathBuf> = urls
                        .iter()
                        .filter_map(|url| {
                            if url.scheme() == "file" {
                                url.to_file_path().ok()
                            } else {
                                None
                            }
                        })
                        .collect();

                    let archive_paths: Vec<String> = paths
                        .iter()
                        .filter(|path| {
                            if let Some(ext) = path.extension().and_then(|e| e.to_str()) {
                                archive_extensions.contains(&ext.to_lowercase().as_str())
                            } else {
                                false
                            }
                        })
                        .filter_map(|p| p.to_str().map(|s| s.to_string()))
                        .collect();

                    if !archive_paths.is_empty() {
                        eprintln!("Found archives: {:?}", archive_paths);

                        if let Some(window) = app_handle.get_webview_window("main") {
                            eprintln!("Emitting to existing window");
                            let _ = window.emit("files_opened", &archive_paths);
                        } else {
                            eprintln!("Window not ready, buffering paths");
                            // Buffer if window not ready yet
                            let state = app_handle.state::<PendingOpens>();
                            state.push_many(paths);
                        }
                    }
                }
                _ => {}
            }
        });
}
