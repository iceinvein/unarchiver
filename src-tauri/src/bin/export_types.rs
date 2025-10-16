// Binary to export TypeScript types
// Run with: cargo run --bin export_types

use ts_rs::TS;

fn main() {
    println!("Exporting TypeScript types...");

    // This will trigger ts-rs to export types
    unarchiver_lib::commands::ExtractOptionsDTO::export()
        .expect("Failed to export ExtractOptionsDTO");
    unarchiver_lib::commands::ProgressEvent::export().expect("Failed to export ProgressEvent");
    unarchiver_lib::commands::CompletionEvent::export().expect("Failed to export CompletionEvent");
    unarchiver_lib::commands::JobStatus::export().expect("Failed to export JobStatus");
    unarchiver_lib::commands::PasswordRequiredEvent::export()
        .expect("Failed to export PasswordRequiredEvent");
    unarchiver_lib::commands::FileSystemEntry::export().expect("Failed to export FileSystemEntry");

    extractor::ArchiveInfo::export().expect("Failed to export ArchiveInfo");
    extractor::ExtractStats::export().expect("Failed to export ExtractStats");

    println!("✓ TypeScript types exported successfully to src/lib/bindings/");
}
