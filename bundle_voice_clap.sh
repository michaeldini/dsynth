#!/bin/bash
# Bundle DSynth Voice Enhancer as a CLAP plugin for macOS

set -e

# Colors for output
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

echo -e "${YELLOW}Building DSynth Voice Enhancer CLAP plugin for macOS...${NC}"

# Build the library with voice-clap feature
cargo build --release --lib --features voice-clap

# Create bundle structure
BUNDLE_NAME="DSynthVoice.clap"
BUNDLE_DIR="target/bundled/${BUNDLE_NAME}"
CONTENTS_DIR="${BUNDLE_DIR}/Contents"
MACOS_DIR="${CONTENTS_DIR}/MacOS"

echo -e "${YELLOW}Creating bundle structure...${NC}"
rm -rf "${BUNDLE_DIR}"
mkdir -p "${MACOS_DIR}"

# Copy the dylib to the MacOS directory with .clap extension
echo -e "${YELLOW}Copying plugin binary...${NC}"
cp "target/release/libdsynth.dylib" "${MACOS_DIR}/DSynthVoice"

# Create Info.plist
echo -e "${YELLOW}Creating Info.plist...${NC}"
cat > "${CONTENTS_DIR}/Info.plist" << EOF
<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0">
<dict>
    <key>CFBundleDevelopmentRegion</key>
    <string>English</string>
    <key>CFBundleExecutable</key>
    <string>DSynthVoice</string>
    <key>CFBundleGetInfoString</key>
    <string>DSynth Voice Enhancer</string>
    <key>CFBundleIconFile</key>
    <string></string>
    <key>CFBundleIdentifier</key>
    <string>com.dsynth.voice-enhancer</string>
    <key>CFBundleInfoDictionaryVersion</key>
    <string>6.0</string>
    <key>CFBundleName</key>
    <string>DSynth Voice Enhancer</string>
    <key>CFBundlePackageType</key>
    <string>BNDL</string>
    <key>CFBundleShortVersionString</key>
    <string>1.0.0</string>
    <key>CFBundleSignature</key>
    <string>????</string>
    <key>CFBundleVersion</key>
    <string>1.0.0</string>
</dict>
</plist>
EOF

# Create PkgInfo
echo -n "BNDL????" > "${CONTENTS_DIR}/PkgInfo"

# Fix library paths using install_name_tool
echo -e "${YELLOW}Fixing library paths...${NC}"
install_name_tool -id "@loader_path/DSynthVoice" "${MACOS_DIR}/DSynthVoice" || true

# Code sign the plugin (ad-hoc signature)
echo -e "${YELLOW}Code signing plugin...${NC}"
codesign --force --sign - --deep "${BUNDLE_DIR}"

echo -e "${GREEN}✓ Bundle created successfully at ${BUNDLE_DIR}${NC}"
echo -e "${GREEN}To install: cp -r ${BUNDLE_DIR} ~/Library/Audio/Plug-Ins/CLAP/${NC}"
