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
DEST_WORKFLOW="$SERVICES_DIR/$QUICK_ACTION_NAME"

echo "=========================================="
echo "Unarchiver Quick Action Uninstaller"
echo "=========================================="
echo ""

# Check if Quick Action exists
if [ ! -d "$DEST_WORKFLOW" ]; then
    echo -e "${YELLOW}Quick Action is not installed.${NC}"
    echo "Nothing to uninstall."
    exit 0
fi

# Remove the workflow
echo "Removing Quick Action..."
rm -rf "$DEST_WORKFLOW"

if [ $? -eq 0 ]; then
    echo -e "${GREEN}✓ Quick Action uninstalled successfully!${NC}"
    echo ""
    echo "The 'Extract Here' Quick Action has been removed from:"
    echo "  $SERVICES_DIR"
    echo ""
    echo -e "${YELLOW}Note:${NC} You may need to restart Finder or log out and back in"
    echo "for the changes to take effect."
    echo ""
    echo "To restart Finder, run:"
    echo "  killall Finder"
    echo ""
else
    echo -e "${RED}✗ Uninstallation failed${NC}"
    exit 1
fi
