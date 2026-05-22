#!/usr/bin/env bash
# Build guestkit release binary on Linux (used by deploy-remote.sh on remote hosts).
# Usage: ./scripts/build-linux-release.sh
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$ROOT"

prepare_cargo_target() {
    if [ -n "${TMPDIR:-}" ] && [ ! -d "$TMPDIR" ]; then
        mkdir -p "$TMPDIR"
    fi
    if [ -d target/release ] && [ ! -d target/release/deps ]; then
        echo "  Incomplete target/release — cargo clean"
        cargo clean
    fi
}

prepare_cargo_target
echo "Building guestkit (release)..."
cargo build --release
echo "  ✅ $(./target/release/guestkit --version 2>/dev/null || echo built)"
