#!/bin/bash
# Bundle script for creating a macOS .app bundle for DSynth standalone application

set -e

APP_NAME="DSynth"
BUILD_DIR="target/release"
BUNDLE_DIR="target/bundled"
APP_BUNDLE="$BUNDLE_DIR/$APP_NAME.app"

echo "Building DSynth standalone .app bundle..."

# Build the standalone application
echo "Building release binary..."
cargo build --release --features standalone

# Create bundle directories
echo "Creating bundle structure..."
rm -rf "$APP_BUNDLE"
mkdir -p "$APP_BUNDLE/Contents/MacOS"
mkdir -p "$APP_BUNDLE/Contents/Resources"

# Copy the executable
echo "Copying executable..."
cp "$BUILD_DIR/dsynth" "$APP_BUNDLE/Contents/MacOS/dsynth"

# Make it executable
chmod +x "$APP_BUNDLE/Contents/MacOS/dsynth"

# Create PkgInfo (legacy but some tools expect it)
echo "APPL????" > "$APP_BUNDLE/Contents/PkgInfo"

# Create Info.plist
echo "Creating Info.plist..."
cat > "$APP_BUNDLE/Contents/Info.plist" << 'EOF'
<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0">
<dict>
    <key>CFBundleDevelopmentRegion</key>
    <string>en</string>
    <key>CFBundleExecutable</key>
    <string>dsynth</string>
    <key>CFBundleIdentifier</key>
    <string>com.dsynth.standalone</string>
    <key>CFBundleInfoDictionaryVersion</key>
    <string>6.0</string>
    <key>CFBundleName</key>
    <string>DSynth</string>
    <key>CFBundleDisplayName</key>
    <string>DSynth</string>
    <key>CFBundlePackageType</key>
    <string>APPL</string>
    <key>CFBundleShortVersionString</key>
    <string>0.3.0</string>
    <key>CFBundleVersion</key>
    <string>0.3.0</string>
    <key>CFBundleIconFile</key>
    <string>AppIcon</string>
    <key>LSMinimumSystemVersion</key>
    <string>10.15</string>
    <key>NSHighResolutionCapable</key>
    <true/>
    <key>NSSupportsAutomaticGraphicsSwitching</key>
    <true/>
    <key>LSApplicationCategoryType</key>
    <string>public.app-category.music</string>
</dict>
</plist>
EOF

# Copy icon if it exists
if [ -f "assets/AppIcon.icns" ]; then
    echo "Copying app icon..."
    cp "assets/AppIcon.icns" "$APP_BUNDLE/Contents/Resources/AppIcon.icns"
else
    echo "Warning: No AppIcon.icns found in assets/"
    echo "  Run ./create_icns.sh to generate it from assets/icon.png"
fi

echo ""
echo "✓ Bundle created successfully!"
echo ""
echo "Location: $APP_BUNDLE"
echo ""
echo "To run:"
echo "  open $APP_BUNDLE"
echo ""
echo "To install to Applications:"
echo "  cp -r $APP_BUNDLE /Applications/"
echo ""
