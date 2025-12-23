#!/usr/bin/env python3
"""
Quick test to check if CLAP plugin can be loaded via ctypes
This will help us determine if the plugin is being found by the system
"""

import ctypes
import os
import sys

# Path to our CLAP plugin
plugin_path = os.path.expanduser("~/Library/Audio/Plug-Ins/CLAP/DSynth.clap/Contents/MacOS/DSynth")

print(f"Testing CLAP plugin: {plugin_path}")
print(f"Plugin exists: {os.path.exists(plugin_path)}")

if os.path.exists(plugin_path):
    try:
        # Try to load the dylib
        lib = ctypes.CDLL(plugin_path)
        print("✓ Plugin dylib loaded successfully")
        
        # Try to get the CLAP entry point
        try:
            entry = lib.clap_entry
            print("✓ CLAP entry point found")
        except AttributeError:
            print("✗ CLAP entry point not found")
            
    except Exception as e:
        print(f"✗ Failed to load plugin: {e}")
else:
    print("✗ Plugin file not found")

# Check the bundle structure
bundle_path = os.path.expanduser("~/Library/Audio/Plug-Ins/CLAP/DSynth.clap")
if os.path.exists(bundle_path):
    print(f"\nBundle structure:")
    for root, dirs, files in os.walk(bundle_path):
        level = root.replace(bundle_path, '').count(os.sep)
        indent = ' ' * 2 * level
        print(f"{indent}{os.path.basename(root)}/")
        subindent = ' ' * 2 * (level + 1)
        for file in files:
            print(f"{subindent}{file}")