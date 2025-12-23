#!/bin/bash
# Bundle DSynth as CLAP plugin (macOS)

set -e

echo "Building DSynth CLAP plugin..."
cargo build --release --lib --features clap

BUNDLE_NAME="DSynth.clap"
BUNDLE_DIR="target/bundled/${BUNDLE_NAME}"
CONTENTS_DIR="${BUNDLE_DIR}/Contents"
MACOS_DIR="${CONTENTS_DIR}/MacOS"

echo "Creating bundle structure..."
rm -rf "${BUNDLE_DIR}"
mkdir -p "${MACOS_DIR}"

echo "Copying plugin binary..."
cp target/release/libdsynth.dylib "${MACOS_DIR}/DSynth"

echo "Creating Info.plist..."
cat > "${CONTENTS_DIR}/Info.plist" << 'EOF'
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
</dict>
</plist>
EOF

echo "Creating PkgInfo..."
echo -n 'BNDL????' > "${CONTENTS_DIR}/PkgInfo"

echo ""
echo "✓ CLAP plugin bundled successfully!"
echo "  Location: ${BUNDLE_DIR}"
echo ""
echo "To install:"
echo "  cp -r '${BUNDLE_DIR}' ~/Library/Audio/Plug-Ins/CLAP/"
echo ""
