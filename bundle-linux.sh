#!/bin/bash
# Bundle script for creating CLAP plugin bundle on Linux
# Note: DSynth is CLAP-only (no longer supports VST3)

set -e

PLUGIN_NAME="DSynth"
BUILD_DIR="target/release"
BUNDLE_DIR="target/bundled"

echo "Building DSynth CLAP plugin..."

# Build the CLAP plugin
echo "Building release binary..."
cargo build --release --lib --features clap

# Create bundle directories
echo "Creating bundle structure..."
rm -rf "$BUNDLE_DIR"
mkdir -p "$BUNDLE_DIR"

# Create CLAP bundle
CLAP_BUNDLE="$BUNDLE_DIR/${PLUGIN_NAME}.clap"
mkdir -p "$CLAP_BUNDLE"

# Copy and rename for CLAP
cp "$BUILD_DIR/libdsynth.so" "$CLAP_BUNDLE/${PLUGIN_NAME}.clap"

echo ""
echo "✅ CLAP plugin bundle created successfully!"
echo ""
echo "CLAP: $CLAP_BUNDLE"
echo ""
echo "To install:"
echo "  mkdir -p ~/.clap"
echo "  cp \"$CLAP_BUNDLE/${PLUGIN_NAME}.clap\" ~/.clap/"
echo ""
