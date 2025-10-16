# Unarchiver

A modern, safe, and efficient archive extraction application for macOS built with Tauri 2.x and React.

## Features

- **Multiple Format Support**: Extract ZIP, 7Z, RAR, TAR, GZ, BZ2, XZ, ISO, and more
- **Multi-Part Archive Support**: Full support for RAR multi-part archives (.part1.rar, .r00, etc.)
- **Modern File Explorer**: Browse and preview archives before extraction
- **Flexible Extraction Options**:
  - Extract to default location (same directory as archive)
  - Extract to custom folder via dropdown or context menu
  - Automatic conflict resolution with rename/skip/replace modes
- **Archive Preview**: View archive contents, file structure, and metadata before extracting
- **Queue Management**: Process multiple archives with real-time progress tracking
- **Password Support**: Handle password-protected archives with secure prompts
- **macOS Integration**: 
  - Double-click archives to open with Unarchiver
  - Finder Quick Action for "Extract Here" functionality
- **Safety First**: Built-in protection against zip-slip and path traversal attacks
- **Customizable Settings**: Control overwrite behavior, size limits, and extraction options
- **Modern UI**: Clean, responsive interface with light/dark/system theme support

## Installation

### macOS

1. Download the latest `.dmg` from the [Releases](https://github.com/yourusername/unarchiver/releases) page
2. Open the DMG and drag Unarchiver to your Applications folder
3. Launch Unarchiver from Applications

### Finder Quick Action (Optional)

To enable the "Extract Here" Quick Action in Finder:

1. Open Terminal
2. Navigate to the Quick Action directory:
   ```bash
   cd /Applications/unarchiver.app/Contents/Resources/quick-action
   ```
3. Run the installer script:
   ```bash
   ./install-quick-action.sh
   ```
4. Restart Finder (optional but recommended):
   ```bash
   killall Finder
   ```

**Usage**: Right-click any archive file(s) in Finder → Services → Extract Here

To uninstall the Quick Action:
```bash
cd /Applications/unarchiver.app/Contents/Resources/quick-action
./uninstall-quick-action.sh
```

## Usage

### GUI Application

1. **Launch the app** from Applications
2. **Browse and select archives** using the file explorer
3. **Extract archives** by:
   - Clicking the "Extract" button to extract to the default location (same directory as archive)
   - Using the dropdown arrow next to "Extract" to choose a custom output folder
   - Right-clicking archives in the file explorer for quick extraction
4. **Monitor progress** in the queue list
5. **Enter passwords** when prompted for encrypted archives

### Command Line Interface

The CLI tool is bundled with the application:

```bash
# Extract archives
/Applications/unarchiver.app/Contents/MacOS/cli extract --out ~/Downloads archive.zip

# Extract multiple archives
/Applications/unarchiver.app/Contents/MacOS/cli extract --out ~/Downloads file1.zip file2.7z

# Probe archive metadata
/Applications/unarchiver.app/Contents/MacOS/cli probe archive.zip

# Probe with JSON output
/Applications/unarchiver.app/Contents/MacOS/cli probe --json archive.zip

# Extract with options
/Applications/unarchiver.app/Contents/MacOS/cli extract \
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
- Navigate using breadcrumb navigation or up/back buttons
- Archives are highlighted with special icons
- Right-click context menu for quick actions

### Archive Preview
- View archive contents in a tree structure
- See file/folder counts and total size
- Expand/collapse folders to explore structure
- Identify password-protected archives before extraction
- Quick metadata display (format, file count, size)

### Extraction Options
- **Quick Extract**: Click the main "Extract" button to extract to default location
- **Custom Location**: Use the dropdown arrow to choose a specific output folder
- **Context Menu**: Right-click any archive for extraction options
- **Keyboard Shortcut**: Cmd+E (Mac) / Ctrl+E (Windows) to extract selected archive

### Queue Management
- View all active and completed extractions
- Real-time progress tracking with file names and bytes written
- Cancel ongoing extractions
- Review extraction statistics (files extracted, duration, etc.)

## Settings

Configure extraction behavior in the Settings panel:

- **Overwrite Mode**: Choose how to handle existing files (replace, skip, or rename)
  - **Replace**: Overwrite existing files
  - **Skip**: Keep existing files, skip new ones
  - **Rename**: Add numbers to create unique filenames (e.g., file (1).txt)
- **Size Limit**: Set maximum extraction size (default: 20 GB)
- **Strip Components**: Remove leading path components from extracted files
- **Allow Symlinks**: Enable/disable symbolic link extraction (disabled by default for security)
- **Allow Hardlinks**: Enable/disable hard link extraction (disabled by default for security)
- **Theme**: Choose between light, dark, or system theme

**Note**: When using "Extract to custom folder", the overwrite mode is automatically set to "replace" for convenience.

## Supported Formats

| Format | Extensions | Password Support | Multi-Part Support |
|--------|-----------|------------------|-------------------|
| ZIP | .zip | ✓ | ⚠️ Limited |
| 7-Zip | .7z | ✓ | ⚠️ Limited |
| RAR | .rar, .part1.rar, .r00 | ✓ | ✅ Full |
| TAR | .tar | ✗ | N/A |
| GZIP | .gz, .tgz | ✗ | N/A |
| BZIP2 | .bz2, .tbz2 | ✗ | N/A |
| XZ | .xz, .txz | ✗ | N/A |
| ISO | .iso | ✗ | N/A |

**Multi-Part Archive Notes:**
- **RAR**: Full support for `.part1.rar`, `.part01.rar`, `.r00`, `.r01`, etc. Select any part and extraction will automatically start from the first part.
- **7-Zip/ZIP**: Limited support. Multi-part `.7z.001`/`.zip.001` archives are detected but may require external tools to combine parts before extraction.

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
- [Bun](https://bun.sh/) (or npm/yarn)
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
├── crates/
│   ├── extractor/     # Core extraction library
│   └── cli/           # Command-line interface
├── src/               # React frontend
├── src-tauri/         # Tauri backend
└── quick-action/      # Finder Quick Action workflow
```

## Troubleshooting

### Quick Action doesn't appear in Finder

1. Ensure the Quick Action is installed: `ls ~/Library/Services/`
2. Restart Finder: `killall Finder`
3. Log out and back in
4. Check System Preferences → Extensions → Finder to enable the service

### App won't open (Gatekeeper warning)

If you see a warning about the app being from an unidentified developer:
1. Right-click the app in Finder
2. Select "Open"
3. Click "Open" in the dialog

For signed releases, this shouldn't be necessary.

### Extraction fails with "Permission Denied"

Ensure you have write permissions to the output directory. Try selecting a different output location.

### Multi-part archives not working

**RAR archives**: Should work automatically. Select any part (e.g., `.part3.rar`) and the app will find and use the first part.

**7-Zip/ZIP archives**: Multi-part `.7z.001`/`.zip.001` archives are not fully supported. You can:
1. Use an external tool (like 7-Zip or The Unarchiver) to combine the parts first
2. Use the command line: `cat file.7z.* > combined.7z` then extract `combined.7z`

### Archive appears corrupted

1. Verify the archive is complete and not corrupted using another tool
2. For multi-part archives, ensure all parts are in the same directory
3. Check that you have the first part (`.part1.rar`, `.001`, etc.)
4. Try extracting with verbose logging enabled (check console output)

## License

[MIT License](LICENSE)

## Recommended IDE Setup

- [VS Code](https://code.visualstudio.com/) + [Tauri](https://marketplace.visualstudio.com/items?itemName=tauri-apps.tauri-vscode) + [rust-analyzer](https://marketplace.visualstudio.com/items?itemName=rust-lang.rust-analyzer)
