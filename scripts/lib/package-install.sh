#!/usr/bin/env bash
set -euo pipefail
ROOT="$(cd "$(dirname "$0")" && pwd)"
cd "$ROOT"
# shellcheck source=/dev/null
[[ -f "${ROOT}/.package-lib/package-ui.sh" ]] && source "${ROOT}/.package-lib/package-ui.sh"

_PKG_SESSION_START=${SECONDS}
pkg_install_welcome "GuestKit"
pkg_banner "GuestKit" "Offline VM disk inspection · client bundle"
pkg_step_init 4

pkg_step "Host dependencies"
pkg_sudo ./install-client-deps.sh 2>/dev/null || ./install-client-deps.sh || pkg_warn "deps issues"
pkg_step_done

pkg_step "Configuration"
pkg_env_bootstrap guestkit.env.example guestkit.env
pkg_step_done

pkg_step "Verify binary"
[[ -x ./guestkit ]] && ./guestkit --version && pkg_ok "guestkit" || { pkg_fail "guestkit"; exit 1; }
pkg_step_done

pkg_step "Smoke test"
[[ -x ./test-package.sh ]] && ./test-package.sh || pkg_warn "test-package.sh"
[[ -x ./test-host.sh ]] && ./test-host.sh || true
pkg_step_done

pkg_summary "GuestKit — ready"
pkg_next_steps \
  "https://zyvor.dev · © @zyvor 2026" \
  "Try: ./guestkit inspect /path/to/disk.qcow2" \
  "./test-selftest.sh --quick (if bundled)" \
  "Docs: HOST_SETUP.txt · PREREQUISITES.txt" \
  "Remove: ./uninstall.sh --yes [--remove-dir]"
