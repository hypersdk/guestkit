#!/usr/bin/env bash
set -uo pipefail
ROOT="$(cd "$(dirname "$0")" && pwd)"
cd "$ROOT"
export PKG_INSTALL_ROOT="${ROOT}"
# shellcheck source=/dev/null
[[ -f "${ROOT}/.package-lib/package-ui.sh" ]] && source "${ROOT}/.package-lib/package-ui.sh"

_PKG_SESSION_START=${SECONDS}
pkg_counters_reset
pkg_banner "GuestKit host test" "qemu-img · nbd · GuestKit"
SUDO=""
[[ "$(id -u)" -ne 0 ]] && command -v sudo &>/dev/null && SUDO=sudo

[[ -x ./guestkit ]] && pkg_ok "guestkit binary" || pkg_fail "guestkit"
command -v qemu-img &>/dev/null && pkg_ok "qemu-img" || pkg_fail "qemu-img"
command -v qemu-nbd &>/dev/null && pkg_ok "qemu-nbd" || pkg_warn "qemu-nbd optional"
command -v guestfish &>/dev/null && pkg_ok "guestfish (optional legacy tool)" || pkg_ok "guestfish not installed (GuestKit does not require libguestfs)"
lsmod 2>/dev/null | grep -q '^nbd ' && pkg_ok "nbd module" || pkg_warn "nbd not loaded"

pkg_summary "Host readiness"
[[ "${_PKG_COUNTERS_FAIL}" -eq 0 ]]
