#!/bin/bash
# Run Phase 3 Windows comprehensive tests
#
# This script runs the comprehensive Phase 3 API tests that create
# a fake Windows disk image and exercise all new APIs.

set -e

echo "==================================================================="
echo "  Phase 3 Windows API Testing"
echo "==================================================================="
echo ""

# Check for required tools
echo "Checking for required tools..."
for tool in cpio parted; do
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
rm -f /tmp/phase3-test-windows.img
rm -f /tmp/stat-test-windows.img
rm -f /tmp/rm-test-windows.img
rm -f /tmp/ntfs-test.img
rm -f /tmp/longpath-test.img
rm -f /tmp/test-archive-windows.cpio
rm -rf /tmp/cpio-test-windows
echo "  ✓ Cleanup complete"
echo ""

# Run the tests
echo "==================================================================="
echo "  Running Tests"
echo "==================================================================="
echo ""

# Run with verbose output
export RUST_BACKTRACE=1

# Test 1: Comprehensive Windows test
echo "Test 1: Comprehensive Phase 3 Windows API test"
echo "-------------------------------------------------------------------"
if cargo test --test phase3_windows test_phase3_windows_comprehensive -- --nocapture --test-threads=1; then
    echo "  ✓ PASSED"
else
    echo "  ✗ FAILED"
    exit 1
fi
echo ""

# Test 2: stat vs lstat on Windows paths
echo "Test 2: stat() vs lstat() on Windows paths"
echo "-------------------------------------------------------------------"
if cargo test --test phase3_windows test_windows_stat_vs_lstat -- --nocapture --test-threads=1; then
    echo "  ✓ PASSED"
else
    echo "  ✗ FAILED"
    exit 1
fi
echo ""

# Test 3: rm operations on Windows paths
echo "Test 3: rm() and rm_rf() on Windows paths"
echo "-------------------------------------------------------------------"
if cargo test --test phase3_windows test_windows_rm_operations -- --nocapture --test-threads=1; then
    echo "  ✓ PASSED"
else
    echo "  ✗ FAILED"
    exit 1
fi
echo ""

# Test 4: NTFS-specific features
echo "Test 4: NTFS-specific features"
echo "-------------------------------------------------------------------"
if cargo test --test phase3_windows test_windows_ntfs_features -- --nocapture --test-threads=1; then
    echo "  ✓ PASSED"
else
    echo "  ✗ FAILED"
    exit 1
fi
echo ""

# Test 5: Windows long path handling
echo "Test 5: Windows long path handling"
echo "-------------------------------------------------------------------"
if cargo test --test phase3_windows test_windows_long_paths -- --nocapture --test-threads=1; then
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
rm -f /tmp/phase3-test-windows.img
rm -f /tmp/stat-test-windows.img
rm -f /tmp/rm-test-windows.img
rm -f /tmp/ntfs-test.img
rm -f /tmp/longpath-test.img
rm -f /tmp/test-archive-windows.cpio
rm -rf /tmp/cpio-test-windows
echo "  ✓ Test artifacts removed"
echo ""

echo "==================================================================="
echo "  ✓ ALL PHASE 3 WINDOWS TESTS PASSED!"
echo "==================================================================="
echo ""
echo "Summary:"
echo "  • Tested on NTFS filesystem (Windows native)"
echo "  • Tested Windows-style directory structures"
echo "  • Tested MBR partition table (common for Windows)"
echo "  • Tested Windows long paths"
echo "  • Tested Windows line endings (\\r\\n)"
echo "  • All 10 Phase 3 APIs validated on Windows image"
echo ""
echo "All APIs work correctly with Windows disk images!"
