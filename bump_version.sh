#!/bin/bash

# Version Bump and Build Script
# Usage: ./bump_version.sh [major|minor|patch]

set -e

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

# Default to patch if no argument provided
BUMP_TYPE=${1:-patch}

# Validate bump type
if [[ ! "$BUMP_TYPE" =~ ^(major|minor|patch)$ ]]; then
    echo -e "${RED}Error: Invalid bump type. Use 'major', 'minor', or 'patch'${NC}"
    exit 1
fi

echo -e "${GREEN}Starting version bump (${BUMP_TYPE})...${NC}"

# Get current version from package.json
CURRENT_VERSION=$(cat package.json | grep '"version"' | head -1 | sed 's/.*"version": "\(.*\)".*/\1/')
echo -e "Current version: ${YELLOW}${CURRENT_VERSION}${NC}"

# Calculate new version
IFS='.' read -r -a VERSION_PARTS <<< "$CURRENT_VERSION"
MAJOR="${VERSION_PARTS[0]}"
MINOR="${VERSION_PARTS[1]}"
PATCH="${VERSION_PARTS[2]}"

case $BUMP_TYPE in
    major)
        MAJOR=$((MAJOR + 1))
        MINOR=0
        PATCH=0
        ;;
    minor)
        MINOR=$((MINOR + 1))
        PATCH=0
        ;;
    patch)
        PATCH=$((PATCH + 1))
        ;;
esac

NEW_VERSION="${MAJOR}.${MINOR}.${PATCH}"
echo -e "New version: ${GREEN}${NEW_VERSION}${NC}"

# Update package.json
echo -e "\n${YELLOW}Updating package.json...${NC}"
sed -i '' "s/\"version\": \".*\"/\"version\": \"${NEW_VERSION}\"/" package.json

# Update src-tauri/Cargo.toml
echo -e "${YELLOW}Updating src-tauri/Cargo.toml...${NC}"
sed -i '' "s/^version = \".*\"/version = \"${NEW_VERSION}\"/" src-tauri/Cargo.toml

# Update src-tauri/tauri.conf.json
echo -e "${YELLOW}Updating src-tauri/tauri.conf.json...${NC}"
sed -i '' "s/\"version\": \".*\"/\"version\": \"${NEW_VERSION}\"/" src-tauri/tauri.conf.json

# Calculate bundle version (integer that increments)
# Get current bundle version from Info.plist
CURRENT_BUNDLE_VERSION=$(grep -A 1 "CFBundleVersion" src-tauri/Info.plist | tail -1 | sed 's/.*<string>\(.*\)<\/string>.*/\1/')
NEW_BUNDLE_VERSION=$((CURRENT_BUNDLE_VERSION + 1))

# Update Info.plist with new version and bundle version
echo -e "${YELLOW}Updating src-tauri/Info.plist...${NC}"
python3 << EOF
import re

with open('src-tauri/Info.plist', 'r') as f:
    content = f.read()

# Update CFBundleShortVersionString
content = re.sub(
    r'(<key>CFBundleShortVersionString</key>\s*<string>)[^<]*(</string>)',
    r'\g<1>${NEW_VERSION}\g<2>',
    content
)

# Update CFBundleVersion
content = re.sub(
    r'(<key>CFBundleVersion</key>\s*<string>)[^<]*(</string>)',
    r'\g<1>${NEW_BUNDLE_VERSION}\g<2>',
    content
)

with open('src-tauri/Info.plist', 'w') as f:
    f.write(content)
EOF

echo -e "Bundle version: ${YELLOW}${CURRENT_BUNDLE_VERSION}${NC} → ${GREEN}${NEW_BUNDLE_VERSION}${NC}"

# Update Cargo.lock
echo -e "${YELLOW}Updating Cargo.lock...${NC}"
cargo update -p unarchiver

echo -e "\n${GREEN}Version bumped to ${NEW_VERSION}${NC}"

# Ask if user wants to build
read -p "Do you want to build the project now? (y/n) " -n 1 -r
echo
if [[ $REPLY =~ ^[Yy]$ ]]; then
    echo -e "\n${GREEN}Building project...${NC}"
    bun tauri build
    echo -e "\n${GREEN}Build complete!${NC}"
fi

# Ask if user wants to commit
read -p "Do you want to commit the version bump? (y/n) " -n 1 -r
echo
if [[ $REPLY =~ ^[Yy]$ ]]; then
    git add package.json src-tauri/Cargo.toml src-tauri/tauri.conf.json src-tauri/Info.plist Cargo.lock
    git commit -m "chore: bump version to ${NEW_VERSION} (bundle: ${NEW_BUNDLE_VERSION})"
    echo -e "${GREEN}Changes committed!${NC}"
    
    read -p "Do you want to create a git tag? (y/n) " -n 1 -r
    echo
    if [[ $REPLY =~ ^[Yy]$ ]]; then
        git tag -a "v${NEW_VERSION}" -m "Release v${NEW_VERSION}"
        echo -e "${GREEN}Tag v${NEW_VERSION} created!${NC}"
        echo -e "${YELLOW}Don't forget to push: git push && git push --tags${NC}"
    fi
fi

echo -e "\n${GREEN}Done!${NC}"
