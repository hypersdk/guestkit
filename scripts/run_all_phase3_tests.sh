#!/bin/bash
# Run all Phase 3 comprehensive tests (Fedora + Windows)
#
# This script runs all Phase 3 API tests covering both Linux (Fedora)
# and Windows scenarios to ensure complete cross-platform compatibility.

set -e

echo "==================================================================="
echo "  Phase 3 Complete Cross-Platform API Testing"
echo "==================================================================="
echo ""
echo "This will test all 10 Phase 3 APIs on both:"
echo "  • Fedora-like disk image (ext4, GPT, Linux paths)"
echo "  • Windows-like disk image (NTFS, MBR, Windows paths)"
echo ""
echo "==================================================================="
echo ""

# Track overall success
FEDORA_SUCCESS=0
WINDOWS_SUCCESS=0

# Run Fedora tests
echo "┌─────────────────────────────────────────────────────────────────┐"
echo "│                    FEDORA TESTS                                  │"
echo "└─────────────────────────────────────────────────────────────────┘"
echo ""

if bash "$(dirname "$0")/run_phase3_tests.sh"; then
    FEDORA_SUCCESS=1
    echo ""
    echo "✅ Fedora tests: PASSED"
else
    echo ""
    echo "❌ Fedora tests: FAILED"
fi

echo ""
echo "==================================================================="
echo ""

# Run Windows tests
echo "┌─────────────────────────────────────────────────────────────────┐"
echo "│                    WINDOWS TESTS                                 │"
echo "└─────────────────────────────────────────────────────────────────┘"
echo ""

if bash "$(dirname "$0")/run_phase3_windows_tests.sh"; then
    WINDOWS_SUCCESS=1
    echo ""
    echo "✅ Windows tests: PASSED"
else
    echo ""
    echo "❌ Windows tests: FAILED"
fi

echo ""
echo "==================================================================="
echo "  FINAL RESULTS"
echo "==================================================================="
echo ""

if [ $FEDORA_SUCCESS -eq 1 ] && [ $WINDOWS_SUCCESS -eq 1 ]; then
    echo "🎉 SUCCESS: All Phase 3 tests passed on both platforms!"
    echo ""
    echo "Test Coverage:"
    echo "  ✓ Linux (Fedora-like): ext4, GPT, Unix paths"
    echo "  ✓ Windows: NTFS, MBR, Windows paths"
    echo "  ✓ 10 Phase 3 APIs validated"
    echo "  ✓ 10 test functions executed"
    echo "  ✓ 50+ test scenarios covered"
    echo ""
    echo "APIs Tested:"
    echo "  • Guestfs::create() - Handle creation alias"
    echo "  • add_drive() - Read-write drive mounting"
    echo "  • add_drive_ro() - Read-only drive mounting"
    echo "  • stat() - File metadata (follows symlinks)"
    echo "  • lstat() - File metadata (doesn't follow symlinks)"
    echo "  • rm() - Single file removal"
    echo "  • rm_rf() - Recursive force removal"
    echo "  • cpio_in() - CPIO archive extraction"
    echo "  • part_get_name() - Get partition label"
    echo "  • part_set_parttype() - Set partition table type"
    echo ""
    echo "Cross-platform compatibility: ✅ VERIFIED"
    echo ""
    exit 0
else
    echo "❌ FAILURE: Some tests failed"
    echo ""
    if [ $FEDORA_SUCCESS -eq 0 ]; then
        echo "  ✗ Fedora tests failed"
    else
        echo "  ✓ Fedora tests passed"
    fi
    if [ $WINDOWS_SUCCESS -eq 0 ]; then
        echo "  ✗ Windows tests failed"
    else
        echo "  ✓ Windows tests passed"
    fi
    echo ""
    echo "Please review the test output above for details."
    echo ""
    exit 1
fi
