#!/usr/bin/env bash
# GuestKit — one-command client install (extracted tarball).
set -euo pipefail
ROOT="$(cd "$(dirname "$0")" && pwd)"
cd "$ROOT"

echo ""
echo "╔══════════════════════════════════════════════════════════╗"
echo "║  GuestKit client install                                 ║"
echo "╚══════════════════════════════════════════════════════════╝"
echo ""

echo "► Step 1/4 — Host dependencies (libguestfs, qemu, nbd)…"
if [ -x ./install-client-deps.sh ]; then
  sudo ./install-client-deps.sh || echo "  (fix deps manually if needed)"
else
  echo "  install-client-deps.sh not found — skip"
fi

echo ""
echo "► Step 2/4 — Configuration…"
if [ -f guestkit.env.example ] && [ ! -f guestkit.env ]; then
  cp guestkit.env.example guestkit.env
  echo "  Created guestkit.env (optional settings)"
elif [ -f guestkit.env ]; then
  echo "  guestkit.env already exists"
fi

echo ""
echo "► Step 3/4 — Verify binary…"
test -x ./guestkit || { echo "ERROR: ./guestkit missing"; exit 1; }
./guestkit --version
echo "  OK: guestkit binary"

echo ""
echo "► Step 4/4 — Smoke test…"
[ -x ./test-package.sh ] && ./test-package.sh || true

echo ""
echo "══════════════════════════════════════════════════════════"
echo "  Install complete."
echo ""
echo "  Inspect a disk image:"
echo "    ./guestkit inspect /path/to/disk.qcow2"
echo "    ./guestkit tui /path/to/disk.qcow2"
echo ""
echo "  Host checks:  ./test-host.sh"
echo "  Full checks:  ./test-selftest.sh"
echo "  Docs:         HOST_SETUP.txt  PREREQUISITES.txt"
echo "  Remove:       ./uninstall.sh --yes [--remove-dir]"
echo "══════════════════════════════════════════════════════════"
