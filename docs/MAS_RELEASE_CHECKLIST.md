# Mac App Store Release Checklist for Unarchiver

## ✅ Pre-Release Setup (Completed)

- [x] Version bumped to 1.0.0 in package.json and tauri.conf.json
- [x] Bundle ID set: `com.dikrana.unarchiver`
- [x] Entitlements files created (mas.plist and mas.inherit.plist)
- [x] Signing scripts created and made executable
- [x] Info.plist configured with copyright and version info

## 📋 Before Building

- [ ] Update certificate identities in `sign_for_mas.sh` and `create_mas_package.sh`
  - Find your identities: `security find-identity -v -p codesigning`
  - Replace "Your Name (TEAMID)" with actual values
- [ ] Ensure you have valid certificates:
  - "3rd Party Mac Developer Application" certificate
  - "3rd Party Mac Developer Installer" certificate
- [ ] Create provisioning profile in Apple Developer portal matching bundle ID

## 🏗️ Build Process

1. **Build the app**
   ```bash
   bun run tauri build
   ```

2. **Sign the app**
   ```bash
   ./sign_for_mas.sh
   ```

3. **Create installer package**
   ```bash
   ./create_mas_package.sh
   ```

## 🍎 App Store Connect Setup

- [ ] Create app record in App Store Connect
  - Name: Unarchiver
  - Bundle ID: com.dikrana.unarchiver
  - SKU: (choose unique identifier)
  - Platform: macOS

- [ ] Complete app information:
  - [ ] Category: Utilities
  - [ ] Subcategory: File Management
  - [ ] Age Rating: 4+
  - [ ] Privacy Policy URL (if collecting data)
  - [ ] Support URL

- [ ] Add screenshots (5 required, 1440×900 or larger)
  - Main window with file browser
  - Archive preview
  - Extraction in progress
  - Settings panel
  - Completed extraction

- [ ] Write app description highlighting:
  - Multi-format support (ZIP, 7Z, RAR, TAR, etc.)
  - Multi-part RAR support
  - Modern UI with dark mode
  - Security features (sandbox, path validation)
  - File associations

- [ ] Keywords (max 30):
  - unarchive, extract, zip, rar, 7z, tar, decompress, unzip, archive, file manager

- [ ] What's New (v1.0.0):
  - Initial release
  - Support for 10+ archive formats
  - Multi-part RAR extraction
  - Modern macOS interface
  - Secure sandboxed extraction

## 📤 Upload & Submit

- [ ] Upload via Transporter:
  1. Open Transporter app
  2. Sign in with Apple ID
  3. Drag Unarchiver.pkg
  4. Click "Deliver"
  5. Wait for processing (5-15 minutes)

- [ ] In App Store Connect:
  - [ ] Select uploaded build
  - [ ] Add export compliance info (no encryption beyond standard HTTPS)
  - [ ] Submit for review

## 🔍 Review Preparation

- [ ] Test app thoroughly in sandbox mode
- [ ] Verify all file associations work
- [ ] Test password-protected archives
- [ ] Test multi-part RAR files
- [ ] Ensure no crashes or hangs
- [ ] Verify all UI strings are correct

## ⚠️ Important Notes

### Entitlements Justification
If Apple asks why you need specific entitlements:

- **files.user-selected.read-only**: Required to read archive files selected by user
- **files.user-selected.read-write**: Required to write extracted files to user-selected locations
- **files.downloads.read-write**: Common location for archive extraction
- **network.client**: Reserved for future features (can be removed if not needed)

### Known Limitations for MAS Build

- Quick Action/Finder extension may not work in sandboxed MAS build
- Consider removing Quick Action references from MAS version
- Auto-updater must be disabled (MAS handles updates)

### If Rejected

Common rejection reasons:
1. **Overly broad entitlements**: Remove network.client if not used
2. **Missing functionality**: Ensure app works fully in sandbox
3. **Crashes**: Test thoroughly before submission
4. **Misleading metadata**: Ensure screenshots match actual functionality

## 📊 Post-Release

- [ ] Monitor App Store Connect Analytics
- [ ] Respond to user reviews
- [ ] Track crash reports
- [ ] Plan updates based on feedback

## 🔄 For Future Updates

1. Increment version in both package.json and tauri.conf.json
2. Update "What's New" in App Store Connect
3. Rebuild, re-sign, re-package
4. Upload and submit

---

**Current Status**: Ready for certificate configuration and build
**Next Step**: Update certificate identities in signing scripts
