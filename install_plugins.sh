#!/bin/bash

# Script to install DSynth VST3 and CLAP plugins to system plugin directories

set -e

echo "Installing DSynth plugins..."

# Check if bundles exist
if [ ! -d "target/bundled/DSynth.vst3" ]; then
    echo "❌ Error: VST3 bundle not found. Run ./bundle.sh first."
    exit 1
fi

if [ ! -d "target/bundled/DSynth.clap" ]; then
    echo "❌ Error: CLAP bundle not found. Run ./bundle.sh first."
    exit 1
fi

# Copy VST3 plugin
echo "Installing VST3 plugin..."
cp -r "target/bundled/DSynth.vst3" ~/Library/Audio/Plug-Ins/VST3/
xattr -cr ~/Library/Audio/Plug-Ins/VST3/DSynth.vst3

# Copy CLAP plugin
echo "Installing CLAP plugin..."
cp -r "target/bundled/DSynth.clap" ~/Library/Audio/Plug-Ins/CLAP/
xattr -cr ~/Library/Audio/Plug-Ins/CLAP/DSynth.clap

echo ""
echo "✅ Plugins installed successfully!"
echo ""
echo "VST3: ~/Library/Audio/Plug-Ins/VST3/DSynth.vst3"
echo "CLAP: ~/Library/Audio/Plug-Ins/CLAP/DSynth.clap"
echo ""
echo "Restart your DAW to see the plugins."
