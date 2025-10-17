# Mac App Store Submission Guide

Complete guide for building, signing, and submitting Unarchiver to the Mac App Store.

## Prerequisites

### 1. Apple Developer Account
- Enroll in the Apple Developer Program ($99/year)
- Access at https://developer.apple.com/account

### 2. Required Certificates

Create and download these certificates from https://developer.apple.com/account/resources/certificates/list:

- **Apple Distribution** (for signing the app)
- **Mac Installer Distribution** (for signing the package)

Install certificates by double-clicking the downloaded files.

Verify installation:
```bash
security find-identity -v
```

You should see:
- `Apple Distribution: Your Name (TEAM_ID)`
- `3rd Party Mac Developer Installer: Your Name (TEAM_ID)`

### 3. App ID

Create an App ID at https://developer.apple.com/account/resources/identifiers/list:
- Type: App IDs
- Bundle ID: `com.dikrana.unarchiver` (explicit)
- Capabilities: Enable any required capabilities (File Access, etc.)

### 4. Provisioning Profile

Create a provisioning profile at https://developer.apple.com/account/resources/profiles/list:
- Type: **Mac App Store** (under Distribution)
- App ID: Select `com.dikrana.unarchiver`
- Certificate: Select your **Apple Distribution** certificate
- Download and save as `Unarchiver_Mac_App_Store.provisionprofile` in project root

## Build Process

### Step 1: Build the App
```bash
# Clean previous builds
cargo clean

# Build release version
cargo build --release
```

### Step 2: Sign for Mac App Store
```bash
./sign_for_mas.sh
```

This script:
- Signs all frameworks and dylibs
- Signs helper executables (export_types, open_file)
- Embeds the provisioning profile
- Signs the main app bundle
- Verifies the signature

### Step 3: Create Installer Package
```bash
./create_mas_package.sh
```

This creates `Unarchiver.pkg` signed with your Mac Installer Distribution certificate.

## Submission

### Using Transporter (Recommended)

1. Open **Transporter** app (download from Mac App Store if needed)
2. Sign in with your Apple ID
3. Drag and drop `Unarchiver.pkg` into Transporter
4. Click **Deliver**
5. Wait for validation and upload to complete

### Using Command Line

```bash
xcrun altool --upload-app --type macos --file Unarchiver.pkg \
  --username YOUR_APPLE_ID \
  --password @keychain:AC_PASSWORD
```

Note: You need to create an app-specific password at https://appleid.apple.com

## App Store Connect

After successful upload:

1. Go to https://appstoreconnect.apple.com
2. Select your app
3. Go to **TestFlight** tab to test the build
4. Or go to **App Store** tab to submit for review

### TestFlight Testing
- Build will appear in TestFlight within 10-15 minutes after processing
- Add internal testers to test before public release
- External testing requires App Review approval

### App Store Submission
1. Create a new version
2. Select the uploaded build
3. Fill in app information, screenshots, description
4. Submit for review
5. Review typically takes 1-3 days

## Troubleshooting

### Signature Verification Failed
```bash
# Check signature details
codesign -d -vvv --entitlements - target/release/bundle/macos/Unarchiver.app

# Verify deep signature
codesign --verify --deep --strict target/release/bundle/macos/Unarchiver.app
```

### Package Verification Failed
```bash
# Check package signature
pkgutil --check-signature Unarchiver.pkg

# Expand package to inspect contents
pkgutil --expand Unarchiver.pkg expanded_pkg
```

### Missing Provisioning Profile
Ensure `Unarchiver_Mac_App_Store.provisionprofile` is in the project root before running `sign_for_mas.sh`.

### Wrong Certificate Type
Make sure you're using the correct certificates for App Store distribution:
- **Apple Distribution** = For signing the app bundle (App Store)
- **Mac Installer Distribution** = For signing the installer package (App Store)
- **Developer ID** = For distribution outside the App Store (not for MAS)

## Files Overview

### Configuration Files
- `src-tauri/tauri.conf.json` - App configuration, bundle ID, version
- `src-tauri/entitlements.mas.plist` - App Store entitlements and sandbox permissions
- `src-tauri/entitlements.mas.inherit.plist` - Entitlements for helper executables
- `Unarchiver_Mac_App_Store.provisionprofile` - Provisioning profile (not in git)

### Scripts
- `sign_for_mas.sh` - Signs app bundle for Mac App Store
- `create_mas_package.sh` - Creates installer package

### Build Artifacts
- `target/release/bundle/macos/Unarchiver.app` - Signed app bundle
- `Unarchiver.pkg` - Installer package for submission

## Version Updates

When releasing a new version:

1. Update version in `src-tauri/tauri.conf.json`
2. Rebuild: `cargo build --release`
3. Re-sign: `./sign_for_mas.sh`
4. Create package: `./create_mas_package.sh`
5. Upload via Transporter
6. Create new version in App Store Connect

## Important Notes

- **Minimum macOS Version**: 12.0 (arm64 only)
- **Architecture**: Apple Silicon only (arm64)
- **Sandbox**: Enabled (required for App Store)
- **Provisioning Profile**: Must be embedded in app bundle
- **Certificate Expiration**: Renew certificates annually
- **Team ID**: UT6J7B9B3Z

## Resources

- [App Store Review Guidelines](https://developer.apple.com/app-store/review/guidelines/)
- [App Sandbox Documentation](https://developer.apple.com/documentation/security/app_sandbox)
- [Distributing Mac Apps](https://developer.apple.com/documentation/xcode/distributing-your-app-for-beta-testing-and-releases)
- [Transporter Help](https://help.apple.com/itc/transporteruserguide/)
