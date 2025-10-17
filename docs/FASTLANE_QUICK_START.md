# Fastlane Quick Start Guide

## Setup (One Time)

1. **Install Fastlane:**
   ```bash
   brew install fastlane
   ```

2. **Configure credentials:**
   ```bash
   cp .env.example .env
   ```
   
   Edit `.env` and add:
   - Your Apple ID email
   - Team IDs (find in App Store Connect)
   - App-specific password (generate at appleid.apple.com)

3. **Add screenshots:**
   - Take 5 screenshots of your app
   - Save as `1.png` through `5.png` in `fastlane/screenshots/en-US/`
   - Recommended size: 1440x900 or larger

4. **Update URLs:**
   Edit these files with your actual URLs:
   - `fastlane/metadata/en-US/marketing_url.txt`
   - `fastlane/metadata/en-US/support_url.txt`
   - `fastlane/metadata/en-US/privacy_url.txt`

## Release Workflow

### First Release

```bash
# 1. Build and sign
bun run tauri build
./sign_for_mas.sh
./create_mas_package.sh

# 2. Upload everything to App Store Connect
fastlane release

# 3. Check App Store Connect web interface to verify

# 4. Submit for review
fastlane submit
```

### Updates

```bash
# 1. Update version in package.json and tauri.conf.json

# 2. Update release notes
# Edit: fastlane/metadata/en-US/release_notes.txt

# 3. Build, sign, and upload
bun run tauri build
./sign_for_mas.sh
./create_mas_package.sh
fastlane release

# 4. Submit for review
fastlane submit
```

## Available Commands

| Command | Description |
|---------|-------------|
| `fastlane metadata` | Upload only metadata and screenshots |
| `fastlane upload` | Upload only the .pkg build |
| `fastlane release` | Upload everything (metadata + build) |
| `fastlane submit` | Submit for App Store review |

## Tips

- Always run `fastlane metadata` first to test your setup
- Check App Store Connect after each upload
- Don't run `fastlane submit` until you've verified everything
- Keep `.env` file private (it's in .gitignore)

## Troubleshooting

**Authentication failed:**
- Check your Apple ID in `.env`
- Verify app-specific password is correct
- Try: `fastlane fastlane-credentials add --username your@email.com`

**Build upload failed:**
- Ensure `Unarchiver.pkg` exists in project root
- Verify package is properly signed: `pkgutil --check-signature Unarchiver.pkg`

**Metadata validation errors:**
- Check App Store Connect for specific error messages
- Ensure all required fields are filled
- Verify URLs are valid and accessible

For more details, see `fastlane/README.md`
