#!/bin/bash
# Run Phase 3 comprehensive tests
#
# This script runs the comprehensive Phase 3 API tests that create
# a fake Fedora disk image and exercise all new APIs.

set -e

echo "==================================================================="
echo "  Phase 3 Comprehensive API Testing"
echo "==================================================================="
echo ""

# Check for required tools
echo "Checking for required tools..."
for tool in cpio parted sgdisk; do
    if ! command -v $tool &> /dev/null; then
        echo "⚠  Warning: $tool not found (some tests may be skipped)"
    else
        echo "  ✓ $tool found"
    fi
done
echo ""

# Check disk space
echo "Checking available disk space..."
AVAILABLE=$(df /tmp | tail -1 | awk '{print $4}')
NEEDED=500000  # 500MB in KB

if [ $AVAILABLE -lt $NEEDED ]; then
    echo "⚠  Warning: Low disk space in /tmp"
    echo "   Available: ${AVAILABLE}KB"
    echo "   Recommended: ${NEEDED}KB"
    echo ""
fi

# Clean up any existing test images
echo "Cleaning up old test images..."
rm -f /tmp/phase3-test-*.img
rm -f /tmp/stat-test.img
rm -f /tmp/rm-test.img
rm -f /tmp/drive-test.img
rm -f /tmp/test-archive.cpio
rm -rf /tmp/cpio-test
echo "  ✓ Cleanup complete"
echo ""

# Run the tests
echo "==================================================================="
echo "  Running Tests"
echo "==================================================================="
echo ""

# Run with verbose output
export RUST_BACKTRACE=1

# Test 1: Comprehensive test
echo "Test 1: Comprehensive Phase 3 API test"
echo "-------------------------------------------------------------------"
if cargo test --test phase3_comprehensive test_phase3_comprehensive -- --nocapture --test-threads=1; then
    echo "  ✓ PASSED"
else
    echo "  ✗ FAILED"
    exit 1
fi
echo ""

# Test 2: stat vs lstat
echo "Test 2: stat() vs lstat() behavior"
echo "-------------------------------------------------------------------"
if cargo test --test phase3_comprehensive test_stat_vs_lstat_behavior -- --nocapture --test-threads=1; then
    echo "  ✓ PASSED"
else
    echo "  ✗ FAILED"
    exit 1
fi
echo ""

# Test 3: rm and rm_rf edge cases
echo "Test 3: rm() and rm_rf() edge cases"
echo "-------------------------------------------------------------------"
if cargo test --test phase3_comprehensive test_rm_rm_rf_edge_cases -- --nocapture --test-threads=1; then
    echo "  ✓ PASSED"
else
    echo "  ✗ FAILED"
    exit 1
fi
echo ""

# Test 4: create vs new
echo "Test 4: create() vs new() compatibility"
echo "-------------------------------------------------------------------"
if cargo test --test phase3_comprehensive test_create_vs_new -- --nocapture --test-threads=1; then
    echo "  ✓ PASSED"
else
    echo "  ✗ FAILED"
    exit 1
fi
echo ""

# Test 5: add_drive vs add_drive_ro
echo "Test 5: add_drive() vs add_drive_ro() behavior"
echo "-------------------------------------------------------------------"
if cargo test --test phase3_comprehensive test_add_drive_vs_add_drive_ro -- --nocapture --test-threads=1; then
    echo "  ✓ PASSED"
else
    echo "  ✗ FAILED"
    exit 1
fi
echo ""

# Final cleanup
echo "==================================================================="
echo "  Cleanup"
echo "==================================================================="
rm -f /tmp/phase3-test-*.img
rm -f /tmp/stat-test.img
rm -f /tmp/rm-test.img
rm -f /tmp/drive-test.img
rm -f /tmp/test-archive.cpio
rm -rf /tmp/cpio-test
echo "  ✓ Test artifacts removed"
echo ""

echo "==================================================================="
echo "  ✓ ALL PHASE 3 TESTS PASSED!"
echo "==================================================================="
echo ""
echo "Summary:"
echo "  • Guestfs::create() alias"
echo "  • add_drive() (read-write)"
echo "  • add_drive_ro() (read-only)"
echo "  • stat() (follows symlinks)"
echo "  • lstat() (doesn't follow symlinks)"
echo "  • rm() (single file)"
echo "  • rm_rf() (recursive force)"
echo "  • cpio_in() (CPIO extraction)"
echo "  • part_get_name() (GPT partition label)"
echo "  • part_set_parttype() (partition table type)"
echo ""
echo "All APIs tested and working correctly!"
