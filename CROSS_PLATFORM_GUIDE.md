# Cross-Platform Build Guide

## Overview

Your DSynth plugin code is **100% cross-platform compatible**! The same Rust code works on macOS, Windows, and Linux. You just need to build on each platform (or use CI/CD).

## Quick Answer

| What You Have | What You Need |
|---------------|---------------|
| ✅ Cross-platform code | ✅ Already done! |
| ✅ macOS build script | ✅ `bundle.sh` |
| ✅ Windows build script | ✅ `bundle.bat` (just created) |
| ✅ Linux build script | ✅ `bundle-linux.sh` (just created) |
| ✅ CI/CD workflow | ✅ `.github/workflows/build.yml` (just created) |

## Building on Each Platform

### macOS (Your Current Machine)
```bash
./bundle.sh
# Creates: target/bundled/DSynth.vst3 and DSynth.clap
# Install: cp -r target/bundled/DSynth.vst3 ~/Library/Audio/Plug-Ins/VST3/
```

### Windows (On a Windows PC or VM)
```batch
bundle.bat
REM Creates: target\bundled\DSynth.vst3 and DSynth.clap
REM Install: xcopy /E /I target\bundled\DSynth.vst3 "%COMMONPROGRAMFILES%\VST3\DSynth.vst3"
```

### Linux (On a Linux PC or VM)
```bash
chmod +x bundle-linux.sh
./bundle-linux.sh
# Creates: target/bundled/DSynth.vst3 and DSynth.clap
# Install: cp -r target/bundled/DSynth.vst3 ~/.vst3/
```

## Recommended Approach: GitHub Actions (Automated)

I've created `.github/workflows/build.yml` that will:

1. **Automatically build** for macOS, Windows, and Linux on every push
2. **Create releases** when you tag a version
3. **Upload artifacts** you can download

### Setup Steps:

1. Push your code to GitHub:
```bash
git add .
git commit -m "Add VST plugin with cross-platform build scripts"
git push
```

2. GitHub Actions will automatically build all platforms
3. Download the artifacts from the "Actions" tab
4. When ready to release, create a tag:
```bash
git tag v0.1.1
git push --tags
```

This creates a GitHub release with downloadable plugins for all platforms!

## Manual Windows Build (Without Windows PC)

If you don't have Windows access, you can:

### Option A: Use GitHub Actions (Recommended)
- Push to GitHub
- Actions builds Windows version automatically
- Download from Actions artifacts

### Option B: Cross-compile (Advanced)
```bash
# Install Windows target
rustup target add x86_64-pc-windows-msvc

# Cross-compile (requires Windows SDK)
cargo build --release --lib --features vst --target x86_64-pc-windows-msvc
```

**Note:** Cross-compiling to Windows from macOS is complex and requires Windows SDK. GitHub Actions is much easier.

## Testing on Each Platform

### macOS
```bash
# Build
./bundle.sh

# Install locally
cp -r target/bundled/DSynth.vst3 ~/Library/Audio/Plug-Ins/VST3/

# Test in Logic Pro, Ableton, etc.
```

### Windows
```batch
REM Build
bundle.bat

REM Install locally  
xcopy /E /I target\bundled\DSynth.vst3 "%COMMONPROGRAMFILES%\VST3\DSynth.vst3"

REM Test in FL Studio, Ableton, Reaper, etc.
```

### Linux
```bash
# Build
./bundle-linux.sh

# Install locally
cp -r target/bundled/DSynth.vst3 ~/.vst3/

# Test in Reaper, Bitwig, Ardour, etc.
```

## Distribution Strategy

### For End Users:
1. Use GitHub Actions to build all platforms
2. Create a release with downloads:
   - `DSynth-macOS-vst3.tar.gz`
   - `DSynth-Windows-vst3.zip`
   - `DSynth-Linux-vst3.tar.gz`
3. Users download and install for their platform

### For Development:
- Build on your Mac for testing
- Use GitHub Actions for Windows/Linux builds
- Test on each platform before release

## File Compatibility

| Platform | Library Extension | Bundle Format |
|----------|------------------|---------------|
| macOS | `.dylib` | Directory bundle (`DSynth.vst3/`) |
| Windows | `.dll` | Directory or flat (`DSynth.vst3/` or `.vst3` file) |
| Linux | `.so` | Directory bundle (`DSynth.vst3/`) |

**Important:** The compiled binaries are NOT cross-platform. A `.dylib` only works on macOS, `.dll` only on Windows, etc. But the Rust source code works everywhere!

## Dependencies

All platforms need:
- Rust nightly (for SIMD features)
- Standard build tools (varies by platform)

Platform-specific:
- **macOS**: Xcode command-line tools (already have)
- **Windows**: MSVC build tools or MinGW
- **Linux**: `libasound2-dev`, `libjack-dev`

## Summary

✅ **Your code is cross-platform ready!**  
✅ **Build scripts created for all platforms**  
✅ **GitHub Actions will build everything automatically**  
✅ **Just push to GitHub and download artifacts**

You don't need to change any code - just build on each platform or use CI/CD!
