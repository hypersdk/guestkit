#!/usr/bin/env bash
set -euo pipefail
# shellcheck source=/dev/null
[[ -f "$(dirname "$0")/.package-lib/package-ui.sh" ]] && source "$(dirname "$0")/.package-lib/package-ui.sh"

pkg_banner "GuestKit host dependencies" "libguestfs · qemu · nbd"
SUDO=""
[[ "$(id -u)" -ne 0 ]] && command -v sudo &>/dev/null && SUDO=sudo
pkg_info "Installing libguestfs-tools, qemu-img, nbd…"
# Delegate to existing logic inline (same as before)
if command -v apt-get &>/dev/null; then
  $SUDO apt-get update -qq
  $SUDO apt-get install -y -qq libguestfs-tools qemu-utils nbd-client lvm2 parted 2>&1 | tail -5 || true
elif command -v dnf &>/dev/null; then
  $SUDO dnf install -y qemu-img nbd lvm2 parted e2fsprogs libvirt libguestfs-tools 2>&1 | tail -5 || true
fi
$SUDO modprobe nbd max_part=16 2>/dev/null || true
pkg_ok "Host packages configured"
pkg_summary "Dependencies"
