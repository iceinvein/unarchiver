#!/bin/bash

# Create Mac App Store Package for Unarchiver
# Usage: ./create_mas_package.sh

set -e

APP_NAME="Unarchiver"
APP_PATH="target/release/bundle/macos/${APP_NAME}.app"
PKG_NAME="${APP_NAME}.pkg"

# Replace with your actual installer certificate identity
# For Mac App Store, use "3rd Party Mac Developer Installer"
INSTALLER_IDENTITY="3rd Party Mac Developer Installer: Dik Rana (UT6J7B9B3Z)"

echo "📦 Creating Mac App Store package..."
echo ""

# Check if signed app exists
if [ ! -d "$APP_PATH" ]; then
    echo "❌ Error: App not found at $APP_PATH"
    echo "Run './sign_for_mas.sh' first"
    exit 1
fi

# Verify app is signed
echo "🔍 Verifying app signature..."
if ! codesign --verify --deep --strict "$APP_PATH" 2>/dev/null; then
    echo "❌ Error: App is not properly signed"
    echo "Run './sign_for_mas.sh' first"
    exit 1
fi

# Create package
echo "📦 Building installer package..."
productbuild --component "$APP_PATH" /Applications \
    --sign "$INSTALLER_IDENTITY" \
    "$PKG_NAME"

# Verify package
echo ""
echo "✅ Verifying package signature..."
pkgutil --check-signature "$PKG_NAME"

echo ""
echo "✅ Package created successfully: $PKG_NAME"
echo ""
echo "Next steps:"
echo "1. Open Transporter app"
echo "2. Sign in with your Apple ID"
echo "3. Drag and drop $PKG_NAME"
echo "4. Click 'Deliver'"
echo ""
echo "Or use command line:"
echo "xcrun altool --upload-app --type macos --file $PKG_NAME \\"
echo "  --username YOUR_APPLE_ID --password @keychain:AC_PASSWORD"
