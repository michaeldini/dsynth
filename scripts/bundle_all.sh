#!/usr/bin/env bash
set -euo pipefail

cd "$(dirname "$0")/.."

echo "==> bundle: DSynth CLAP (macOS)"
./scripts/bundle_clap_macos.sh

echo "==> bundle: DSynth Kick CLAP (macOS)"
./scripts/bundle_kick_clap_macos.sh

echo "==> bundle: DSynth Voice CLAP (macOS)"
./scripts/bundle_voice_clap_macos.sh

echo "==> build: DSynth Standalone (macOS)"
./scripts/bundle_standalone_macos.sh

echo "OK: all bundles built"
