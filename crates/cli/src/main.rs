//! Command-line interface for archive extraction.
//!
//! This CLI tool provides a simple interface for extracting archives
//! and probing archive metadata from the command line.

use clap::{Parser, Subcommand};
use std::path::PathBuf;
use std::process;

#[derive(Parser)]
#[command(name = "unarchive")]
#[command(version, about = "Extract archives from the command line", long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Extract one or more archives
    Extract {
        /// Archive files to extract
        #[arg(required = true)]
        archives: Vec<PathBuf>,

        /// Output directory
        #[arg(short, long)]
        out: PathBuf,

        /// Overwrite mode: replace, skip, rename
        #[arg(long, default_value = "rename")]
        overwrite: String,

        /// Password for encrypted archives
        #[arg(long)]
        password: Option<String>,

        /// Strip leading path components
        #[arg(long, default_value = "0")]
        strip_components: u32,

        /// Size limit in bytes
        #[arg(long)]
        size_limit: Option<u64>,
    },

    /// Probe archive metadata
    Probe {
        /// Archive file to probe
        archive: PathBuf,

        /// Output as JSON
        #[arg(long)]
        json: bool,
    },
}

fn main() {
    // Initialize tracing
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new("info")),
        )
        .init();

    let cli = Cli::parse();

    let result = match cli.command {
        Commands::Extract {
            archives,
            out,
            overwrite,
            password,
            strip_components,
            size_limit,
        } => handle_extract(archives, out, overwrite, password, strip_components, size_limit),
        Commands::Probe { archive, json } => handle_probe(archive, json),
    };

    if let Err(e) = result {
        eprintln!("Error: {}", e);
        process::exit(1);
    }
}

fn handle_extract(
    archives: Vec<PathBuf>,
    out: PathBuf,
    overwrite: String,
    password: Option<String>,
    strip_components: u32,
    size_limit: Option<u64>,
) -> Result<(), Box<dyn std::error::Error>> {
    use extractor::{extract, ExtractOptions, OverwriteMode};
    use indicatif::{ProgressBar, ProgressStyle};
    use std::sync::atomic::{AtomicBool, Ordering};
    use std::sync::Arc;

    // Parse overwrite mode
    let overwrite_mode = match overwrite.as_str() {
        "replace" => OverwriteMode::Replace,
        "skip" => OverwriteMode::Skip,
        "rename" => OverwriteMode::Rename,
        _ => {
            eprintln!("Invalid overwrite mode: {}. Use 'replace', 'skip', or 'rename'.", overwrite);
            process::exit(1);
        }
    };

    // Create extraction options
    let options = ExtractOptions {
        overwrite: overwrite_mode,
        size_limit_bytes: size_limit,
        strip_components,
        allow_symlinks: false,
        allow_hardlinks: false,
        password: password.clone(),
    };

    // Create output directory if it doesn't exist
    if !out.exists() {
        std::fs::create_dir_all(&out)?;
    }

    // Process each archive
    for archive_path in archives {
        println!("\nExtracting: {}", archive_path.display());

        // Check if archive exists
        if !archive_path.exists() {
            eprintln!("Error: Archive not found: {}", archive_path.display());
            process::exit(1);
        }

        // Create progress bar
        let pb = ProgressBar::new(100);
        pb.set_style(
            ProgressStyle::default_bar()
                .template("{spinner:.green} [{bar:40.cyan/blue}] {percent}% {msg}")
                .expect("Invalid progress bar template")
                .progress_chars("#>-"),
        );

        let cancel_flag = Arc::new(AtomicBool::new(false));
        let cancel_flag_clone = cancel_flag.clone();

        // Set up Ctrl+C handler
        ctrlc::set_handler(move || {
            cancel_flag_clone.store(true, Ordering::SeqCst);
        })
        .ok(); // Ignore error if handler already set

        // Track progress
        let pb_clone = pb.clone();
        let progress_cb = move |file: &str, _bytes_written: u64, total_bytes: Option<u64>| {
            pb_clone.set_message(format!("{}", file));
            
            if let Some(total) = total_bytes {
                if total > 0 {
                    let percent = (_bytes_written as f64 / total as f64 * 100.0) as u64;
                    pb_clone.set_position(percent.min(100));
                }
            }
            
            true // Continue extraction
        };

        // Extract archive
        match extract(&archive_path, &out, &options, &progress_cb, cancel_flag.clone()) {
            Ok(stats) => {
                pb.finish_with_message("Done");
                
                if stats.cancelled {
                    println!("✗ Extraction cancelled");
                    process::exit(130); // Standard exit code for SIGINT
                } else {
                    println!(
                        "✓ Extracted {} files ({:.2} MB) in {:.2}s",
                        stats.files_extracted,
                        stats.bytes_written as f64 / 1_048_576.0,
                        stats.duration.as_secs_f64()
                    );
                }
            }
            Err(e) => {
                pb.finish_with_message("Failed");
                eprintln!("Error extracting {}: {}", archive_path.display(), e);
                process::exit(1);
            }
        }
    }

    Ok(())
}

fn handle_probe(archive: PathBuf, json: bool) -> Result<(), Box<dyn std::error::Error>> {
    use extractor::probe;

    // Check if archive exists
    if !archive.exists() {
        eprintln!("Error: Archive not found: {}", archive.display());
        process::exit(1);
    }

    // Probe the archive
    match probe(&archive) {
        Ok(info) => {
            if json {
                // Output as JSON
                let json_output = serde_json::to_string_pretty(&info)?;
                println!("{}", json_output);
            } else {
                // Output as human-readable text
                println!("Archive: {}", archive.display());
                println!("Format: {}", info.format);
                println!("Entries: {}", info.entries);
                
                if let Some(compressed) = info.compressed_bytes {
                    println!("Compressed: {:.2} MB", compressed as f64 / 1_048_576.0);
                }
                
                if let Some(uncompressed) = info.uncompressed_estimate {
                    println!("Uncompressed: {:.2} MB (estimated)", uncompressed as f64 / 1_048_576.0);
                }
                
                println!("Encrypted: {}", if info.encrypted { "Yes" } else { "No" });
            }
            Ok(())
        }
        Err(e) => {
            eprintln!("Error probing archive: {}", e);
            process::exit(1);
        }
    }
}
