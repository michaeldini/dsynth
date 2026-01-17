#!/bin/bash
# Bundle DSynth Voice Enhancer as a CLAP plugin for macOS (flat .clap Mach-O)

set -e

# Colors for output
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

echo -e "${YELLOW}Building DSynth Voice Enhancer CLAP plugin for macOS...${NC}"

# Build the library with voice-clap feature
cargo build --release --lib --features voice-clap

# Output is a single flat file (REAPER-friendly)
OUT_DIR="target/bundled"
OUT_FILE="${OUT_DIR}/DSynthVoice.clap"

echo -e "${YELLOW}Creating output directory...${NC}"
mkdir -p "${OUT_DIR}"

# Copy the dylib to a flat .clap file
echo -e "${YELLOW}Copying plugin binary...${NC}"
cp "target/release/libdsynth.dylib" "${OUT_FILE}"

# Fix library paths using install_name_tool
echo -e "${YELLOW}Fixing library paths...${NC}"
install_name_tool -id "@loader_path/DSynthVoice.clap" "${OUT_FILE}" || true

# Code sign the plugin (ad-hoc signature)
echo -e "${YELLOW}Code signing plugin...${NC}"
codesign --force --sign - "${OUT_FILE}" || true

echo -e "${GREEN}✓ Plugin created successfully at ${OUT_FILE}${NC}"
echo -e "${GREEN}To install: cp ${OUT_FILE} ~/Library/Audio/Plug-Ins/CLAP/${NC}"
