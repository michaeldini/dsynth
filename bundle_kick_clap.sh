#!/bin/bash
# Bundle DSynth Kick CLAP Plugin for macOS
# This script creates a CLAP plugin bundle that can be loaded by DAWs

set -e

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

echo -e "${GREEN}Building DSynth Kick CLAP plugin...${NC}"

# Build the CLAP plugin (dynamic library)
echo -e "${YELLOW}Compiling...${NC}"
cargo build --release --lib --features kick-clap --no-default-features

# Bundle name and paths
BUNDLE_NAME="DSynthKick.clap"
BUNDLE_DIR="target/bundled"
BUNDLE_PATH="$BUNDLE_DIR/$BUNDLE_NAME"
CONTENTS_DIR="$BUNDLE_PATH/Contents"
MACOS_DIR="$CONTENTS_DIR/MacOS"

# Create bundle directory structure
echo -e "${YELLOW}Creating bundle structure...${NC}"
rm -rf "$BUNDLE_PATH"
mkdir -p "$MACOS_DIR"

# Copy the dylib to the bundle
echo -e "${YELLOW}Copying plugin binary...${NC}"
cp "target/release/libdsynth.dylib" "$MACOS_DIR/DSynthKick"

# Create Info.plist
echo -e "${YELLOW}Creating Info.plist...${NC}"
cat > "$CONTENTS_DIR/Info.plist" << EOF
<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0">
<dict>
    <key>CFBundleDevelopmentRegion</key>
    <string>English</string>
    <key>CFBundleExecutable</key>
    <string>DSynthKick</string>
    <key>CFBundleIdentifier</key>
    <string>com.dsynth.kick</string>
    <key>CFBundleInfoDictionaryVersion</key>
    <string>6.0</string>
    <key>CFBundleName</key>
    <string>DSynth Kick</string>
    <key>CFBundlePackageType</key>
    <string>BNDL</string>
    <key>CFBundleShortVersionString</key>
    <string>0.3.0</string>
    <key>CFBundleVersion</key>
    <string>0.3.0</string>
    <key>CFBundleSignature</key>
    <string>????</string>
</dict>
</plist>
EOF

# Create PkgInfo file
echo -e "${YELLOW}Creating PkgInfo...${NC}"
echo -n "BNDL????" > "$CONTENTS_DIR/PkgInfo"

# Verify the bundle
echo -e "${YELLOW}Verifying bundle...${NC}"
if [ -f "$MACOS_DIR/DSynthKick" ]; then
    echo -e "${GREEN}✓ Plugin binary present${NC}"
else
    echo -e "${RED}✗ Plugin binary missing!${NC}"
    exit 1
fi

if [ -f "$CONTENTS_DIR/Info.plist" ]; then
    echo -e "${GREEN}✓ Info.plist present${NC}"
else
    echo -e "${RED}✗ Info.plist missing!${NC}"
    exit 1
fi

# Print success message
echo ""
echo -e "${GREEN}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"
echo -e "${GREEN}✓ DSynth Kick CLAP plugin built successfully!${NC}"
echo -e "${GREEN}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"
echo ""
echo -e "${YELLOW}Bundle location:${NC}"
echo "  $BUNDLE_PATH"
echo ""
echo -e "${YELLOW}To install (macOS):${NC}"
echo "  cp -r \"$BUNDLE_PATH\" ~/Library/Audio/Plug-Ins/CLAP/"
echo ""
echo -e "${YELLOW}To test:${NC}"
echo "  1. Copy bundle to CLAP plugins folder"
echo "  2. Open your DAW (Bitwig, Reaper, etc.)"
echo "  3. Scan for new plugins"
echo "  4. Load 'DSynth Kick' as an instrument"
echo ""
