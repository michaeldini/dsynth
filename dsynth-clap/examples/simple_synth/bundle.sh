#!/bin/bash
# Bundle Simple Synth as CLAP plugin for macOS

set -e

echo "Building Simple Synth..."
cargo build --release

echo "Creating CLAP plugin flat file..."
mkdir -p "target/bundled"
FLAT_FILE="target/bundled/SimpleSynth.clap"
rm -rf "$FLAT_FILE"
cp "target/release/libsimple_synth.dylib" "$FLAT_FILE"
chmod +w "$FLAT_FILE"
install_name_tool -id "@loader_path/SimpleSynth.clap" "$FLAT_FILE"

echo "✅ Plugin created at: $FLAT_FILE"
echo ""
echo "To install:"
echo "  cp $FLAT_FILE ~/Library/Audio/Plug-Ins/CLAP/"
echo ""
echo "To test:"
echo "  file $FLAT_FILE"
