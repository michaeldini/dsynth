#!/bin/bash
# Simple CLAP plugin validator for DSynth Kick

set -e

PLUGIN_PATH="target/bundled/DSynthKick.clap/Contents/MacOS/DSynthKick"
LOG_FILE="/tmp/dsynth_kick_clap.log"

echo "Testing DSynth Kick CLAP Plugin"
echo "================================"
echo ""

# Clear old log
rm -f "$LOG_FILE"

# Check if plugin exists
if [ ! -f "$PLUGIN_PATH" ]; then
    echo "❌ Plugin binary not found at: $PLUGIN_PATH"
    exit 1
fi

echo "✓ Plugin binary exists"

# Check architecture
ARCH=$(file "$PLUGIN_PATH" | grep -o "arm64\|x86_64")
echo "✓ Architecture: $ARCH"

# Check if clap_entry symbol is exported
if nm -g "$PLUGIN_PATH" | grep -q "clap_entry"; then
    echo "✓ clap_entry symbol exported"
else
    echo "❌ clap_entry symbol NOT found"
    exit 1
fi

# Check bundle structure
if [ -f "target/bundled/DSynthKick.clap/Contents/Info.plist" ]; then
    echo "✓ Info.plist present"
else
    echo "❌ Info.plist missing"
    exit 1
fi

# Try to load with Python (if available)
echo ""
echo "Checking plugin load with Python ctypes..."
python3 << 'EOF'
import ctypes
import sys

try:
    lib = ctypes.CDLL("target/bundled/DSynthKick.clap/Contents/MacOS/DSynthKick")
    print("✓ Plugin dylib loads successfully")
    
    # Try to get clap_entry function
    clap_entry = lib.clap_entry
    print("✓ clap_entry function accessible")
    
    # Call it (this should return a pointer to clap_plugin_entry)
    result = clap_entry(None)
    if result:
        print(f"✓ clap_entry returned pointer: 0x{result:x}")
    else:
        print("❌ clap_entry returned NULL")
        sys.exit(1)
        
except Exception as e:
    print(f"❌ Error: {e}")
    sys.exit(1)
EOF

if [ $? -eq 0 ]; then
    echo ""
    echo "✓ All basic checks passed!"
    echo ""
    echo "Next steps:"
    echo "1. Copy plugin: cp -r target/bundled/DSynthKick.clap ~/Library/Audio/Plug-Ins/CLAP/"
    echo "2. In Reaper, rescan plugins"
    echo "3. Check log file: tail -f /tmp/dsynth_kick_clap.log"
    echo "4. Try to load the plugin and watch the log file"
else
    echo ""
    echo "❌ Plugin validation failed"
    exit 1
fi
