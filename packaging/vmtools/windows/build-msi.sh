#!/usr/bin/env bash
# Build Zeus VM Tools Windows MSI (WiX) or publish a zip scaffold.
set -euo pipefail

ROOT="$(cd "$(dirname "$0")/../../.." && pwd)"
VERSION="${VERSION:-0.1.0}"
DIST="${ROOT}/dist/vmtools/windows"
WXS="${ROOT}/packaging/vmtools/windows/zyvor-guest-agent.wxs"
STAGING="${DIST}/staging"

mkdir -p "${STAGING}"

build_windows_agent() {
  echo "Building zyvor-guest-agent for Windows (x86_64-pc-windows-gnu)..."
  cd "${ROOT}"
  rustup target add x86_64-pc-windows-gnu 2>/dev/null || true
  cargo build --release --features agent --no-default-features \
    --target x86_64-pc-windows-gnu --bin zyvor-guest-agent
  cp "target/x86_64-pc-windows-gnu/release/zyvor-guest-agent.exe" "${STAGING}/"
}

if command -v cargo >/dev/null 2>&1; then
  build_windows_agent || {
    echo "Windows cross-compile unavailable — place zyvor-guest-agent.exe in ${STAGING}/ manually."
    touch "${STAGING}/.agent-missing"
  }
else
  echo "cargo not found — skipping agent build."
fi

cp "${ROOT}/packaging/vmtools/windows/install.ps1" "${STAGING}/"

if [[ -f "${STAGING}/zyvor-guest-agent.exe" ]] && command -v candle >/dev/null && command -v light >/dev/null; then
  echo "Building MSI with WiX..."
  WIX_OBJ="${DIST}/zyvor-vm-tools.wixobj"
  sed "s/Version=\"0.1.0.0\"/Version=\"${VERSION}.0\"/" "${WXS}" > "${DIST}/zyvor-guest-agent.wxs"
  candle -out "${WIX_OBJ}" -dStaging="${STAGING}" "${DIST}/zyvor-guest-agent.wxs"
  light -out "${DIST}/zyvor-vm-tools-${VERSION}.msi" "${WIX_OBJ}"
  echo "MSI: ${DIST}/zyvor-vm-tools-${VERSION}.msi"
else
  echo "WiX not available or agent binary missing — publishing zip scaffold."
  (cd "${STAGING}" && zip -r "${DIST}/zyvor-vm-tools-windows-${VERSION}.zip" .)
  echo "Zip: ${DIST}/zyvor-vm-tools-windows-${VERSION}.zip"
  echo "For MSI: install WiX Toolset, ensure zyvor-guest-agent.exe is in staging, re-run."
fi
