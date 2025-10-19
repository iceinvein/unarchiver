# Unarchiver

A modern, safe, and efficient archive extraction application for macOS built with Tauri 2.x and React.

## Features

- **Multiple Format Support**: Extract ZIP, 7Z, RAR, TAR, GZ, BZ2, and XZ archives
- **Multi-Part Archive Support**: Full support for RAR multi-part archives (.part1.rar, .r00, etc.)
- **Modern File Explorer**: Browse your file system and preview archives before extraction
- **Flexible Extraction Options**:
  - Extract to default location (same directory as archive)
  - Extract to custom folder via split button dropdown
  - Automatic conflict resolution with rename/skip/replace modes
- **Archive Preview**: View archive contents, file structure, and metadata before extracting
- **Queue Management**: Process multiple archives with real-time progress tracking in a drawer
- **Password Support**: Handle password-protected archives with secure prompts
- **macOS Integration**: 
  - File associations - double-click archives to open with Unarchiver
  - Deep-link support for opening archives from Finder
  - Single-instance app - opening multiple archives adds them to the queue
- **Safety First**: Built-in protection against zip-slip and path traversal attacks
- **Customizable Settings**: Control overwrite behavior, size limits, and extraction options
- **Modern UI**: Clean, responsive interface with light/dark/system theme support
- **Permission Management**: Guided folder access setup for macOS sandbox compliance

## Installation

### Mac App Store

Download Unarchiver from the Mac App Store (coming soon).

### Direct Download

1. Download the latest `.dmg` from the [Releases](https://github.com/yourusername/unarchiver/releases) page
2. Open the DMG and drag Unarchiver to your Applications folder
3. Launch Unarchiver from Applications

### First Launch

On first launch, you'll be prompted to grant folder access permissions. This is required for the app to read archives and write extracted files due to macOS sandboxing requirements.

## Usage

### GUI Application

1. **Launch the app** from Applications
2. **Browse and select archives** using the built-in file explorer
3. **Preview archive contents** - click on an archive to see its contents, structure, and metadata
4. **Extract archives** by:
   - Clicking the "Extract" button to extract to the default location (same directory as archive)
   - Using the split button dropdown to choose "Extract to..." for a custom output folder
   - Double-clicking archives from Finder (if file associations are set up)
5. **Monitor progress** - click the queue icon (top-right) to view active and completed extractions
6. **Enter passwords** when prompted for encrypted archives

### Opening Archives from Finder

Once file associations are configured, you can:
- Double-click any supported archive in Finder to open it in Unarchiver
- Drag and drop archives onto the Unarchiver app icon
- Right-click archives and select "Open With → Unarchiver"

The app will automatically navigate to the archive location and show a preview.

### Command Line Interface

A standalone CLI tool is available in the `crates/cli` directory:

```bash
# Build the CLI
npm run build:cli

# The binary will be at: target/release/unarchive-cli

# Extract archives
./target/release/unarchive-cli extract --out ~/Downloads archive.zip

# Extract multiple archives
./target/release/unarchive-cli extract --out ~/Downloads file1.zip file2.7z

# Probe archive metadata
./target/release/unarchive-cli probe archive.zip

# Probe with JSON output
./target/release/unarchive-cli probe --json archive.zip

# Extract with options
./target/release/unarchive-cli extract \
  --out ~/Downloads \
  --overwrite rename \
  --strip-components 1 \
  --password mypassword \
  archive.zip
```

**CLI Options**:
- `--out <DIR>`: Output directory (required)
- `--overwrite <MODE>`: Overwrite mode: `replace`, `skip`, or `rename` (default: `rename`)
- `--password <PASS>`: Password for encrypted archives
- `--strip-components <N>`: Remove N leading path components (default: 0)
- `--size-limit <BYTES>`: Maximum extraction size in bytes
- `--json`: Output probe results as JSON

## User Interface

### File Explorer
- Browse your file system to find archives
- Navigate using breadcrumb navigation
- Archives are highlighted with special icons
- Click on archives to preview their contents

### Archive Preview
- View archive contents in a tree structure
- See file/folder counts and total size
- Expand/collapse folders to explore structure
- Identify password-protected archives before extraction
- Quick metadata display (format, file count, size)

### Extraction Options
- **Quick Extract**: Click the main "Extract" button to extract to default location (same directory as archive)
- **Custom Location**: Use the split button dropdown to select "Extract to..." and choose a specific output folder
- **Drag & Drop**: Drop archives onto the app window to open them

### Queue Management (Drawer)
- Click the queue icon (top-right) to open the extraction queue drawer
- View all active and completed extractions
- Real-time progress tracking with file names and bytes written
- Cancel ongoing extractions
- Badge indicator shows number of active extractions
- Review extraction statistics (files extracted, duration, etc.)

## Settings

Configure extraction behavior in the Settings panel (gear icon in top-right):

- **Overwrite Mode**: Choose how to handle existing files (replace, skip, or rename)
  - **Replace**: Overwrite existing files
  - **Skip**: Keep existing files, skip new ones
  - **Rename**: Add numbers to create unique filenames (e.g., file (1).txt)
- **Size Limit**: Set maximum extraction size (default: 20 GB)
- **Strip Components**: Remove leading path components from extracted files
- **Allow Symlinks**: Enable/disable symbolic link extraction (disabled by default for security)
- **Allow Hardlinks**: Enable/disable hard link extraction (disabled by default for security)
- **Theme**: Choose between light, dark, or system theme (toggle with theme icon in navbar)

**Note**: When using "Extract to..." for a custom folder, the overwrite mode is automatically set to "replace" for convenience.

## Supported Formats

| Format | Extensions | Password Support | Multi-Part Support | Notes |
|--------|-----------|------------------|-------------------|-------|
| ZIP | .zip | ✓ | ⚠️ Limited | Includes ZIP64 support |
| 7-Zip | .7z | ✓ | ⚠️ Limited | |
| RAR | .rar, .part1.rar, .r00 | ✓ | ✅ Full | Read-only |
| TAR | .tar | ✗ | N/A | |
| TAR+GZIP | .tar.gz, .tgz | ✗ | N/A | |
| TAR+BZIP2 | .tar.bz2, .tbz2, .tbz | ✗ | N/A | |
| TAR+XZ | .tar.xz, .txz | ✗ | N/A | Pure Rust LZMA |
| GZIP | .gz | ✗ | N/A | Single file compression |
| BZIP2 | .bz2 | ✗ | N/A | Single file compression |
| XZ | .xz | ✗ | N/A | Single file compression |

**Multi-Part Archive Notes:**
- **RAR**: Full support for `.part1.rar`, `.part01.rar`, `.r00`, `.r01`, etc. Select any part and extraction will automatically start from the first part.
- **7-Zip/ZIP**: Limited support. Multi-part `.7z.001`/`.zip.001` archives are detected but may require external tools to combine parts before extraction.

**Note**: ISO format is not currently supported.

## Security

Unarchiver includes built-in security features:

- **Path Traversal Protection**: Prevents zip-slip attacks
- **Size Limits**: Configurable extraction size limits
- **Safe Defaults**: Symlinks and hardlinks are blocked by default
- **Path Validation**: All entry paths are normalized and validated

## Development

### Prerequisites

- [Rust](https://www.rust-lang.org/) (latest stable)
- [Node.js](https://nodejs.org/) (v18 or later)
- [Bun](https://bun.sh/) (recommended) or npm
- Xcode Command Line Tools (macOS)

### Setup

```bash
# Clone the repository
git clone https://github.com/yourusername/unarchiver.git
cd unarchiver

# Install dependencies
bun install

# Run in development mode
bun run tauri dev

# Build for production
bun run tauri build
```

### Project Structure

```
.
├── Cargo.toml         # Rust workspace configuration
├── package.json       # Frontend dependencies
├── crates/
│   ├── extractor/     # Core extraction library (shared)
│   └── cli/           # Standalone command-line tool
├── src/               # React frontend (Vite + TypeScript)
├── src-tauri/         # Tauri backend (main app)
├── docs/              # Documentation
└── fastlane/          # Mac App Store automation
```

See [WORKSPACE_STRUCTURE.md](docs/WORKSPACE_STRUCTURE.md) for detailed information.

## Troubleshooting

### App won't open (Gatekeeper warning)

If you see a warning about the app being from an unidentified developer:
1. Right-click the app in Finder
2. Select "Open"
3. Click "Open" in the dialog

For signed releases, this shouldn't be necessary.

### Extraction fails with "Permission Denied"

This is usually due to macOS sandbox restrictions:
1. On first launch, grant folder access when prompted
2. If you dismissed the dialog, go to Settings and click "Request Folder Access"
3. Select your home folder or a specific folder you want to extract to
4. Ensure you have write permissions to the output directory

### Multi-part archives not working

**RAR archives**: Should work automatically. Select any part (e.g., `.part3.rar`) and the app will find and use the first part.

**7-Zip/ZIP archives**: Multi-part `.7z.001`/`.zip.001` archives are not fully supported. You can:
1. Use an external tool (like 7-Zip or The Unarchiver) to combine the parts first
2. Use the command line: `cat file.7z.* > combined.7z` then extract `combined.7z`

### Archive appears corrupted

1. Verify the archive is complete and not corrupted using another tool
2. For multi-part archives, ensure all parts are in the same directory
3. Check that you have the first part (`.part1.rar`, `.001`, etc.)
4. Check the app logs for detailed error messages

### File associations not working

If double-clicking archives doesn't open Unarchiver:
1. Right-click an archive file in Finder
2. Select "Get Info"
3. Under "Open with:", select Unarchiver
4. Click "Change All..." to apply to all files of this type

## Mac App Store

This app is designed for distribution via the Mac App Store. See [MAS_SUBMISSION_GUIDE.md](MAS_SUBMISSION_GUIDE.md) for build and submission instructions.

Key features for App Store compliance:
- App Sandbox enabled
- Proper entitlements configuration
- Signed with Apple Distribution certificate
- Minimum macOS 12.0 (Apple Silicon only)

## Contributing

Contributions are welcome! Please feel free to submit a Pull Request.

## License

[MIT License](LICENSE)

## Recommended IDE Setup

- [VS Code](https://code.visualstudio.com/) + [Tauri](https://marketplace.visualstudio.com/items?itemName=tauri-apps.tauri-vscode) + [rust-analyzer](https://marketplace.visualstudio.com/items?itemName=rust-lang.rust-analyzer)
