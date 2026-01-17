#!/bin/bash
# Build DSynth Kick CLAP Plugin for macOS (flat .clap file)
# Produces a single Mach-O file named DSynthKick.clap.

set -e

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

echo -e "${GREEN}Building DSynth Kick CLAP plugin (flat file)...${NC}"

# Build the CLAP plugin (dynamic library)
echo -e "${YELLOW}Compiling...${NC}"
cargo build --release --lib --features kick-clap --no-default-features

FLAT_NAME="DSynthKick.clap"
OUT_DIR="target/bundled"
OUT_PATH="$OUT_DIR/$FLAT_NAME"

echo -e "${YELLOW}Preparing output path...${NC}"
mkdir -p "$OUT_DIR"

# If an old bundle directory exists, remove it.
rm -rf "$OUT_PATH"

echo -e "${YELLOW}Creating flat .clap file...${NC}"
cp "target/release/libdsynth.dylib" "$OUT_PATH"

# Set install name so the file is self-contained when loaded by a host.
install_name_tool -id "@loader_path/$FLAT_NAME" "$OUT_PATH" || true

echo -e "${YELLOW}Verifying output...${NC}"
if [ -f "$OUT_PATH" ]; then
    echo -e "${GREEN}✓ Plugin file present${NC}"
else
    echo -e "${RED}✗ Plugin file missing!${NC}"
    exit 1
fi

# Print success message
echo ""
echo -e "${GREEN}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"
echo -e "${GREEN}✓ DSynth Kick CLAP plugin built successfully!${NC}"
echo -e "${GREEN}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"
echo ""
echo -e "${YELLOW}Bundle location:${NC}"
echo "  $OUT_PATH"
echo ""
echo -e "${YELLOW}To install (macOS):${NC}"
echo "  cp -f \"$OUT_PATH\" ~/Library/Audio/Plug-Ins/CLAP/"
echo ""
echo -e "${YELLOW}To test:${NC}"
echo "  1. Copy bundle to CLAP plugins folder"
echo "  2. Open your DAW (Bitwig, Reaper, etc.)"
echo "  3. Scan for new plugins"
echo "  4. Load 'DSynth Kick' as an instrument"
echo ""
