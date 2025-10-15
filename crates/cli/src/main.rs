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
    _archives: Vec<PathBuf>,
    _out: PathBuf,
    _overwrite: String,
    _password: Option<String>,
    _strip_components: u32,
    _size_limit: Option<u64>,
) -> Result<(), Box<dyn std::error::Error>> {
    // TODO: Implement in task 5.2
    println!("Extract command not yet implemented");
    Ok(())
}

fn handle_probe(_archive: PathBuf, _json: bool) -> Result<(), Box<dyn std::error::Error>> {
    // TODO: Implement in task 5.3
    println!("Probe command not yet implemented");
    Ok(())
}
