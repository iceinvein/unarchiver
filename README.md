# Unarchiver

A modern, safe, and efficient archive extraction application for macOS built with Tauri 2.x and React.

## Features

- **Multiple Format Support**: Extract ZIP, 7Z, RAR, TAR, GZ, BZ2, XZ, ISO, and more
- **Drag & Drop**: Simply drag archives into the app window
- **Queue Management**: Process multiple archives with real-time progress tracking
- **Password Support**: Handle password-protected archives with secure prompts
- **macOS Integration**: 
  - Double-click archives to open with Unarchiver
  - Finder Quick Action for "Extract Here" functionality
- **Safety First**: Built-in protection against zip-slip and path traversal attacks
- **Customizable Settings**: Control overwrite behavior, size limits, and extraction options

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
2. **Add archives** by:
   - Dragging and dropping files into the window
   - Clicking "Select Archives" to browse
   - Double-clicking archive files in Finder (if file associations are set)
3. **Choose output directory** when prompted
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

## Settings

Configure extraction behavior in the Settings panel:

- **Overwrite Mode**: Choose how to handle existing files (replace, skip, or rename)
- **Size Limit**: Set maximum extraction size (default: 20 GB)
- **Strip Components**: Remove leading path components from extracted files
- **Allow Symlinks**: Enable/disable symbolic link extraction (disabled by default for security)
- **Allow Hardlinks**: Enable/disable hard link extraction (disabled by default for security)
- **Theme**: Choose between light, dark, or system theme

## Supported Formats

| Format | Extensions | Password Support |
|--------|-----------|------------------|
| ZIP | .zip | ✓ |
| 7-Zip | .7z | ✓ |
| RAR | .rar | ✓ (read-only) |
| TAR | .tar | ✗ |
| GZIP | .gz, .tgz | ✗ |
| BZIP2 | .bz2, .tbz2 | ✗ |
| XZ | .xz, .txz | ✗ |
| ISO | .iso | ✗ |

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

## License

[MIT License](LICENSE)

## Recommended IDE Setup

- [VS Code](https://code.visualstudio.com/) + [Tauri](https://marketplace.visualstudio.com/items?itemName=tauri-apps.tauri-vscode) + [rust-analyzer](https://marketplace.visualstudio.com/items?itemName=rust-lang.rust-analyzer)
