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
