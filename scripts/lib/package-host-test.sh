#!/usr/bin/env bash
# Verify host prerequisites for GuestKit (offline VM disk inspection).
set -uo pipefail
ROOT="$(cd "$(dirname "$0")/.." && pwd)"
cd "$ROOT"

PASS=0 WARN=0 FAIL=0
SUDO=""
[[ "$(id -u)" -ne 0 ]] && command -v sudo &>/dev/null && SUDO=sudo
ok()   { echo "  OK: $*"; PASS=$((PASS + 1)); }
warn() { echo "  WARN: $*"; WARN=$((WARN + 1)); }
fail() { echo "  FAIL: $*"; FAIL=$((FAIL + 1)); }

echo "== GuestKit host test =="

if [[ -x ./guestkit ]]; then
  ok "guestkit binary in bundle"
elif command -v guestkit &>/dev/null; then
  ok "guestkit in PATH"
else
  fail "guestkit binary missing"
fi

if command -v qemu-img &>/dev/null; then
  ok "qemu-img ($(qemu-img --version 2>/dev/null | head -1))"
else
  fail "qemu-img not found (run ./install-client-deps.sh)"
fi

if command -v qemu-nbd &>/dev/null; then
  ok "qemu-nbd"
else
  warn "qemu-nbd not in PATH (some QCOW2/NBD paths limited)"
fi

if command -v guestfish &>/dev/null || ldconfig -p 2>/dev/null | grep -q libguestfs; then
  ok "libguestfs runtime"
else
  warn "libguestfs not detected — install libguestfs-tools"
fi

if lsmod 2>/dev/null | grep -q '^nbd '; then
  ok "nbd kernel module loaded"
elif $SUDO modprobe nbd max_part=16 2>/dev/null && lsmod | grep -q '^nbd '; then
  ok "nbd module loaded (modprobe)"
else
  warn "nbd module not loaded — run: sudo modprobe nbd max_part=16"
fi

echo ""
echo "Summary: ${PASS} ok, ${WARN} warn, ${FAIL} fail"
[[ "${FAIL}" -eq 0 ]]
