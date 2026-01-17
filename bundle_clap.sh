#!/bin/bash
# Bundle DSynth as a CLAP plugin for macOS (flat .clap Mach-O)

set -e

# Colors for output
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

echo -e "${YELLOW}Building DSynth CLAP plugin for macOS...${NC}"
cargo build --release --lib --features clap

# Output is a single flat file (REAPER-friendly)
OUT_DIR="target/bundled"
OUT_FILE="${OUT_DIR}/DSynth.clap"

echo -e "${YELLOW}Creating output directory...${NC}"
mkdir -p "${OUT_DIR}"

echo -e "${YELLOW}Copying plugin binary...${NC}"
cp "target/release/libdsynth.dylib" "${OUT_FILE}"

echo -e "${YELLOW}Fixing library paths...${NC}"
install_name_tool -id "@loader_path/DSynth.clap" "${OUT_FILE}" || true

echo -e "${YELLOW}Code signing plugin...${NC}"
codesign --force --sign - "${OUT_FILE}" || true

echo -e "${GREEN}✓ Plugin created successfully at ${OUT_FILE}${NC}"
echo -e "${GREEN}To install: cp ${OUT_FILE} ~/Library/Audio/Plug-Ins/CLAP/${NC}"
