#!/usr/bin/env bash
set -euo pipefail
ROOT="$(cd "$(dirname "$0")" && pwd)"
cd "$ROOT"
echo "== GuestKit package test =="
test -x ./guestkit || { echo "FAIL: ./guestkit missing"; exit 1; }
./guestkit --version && echo "  OK: guestkit --version"
./guestkit --help >/dev/null 2>&1 && echo "  OK: guestkit --help"
if [ -x ./test-host.sh ]; then
  ./test-host.sh || echo "  WARN: test-host.sh — see HOST_SETUP.txt"
fi
if [ -x ./test-selftest.sh ]; then
  ./test-selftest.sh --quick && echo "  OK: test-selftest.sh --quick" || echo "  WARN: test-selftest.sh (see HOST_SETUP.txt)"
fi
echo "Done."
