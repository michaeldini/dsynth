#!/usr/bin/env bash
# Convert PNG icon to macOS ICNS format
# Requires a source icon.png in assets/ directory

set -euo pipefail

cd "$(dirname "$0")/.."

SOURCE_PNG="assets/icon.png"
ICONSET_DIR="assets/AppIcon.iconset"
OUTPUT_ICNS="assets/AppIcon.icns"

if [ ! -f "$SOURCE_PNG" ]; then
    echo "Error: $SOURCE_PNG not found!"
    echo "Please place a PNG icon (512x512 or larger recommended) at $SOURCE_PNG"
    exit 1
fi

echo "Creating ICNS from $SOURCE_PNG..."

# Verify the source is actually a PNG
file_type=$(file -b "$SOURCE_PNG")
echo "Source file type: $file_type"

# Remove old iconset and output if they exist
rm -rf "$ICONSET_DIR"
rm -f "$OUTPUT_ICNS"

# Create iconset directory
mkdir -p "$ICONSET_DIR"

# Generate all required icon sizes
# macOS requires specific sizes and naming conventions
echo "Generating icon sizes..."

# Use sips with explicit format specification
sips -s format png -z 16 16     "$SOURCE_PNG" --out "$ICONSET_DIR/icon_16x16.png" 2>&1 | grep -v "Warning:"
sips -s format png -z 32 32     "$SOURCE_PNG" --out "$ICONSET_DIR/icon_16x16@2x.png" 2>&1 | grep -v "Warning:"
sips -s format png -z 32 32     "$SOURCE_PNG" --out "$ICONSET_DIR/icon_32x32.png" 2>&1 | grep -v "Warning:"
sips -s format png -z 64 64     "$SOURCE_PNG" --out "$ICONSET_DIR/icon_32x32@2x.png" 2>&1 | grep -v "Warning:"
sips -s format png -z 128 128   "$SOURCE_PNG" --out "$ICONSET_DIR/icon_128x128.png" 2>&1 | grep -v "Warning:"
sips -s format png -z 256 256   "$SOURCE_PNG" --out "$ICONSET_DIR/icon_128x128@2x.png" 2>&1 | grep -v "Warning:"
sips -s format png -z 256 256   "$SOURCE_PNG" --out "$ICONSET_DIR/icon_256x256.png" 2>&1 | grep -v "Warning:"
sips -s format png -z 512 512   "$SOURCE_PNG" --out "$ICONSET_DIR/icon_256x256@2x.png" 2>&1 | grep -v "Warning:"
sips -s format png -z 512 512   "$SOURCE_PNG" --out "$ICONSET_DIR/icon_512x512.png" 2>&1 | grep -v "Warning:"
sips -s format png -z 1024 1024 "$SOURCE_PNG" --out "$ICONSET_DIR/icon_512x512@2x.png" 2>&1 | grep -v "Warning:"

# Verify files were created
echo "Verifying generated files..."
ls -la "$ICONSET_DIR/" | grep icon_ || {
    echo "Error: Failed to generate icon files"
    exit 1
}

# Convert iconset to ICNS
echo "Converting to ICNS..."
iconutil -c icns "$ICONSET_DIR" -o "$OUTPUT_ICNS"

if [ ! -f "$OUTPUT_ICNS" ]; then
    echo "Error: iconutil failed to create ICNS file"
    echo "Iconset contents:"
    ls -la "$ICONSET_DIR/"
    exit 1
fi

# Clean up iconset directory
rm -rf "$ICONSET_DIR"

echo ""
echo "✓ ICNS icon created successfully!"
echo ""
echo "Output: $OUTPUT_ICNS"
echo ""
echo "Now run: ./scripts/bundle_standalone_macos.sh"
echo ""
