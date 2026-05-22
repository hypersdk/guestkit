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

# Run a command as root when needed (install-full, libvirt deps).
pkg_sudo() {
    if [[ "$(id -u)" -eq 0 ]]; then
        "$@"
    elif command -v sudo >/dev/null 2>&1; then
        sudo "$@"
    else
        pkg_fail "This step needs root. Re-run with sudo or as root."
        return 1
    fi
}

pkg_sudo_available() {
    [[ "$(id -u)" -eq 0 ]] && return 0
    command -v sudo >/dev/null 2>&1 && sudo -n true 2>/dev/null && return 0
    command -v sudo >/dev/null 2>&1 && return 0
    return 1
}

# Copy product.env.example → product.env when missing.
pkg_env_bootstrap() {
    local example="$1"
    local target="${2:-${example%.example}}"
    if [[ -f "${example}" && ! -f "${target}" ]]; then
        cp "${example}" "${target}"
        pkg_ok "Created ${target} (edit before production)"
        return 0
    fi
    if [[ -f "${target}" ]]; then
        pkg_ok "${target} already present"
    else
        pkg_warn "No ${example} — create ${target} manually"
    fi
}

# One-screen guide at start of ./install.sh
pkg_install_welcome() {
    local product="$1"
    echo ""
    pkg_box_begin "Automatic install — ${product}"
    pkg_box_line "You are in the extracted tarball — nothing else to download"
    pkg_box_line "We install OS deps, create config, verify binaries, run tests"
    pkg_box_line "Faster path: ./install-everything.sh (same + host/production checks)"
    pkg_box_end
    pkg_detail "This server: $(pkg_primary_host_label)"
    pkg_detail "Zyvor · https://zyvor.dev · © @zyvor 2026"
    echo ""
}

# Machina-style production install (systemd, TLS, firewall) when install-full.sh exists.
pkg_maybe_run_full_install() {
    [[ -x ./install-full.sh ]] || return 0
    if [[ "${ZYVOR_AUTO_INSTALL:-1}" == "0" ]]; then
        pkg_skip "install-full.sh (ZYVOR_AUTO_INSTALL=0)"
        return 0
    fi
    pkg_info "Production setup: systemd + TLS + firewall (install-full.sh)…"
    if pkg_sudo ./install-full.sh --open-firewall; then
        pkg_ok "Service installed and started"
    else
        pkg_warn "install-full.sh had issues — fix log then: sudo ./install-full.sh --open-firewall"
    fi
}

# Friendly finish banner with live URL on this host.
pkg_install_finish() {
    local product="$1" scheme="$2" port="$3" ui_path="${4:-}"
    shift 4
    local -a extras=()
    local line
    for line in "$@"; do
        [[ -n "${line}" ]] && extras+=("${line}")
    done

    local base host_label url
    base=$(pkg_access_url "${scheme}" "${port}")
    host_label=$(pkg_primary_host_label)
    url="${base}${ui_path}"

    pkg_summary "${product} — ready to use"
    echo ""
    pkg_box_begin "Open on this server"
    pkg_box_line "${url}" "${PKG_C_GREEN}${PKG_C_BOLD}"
    pkg_box_line "Network: ${host_label}" "${PKG_C_DIM}"
    if [[ "${base}" != *"://localhost:"* ]]; then
        pkg_box_line "On this machine: ${scheme}://127.0.0.1:${port}${ui_path}" "${PKG_C_DIM}"
    fi
    pkg_box_end

    local -a steps=("https://zyvor.dev · © @zyvor 2026")
    if [[ ${#extras[@]} -gt 0 ]]; then
        steps+=("${extras[@]}")
    fi
    steps+=("Re-run checks: ./test-package.sh" "Remove: ./uninstall.sh --yes [--remove-dir]")
    pkg_next_steps "${steps[@]}"
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
