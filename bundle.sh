#!/bin/bash
# Bundle script for creating CLAP plugin bundle on macOS
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
mkdir -p "$CLAP_BUNDLE/Contents/MacOS"
mkdir -p "$CLAP_BUNDLE/Contents/Resources"

# Copy the dylib
echo "Copying plugin binary..."
cp "$BUILD_DIR/libdsynth.dylib" "$CLAP_BUNDLE/Contents/MacOS/$PLUGIN_NAME"

# Create PkgInfo
echo "BNDL????" > "$CLAP_BUNDLE/Contents/PkgInfo"

# Create Info.plist for CLAP
echo "Creating Info.plist..."
cat > "$CLAP_BUNDLE/Contents/Info.plist" << 'EOF'
<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0">
<dict>
    <key>CFBundleDevelopmentRegion</key>
    <string>English</string>
    <key>CFBundleExecutable</key>
    <string>DSynth</string>
    <key>CFBundleIdentifier</key>
    <string>com.dsynth.dsynth</string>
    <key>CFBundleInfoDictionaryVersion</key>
    <string>6.0</string>
    <key>CFBundleName</key>
    <string>DSynth</string>
    <key>CFBundlePackageType</key>
    <string>BNDL</string>
    <key>CFBundleShortVersionString</key>
    <string>0.3.0</string>
    <key>CFBundleVersion</key>
    <string>0.3.0</string>
    <key>CFBundleSignature</key>
    <string>????</string>
    <key>NSHumanReadableCopyright</key>
    <string>Copyright 2025</string>
</dict>
</plist>
EOF

# Code sign the bundle (ad-hoc signing for development)
echo "Signing bundle..."
codesign --force --deep --sign - "$CLAP_BUNDLE"

echo ""
echo "✅ CLAP plugin bundle created and signed successfully!"
echo ""
echo "CLAP: $CLAP_BUNDLE"
echo ""
echo "To install:"
echo "  cp -r \"$CLAP_BUNDLE\" ~/Library/Audio/Plug-Ins/CLAP/"
echo "  # Remove quarantine attributes if needed:"
echo "  xattr -cr ~/Library/Audio/Plug-Ins/CLAP/${PLUGIN_NAME}.clap"
echo ""
echo "Or use the install script:"
echo "  ./install_plugins.sh"
echo ""
