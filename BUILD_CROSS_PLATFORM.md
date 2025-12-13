# Cross-Platform Build Instructions

## macOS

The `bundle.sh` script handles everything:

```bash
./bundle.sh
```

This creates:
- `target/bundled/DSynth.vst3` - VST3 plugin bundle
- `target/bundled/DSynth.clap` - CLAP plugin bundle

Install with:
```bash
cp -r target/bundled/DSynth.vst3 ~/Library/Audio/Plug-Ins/VST3/
cp -r target/bundled/DSynth.clap ~/Library/Audio/Plug-Ins/CLAP/
```

## Windows

### Build
```bash
cargo build --release --lib --features vst
```

This creates: `target/release/dsynth.dll`

### Create VST3 Bundle
```powershell
# Create directory structure
mkdir target\bundled\DSynth.vst3\Contents\x86_64-win

# Copy the DLL
copy target\release\dsynth.dll target\bundled\DSynth.vst3\Contents\x86_64-win\DSynth.vst3

# Create moduleinfo.json
@"
{
  "Name": "DSynth",
  "Version": "0.1.1",
  "Factory Info": {
    "Vendor": "DSynth",
    "URL": "",
    "E-Mail": ""
  },
  "Compatibility": {
    "Classes": [
      {
        "CID": "44535374445300000000000000000000"
      }
    ]
  }
}
"@ | Out-File -Encoding utf8 target\bundled\DSynth.vst3\Contents\x86_64-win\moduleinfo.json
```

### Create CLAP Bundle
```powershell
mkdir target\bundled\DSynth.clap\Contents\x86_64-win
copy target\release\dsynth.dll target\bundled\DSynth.clap\Contents\x86_64-win\DSynth.clap
```

### Install
```powershell
# VST3
xcopy /E /I target\bundled\DSynth.vst3 "%COMMONPROGRAMFILES%\VST3\DSynth.vst3"

# CLAP
xcopy /E /I target\bundled\DSynth.clap "%COMMONPROGRAMFILES%\CLAP\DSynth.clap"
```

## Linux

### Build
```bash
cargo build --release --lib --features vst
```

This creates: `target/release/libdsynth.so`

### Create VST3 Bundle
```bash
#!/bin/bash
# Create VST3 bundle
mkdir -p target/bundled/DSynth.vst3/Contents/x86_64-linux
cp target/release/libdsynth.so target/bundled/DSynth.vst3/Contents/x86_64-linux/DSynth.so

# Create moduleinfo.json
cat > target/bundled/DSynth.vst3/Contents/x86_64-linux/moduleinfo.json << 'EOF'
{
  "Name": "DSynth",
  "Version": "0.1.1",
  "Factory Info": {
    "Vendor": "DSynth",
    "URL": "",
    "E-Mail": ""
  },
  "Compatibility": {
    "Classes": [
      {
        "CID": "44535374445300000000000000000000"
      }
    ]
  }
}
EOF
```

### Create CLAP Bundle
```bash
mkdir -p target/bundled/DSynth.clap/Contents/x86_64-linux
cp target/release/libdsynth.so target/bundled/DSynth.clap/Contents/x86_64-linux/DSynth.clap
```

### Install
```bash
# VST3
cp -r target/bundled/DSynth.vst3 ~/.vst3/

# CLAP
cp -r target/bundled/DSynth.clap ~/.clap/
```

## Quick Reference

| Platform | Build Command | Output | Install Location |
|----------|--------------|--------|------------------|
| macOS | `./bundle.sh` | `DSynth.vst3`/`.clap` | `~/Library/Audio/Plug-Ins/VST3/` or `/CLAP/` |
| Windows | `cargo build --release --lib --features vst` | `dsynth.dll` | `%COMMONPROGRAMFILES%\VST3\` or `\CLAP\` |
| Linux | `cargo build --release --lib --features vst` | `libdsynth.so` | `~/.vst3/` or `~/.clap/` |

## Notes

- **macOS**: Bundle structure is required (`.vst3` and `.clap` are directories)
- **Windows**: Can use flat directory structure, but bundle is recommended
- **Linux**: Bundle structure recommended for better DAW compatibility
- All platforms: The binary must match the expected name (`DSynth` on macOS, `DSynth.vst3`/`DSynth.clap` on Win/Linux)
- Ensure the binary has execute permissions on macOS/Linux: `chmod +x <binary>`
