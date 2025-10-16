# Workspace Structure

This project uses a Cargo workspace to organize the codebase into multiple crates.

## Directory Layout

```
unarchiver/
├── Cargo.toml                 # Workspace root configuration
├── package.json               # Frontend dependencies (React + Vite)
├── src/                       # React frontend source
├── src-tauri/                 # Tauri application (main app)
│   ├── Cargo.toml
│   ├── src/
│   │   ├── main.rs
│   │   └── lib.rs
│   └── tauri.conf.json
├── crates/                    # Rust workspace crates
│   ├── extractor/             # Core extraction library
│   │   ├── Cargo.toml
│   │   └── src/
│   │       ├── lib.rs         # Public API
│   │       ├── error.rs       # Error types
│   │       └── types.rs       # Data structures
│   └── cli/                   # Command-line interface
│       ├── Cargo.toml
│       └── src/
│           └── main.rs
└── dist/                      # Built frontend assets
```

## Crates

### 1. `unarchiver` (src-tauri/)
The main Tauri desktop application that provides the GUI and integrates with macOS.

**Dependencies:**
- `extractor` - Core extraction logic
- `tauri` - Desktop app framework
- `tokio` - Async runtime

### 2. `extractor` (crates/extractor/)
Core library for safe archive extraction. Platform-agnostic and reusable.

**Features:**
- Multiple format support (ZIP, TAR, 7Z, RAR, ISO)
- Security features (path traversal prevention, size limits)
- Progress tracking and cancellation
- Password-protected archives

**Public API:**
```rust
pub fn probe(path: &Path) -> Result<ArchiveInfo, ExtractError>;
pub fn extract(...) -> Result<ExtractStats, ExtractError>;
```

### 3. `unarchive-cli` (crates/cli/)
Command-line interface for archive operations.

**Commands:**
- `unarchive extract` - Extract archives
- `unarchive probe` - Inspect archive metadata

## Building

```bash
# Build entire workspace
cargo build --workspace

# Build specific crate
cargo build -p extractor
cargo build -p unarchive-cli
cargo build -p unarchiver

# Run tests
cargo test --workspace

# Run the Tauri app in dev mode
npm run tauri dev
```

## Workspace Dependencies

Shared dependencies are defined in the root `Cargo.toml` under `[workspace.dependencies]`:
- `serde` - Serialization
- `serde_json` - JSON support
- `thiserror` - Error handling
- `tokio` - Async runtime
- `tracing` - Logging

This ensures version consistency across all crates.
