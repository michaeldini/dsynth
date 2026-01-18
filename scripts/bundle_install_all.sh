#!/usr/bin/env bash
set -euo pipefail

cd "$(dirname "$0")/.."

echo "==> checks"
./scripts/check.sh --no-release "$@"

echo "==> bundles"
./scripts/bundle_all.sh

echo "==> install CLAP plugins"
./scripts/install_clap_plugins.sh

echo "OK: checks, bundles, install completed"
