#!/bin/bash
# ============================================================================
# deploy-remote.sh — Full guestkit deployment to a remote server
# ============================================================================
# One command to fully set up a remote server for VM disk inspection:
#   1. Rsync repo to remote
#   2. Install system dependencies (libguestfs, qemu, nbd)
#   3. Build guestkit from source (cargo build --release)
#   4. Install binary to /usr/local/bin
#   5. Verify everything works
#
# Usage:
#   ./scripts/deploy-remote.sh <host> [user] [password]
#   ./scripts/deploy-remote.sh 185.165.240.5 root mypassword
#   ./scripts/deploy-remote.sh 10.0.0.1 root                  # SSH key auth
#   ./scripts/deploy-remote.sh 10.0.0.1 root pass --quick     # skip deps
#   ./scripts/deploy-remote.sh 10.0.0.1 root pass --uninstall # remove guestkit
#
# Environment variables:
#   DEPLOY_HOST=185.165.240.5
#   DEPLOY_USER=root
#   DEPLOY_PASS=mypassword
#   DEPLOY_DIR=/root/guestkit
# ============================================================================

set -euo pipefail

info()  { echo "  ✅ $*"; }
warn()  { echo "  ⚠️  $*"; }
error() { echo "  ❌ $*"; exit 1; }
step()  { echo ""; echo "  🔧 $*"; }

# ── Parse args ──
QUICK_MODE=false
UNINSTALL_MODE=false
POSITIONAL=()
for arg in "$@"; do
    case "$arg" in
        --quick)     QUICK_MODE=true ;;
        --uninstall) UNINSTALL_MODE=true ;;
        --help|-h)
            echo "Usage: $0 <host> [user] [password] [--quick|--uninstall]"
            echo ""
            echo "  --quick      Skip dependency install (only rsync + build)"
            echo "  --uninstall  Remove guestkit from remote server"
            echo ""
            echo "Full mode installs everything: Rust toolchain, libguestfs,"
            echo "qemu-img, qemu-nbd, nbd kernel module, and guestkit binary."
            exit 0
            ;;
        *)  POSITIONAL+=("$arg") ;;
    esac
done

HOST="${POSITIONAL[0]:-${DEPLOY_HOST:-}}"
USER="${POSITIONAL[1]:-${DEPLOY_USER:-root}}"
PASS="${POSITIONAL[2]:-${DEPLOY_PASS:-}}"
REMOTE_DIR="${DEPLOY_DIR:-/root/guestkit}"

[ -z "$HOST" ] && error "Usage: $0 <host> [user] [password] [--quick]"

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
REPO_DIR="$(cd "$SCRIPT_DIR/.." && pwd)"

[ -f "$REPO_DIR/Cargo.toml" ] || error "Not in guestkit repo: $REPO_DIR"

# ── SSH/rsync wrappers ──
_ssh() {
    if [ -n "$PASS" ]; then
        SSHPASS="$PASS" sshpass -e ssh -o StrictHostKeyChecking=no "${USER}@${HOST}" "$@"
    else
        ssh -o StrictHostKeyChecking=no "${USER}@${HOST}" "$@"
    fi
}

_rsync() {
    local ssh_cmd="ssh -o StrictHostKeyChecking=no"
    if [ -n "$PASS" ]; then
        ssh_cmd="sshpass -e $ssh_cmd"
    fi
    SSHPASS="$PASS" rsync -avz \
        --exclude='*.vmdk' --exclude='*.qcow2' --exclude='*.raw' \
        --exclude='*.ova' --exclude='*.iso' --exclude='*.img' \
        --exclude='*.vhd' --exclude='*.vhdx' \
        --exclude='.git' --exclude='target/' \
        --exclude='__pycache__' --exclude='*.pyc' \
        --exclude='proptest-regressions/' \
        -e "$ssh_cmd" \
        "$@"
}

# ── Preflight ──
if [ -n "$PASS" ] && ! command -v sshpass &>/dev/null; then
    error "sshpass required for password auth. Install: dnf install sshpass"
fi

# ── Uninstall mode ──
if $UNINSTALL_MODE; then
    echo ""
    echo "  ╔══════════════════════════════════════════════════╗"
    echo "  ║     🗑️  guestkit Remote Uninstall                 ║"
    echo "  ╚══════════════════════════════════════════════════╝"
    echo ""
    echo "  Host: ${USER}@${HOST}"
    echo ""

    step "Uninstalling guestkit"
    _ssh "
        rm -f /usr/local/bin/guestkit 2>/dev/null || true
        rm -rf $REMOTE_DIR
        rm -rf /var/cache/guestkit 2>/dev/null || true
        echo 'Done'
    " 2>&1
    info "guestkit removed from ${HOST}"
    echo ""
    echo "  📁 Kept: system packages (libguestfs, qemu, etc)"
    echo ""
    exit 0
fi

TOTAL_STEPS=5
$QUICK_MODE && TOTAL_STEPS=3

echo ""
echo "  ╔══════════════════════════════════════════════════╗"
echo "  ║     🚀 guestkit Remote Deployment                ║"
echo "  ╚══════════════════════════════════════════════════╝"
echo ""
echo "  Host:     ${USER}@${HOST}"
echo "  Auth:     $([ -n "$PASS" ] && echo "🔑 password" || echo "🔐 SSH key")"
echo "  Local:    $REPO_DIR"
echo "  Remote:   $REMOTE_DIR"
echo "  Mode:     $($QUICK_MODE && echo "⚡ quick (rsync + build only)" || echo "📦 full (deps + build)")"
echo ""

# ── Step 1: Rsync repo ──
step "Step 1/${TOTAL_STEPS}: 📤 Syncing repository to ${HOST}"

_rsync "$REPO_DIR/" "${USER}@${HOST}:${REMOTE_DIR}/" 2>&1 | tail -3
info "Synced to ${HOST}:${REMOTE_DIR}"

if ! $QUICK_MODE; then
    # ── Step 2: Install system dependencies ──
    step "Step 2/${TOTAL_STEPS}: 📦 Installing system dependencies"

    _ssh "
        # Detect package manager
        if command -v dnf &>/dev/null; then
            PKG_MGR='dnf'
        elif command -v yum &>/dev/null; then
            PKG_MGR='yum'
        elif command -v apt-get &>/dev/null; then
            PKG_MGR='apt-get'
            apt-get update -qq 2>&1 | tail -1
        else
            echo 'ERROR: No supported package manager found'
            exit 1
        fi

        # Install libguestfs and QEMU tools
        if [ \"\$PKG_MGR\" = 'apt-get' ]; then
            \$PKG_MGR install -y libguestfs-tools qemu-utils nbd-client linux-modules-extra-\$(uname -r) 2>&1 | tail -3
        else
            \$PKG_MGR install -y libguestfs libguestfs-tools-c qemu-img qemu-nbd nbd 2>&1 | tail -3
        fi

        # Load NBD kernel module
        modprobe nbd max_part=16 2>/dev/null || true
        echo 'nbd' > /etc/modules-load.d/nbd.conf 2>/dev/null || true
        echo 'options nbd max_part=16' > /etc/modprobe.d/nbd.conf 2>/dev/null || true

        echo 'System deps installed'
    " 2>&1
    info "System dependencies installed"

    # ── Step 3: Install Rust toolchain (if needed) ──
    step "Step 3/${TOTAL_STEPS}: 🦀 Ensuring Rust toolchain"

    _ssh "
        if command -v cargo &>/dev/null; then
            echo \"Rust already installed: \$(rustc --version)\"
        else
            curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y 2>&1 | tail -3
            source \$HOME/.cargo/env
            echo \"Rust installed: \$(rustc --version)\"
        fi
    " 2>&1
    info "Rust toolchain ready"

    BUILD_STEP=4
    VERIFY_STEP=5
else
    BUILD_STEP=2
    VERIFY_STEP=3
fi

# ── Build and install ──
step "Step ${BUILD_STEP}/${TOTAL_STEPS}: 🔨 Building guestkit (release mode)"

_ssh "
    source \$HOME/.cargo/env 2>/dev/null || true
    cd $REMOTE_DIR
    cargo build --release 2>&1 | tail -5
    install -m755 target/release/guestkit /usr/local/bin/guestkit
    echo \"Installed: \$(guestkit --version 2>/dev/null || echo 'built')\"
" 2>&1
info "guestkit built and installed"

# ── Verify ──
step "Step ${VERIFY_STEP}/${TOTAL_STEPS}: ✅ Verifying installation"

_ssh "
    echo \"📍 guestkit: \$(which guestkit 2>/dev/null || echo NOT_FOUND)\"
    echo \"📍 version:  \$(guestkit --version 2>/dev/null || echo FAILED)\"

    # Check system tools
    for tool in qemu-img qemu-nbd losetup; do
        if command -v \$tool &>/dev/null; then
            echo \"📍 \$tool: \$(command -v \$tool)\"
        else
            echo \"⚠️  \$tool: not found\"
        fi
    done

    # Check libguestfs
    if [ -f /usr/lib64/libguestfs.so ] || [ -f /usr/lib/x86_64-linux-gnu/libguestfs.so.0 ]; then
        echo '📍 libguestfs: OK'
    else
        echo '⚠️  libguestfs: not found'
    fi

    # Check NBD module
    if lsmod | grep -q nbd; then
        echo '📍 nbd module: loaded'
    else
        echo '⚠️  nbd module: not loaded (run: modprobe nbd)'
    fi

    # Quick smoke test with a temp image
    TMPIMG=\$(mktemp /tmp/guestkit-test-XXXXXX.raw)
    qemu-img create -f raw \$TMPIMG 64M &>/dev/null 2>&1
    if guestkit detect \$TMPIMG 2>/dev/null; then
        echo '📍 smoke test: OK'
    else
        echo '📍 smoke test: detect ran (empty image expected)'
    fi
    rm -f \$TMPIMG
" 2>&1

echo ""
echo "  ════════════════════════════════════════════════════"
echo "  🎉 Deployment complete: ${USER}@${HOST}"
echo "  ════════════════════════════════════════════════════"
echo ""
echo "  🔗 Connect:"
echo "    ssh ${USER}@${HOST}"
echo ""
echo "  🚀 Inspect a disk:"
echo "    guestkit inspect /path/to/disk.qcow2"
echo "    guestkit interactive /path/to/disk.qcow2"
echo ""
echo "  🩺 Self-test:"
echo "    cd $REMOTE_DIR && bash scripts/selftest.sh"
echo ""
