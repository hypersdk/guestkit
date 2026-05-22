#!/usr/bin/env bash
set -euo pipefail
ROOT="$(cd "$(dirname "$0")" && pwd)"
cd "$ROOT"
# shellcheck source=/dev/null
[[ -f "${ROOT}/.package-lib/package-ui.sh" ]] && source "${ROOT}/.package-lib/package-ui.sh"

_PKG_SESSION_START=${SECONDS}
pkg_install_welcome "guestkit"

pkg_banner "GuestKit client install" "Offline VM disk inspection · not Kubernetes"
pkg_step_init 4

pkg_step "Host dependencies (libguestfs, qemu, nbd)"
sudo ./install-client-deps.sh 2>/dev/null || ./install-client-deps.sh || pkg_warn "deps issues"
pkg_step_done

pkg_step "Configuration"
[[ -f guestkit.env.example ]] && [[ ! -f guestkit.env ]] && cp guestkit.env.example guestkit.env && pkg_ok "guestkit.env created" || pkg_ok "config OK"
pkg_step_done

pkg_step "Verify binary"
[[ -x ./guestkit ]] && ./guestkit --version && pkg_ok "guestkit" || { pkg_fail "guestkit"; exit 1; }
pkg_step_done

pkg_step "Tests"
[[ -x ./test-package.sh ]] && ./test-package.sh || true
pkg_step_done

pkg_summary "Install complete"
pkg_next_steps \
  "https://zyvor.dev · © @zyvor 2026" \
  "./test-host.sh && ./test-selftest.sh --quick" \
  "./guestkit inspect /path/to/disk.qcow2" \
  "Docs: HOST_SETUP.txt · PREREQUISITES.txt"
