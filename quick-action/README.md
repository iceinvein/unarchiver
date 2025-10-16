# Finder Quick Action - Extract Here

This directory contains the macOS Finder Quick Action workflow for extracting archives directly from Finder.

## What is a Quick Action?

Quick Actions (formerly Services) are system-wide shortcuts that appear in the Finder context menu. The "Extract Here" Quick Action allows you to right-click on archive files in Finder and extract them to the same directory without opening the main application.

## Installation

### Automatic Installation

Run the installer script:

```bash
./install-quick-action.sh
```

This will:
1. Copy the workflow to `~/Library/Services/`
2. Make it available in Finder's Services menu
3. Display usage instructions

### Manual Installation

1. Copy the `Extract Here.workflow` folder to `~/Library/Services/`
2. Restart Finder: `killall Finder`
3. Log out and back in (if needed)

## Usage

1. In Finder, select one or more archive files
2. Right-click and navigate to: **Services → Extract Here**
3. The archives will be extracted to their parent directory
4. You'll see a notification for each archive processed

## Supported Formats

- ZIP (.zip)
- 7-Zip (.7z)
- RAR (.rar)
- TAR (.tar, .tgz, .tbz2, .txz)
- GZIP (.gz)
- BZIP2 (.bz2)
- XZ (.xz)
- ISO (.iso)

## How It Works

The Quick Action workflow:
1. Receives selected files from Finder
2. Filters for supported archive formats
3. Calls the Unarchiver CLI tool for each archive
4. Extracts to the same directory as the source file
5. Uses "rename" mode to avoid overwriting existing files
6. Shows macOS notifications for success/failure

## Requirements

- macOS 10.14 (Mojave) or later
- Unarchiver app installed in `/Applications/`
- The CLI binary must be present at: `/Applications/unarchiver.app/Contents/MacOS/cli`

## Uninstallation

Run the uninstaller script:

```bash
./uninstall-quick-action.sh
```

Or manually remove:

```bash
rm -rf ~/Library/Services/Extract\ Here.workflow
killall Finder
```

## Troubleshooting

### Quick Action doesn't appear

1. Check if it's installed:
   ```bash
   ls ~/Library/Services/
   ```

2. Restart Finder:
   ```bash
   killall Finder
   ```

3. Check System Preferences:
   - Go to System Preferences → Extensions → Finder
   - Ensure "Extract Here" is enabled

4. Log out and back in

### "CLI not found" error

The Quick Action expects the CLI binary at:
```
/Applications/unarchiver.app/Contents/MacOS/cli
```

If you installed the app in a different location, you'll need to edit the workflow:
1. Open `Extract Here.workflow` in Automator
2. Update the `CLI_PATH` variable in the shell script
3. Save and reinstall

### Permission issues

If you get permission errors:
1. Ensure the installer script is executable:
   ```bash
   chmod +x install-quick-action.sh
   ```

2. Check that you have write access to `~/Library/Services/`

## Customization

You can customize the Quick Action by editing the workflow in Automator:

1. Open the workflow:
   ```bash
   open ~/Library/Services/Extract\ Here.workflow
   ```

2. Modify the shell script to change:
   - CLI path
   - Overwrite mode (replace, skip, rename)
   - Notification messages
   - Error handling

3. Save and the changes will take effect immediately

## Technical Details

### Workflow Structure

```
Extract Here.workflow/
├── Contents/
│   ├── Info.plist          # Service metadata and file type associations
│   └── document.wflow      # Automator workflow definition
```

### Shell Script

The workflow runs a bash script that:
- Validates the CLI binary exists
- Iterates through selected files
- Extracts each archive using the CLI
- Shows notifications via `osascript`

### File Type Associations

The workflow is registered for these UTIs (Uniform Type Identifiers):
- `public.zip-archive`
- `org.7-zip.7-zip-archive`
- `com.rarlab.rar-archive`
- `public.tar-archive`
- `org.gnu.gnu-tar-archive`
- `org.gnu.gnu-zip-archive`
- `public.bzip2-archive`
- `org.tukaani.xz-archive`
- `public.iso-image`

## License

Same as the main Unarchiver project (MIT License).
