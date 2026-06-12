#!/usr/bin/env bash
# Build zyvor-vm-tools release artifacts and optional ISO.
set -euo pipefail

ROOT="$(cd "$(dirname "$0")/../.." && pwd)"
VERSION="${VERSION:-0.1.0}"
DIST="${ROOT}/dist/vmtools"
ISO_DIR="${DIST}/iso-root"

echo "Building zyvor-guest-agent (musl)..."
cd "${ROOT}"
rustup target add x86_64-unknown-linux-musl 2>/dev/null || true
cargo build --release --features agent --no-default-features \
  --target x86_64-unknown-linux-musl --bin zyvor-guest-agent

mkdir -p "${DIST}/linux" "${ISO_DIR}/linux"
cp "target/x86_64-unknown-linux-musl/release/zyvor-guest-agent" "${DIST}/linux/zyvor-guest-agent"
cp templates/agent/zyvor-guest-agent.service "${DIST}/linux/"
tar czf "${DIST}/linux/zyvor-vm-tools-linux-amd64.tar.gz" \
  -C "${DIST}/linux" zyvor-guest-agent zyvor-guest-agent.service

cat > "${ISO_DIR}/linux/install.sh" <<'EOF'
#!/bin/sh
set -eu
ARCH="$(uname -m)"
if [ -f /etc/redhat-release ]; then
  rpm -Uvh --force linux/zyvor-vm-tools-*.rpm 2>/dev/null || cp linux/zyvor-guest-agent /usr/bin/zyvor-guest-agent
elif [ -f /etc/debian_version ]; then
  dpkg -i linux/zyvor-vm-tools_*.deb 2>/dev/null || cp linux/zyvor-guest-agent /usr/bin/zyvor-guest-agent
else
  cp linux/zyvor-guest-agent /usr/bin/zyvor-guest-agent
fi
chmod 755 /usr/bin/zyvor-guest-agent
install -Dm644 linux/zyvor-guest-agent.service /etc/systemd/system/zyvor-guest-agent.service
systemctl daemon-reload
systemctl enable --now zyvor-guest-agent
echo "Zyvor VM Tools installed"
EOF
chmod +x "${ISO_DIR}/linux/install.sh"
cp "${DIST}/linux/zyvor-guest-agent" "${ISO_DIR}/linux/"
cp "${DIST}/linux/zyvor-guest-agent.service" "${ISO_DIR}/linux/"

cat > "${ISO_DIR}/manifest.json" <<EOF
{"version":"${VERSION}","product":"Zeus VM Tools","platforms":["linux-amd64"]}
EOF

if command -v genisoimage >/dev/null; then
  genisoimage -o "${DIST}/zyvor-vm-tools.iso" -V ZYVOR_VM_TOOLS -r "${ISO_DIR}"
  echo "ISO: ${DIST}/zyvor-vm-tools.iso"
fi

echo "Artifacts in ${DIST}/linux"
