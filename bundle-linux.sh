#!/bin/bash
# Bundle script for creating VST3/CLAP plugin bundles on Linux

set -e

PLUGIN_NAME="DSynth"
BUILD_DIR="target/release"
BUNDLE_DIR="target/bundled"

echo "Building DSynth plugin bundles for Linux..."

# Build the plugin
echo "Building release binary..."
cargo build --release --lib --features vst

# Create bundle directories
echo "Creating bundle structure..."
rm -rf "$BUNDLE_DIR"
mkdir -p "$BUNDLE_DIR"

# Create VST3 bundle
VST3_BUNDLE="$BUNDLE_DIR/${PLUGIN_NAME}.vst3"
mkdir -p "$VST3_BUNDLE/Contents/x86_64-linux"

# Copy and rename the library
echo "Copying plugin binary..."
cp "$BUILD_DIR/libdsynth.so" "$VST3_BUNDLE/Contents/x86_64-linux/${PLUGIN_NAME}.so"

# Create moduleinfo.json
echo "Creating moduleinfo.json..."
cat > "$VST3_BUNDLE/Contents/x86_64-linux/moduleinfo.json" << 'EOF'
{
  "Name": "DSynth",
  "Version": "0.1.1",
  "Factory Info": {
    "Vendor": "DSynth",
    "URL": "",
    "E-Mail": ""
  },
  "Compatibility": {
    "Classes": [
      {
        "CID": "44535374445300000000000000000000"
      }
    ]
  }
}
EOF

# Create CLAP bundle
CLAP_BUNDLE="$BUNDLE_DIR/${PLUGIN_NAME}.clap"
mkdir -p "$CLAP_BUNDLE"

# Copy and rename for CLAP
cp "$BUILD_DIR/libdsynth.so" "$CLAP_BUNDLE/${PLUGIN_NAME}.clap"

echo ""
echo "✅ Plugin bundles created successfully!"
echo ""
echo "VST3: $VST3_BUNDLE"
echo "CLAP: $CLAP_BUNDLE"
echo ""
echo "To install:"
echo "  mkdir -p ~/.vst3 ~/.clap"
echo "  cp -r \"$VST3_BUNDLE\" ~/.vst3/"
echo "  cp \"$CLAP_BUNDLE/${PLUGIN_NAME}.clap\" ~/.clap/"
echo ""
