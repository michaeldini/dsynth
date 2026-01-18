# DSynth Assets

## macOS App Bundle Icon (.app)

For the **standalone macOS application** to show an icon in Finder and the Dock, you need to create a `.app` bundle with an ICNS icon.

### Quick Start (3 steps):

1. **Place your icon**: Put a PNG file (512x512 or larger) at `assets/icon.png`

2. **Generate ICNS**: 
   ```bash
   ./scripts/create_icons.sh
   ```
   This creates `assets/AppIcon.icns` with all required resolutions.

3. **Build app bundle**:
   ```bash
   ./scripts/bundle_standalone_macos.sh
   ```
   This creates `target/bundled/DSynth.app`

4. **Run or install**:
   ```bash
   open target/bundled/DSynth.app
   # Or install to Applications:
   cp -r target/bundled/DSynth.app /Applications/
   ```

---

## Window Icon (Embedded in Binary)

The icon in [src/gui/mod.rs](../src/gui/mod.rs) embeds `assets/icon.png` directly into the binary. This is mainly visible on **Windows and Linux** - macOS uses the app bundle icon instead.

**Note**: On macOS, window icons from code don't typically show. The `.app` bundle icon (above) is what matters.

---

## Icon Requirements

### For icon.png (embedded):
- Format: PNG
- Recommended size: 512x512 pixels
- Transparent background recommended
- Used for: Windows/Linux window icons, source for ICNS conversion

### For AppIcon.icns (macOS bundle):
- Auto-generated from icon.png by `scripts/create_icons.sh`
- Contains multiple resolutions: 16x16 through 1024x1024
- Used for: Finder, Dock, app switcher on macOS

---

## Converting SVG to PNG

If you have an SVG icon, convert it first:

**Using ImageMagick:**
```bash
convert -background none -resize 512x512 icon.svg assets/icon.png
```

**Using Inkscape:**
```bash
inkscape --export-type=png --export-width=512 --export-height=512 icon.svg -o assets/icon.png
```

**Using rsvg-convert:**
```bash
rsvg-convert -w 512 -h 512 icon.svg -o assets/icon.png
```

**Online converters:**
- https://cloudconvert.com/svg-to-png
- https://convertio.co/svg-png/

---

## What Gets Built

- **`cargo run`**: Debug build, no icon visible (running from terminal)
- **`cargo build --release`**: Release binary, embedded icon (Windows/Linux only)
- **`./scripts/bundle_standalone_macos.sh`**: Complete .app bundle with ICNS icon (macOS)

---

## Platform Support

- **macOS**: Requires `.app` bundle for visible icons (use `scripts/bundle_standalone_macos.sh`)
- **Windows**: Window icon from embedded PNG works directly
- **Linux**: Window icon from embedded PNG works directly
