#!/usr/bin/env bash
set -euo pipefail

cd "$(dirname "$0")/.."

run_release=1
fix_fmt=0

usage() {
  cat <<'USAGE'
Usage: ./scripts/check.sh [--no-release] [--fix]

Runs the local "quality gate" checks:
  1) cargo fmt (check-only by default)
  2) cargo clippy (all targets; multiple feature sets)
  3) cargo test
  4) cargo test --release (optional; on by default)

Options:
  --no-release   Skip the release-mode test run
  --fix          Run `cargo fmt --all` to auto-format instead of failing
USAGE
}

while [[ $# -gt 0 ]]; do
  case "$1" in
    --no-release)
      run_release=0
      shift
      ;;
    --fix)
      fix_fmt=1
      shift
      ;;
    -h|--help)
      usage
      exit 0
      ;;
    *)
      echo "Unknown argument: $1" >&2
      usage >&2
      exit 2
      ;;
  esac
done

echo "==> rustfmt"
if [[ "$fix_fmt" -eq 1 ]]; then
  cargo fmt --all
else
  cargo fmt --all -- --check
fi

echo "==> clippy"
echo "  - default features"
cargo clippy --workspace --all-targets

echo "  - clap"
cargo clippy --workspace --all-targets --no-default-features --features clap

echo "  - kick-clap"
cargo clippy --workspace --all-targets --no-default-features --features kick-clap

echo "  - voice-clap"
cargo clippy --workspace --all-targets --no-default-features --features voice-clap

echo "==> tests (debug)"
cargo test -q

if [[ "$run_release" -eq 1 ]]; then
  echo "==> tests (release)"
  cargo test --release -q
fi

echo "OK: checks passed"
