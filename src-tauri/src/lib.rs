pub mod commands;
mod state;

use state::AppState;
use tauri::Emitter;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_fs::init())
        .manage(AppState::new())
        .invoke_handler(tauri::generate_handler![
            commands::extract,
            commands::probe,
            commands::cancel_job,
            commands::provide_password,
        ])
        .on_window_event(|window, event| {
            if let tauri::WindowEvent::DragDrop(tauri::DragDropEvent::Drop { paths, .. }) = event {
                // Filter for supported archive extensions
                let archive_extensions = [
                    "zip", "7z", "rar", "tar", "gz", "bz2", "xz",
                    "tgz", "tbz2", "txz", "iso"
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
                    // Emit event to frontend to queue these archives
                    let _ = window.emit("files_opened", archive_paths);
                }
            }
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
