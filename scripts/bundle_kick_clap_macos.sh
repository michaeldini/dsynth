#!/usr/bin/env bash
set -euo pipefail

cd "$(dirname "$0")/.."

# Colors for output
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m'

TARGET_DIR="target/release"
TARGET_ARGS=()
if [[ -n "${TARGET:-}" ]]; then
  TARGET_DIR="target/$TARGET/release"
  TARGET_ARGS=(--target "$TARGET")
fi

echo -e "${YELLOW}Building DSynth Kick CLAP plugin for macOS (flat .clap)...${NC}"

# Build the CLAP plugin (no default features)
if ((${#TARGET_ARGS[@]})); then
  cargo build --release --lib --no-default-features --features kick-clap "${TARGET_ARGS[@]}"
else
  cargo build --release --lib --no-default-features --features kick-clap
fi

OUT_DIR="target/bundled"
OUT_FILE="${OUT_DIR}/DSynthKick.clap"

echo -e "${YELLOW}Creating output directory...${NC}"
mkdir -p "${OUT_DIR}"

echo -e "${YELLOW}Copying plugin binary...${NC}"
cp "${TARGET_DIR}/libdsynth.dylib" "${OUT_FILE}"

# Fix library paths
echo -e "${YELLOW}Fixing library paths...${NC}"
install_name_tool -id "@loader_path/DSynthKick.clap" "${OUT_FILE}" || true

# Code sign the plugin (ad-hoc)
echo -e "${YELLOW}Code signing plugin...${NC}"
codesign --force --sign - "${OUT_FILE}" || true

echo -e "${GREEN}✓ Plugin created successfully at ${OUT_FILE}${NC}"