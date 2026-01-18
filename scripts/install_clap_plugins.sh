#!/usr/bin/env bash
set -euo pipefail

cd "$(dirname "$0")/.."

CLAP_DIR="$HOME/Library/Audio/Plug-Ins/CLAP"

echo "Installing DSynth CLAP plugins..."
mkdir -p "$CLAP_DIR"

plugins=(
  "target/bundled/DSynth.clap"
  "target/bundled/DSynthKick.clap"
  "target/bundled/DSynthVoice.clap"
)

for plugin in "${plugins[@]}"; do
  if [[ ! -e "$plugin" ]]; then
    echo "❌ Missing plugin bundle: $plugin"
    echo "   Run ./scripts/bundle_all.sh first."
    exit 1
  fi
done

for plugin in "${plugins[@]}"; do
  echo "Installing: $plugin"
  dest="$CLAP_DIR/$(basename "$plugin")"
  if [[ -d "$plugin" ]]; then
    rm -rf "$dest"
    cp -R "$plugin" "$CLAP_DIR/"
  else
    if [[ -d "$dest" ]]; then
      rm -rf "$dest"
    fi
    cp -f "$plugin" "$CLAP_DIR/"
  fi
  xattr -cr "$dest" || true
done

echo ""
echo "✅ Installed CLAP plugins to $CLAP_DIR"
