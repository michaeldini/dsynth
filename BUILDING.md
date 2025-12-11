# Building DSynth for Distribution

## Overview

DSynth is a cross-platform synthesizer that can be built for macOS, Linux, and Windows. This guide covers building binaries for distribution.

## Prerequisites by Platform

### macOS
- Rust nightly toolchain
- Xcode Command Line Tools: `xcode-select --install`

### Linux (Ubuntu/Debian)
```bash
# Install required system dependencies
sudo apt-get update
sudo apt-get install -y \
    libasound2-dev \
    pkg-config \
    libx11-dev \
    libxcb1-dev \
    libxcb-render0-dev \
    libxcb-shape0-dev \
    libxcb-xfixes0-dev
```

### Windows
- Rust nightly toolchain
- Visual Studio 2019 or later with C++ build tools
- Windows SDK

## Building Release Binaries

### Current Platform
```bash
# Build optimized release binary
cargo build --release

# Binary location:
# macOS/Linux: target/release/dsynth
# Windows: target/release/dsynth.exe
```

### Cross-Compilation

#### On macOS - Build for Intel and Apple Silicon
```bash
# Add targets
rustup target add x86_64-apple-darwin
rustup target add aarch64-apple-darwin

# Build Intel
cargo build --release --target x86_64-apple-darwin

# Build Apple Silicon
cargo build --release --target aarch64-apple-darwin
```

#### On Linux - Build for different architectures
```bash
# Add target
rustup target add x86_64-unknown-linux-gnu

# Build
cargo build --release --target x86_64-unknown-linux-gnu
```

#### Cross-compile from Linux to Windows
```bash
# Install MinGW
sudo apt-get install mingw-w64

# Add target
rustup target add x86_64-pc-windows-gnu

# Build
cargo build --release --target x86_64-pc-windows-gnu
```

## Optimizing Binary Size

### Strip Debug Symbols
```bash
# macOS/Linux
strip target/release/dsynth

# Windows (using MSVC toolchain)
# Already optimized by default in release mode
```

### Additional Size Optimization
Add to `Cargo.toml`:
```toml
[profile.release]
opt-level = "z"     # Optimize for size
lto = true          # Enable Link Time Optimization
codegen-units = 1   # Better optimization
strip = true        # Strip symbols
```

## Distribution Packaging

### macOS - Create .app Bundle
```bash
#!/bin/bash
# create_macos_bundle.sh

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
Consider using:
- **WiX Toolset** for MSI installers
- **Inno Setup** for simple installers
- **cargo-wix**: `cargo install cargo-wix`

## Automated Builds with GitHub Actions

The repository includes `.github/workflows/release.yml` for automated builds.

### Trigger a Release
```bash
# Tag a version
git tag v1.0.0
git push origin v1.0.0

# GitHub Actions will automatically build for all platforms
```

### Manual Workflow Dispatch
Go to GitHub Actions → Build Release Binaries → Run workflow

## Platform-Specific Notes

### macOS Code Signing
For distribution outside App Store:
```bash
# Sign the binary
codesign --force --deep --sign "Developer ID Application: Your Name" DSynth.app

# Notarize (required for macOS 10.15+)
xcrun notarytool submit DSynth.app --apple-id "you@example.com" --wait
```

### Linux Dependencies
Users need ALSA libraries:
```bash
# Ubuntu/Debian
sudo apt-get install libasound2

# Fedora/RHEL
sudo dnf install alsa-lib
```

### Windows Runtime
No additional runtime dependencies required. The binary is self-contained.

## Testing Binaries

Before distribution, test on target platforms:

```bash
# Check binary is not dependent on local paths
ldd target/release/dsynth  # Linux
otool -L target/release/dsynth  # macOS

# Run smoke test
./target/release/dsynth
```

## Distribution Checklist

- [ ] Build for all target platforms
- [ ] Strip debug symbols
- [ ] Test on clean systems (no Rust installed)
- [ ] Verify audio output works
- [ ] Check MIDI input (optional)
- [ ] Verify GUI renders correctly
- [ ] Test keyboard input
- [ ] Include README and license
- [ ] Document system requirements
- [ ] Provide quick start guide

## System Requirements

### Minimum
- **CPU**: Dual-core 2.0 GHz or faster
- **RAM**: 256 MB
- **OS**: 
  - macOS 10.15+ (Catalina or later)
  - Linux with ALSA or PulseAudio
  - Windows 10 or later
- **Audio**: Any audio output device

### Recommended
- **CPU**: Quad-core 2.5 GHz or faster
- **RAM**: 512 MB
- Low-latency audio interface

## Support

For build issues, check:
1. Rust nightly is installed: `rustup toolchain list`
2. All dependencies are installed
3. Target is added: `rustup target list --installed`
4. Build with verbose output: `cargo build --release -vv`
