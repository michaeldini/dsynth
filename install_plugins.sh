#!/bin/bash

# Script to install DSynth CLAP plugins to system plugin directories

set -e

echo "Installing DSynth CLAP plugin..."

# Check if bundle exists
if [ ! -d "target/bundled/DSynth.clap" ]; then
    echo "❌ Error: CLAP bundle not found. Run ./bundle_clap.sh first."
    exit 1
fi

# Copy CLAP plugin
echo "Installing CLAP plugin..."
cp -r "target/bundled/DSynth.clap" ~/Library/Audio/Plug-Ins/CLAP/
cp -r target/bundled/DSynthVoice.clap ~/Library/Audio/Plug-Ins/CLAP/
xattr -cr ~/Library/Audio/Plug-Ins/CLAP/DSynth.clap
xattr -cr ~/Library/Audio/Plug-Ins/CLAP/DSynthVoice.clap

echo ""
echo "✅ Plugin installed successfully!"
echo ""
echo "CLAP: ~/Library/Audio/Plug-Ins/CLAP/DSynth.clap"
echo "CLAP: ~/Library/Audio/Plug-Ins/CLAP/DSynthVoice.clap"
echo ""
echo "Restart your DAW to see the plugins."
