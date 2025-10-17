#!/bin/bash

# Mac App Store Signing Script for Unarchiver
# Usage: ./sign_for_mas.sh

set -e

# Configuration
APP_NAME="Unarchiver"
APP_PATH="target/release/bundle/macos/${APP_NAME}.app"
ENTITLEMENTS="src-tauri/entitlements.mas.plist"
CHILD_ENTITLEMENTS="src-tauri/entitlements.mas.inherit.plist"
PROVISIONING_PROFILE="Unarchiver.provisionprofile"

# You need to replace these with your actual certificate identities
# Find them with: security find-identity -v -p codesigning
# For Mac App Store, use "3rd Party Mac Developer" certificates
INSTALLER_IDENTITY="3rd Party Mac Developer Installer: Dik Rana (UT6J7B9B3Z)"
APP_IDENTITY="Apple Distribution: Dik Rana (UT6J7B9B3Z)"

echo "🔐 Signing ${APP_NAME} for Mac App Store..."
echo ""

# Check if app exists
if [ ! -d "$APP_PATH" ]; then
    echo "❌ Error: App not found at $APP_PATH"
    echo "Run 'bun run tauri build' first"
    exit 1
fi

# Sign frameworks and helpers first (inside-out signing)
echo "📦 Signing frameworks and helpers..."

# Sign all frameworks
find "$APP_PATH/Contents/Frameworks" -type d -name "*.framework" -o -name "*.dylib" 2>/dev/null | while read framework; do
    echo "  Signing: $framework"
    codesign --force --sign "$APP_IDENTITY" \
        --entitlements "$CHILD_ENTITLEMENTS" \
        --timestamp \
        "$framework" 2>/dev/null || true
done

# Sign XPC services if they exist
if [ -d "$APP_PATH/Contents/XPCServices" ]; then
    find "$APP_PATH/Contents/XPCServices" -type d -name "*.xpc" | while read xpc; do
        echo "  Signing XPC: $xpc"
        codesign --force --sign "$APP_IDENTITY" \
            --entitlements "$CHILD_ENTITLEMENTS" \
            --timestamp \
            "$xpc"
    done
fi

# Sign all helper executables in MacOS folder
echo ""
echo "🔧 Signing helper executables..."
find "$APP_PATH/Contents/MacOS" -type f -perm +111 | while read executable; do
    # Skip the main app executable (we'll sign it separately)
    if [ "$(basename "$executable")" != "$APP_NAME" ]; then
        helper_name=$(basename "$executable")
        echo "  Signing helper: $helper_name"
        codesign --force --sign "$APP_IDENTITY" \
            --entitlements "$CHILD_ENTITLEMENTS" \
            --identifier "com.dikrana.unarchiver.$helper_name" \
            --timestamp \
            "$executable"
    fi
done

# Embed provisioning profile
echo ""
echo "📄 Embedding provisioning profile..."
if [ -f "$PROVISIONING_PROFILE" ]; then
    cp "$PROVISIONING_PROFILE" "$APP_PATH/Contents/embedded.provisionprofile"
    echo "  ✓ Provisioning profile embedded"
else
    echo "  ⚠️  Warning: Provisioning profile not found at $PROVISIONING_PROFILE"
    echo "  Download it from https://developer.apple.com/account/resources/profiles/list"
fi

# Sign the main executable
echo ""
echo "🎯 Signing main application..."
codesign --force --sign "$APP_IDENTITY" \
    --entitlements "$ENTITLEMENTS" \
    --identifier "com.dikrana.unarchiver" \
    --timestamp \
    "$APP_PATH"

# Verify signature
echo ""
echo "✅ Verifying signature..."
codesign --verify --deep --strict --verbose=2 "$APP_PATH"

echo ""
echo "📋 Signature details:"
codesign -d -vvv --entitlements - "$APP_PATH" 2>&1 | grep -E "(Identifier|TeamIdentifier)"

echo ""
echo "✅ App signed successfully!"
echo ""
echo "Next steps:"
echo "1. Create installer package:"
echo "   productbuild --component \"$APP_PATH\" /Applications \\"
echo "     --sign \"$INSTALLER_IDENTITY\" \\"
echo "     ${APP_NAME}.pkg"
echo ""
echo "2. Verify package:"
echo "   pkgutil --check-signature ${APP_NAME}.pkg"
echo ""
echo "3. Upload to App Store Connect via Transporter"
