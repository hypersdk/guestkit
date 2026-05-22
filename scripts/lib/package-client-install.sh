#!/usr/bin/env bash
# GuestKit client runtime dependencies (libguestfs, qemu, nbd).
set -euo pipefail
echo "== GuestKit client dependencies =="
SUDO=""
[ "$(id -u)" -ne 0 ] && command -v sudo &>/dev/null && SUDO=sudo

pkg_install() { $SUDO "$@"; }

if command -v apt-get &>/dev/null; then
  pkg_install apt-get update -qq
  pkg_install apt-get install -y -qq \
    libguestfs-tools qemu-utils nbd-client lvm2 parted e2fsprogs \
    2>&1 | tail -10 || true
elif command -v dnf &>/dev/null; then
  PKG=dnf
  pkg_install "$PKG" install -y qemu-img nbd lvm2 parted e2fsprogs 2>&1 | tail -8 || true
  pkg_install "$PKG" install -y libguestfs libguestfs-tools 2>/dev/null \
    || pkg_install "$PKG" install -y libguestfs-tools 2>/dev/null || true
  if ! command -v qemu-nbd &>/dev/null; then
    pkg_install "$PKG" install -y qemu-nbd 2>/dev/null \
      || pkg_install "$PKG" install -y qemu-kvm-tools 2>/dev/null || true
  fi
elif command -v yum &>/dev/null; then
  PKG=yum
  pkg_install "$PKG" install -y qemu-img nbd lvm2 parted e2fsprogs libguestfs-tools 2>&1 | tail -8 || true
else
  echo "  WARNING: install libguestfs-tools qemu-utils nbd-client manually"
fi

$SUDO modprobe nbd max_part=16 2>/dev/null || true
echo "  qemu-img: $(command -v qemu-img 2>/dev/null || echo missing)"
echo "  qemu-nbd: $(command -v qemu-nbd 2>/dev/null || echo optional/missing)"
echo "  guestfish: $(command -v guestfish 2>/dev/null || echo optional)"
echo "Done."
