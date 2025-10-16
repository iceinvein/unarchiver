#!/bin/bash
set -euo pipefail

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

# Quick Action name
QUICK_ACTION_NAME="Extract Here.workflow"
SERVICES_DIR="$HOME/Library/Services"
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
SOURCE_WORKFLOW="$SCRIPT_DIR/$QUICK_ACTION_NAME"
DEST_WORKFLOW="$SERVICES_DIR/$QUICK_ACTION_NAME"

echo "=========================================="
echo "Unarchiver Quick Action Installer"
echo "=========================================="
echo ""

# Check if source workflow exists
if [ ! -d "$SOURCE_WORKFLOW" ]; then
    echo -e "${RED}Error: Quick Action workflow not found at:${NC}"
    echo "  $SOURCE_WORKFLOW"
    echo ""
    echo "Please ensure you're running this script from the quick-action directory."
    exit 1
fi

# Create Services directory if it doesn't exist
if [ ! -d "$SERVICES_DIR" ]; then
    echo -e "${YELLOW}Creating Services directory...${NC}"
    mkdir -p "$SERVICES_DIR"
fi

# Check if Quick Action already exists
if [ -d "$DEST_WORKFLOW" ]; then
    echo -e "${YELLOW}Quick Action already exists. Removing old version...${NC}"
    rm -rf "$DEST_WORKFLOW"
fi

# Copy the workflow
echo "Installing Quick Action..."
cp -R "$SOURCE_WORKFLOW" "$SERVICES_DIR/"

if [ $? -eq 0 ]; then
    echo -e "${GREEN}✓ Quick Action installed successfully!${NC}"
    echo ""
    echo "The 'Extract Here' Quick Action has been installed to:"
    echo "  $DEST_WORKFLOW"
    echo ""
    echo "=========================================="
    echo "How to use:"
    echo "=========================================="
    echo "1. In Finder, right-click on one or more archive files"
    echo "2. Navigate to: Services > Extract Here"
    echo "3. The archives will be extracted to their parent directory"
    echo ""
    echo "Supported formats: ZIP, 7Z, RAR, TAR, GZ, BZ2, XZ, ISO"
    echo ""
    echo -e "${YELLOW}Note:${NC} You may need to restart Finder or log out and back in"
    echo "for the Quick Action to appear in the Services menu."
    echo ""
    echo "To restart Finder, run:"
    echo "  killall Finder"
    echo ""
else
    echo -e "${RED}✗ Installation failed${NC}"
    exit 1
fi
