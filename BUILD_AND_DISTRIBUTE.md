# DSynth Build & Distribution Guide

## Overview

DSynth is a **100% cross-platform** synthesizer plugin. The same Rust code builds for macOS, Windows, and Linux without modification. This guide covers building for all platforms, creating distributions, and automated builds via GitHub Actions.

---

## Table of Contents

1. [Quick Start](#quick-start)
2. [Platform-Specific Prerequisites](#platform-specific-prerequisites)
3. [Building on Current Platform](#building-on-current-platform)
4. [Cross-Compilation](#cross-compilation)
5. [Plugin Bundle Creation](#plugin-bundle-creation)
6. [Distribution Packaging](#distribution-packaging)
7. [GitHub Actions Automation](#github-actions-automation)
8. [Testing](#testing)
9. [Distribution Checklist](#distribution-checklist)

---

## Quick Start

### macOS
```bash
./scripts/bundle_clap_macos.sh
# Creates CLAP plugin in target/bundled/

# Install
cp -r target/bundled/DSynth.clap ~/Library/Audio/Plug-Ins/CLAP/
```

### Windows
```batch
scripts\bundle_clap_windows.bat
REM Creates CLAP plugin in target\bundled\

REM Install
copy target\bundled\DSynth.clap\DSynth.clap "%COMMONPROGRAMFILES%\CLAP\"
```

---

## Platform-Specific Prerequisites

### macOS
- Rust nightly toolchain
- Xcode Command Line Tools: `xcode-select --install`
- ✅ Already have this

### Windows
- Rust nightly toolchain
- Visual Studio 2019+ with C++ build tools
- Windows SDK

---

## Building on Current Platform

### Release Binary (Standalone)
```bash
# Build optimized release binary
cargo build --release

# Binary location:
# macOS/Linux: target/release/dsynth
# Windows: target/release/dsynth.exe
```

### Plugin Library
```bash
# Build plugin library (not a standalone executable)
cargo build --release --lib --features clap

# Output:
# macOS: target/release/libdsynth.dylib
# Windows: target/release/dsynth.dll
# Linux: target/release/libdsynth.so
```

### Using Build Scripts
Each platform has a ready-to-use build script:

- **macOS**: `./scripts/bundle_clap_macos.sh` - Creates CLAP plugin (flat .clap)
- **Windows**: `scripts\bundle_clap_windows.bat` - Creates CLAP plugin (flat .clap)

---

## Cross-Compilation

### macOS → Universal Binary (Intel + Apple Silicon)

```bash
# Add targets
rustup target add x86_64-apple-darwin
rustup target add aarch64-apple-darwin

# Build Intel (x86_64)
cargo build --release --target x86_64-apple-darwin

# Build Apple Silicon (ARM64)
cargo build --release --target aarch64-apple-darwin

# Create universal binary (requires lipo)
lipo -create \
  target/x86_64-apple-darwin/release/dsynth \
  target/aarch64-apple-darwin/release/dsynth \
  -output target/release/dsynth-universal
```

### Linux → x86_64

```bash
# Add target
rustup target add x86_64-unknown-linux-gnu

# Build
cargo build --release --target x86_64-unknown-linux-gnu
```

### Linux → Windows (Advanced)

```bash
# Install MinGW
sudo apt-get install mingw-w64

# Add target
rustup target add x86_64-pc-windows-gnu

# Build
cargo build --release --target x86_64-pc-windows-gnu --lib
```

### macOS → Windows (Not Recommended)

**Note:** Cross-compiling to Windows from macOS is complex. **Use GitHub Actions instead** (see below).

---

## Plugin Bundle Creation

### macOS CLAP Bundle Structure
```
DSynth.clap/
├── Contents/
│   ├── MacOS/
│   │   └── DSynth (executable)
│   ├── PkgInfo
│   └── Info.plist
```

**Created by**: `./scripts/bundle_clap_macos.sh`

### Windows CLAP Bundle Structure
```
DSynth.clap/
└── DSynth.clap (DLL)
```

**Created by**: `scripts\\bundle_clap_windows.bat`

---

## Distribution Packaging

### Binary Size Optimization

Add to `Cargo.toml` for smaller binaries:
```toml
[profile.release]
opt-level = "z"     # Optimize for size
lto = true          # Link Time Optimization
codegen-units = 1   # Better optimization
strip = true        # Strip debug symbols
```

### macOS - Create .app Bundle

Use the provided script:
```bash
./scripts/bundle_standalone_macos.sh
```

Example manual steps (if you need a custom bundle):
```bash
#!/bin/bash
# create_macos_bundle.sh (example only)

APP_NAME="DSynth"
BUNDLE_NAME="${APP_NAME}.app"
BINARY="target/release/dsynth"

mkdir -p "${BUNDLE_NAME}/Contents/MacOS"
mkdir -p "${BUNDLE_NAME}/Contents/Resources"

cp "${BINARY}" "${BUNDLE_NAME}/Contents/MacOS/${APP_NAME}"

cat > "${BUNDLE_NAME}/Contents/Info.plist" << EOF
<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0">
<dict>
    <key>CFBundleExecutable</key>
    <string>${APP_NAME}</string>
    <key>CFBundleIdentifier</key>
    <string>com.dsynth.app</string>
    <key>CFBundleName</key>
    <string>${APP_NAME}</string>
    <key>CFBundleVersion</key>
    <string>1.0.0</string>
    <key>CFBundleShortVersionString</key>
    <string>1.0.0</string>
    <key>NSHighResolutionCapable</key>
    <true/>
</dict>
</plist>
EOF

echo "Created ${BUNDLE_NAME}"
```

### macOS Code Signing

For distribution outside App Store:
```bash
# Sign the binary
codesign --force --deep --sign "Developer ID Application: Your Name" DSynth.app

# Notarize (required for macOS 10.15+)
xcrun notarytool submit DSynth.app --apple-id "you@example.com" --wait
```

### Linux - Create .deb Package

```bash
# Install cargo-deb
cargo install cargo-deb

# Add to Cargo.toml:
# [package.metadata.deb]
# maintainer = "Your Name <you@example.com>"
# depends = "$auto, libasound2"
# section = "sound"
# priority = "optional"
# assets = [
#     ["target/release/dsynth", "usr/bin/", "755"],
#     ["README.md", "usr/share/doc/dsynth/", "644"],
# ]

# Build .deb package
cargo deb
```

### Windows - Create Installer

Options:
- **WiX Toolset** - Professional MSI installers
- **Inno Setup** - Simple NSIS installers  
- **cargo-wix** - Rust integration: `cargo install cargo-wix`

---

## GitHub Actions Automation

### What It Builds

The workflow **automatically builds** for all platforms:

| Platform | Standalone | VST3 Plugin | CLAP Plugin |
|----------|-----------|-------------|-------------|
| macOS Intel | ✅ | ✅ | ✅ |
| macOS ARM64 | ✅ | ✅ | ✅ |
| Windows x64 | ✅ | ✅ | ✅ |
| Linux x64 | ✅ | ✅ | ✅ |

**Total: 12 artifacts** (3 formats × 4 platforms)

### When It Runs

1. **Manual trigger**: GitHub Actions → "Build Release Binaries" → "Run workflow"
2. **On tag push**: Push a version tag (e.g., `git tag v0.1.1 && git push --tags`)

### How to Use

#### Option 1: Test Build (Manual)
```bash
# Push your changes
git add .
git commit -m "Update plugin"
git push

# Go to GitHub → Actions → "Build Release Binaries"
# Click "Run workflow" → Select branch → Run

# Wait 5-10 minutes, download artifacts
```

#### Option 2: Create a Release
```bash
# Tag your version
git tag v0.1.1
git push --tags

# GitHub Actions automatically:
# 1. Builds all platforms
# 2. Creates a GitHub ReleCLAP Plugin |
|----------|-----------|-------------|
| macOS ARM64 | ✅ | ✅ |
| Windows x64 | ✅ | ✅ |

**Total: 4 artifacts** (2 formats × 2
**Standalone Binaries:**
- `dsynth-macos-x86_64`
- `dsynth-macos-arm64`
- `dsynth-windows-x86_64.exe`
- `dsynth-linux-x86_64`

**VST3 Plugins:**
- `DSynth-x86_64-apple-darwin-vst3.tar.gz`
- `DSynth-aarch64-apple-darwin-vst3.tar.gz`
- `DSynth-x86_64-pc-windows-msvc-vst3.zip`
- `DSynth-x86_64-unknown-linux-gnu-vst3.tar.gz`

**CLAP Plugins:**
- `DSynth-x86_64-apple-darwin-clap.tar.gz`
- `DSynth-aarch64-apple-darwin-clap.tar.gz`
- `DSynth-x86_64-pc-windows-msvc-clap.zip`
- `DSynth-x86_64-unknown-linux-gnu-clap.tar.gz`

### Build Process Per Platform

**macOS:**
1. Build standalone: `cargo build --release --features standalone`
2. Build plugin: `cargo build --release --lib --features vst`
3. Create VST3 bundle (`.vst3/Contents/MacOS/`)
4. Create CLAP bundle (`.clap/Contents/MacOS/`)
5. Archive as `.tar.gz`

**Windows:**
1. Build standalone: `cargo build --release --features standalone`
2. Build plugin DLL: `cargo build --release --lib --features vst`
3. Create VST3 bundle (`.vst3/Contents/x86_64-win/`)
4. Create CLAP bundle
5. Archive as `.zip`

**Linux:**
1. Build standalone: `cargo build --release --features standalone`
2. Build plugin `.so`: `cargo build --release --lib --features vst`
3. Create VST3 bundle (`.vst3/Contents/x86_64-linux/`)
4. Create CLAP bundle
5. Archive as `.tar.gz`
arm64`
- `dsynth-windows-x86_64.exe`

**CLAP Plugins:**
- `DSynth-aarch64-apple-darwin-clap.tar.gz`
- `DSynth-x86_64-pc-windows-msvc-clap.zip
# Linux
ldd target/release/dsynth

# macOS
otool -L target/release/dsynth
```

### Test Standalone App
```bash
./target/release/dsynth
```

### Test Plugin Installation

**macOS:**
```bashclap`
3. Create CLAP bundle (`.clap/Contents/MacOS/`)
4. Archive as `.tar.gz`

**Windows:**
1. Build standalone: `cargo build --release --features standalone`
2. Build plugin DLL: `cargo build --release --lib --features clap`
3. Create CLAP bundle
4. Archive as `.zipem

Before releasing, test on a clean system with no Rust installed:
- Verify audio output works
- Check MIDI input (if applicable)
- Verify GUI renders correctly
- Test keyboard input
- Check performance under load

---

## Distribution Strategy

### For End Users
1. Use GitHub Actions to build all platforms automatically
2. Create releases on GitHub with downloadable archives:
   - `DSynth-macOS-vst3.tar.gz` (Intel + ARM64)
   - `DSynth-Windows-vst3.zip`
   - `DSynth-Linux-vst3.tar.gz`
3. Users download for their platform and extract

### For Development
- Build on macOS for local testing
- Use GitHub Actions for Windows/Linux builds
- Test on each platform before release
- Share GitHub Actions artifacts for testing

---

## Distribution Checklist

- [ ] Build for all target platforms
- [ ] Strip debug symbols (optional, can do in Cargo.toml)
- [ ] Test on clean systems (no development tools)
- [ ] Verify audio output works
- [ ] Check MIDI input/output
- [ ] Verify GUI renders correctly (all screen scales)
- [ ] Test keyboard input and shortcuts
- [ ] Verify performance (CPU usage reasonable)
- [ ] Include README and license
- [ ] Document system requirements
- [ ] Provide quick start guide
- [ ] Test plugin on multiplclap ~/Library/Audio/Plug-Ins/CLAP/
# Test in Bitwig Studio, Reaper, etc.
```

**Windows:**
```batch
copy target\bundled\DSynth.clap\DSynth.clap "%COMMONPROGRAMFILES%\CLAP\"
REM Test in Bitwig Studio, Reaper, FL Studio, etc.
```

**Linux:**
```bash
cp target/bundled/DSynth.clap/DSynth.clap ~/.clap
  - Linux with ALSA or PulseAudio
- **Audio**: Any audio output device

### Recommended
- **CPU**: Quad-core 2.5 GHz or faster
- **RAM**: 512 MB or more
- **Audio**: Low-latency audio interface

### PlatforCLAP plugin on multiple DAWs (Bitwig Studio, Reaper, FL Studio)

**macOS:**
- Universal binary supports both Intel and Apple Silicon
- Code signing and notarization required for distribution

**Windows:**
- No additional runtime dependencies (self-contained)
- Works with ASIO, DirectSound, WASAPI

**Linux:**
- Requires ALaarch64-apple-darwin-clap.tar.gz` (macOS ARM64)
   - `DSynth-x86_64-pc-windows-msvc-clap.zip` (Windows)JACK

---

## File Compatibility Summary

| Platform | Library Extension | Bundle Format |
|----------|------------------|---------------|
| macOS | `.dylib` | Directory bundle (`DSynth.vst3/`) |
| Windows | `.dll` | Directory or flat file |
| Linux | `.so` | Directory bundle (`DSynth.vst3/`) |

- ARM64 (Apple Silicon) builds available
- Code signing and notarization required for distribution

**Windows:**
- No additional runtime dependencies (self-contained)
- Works with ASIO, DirectSound, WASAPIag to trigger builds |
| ✅ Plugin bundles | Ready | Scripts create autclap/Contents/MacOS/`) |
| Windows | `.dll` | Flat file (`DSynth.clap`) |

**Important:** Compiled binaries are NOT cross-platform. A `.dylib` only works on macOS, `.dll` only on Windows(CLAP-only) | Ready | Run `./scripts/bundle_clap_macos.sh` on macOS |
| ✅ GitHub Actions (CLAP-only) | Ready | Push tag to trigger builds |
| ✅ CLAP bundles | Ready | Scripts create automatically |
| ✅ Distribution ready | Ready | Use GitHub Releases for distribution |

**Note:** DSynth is now CLAP-only. All VST3 support has been removed.