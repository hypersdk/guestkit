# Phase 3 API Testing Documentation

## Overview

Phase 3 introduced 10 critical libguestfs APIs to achieve 100% core API coverage (145/145 APIs). This document describes the comprehensive testing strategy that validates all APIs across multiple platforms and scenarios.

## Test Coverage

### APIs Tested

All 10 Phase 3 APIs are tested in realistic scenarios:

1. **`Guestfs::create()`** - Alias for `new()`, libguestfs compatibility
2. **`add_drive()`** - Add disk image in read-write mode
3. **`add_drive_ro()`** - Add disk image in read-only mode
4. **`stat()`** - Get file metadata (follows symlinks)
5. **`lstat()`** - Get file metadata (doesn't follow symlinks)
6. **`rm()`** - Remove single file (errors on directories)
7. **`rm_rf()`** - Recursive force removal (silent on non-existent)
8. **`cpio_in()`** - Extract CPIO archive
9. **`part_get_name()`** - Get GPT partition label
10. **`part_set_parttype()`** - Set partition table type

### Cross-Platform Testing

Tests validate functionality across different environments:

| Platform | Filesystem | Partition Table | Test Suite |
|----------|-----------|----------------|------------|
| **Linux (Fedora)** | ext4 | GPT | `tests/phase3_comprehensive.rs` |
| **Windows** | NTFS | MBR | `tests/phase3_windows.rs` |

### Test Scenarios

#### Linux (Fedora) Tests

**File:** `tests/phase3_comprehensive.rs`

Creates a 200MB Fedora-like disk image with:
- GPT partition table
- ext4 filesystems with labels (`boot`, `root`)
- Standard Linux directory structure (`/etc`, `/usr`, `/var`, etc.)
- Fedora release files and configuration
- Test files with various attributes
- Symbolic links

**Test Functions:**
1. `test_phase3_comprehensive()` - Complete workflow test (all 10 APIs)
2. `test_stat_vs_lstat_behavior()` - Symlink handling validation
3. `test_rm_rm_rf_edge_cases()` - Removal operation edge cases
4. `test_create_vs_new()` - API alias compatibility
5. `test_add_drive_vs_add_drive_ro()` - Read-only enforcement

**Run:** `./scripts/run_phase3_tests.sh`

#### Windows Tests

**File:** `tests/phase3_windows.rs`

Creates a 200MB Windows-like disk image with:
- MBR partition table
- NTFS filesystems with labels (`System Reserved`, `Windows`)
- Standard Windows directory structure (`C:\Windows`, `C:\Program Files`, etc.)
- Windows system files (fake registry, version info)
- Windows-style line endings (`\r\n`)
- Long Windows paths

**Test Functions:**
1. `test_phase3_windows_comprehensive()` - Complete workflow test (all 10 APIs)
2. `test_windows_stat_vs_lstat()` - Symlink handling on Windows paths
3. `test_windows_rm_operations()` - Removal operations on Windows paths
4. `test_windows_ntfs_features()` - NTFS-specific functionality
5. `test_windows_long_paths()` - Deep directory structure handling

**Run:** `./scripts/run_phase3_windows_tests.sh`

## Running Tests

### Quick Start

```bash
# Run all Phase 3 tests (Fedora + Windows)
./scripts/run_all_phase3_tests.sh

# Run only Fedora tests
./scripts/run_phase3_tests.sh

# Run only Windows tests
./scripts/run_phase3_windows_tests.sh

# Run specific test
cargo test --test phase3_comprehensive test_stat_vs_lstat_behavior -- --nocapture
```

### Requirements

**System Tools:**
- `cpio` - For CPIO archive operations
- `parted` - For partition table operations
- `sgdisk` - For GPT partition operations (Fedora tests)

**Disk Space:**
- Minimum: 500MB free in `/tmp`
- Recommended: 1GB for comfortable testing

### Test Output

Tests provide detailed output showing:
- Each API being tested with step numbers
- Success/failure indicators (✓/✗)
- File sizes, partition information, and metadata
- Error messages if tests fail

Example output:
```
[1/10] Testing Guestfs::create() alias...
  Creating 200MB disk image...
  ✓ Drive added in read-write mode

[2/10] Testing add_drive() (read-write mode)...
  ✓ Drive added in read-write mode

[4/10] Testing stat() on regular file...
  File size: 26 bytes
  Mode: 0100644
  UID: 0, GID: 0
  ✓ stat() works correctly
```

## Test Methodology

### Realistic Image Creation

Following libguestfs testing practices, we create realistic disk images that:
- Use proper partition tables (GPT for Linux, MBR for Windows)
- Create appropriate filesystems (ext4, NTFS)
- Populate with OS-specific directory structures
- Include system files and configuration

This ensures APIs work in production-like scenarios, not just trivial cases.

### Comprehensive API Validation

Each API is tested:
1. **Positive cases** - Normal, expected usage
2. **Negative cases** - Error conditions and edge cases
3. **Cross-platform** - Same API on different OS types
4. **Integration** - APIs working together in workflows

### Examples of Edge Cases Tested

**`rm()` edge cases:**
- File that doesn't exist → should error
- Directory instead of file → should error
- Valid file → should succeed

**`rm_rf()` edge cases:**
- Non-existent path → should succeed silently
- Empty directory → should remove
- Nested directory tree → should remove recursively
- Single file → should remove

**`stat()` vs `lstat()` edge cases:**
- Regular file → both should return same metadata
- Symbolic link → stat follows, lstat doesn't
- Broken symlink → stat errors, lstat succeeds

**Read-only enforcement:**
- Read operations on read-only mount → should succeed
- Write operations on read-only mount → should error

## Test Statistics

### Overall Coverage

- **Total Phase 3 APIs:** 10
- **Test files:** 2 (Fedora + Windows)
- **Test functions:** 10
- **Test scenarios:** 50+
- **Lines of test code:** ~1,100
- **Disk images created:** 5 per full test run

### Test Execution Time

Approximate times (varies by system):
- Fedora tests: 15-30 seconds
- Windows tests: 15-30 seconds
- Total (both): 30-60 seconds

## Continuous Integration

These tests are designed to run in CI/CD pipelines:

```yaml
# Example GitHub Actions workflow
- name: Run Phase 3 Tests
  run: |
    sudo apt-get install -y cpio parted gdisk
    ./scripts/run_all_phase3_tests.sh
```

## Troubleshooting

### Common Issues

**"cpio not found"**
```bash
# Ubuntu/Debian
sudo apt-get install cpio

# Fedora/RHEL
sudo dnf install cpio
```

**"Low disk space"**
- Tests need ~500MB in `/tmp`
- Clean up old test images: `rm -f /tmp/phase3-test-*.img`

**"Permission denied"**
- Some operations may require elevated privileges
- Run with `sudo` if needed: `sudo ./scripts/run_all_phase3_tests.sh`

**Tests hang or timeout**
- Check if qemu-nbd is running: `pgrep qemu-nbd`
- Kill stale processes: `pkill qemu-nbd`

## Future Enhancements

Planned test improvements:
- [ ] macOS disk image testing (HFS+/APFS)
- [ ] Extended attributes testing
- [ ] ACL testing
- [ ] Compression testing (qcow2)
- [ ] Snapshot testing
- [ ] Performance benchmarking
- [ ] Memory leak detection

## References

- [Phase 3 Implementation Status](../GUESTFS_IMPLEMENTATION_STATUS.md)
- [Phase 3 Code Review](../PHASE3_REVIEW.md)
- [API Coverage Report](../docs/MISSING_APIS.md)
- [libguestfs Documentation](https://libguestfs.org/guestfs.3.html)
