#!/bin/bash
# Bundle script for creating VST3/CLAP plugin bundles on macOS

set -e

PLUGIN_NAME="DSynth"
BUILD_DIR="target/release"
BUNDLE_DIR="target/bundled"

echo "Building DSynth plugin bundles..."

# Build the plugin
echo "Building release binary..."
cargo build --release --lib --features vst

# Create bundle directories
echo "Creating bundle structure..."
rm -rf "$BUNDLE_DIR"
mkdir -p "$BUNDLE_DIR"

# Create VST3 bundle for macOS
VST3_BUNDLE="$BUNDLE_DIR/${PLUGIN_NAME}.vst3"
mkdir -p "$VST3_BUNDLE/Contents/MacOS"
mkdir -p "$VST3_BUNDLE/Contents/Resources"

# Copy the dylib
echo "Copying plugin binary..."
cp "$BUILD_DIR/libdsynth.dylib" "$VST3_BUNDLE/Contents/MacOS/$PLUGIN_NAME"

# Create PkgInfo
echo "BNDL????" > "$VST3_BUNDLE/Contents/PkgInfo"

# Create Info.plist
echo "Creating Info.plist..."
cat > "$VST3_BUNDLE/Contents/Info.plist" << 'EOF'
<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0">
<dict>
    <key>CFBundleDevelopmentRegion</key>
    <string>English</string>
    <key>CFBundleExecutable</key>
    <string>DSynth</string>
    <key>CFBundleIdentifier</key>
    <string>com.dsynth.vst3</string>
    <key>CFBundleInfoDictionaryVersion</key>
    <string>6.0</string>
    <key>CFBundleName</key>
    <string>DSynth</string>
    <key>CFBundlePackageType</key>
    <string>BNDL</string>
    <key>CFBundleShortVersionString</key>
    <string>0.1.1</string>
    <key>CFBundleVersion</key>
    <string>0.1.1</string>
    <key>CFBundleSignature</key>
    <string>????</string>
    <key>NSHumanReadableCopyright</key>
    <string>Copyright 2025</string>
</dict>
</plist>
EOF

# Create CLAP bundle
CLAP_BUNDLE="$BUNDLE_DIR/${PLUGIN_NAME}.clap"
mkdir -p "$CLAP_BUNDLE/Contents/MacOS"
mkdir -p "$CLAP_BUNDLE/Contents/Resources"

# Copy the dylib
cp "$BUILD_DIR/libdsynth.dylib" "$CLAP_BUNDLE/Contents/MacOS/$PLUGIN_NAME"

# Create PkgInfo
echo "BNDL????" > "$CLAP_BUNDLE/Contents/PkgInfo"

# Create Info.plist for CLAP
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
    <string>com.dsynth.clap</string>
    <key>CFBundleInfoDictionaryVersion</key>
    <string>6.0</string>
    <key>CFBundleName</key>
    <string>DSynth</string>
    <key>CFBundlePackageType</key>
    <string>BNDL</string>
    <key>CFBundleShortVersionString</key>
    <string>0.1.1</string>
    <key>CFBundleVersion</key>
    <string>0.1.1</string>
    <key>CFBundleSignature</key>
    <string>????</string>
    <key>NSHumanReadableCopyright</key>
    <string>Copyright 2025</string>
</dict>
</plist>
EOF

# Code sign the bundles (ad-hoc signing for development)
echo "Signing bundles..."
codesign --force --deep --sign - "$VST3_BUNDLE"
codesign --force --deep --sign - "$CLAP_BUNDLE"

echo ""
echo "✅ Plugin bundles created and signed successfully!"
echo ""
echo "VST3: $VST3_BUNDLE"
echo "CLAP: $CLAP_BUNDLE"
echo ""
echo "To install:"
echo "  cp -r \"$VST3_BUNDLE\" ~/Library/Audio/Plug-Ins/VST3/"
echo "  cp -r \"$CLAP_BUNDLE\" ~/Library/Audio/Plug-Ins/CLAP/"
echo "  # Remove quarantine attributes if needed:"
echo "  xattr -cr ~/Library/Audio/Plug-Ins/VST3/${PLUGIN_NAME}.vst3"
echo "  xattr -cr ~/Library/Audio/Plug-Ins/CLAP/${PLUGIN_NAME}.clap"
echo ""
