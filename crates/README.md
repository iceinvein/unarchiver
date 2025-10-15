# Crates

This directory contains the Rust workspace crates for the Unarchiver project.

## Structure

### `extractor/`
Core archive extraction library with security features. This is a standalone library that can be used independently of the Tauri app.

**Features:**
- Safe extraction of multiple archive formats (ZIP, TAR, 7Z, RAR, ISO)
- Path traversal attack prevention (zip-slip protection)
- Size limit enforcement
- Progress tracking and cancellation support
- Password-protected archive support

**Usage:**
```rust
use extractor::{probe, extract, ExtractOptions};

// Probe archive metadata
let info = probe(Path::new("archive.zip"))?;

// Extract with options
let options = ExtractOptions::default();
let stats = extract(archive_path, output_dir, &options, &progress_cb, cancel_flag)?;
```

### `cli/`
Command-line interface for archive extraction. Provides a simple CLI tool for extracting archives and probing metadata.

**Usage:**
```bash
# Extract archives
unarchive extract archive.zip -o output/

# Probe archive metadata
unarchive probe archive.zip --json
```

## Building

Build all crates:
```bash
cargo build --workspace
```

Build specific crate:
```bash
cargo build -p extractor
cargo build -p unarchive-cli
```

Run tests:
```bash
cargo test --workspace
```

## Dependencies

Shared workspace dependencies are defined in the root `Cargo.toml` to ensure version consistency across all crates.
