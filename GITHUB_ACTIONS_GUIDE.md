# GitHub Actions Release Workflow

## What It Does

Your `release.yml` workflow now builds **BOTH standalone applications AND VST/CLAP plugins** for all platforms automatically!

## When It Runs

1. **Manual trigger**: Click "Run workflow" in GitHub Actions tab
2. **On tag push**: When you push a version tag (e.g., `v0.1.1`)

## What Gets Built

### For Each Platform:

| Platform | Standalone | VST3 Plugin | CLAP Plugin |
|----------|-----------|-------------|-------------|
| macOS Intel | ✅ dsynth | ✅ DSynth.vst3 | ✅ DSynth.clap |
| macOS ARM64 | ✅ dsynth | ✅ DSynth.vst3 | ✅ DSynth.clap |
| Windows x64 | ✅ dsynth.exe | ✅ DSynth.vst3 | ✅ DSynth.clap |
| Linux x64 | ✅ dsynth | ✅ DSynth.vst3 | ✅ DSynth.clap |

**Total: 12 artifacts** (4 standalone + 4 VST3 + 4 CLAP)

## How to Use

### Option 1: Manual Test Build
```bash
# Push your changes to GitHub
git add .
git commit -m "Update plugin"
git push

# Go to GitHub → Actions → "Build Release Binaries"
# Click "Run workflow" → Select branch → Run

# Wait for build to complete (~5-10 minutes)
# Download artifacts from the workflow run
```

### Option 2: Create a Release
```bash
# Tag your version
git tag v0.1.1
git push --tags

# GitHub Actions automatically:
# 1. Builds all platforms
# 2. Creates a GitHub Release
# 3. Uploads all files as release assets
```

## Release Assets

When you push a tag, GitHub creates a release with these downloadable files:

**Standalone:**
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

## What Changed

### Before:
- Only built standalone applications
- Had to manually build plugins on each platform

### After:
- Builds standalone + VST3 + CLAP for all platforms
- Completely automated
- Cross-platform without needing Windows/Linux machines
- Creates proper plugin bundles with correct structure

## Build Process Per Platform

### macOS
1. Build standalone: `cargo build --release --features standalone`
2. Build plugin lib: `cargo build --release --lib --features vst`
3. Create VST3 bundle with proper `.vst3/Contents/MacOS/` structure
4. Create CLAP bundle with proper `.clap/Contents/MacOS/` structure
5. Archive as `.tar.gz`

### Windows
1. Build standalone: `cargo build --release --features standalone`
2. Build plugin DLL: `cargo build --release --lib --features vst`
3. Create VST3 bundle with `.vst3/Contents/x86_64-win/` structure
4. Create CLAP bundle
5. Archive as `.zip`

### Linux
1. Build standalone: `cargo build --release --features standalone`
2. Build plugin `.so`: `cargo build --release --lib --features vst`
3. Create VST3 bundle with `.vst3/Contents/x86_64-linux/` structure
4. Create CLAP bundle
5. Archive as `.tar.gz`

## Testing Without Release

You can test builds anytime without creating a release:

```bash
# Push to any branch
git push origin your-branch

# Go to GitHub → Actions → "Build Release Binaries"
# Click "Run workflow"
# Select your branch
# Click "Run workflow" button

# Download artifacts from the workflow summary page
```

## First Time Setup

No additional setup needed! Your workflow is ready to go:

```bash
# Just push and tag
git add .
git commit -m "Add VST plugin support"
git push

git tag v0.1.1
git push --tags

# Check GitHub → Releases tab
# All binaries will appear there!
```

## Build Time

Expect ~5-10 minutes for complete build:
- macOS Intel: ~2 min
- macOS ARM64: ~2 min
- Windows: ~3 min
- Linux: ~2 min

Builds run in parallel, so total time is ~5-7 minutes.

## Next Steps

1. **Test it**: `git push --tags` to trigger a build
2. **Download artifacts**: Go to GitHub → Actions → click on the workflow run
3. **Test plugins**: Install the downloaded plugins in your DAW
4. **Share**: Send users to your GitHub Releases page

## Notes

- Builds require Rust nightly (automatically installed in workflow)
- All platform dependencies are installed automatically
- Plugin bundles follow VST3/CLAP specifications exactly
- Archives are compressed for faster downloads
- No code changes needed - workflow handles everything!
