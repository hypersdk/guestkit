# shellcheck shell=bash
# Shared UI helpers for VMRogue-family client packaging scripts.
# Source from scripts/lib/*.sh or .package-lib/package-ui.sh inside tarballs.

[[ -n "${_PKG_UI_LOADED:-}" ]] && return 0
_PKG_UI_LOADED=1

PKG_UI_WIDTH="${PKG_UI_WIDTH:-62}"
PKG_STEP=0
PKG_STEP_TOTAL="${PKG_STEP_TOTAL:-0}"
_PKG_STEP_START=0
_PKG_COUNTERS_OK=0
_PKG_COUNTERS_WARN=0
_PKG_COUNTERS_FAIL=0
_PKG_COUNTERS_SKIP=0
_PKG_SESSION_START=${SECONDS:-0}

pkg_ui_tty() {
    [[ -t 1 ]] && [[ -z "${NO_COLOR:-}" ]]
}

if pkg_ui_tty; then
    PKG_C_RESET=$'\033[0m'
    PKG_C_BOLD=$'\033[1m'
    PKG_C_DIM=$'\033[2m'
    PKG_C_CYAN=$'\033[36m'
    PKG_C_GREEN=$'\033[32m'
    PKG_C_YELLOW=$'\033[33m'
    PKG_C_RED=$'\033[31m'
    PKG_C_BLUE=$'\033[34m'
    PKG_C_MAGENTA=$'\033[35m'
else
    PKG_C_RESET="" PKG_C_BOLD="" PKG_C_DIM="" PKG_C_CYAN=""
    PKG_C_GREEN="" PKG_C_YELLOW="" PKG_C_RED="" PKG_C_BLUE="" PKG_C_MAGENTA=""
fi

pkg_divider() {
    local ch="${1:-─}"
    printf '%*s\n' "${PKG_UI_WIDTH}" '' | tr ' ' "${ch}"
}

pkg_banner() {
    local title="$1" subtitle="${2:-}"
    echo ""
    pkg_divider "═"
    printf "%s%s%s\n" "${PKG_C_BOLD}${PKG_C_CYAN}" "${title}" "${PKG_C_RESET}"
    [[ -n "${subtitle}" ]] && printf "%s%s%s\n" "${PKG_C_DIM}" "${subtitle}" "${PKG_C_RESET}"
    pkg_divider "═"
    echo ""
}

pkg_box_line() {
    local msg="$1" color="${2:-}"
    local inner=$((PKG_UI_WIDTH - 4))
    printf "  %s┃%s %-*s %s┃%s\n" \
        "${color}" "${PKG_C_RESET}" "${inner}" "${msg}" "${color}" "${PKG_C_RESET}"
}

pkg_box_begin() {
    local title="${1:-}"
    echo ""
    pkg_box_line "${title}" "${PKG_C_BOLD}${PKG_C_BLUE}"
    pkg_divider "─"
}

pkg_box_end() {
    pkg_divider "─"
    echo ""
}

pkg_step_init() {
    local total="$1"
    PKG_STEP_TOTAL="${total}"
    PKG_STEP=0
}

pkg_step() {
    local label="$1"
    PKG_STEP=$((PKG_STEP + 1))
    _PKG_STEP_START=${SECONDS}
    echo ""
    if [[ "${PKG_STEP_TOTAL}" -gt 0 ]]; then
        printf "%s▶ Step %d/%d — %s%s\n" \
            "${PKG_C_BOLD}${PKG_C_MAGENTA}" "${PKG_STEP}" "${PKG_STEP_TOTAL}" "${label}" "${PKG_C_RESET}"
    else
        printf "%s▶ %s%s\n" "${PKG_C_BOLD}${PKG_C_MAGENTA}" "${label}" "${PKG_C_RESET}"
    fi
}

pkg_step_done() {
    local elapsed=$((SECONDS - _PKG_STEP_START))
    printf "%s  ✓ done (%ds)%s\n" "${PKG_C_DIM}" "${elapsed}" "${PKG_C_RESET}"
}

pkg_ok() {
    _PKG_COUNTERS_OK=$((_PKG_COUNTERS_OK + 1))
    printf "  %s✓%s %s\n" "${PKG_C_GREEN}" "${PKG_C_RESET}" "$*"
}

pkg_warn() {
    _PKG_COUNTERS_WARN=$((_PKG_COUNTERS_WARN + 1))
    printf "  %s⚠%s %s\n" "${PKG_C_YELLOW}" "${PKG_C_RESET}" "$*"
}

pkg_fail() {
    _PKG_COUNTERS_FAIL=$((_PKG_COUNTERS_FAIL + 1))
    printf "  %s✗%s %s\n" "${PKG_C_RED}" "${PKG_C_RESET}" "$*"
}

pkg_skip() {
    _PKG_COUNTERS_SKIP=$((_PKG_COUNTERS_SKIP + 1))
    printf "  %s○%s %s\n" "${PKG_C_DIM}" "${PKG_C_RESET}" "$*"
}

pkg_info() {
    printf "  %s›%s %s\n" "${PKG_C_CYAN}" "${PKG_C_RESET}" "$*"
}

pkg_detail() {
    printf "    %s%s%s\n" "${PKG_C_DIM}" "$*" "${PKG_C_RESET}"
}

pkg_counters_reset() {
    _PKG_COUNTERS_OK=0 _PKG_COUNTERS_WARN=0 _PKG_COUNTERS_FAIL=0 _PKG_COUNTERS_SKIP=0
}

pkg_summary() {
    local title="${1:-Summary}"
    echo ""
    pkg_box_begin "${title}"
    pkg_box_line "Passed:  ${_PKG_COUNTERS_OK}" "${PKG_C_GREEN}"
    [[ "${_PKG_COUNTERS_WARN}" -gt 0 ]] && pkg_box_line "Warnings: ${_PKG_COUNTERS_WARN}" "${PKG_C_YELLOW}"
    [[ "${_PKG_COUNTERS_SKIP}" -gt 0 ]] && pkg_box_line "Skipped:  ${_PKG_COUNTERS_SKIP}" "${PKG_C_DIM}"
    [[ "${_PKG_COUNTERS_FAIL}" -gt 0 ]] && pkg_box_line "Failed:  ${_PKG_COUNTERS_FAIL}" "${PKG_C_RED}"
    local elapsed=$((SECONDS - _PKG_SESSION_START))
    pkg_box_line "Elapsed: ${elapsed}s" "${PKG_C_DIM}"
    pkg_box_end
}

pkg_phase() {
    local phase="$1"
    echo ""
    printf "%s━━ %s ━━%s\n" "${PKG_C_BOLD}${PKG_C_BLUE}" "${phase}" "${PKG_C_RESET}"
}

pkg_next_steps() {
    local -a lines=("$@")
    echo ""
    printf "%s%sNext steps%s\n" "${PKG_C_BOLD}" "${PKG_C_CYAN}" "${PKG_C_RESET}"
    local line
    for line in "${lines[@]}"; do
        pkg_info "${line}"
    done
    echo ""
}

# One-screen install guide shown at start of ./install.sh (hassle-free path).
pkg_install_welcome() {
    local product="$1"
    echo ""
    pkg_box_begin "Hassle-free install"
    pkg_box_line "No git clone · no compile on this machine (Python bundles use venv/)"
    pkg_box_line "1. You are already in the extracted tarball folder"
    pkg_box_line "2. This script installs deps + verifies binaries"
    pkg_box_line "3. Run ./test-package.sh when finished"
    pkg_box_end
    pkg_detail "Suite: https://zyvor.dev · © @zyvor 2026"
    [[ -n "${product}" ]] && pkg_detail "Product: ${product}"
    echo ""
}

# Best-effort routable IPv4 for install URLs (default-route source, then first global UP iface).
pkg_primary_ipv4() {
    local ip=""
    if command -v ip >/dev/null 2>&1; then
        ip=$(ip -4 route get 1.1.1.1 2>/dev/null | awk '{for (i = 1; i <= NF; i++) if ($i == "src") { print $(i + 1); exit }}')
        if [[ -z "${ip}" || "${ip}" == "127.0.0.1" ]]; then
            ip=$(
                ip -4 -o addr show scope global up 2>/dev/null | awk '
                    $2 !~ /^(lo|docker|virbr|veth|br-|cni|flannel|tailscale|wg)/ {
                        split($4, a, "/"); print a[1]; exit
                    }'
            )
        fi
    fi
    if [[ -z "${ip}" ]]; then
        ip=$(hostname -I 2>/dev/null | awk '{print $1}')
    fi
    if [[ -n "${ip}" && "${ip}" != "127.0.0.1" ]]; then
        echo "${ip}"
    else
        echo "127.0.0.1"
    fi
}

# Human label, e.g. 212.8.252.194 (eno2)
pkg_primary_host_label() {
    local ip iface
    ip=$(pkg_primary_ipv4)
    if [[ "${ip}" == "127.0.0.1" ]]; then
        echo "localhost"
        return 0
    fi
    iface=$(
        ip -4 -o addr show scope global 2>/dev/null | awk -v want="${ip}" '
            { split($4, a, "/"); if (a[1] == want) { print $2; exit } }'
    )
    if [[ -n "${iface}" ]]; then
        echo "${ip} (${iface})"
    else
        echo "${ip}"
    fi
}

# scheme://host:port with detected host (http or https).
pkg_access_url() {
    local scheme="${1:-http}" port="${2:-80}"
    local ip
    ip=$(pkg_primary_ipv4)
    if [[ "${ip}" == "127.0.0.1" ]]; then
        echo "${scheme}://localhost:${port}"
    else
        echo "${scheme}://${ip}:${port}"
    fi
}

pkg_install_done_message() {
    local product="${1:-}"
    pkg_summary "Install complete"
    pkg_next_steps \
        "Web UI footer: https://zyvor.dev · © @zyvor 2026" \
        "Questions: https://zyvor.dev" \
        "Remove: ./uninstall.sh --yes [--remove-dir]"
    [[ -n "${product}" ]] && pkg_ok "Ready — ${product}"
}

pkg_source_ui() {
    local root="${1:-}"
    if [[ -f "${root}/.package-lib/package-ui.sh" ]]; then
        # shellcheck source=/dev/null
        source "${root}/.package-lib/package-ui.sh"
    elif [[ -f "$(dirname "${BASH_SOURCE[1]:-$0}")/package-ui.sh" ]]; then
        # shellcheck source=/dev/null
        source "$(dirname "${BASH_SOURCE[1]:-$0}")/package-ui.sh"
    fi
}
