#!/usr/bin/env bash
set -euo pipefail
ROOT="$(cd "$(dirname "$0")" && pwd)"
# shellcheck source=/dev/null
source "${ROOT}/.package-lib/package-uninstall-lib.sh"

PRODUCT="GuestKit"
BINARIES=(guestkit)
LOCAL_CONFIGS=(guestkit.env)
SYSTEM_PATHS=("${HOME}/.cache/guestkit")

package_uninstall_main "${PRODUCT}" "${ROOT}" "$@"
