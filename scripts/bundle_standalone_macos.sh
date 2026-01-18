#!/usr/bin/env bash
set -euo pipefail

cd "$(dirname "$0")/.."

APP_NAME="DSynth"
BUNDLE_DIR="target/bundled"
APP_BUNDLE="$BUNDLE_DIR/$APP_NAME.app"

TARGET_DIR="target/release"
TARGET_ARGS=()
if [[ -n "${TARGET:-}" ]]; then
  TARGET_DIR="target/$TARGET/release"
  TARGET_ARGS=(--target "$TARGET")
fi

echo "Building DSynth standalone .app bundle..."

echo "Building release binary..."
if ((${#TARGET_ARGS[@]})); then
    cargo build --release --features standalone "${TARGET_ARGS[@]}"
else
    cargo build --release --features standalone
fi

echo "Creating bundle structure..."
rm -rf "$APP_BUNDLE"
mkdir -p "$APP_BUNDLE/Contents/MacOS"
mkdir -p "$APP_BUNDLE/Contents/Resources"

echo "Copying executable..."
cp "$TARGET_DIR/dsynth" "$APP_BUNDLE/Contents/MacOS/dsynth"
chmod +x "$APP_BUNDLE/Contents/MacOS/dsynth"

echo "APPL????" > "$APP_BUNDLE/Contents/PkgInfo"

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

if [ -f "assets/AppIcon.icns" ]; then
    echo "Copying app icon..."
    cp "assets/AppIcon.icns" "$APP_BUNDLE/Contents/Resources/AppIcon.icns"
else
    echo "Warning: No AppIcon.icns found in assets/"
    echo "  Run ./create_icns.sh to generate it from assets/icon.png"
fi

echo ""
echo "✓ Bundle created successfully!"
echo "Location: $APP_BUNDLE"