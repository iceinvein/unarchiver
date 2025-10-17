// Simple CLI tool to send file paths to the running app
use std::env;
use std::process::Command;

fn main() {
    let args: Vec<String> = env::args().collect();

    if args.len() < 2 {
        eprintln!("Usage: open_file <file_path>");
        std::process::exit(1);
    }

    let file_path = &args[1];

    // Use AppleScript to tell the app to open the file
    let script = format!(
        r#"tell application "unarchiver" to activate
        tell application "System Events"
            keystroke "{}"
        end tell"#,
        file_path
    );

    let _ = Command::new("osascript").arg("-e").arg(&script).output();
}
