# GuestCtl Testing Guide

This document describes how to run tests for GuestCtl, including both Rust and Python tests.

## Table of Contents

- [Rust Tests](#rust-tests)
- [Python Tests](#python-tests)
- [Integration Tests](#integration-tests)
- [Test Coverage](#test-coverage)
- [CI/CD](#cicd)

## Rust Tests

### Running All Rust Tests

```bash
# Run all tests
cargo test

# Run tests with output
cargo test -- --nocapture

# Run specific test
cargo test test_name

# Run tests in a specific file
cargo test --test integration_basic
```

### Test Categories

#### Unit Tests

Located in `src/` alongside the code:
- Guestfs API tests
- Disk operations tests
- Partition parsing tests
- Filesystem detection tests

```bash
# Run only unit tests (library tests)
cargo test --lib
```

#### Integration Tests

Located in `tests/` directory:
- `integration_basic.rs` - Basic functionality tests
- `integration_error_handling.rs` - Error handling tests
- `integration_lvm_luks.rs` - LVM and LUKS tests
- `ubuntu_realistic.rs` - Ubuntu VM simulation
- `debian_realistic.rs` - Debian VM simulation
- `arch_realistic.rs` - Arch Linux VM simulation
- `windows_realistic.rs` - Windows VM simulation
- `phase3_comprehensive.rs` - Comprehensive tests
- `phase3_windows.rs` - Windows-specific tests
- `panic_safety.rs` - Panic safety tests
- `security_tests.rs` - Security tests
- `output_formats_test.rs` - Output format tests

```bash
# Run integration tests
cargo test --test integration_basic

# Run realistic distribution tests
cargo test --test ubuntu_realistic
cargo test --test debian_realistic
cargo test --test arch_realistic
cargo test --test windows_realistic
```

#### Documentation Tests

Embedded in documentation comments:

```bash
# Run doc tests
cargo test --doc
```

### Test with Features

```bash
# Test with all features
cargo test --all-features

# Test with specific features
cargo test --features python-bindings
cargo test --features guest-inspect
cargo test --features disk-ops
```

### Release Mode Tests

```bash
# Run tests in release mode (faster, optimized)
cargo test --release
```

## Python Tests

### Prerequisites

```bash
# Install pytest
pip install pytest

# Or in virtual environment
source .venv/bin/activate
pip install pytest
```

### Running Python Tests

```bash
# Run all Python tests
pytest tests/test_python_bindings.py -v

# Run specific test
pytest tests/test_python_bindings.py::test_import_guestctl -v

# Run with detailed output
pytest tests/test_python_bindings.py -vv

# Run tests matching pattern
pytest tests/test_python_bindings.py -k "guestfs" -v
```

### Test Categories

#### Basic Tests (No Disk Required)

These tests run without needing a disk image:
- Module import tests
- Class creation tests
- Method existence tests
- Error handling tests
- Version consistency tests

```bash
# Run only basic tests (skip image-required tests)
pytest tests/test_python_bindings.py -m "not requires_image" -v
```

#### Disk Image Tests

These tests require a VM disk image to test full functionality:
- OS inspection
- Device operations
- Filesystem operations
- Mount and file operations
- Package listing

```bash
# Set disk image and run all tests
export GUESTKIT_TEST_IMAGE=/path/to/test.qcow2
pytest tests/test_python_bindings.py -v

# Run only image-requiring tests
pytest tests/test_python_bindings.py -m requires_image -v
```

### Test Results

Example output:
```
============================= test session starts ==============================
platform linux -- Python 3.14.2, pytest-9.0.2, pluggy-1.6.0
collected 16 items

tests/test_python_bindings.py::test_import_guestctl PASSED               [  6%]
tests/test_python_bindings.py::test_import_classes PASSED                [ 12%]
tests/test_python_bindings.py::test_guestfs_creation PASSED              [ 18%]
tests/test_python_bindings.py::test_guestfs_methods_exist PASSED         [ 25%]
tests/test_python_bindings.py::test_disk_converter_creation PASSED       [ 31%]
tests/test_python_bindings.py::test_disk_converter_methods_exist PASSED  [ 37%]
tests/test_python_bindings.py::test_guestfs_method_count PASSED          [ 43%]
tests/test_python_bindings.py::test_error_handling PASSED                [ 50%]
tests/test_python_bindings.py::test_version_consistency PASSED           [ 56%]
tests/test_python_bindings.py::TestWithDiskImage::test_add_drive_and_launch SKIPPED [ 62%]
tests/test_python_bindings.py::TestWithDiskImage::test_inspect_os SKIPPED [ 68%]
tests/test_python_bindings.py::TestWithDiskImage::test_device_operations SKIPPED [ 75%]
tests/test_python_bindings.py::TestWithDiskImage::test_filesystem_operations SKIPPED [ 81%]
tests/test_python_bindings.py::TestWithDiskImage::test_mount_and_file_operations SKIPPED [ 87%]
tests/test_python_bindings.py::TestWithDiskImage::test_package_listing SKIPPED [ 93%]
tests/test_python_bindings.py::TestDiskConverter::test_detect_format_nonexistent PASSED [100%]

======================== 10 passed, 6 skipped in 0.08s =========================
```

## Integration Tests

### Manual Integration Testing

#### Test with Real Disk Images

```bash
# Test CLI with real disk
sudo ./target/release/guestctl inspect /path/to/vm.qcow2

# Test Python bindings
cd examples/python
sudo python3 test_bindings.py /path/to/vm.qcow2
```

#### Test Different Formats

```bash
# QCOW2
sudo ./target/release/guestctl inspect disk.qcow2

# VMDK
sudo ./target/release/guestctl inspect disk.vmdk

# RAW
sudo ./target/release/guestctl inspect disk.img
```

## Test Coverage

### Rust Test Coverage

```bash
# Install tarpaulin for coverage
cargo install cargo-tarpaulin

# Generate coverage report
cargo tarpaulin --out Html --output-dir coverage

# View coverage
firefox coverage/index.html
```

### Python Test Coverage

```bash
# Install pytest-cov
pip install pytest-cov

# Run tests with coverage
pytest tests/test_python_bindings.py --cov=guestctl --cov-report=html

# View coverage
firefox htmlcov/index.html
```

## Test Best Practices

### Writing Rust Tests

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_feature() {
        // Arrange
        let input = setup_test_data();

        // Act
        let result = function_under_test(input);

        // Assert
        assert_eq!(result, expected_value);
    }

    #[test]
    #[should_panic(expected = "error message")]
    fn test_error_handling() {
        function_that_should_panic();
    }
}
```

### Writing Python Tests

```python
def test_feature():
    """Test description"""
    from guestctl import Guestfs

    # Arrange
    g = Guestfs()

    try:
        # Act
        result = g.some_method()

        # Assert
        assert result is not None
        assert isinstance(result, str)
    finally:
        # Cleanup
        g.shutdown()
```

## CI/CD

### GitHub Actions

Example `.github/workflows/test.yml`:

```yaml
name: Tests

on: [push, pull_request]

jobs:
  rust-tests:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v2
      - uses: actions-rs/toolchain@v1
        with:
          toolchain: stable
      - name: Run tests
        run: cargo test --all-features

  python-tests:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v2
      - uses: actions/setup-python@v2
        with:
          python-version: '3.11'
      - name: Install dependencies
        run: |
          pip install maturin pytest
      - name: Build Python bindings
        run: |
          PYO3_USE_ABI3_FORWARD_COMPATIBILITY=1 maturin develop --features python-bindings
      - name: Run Python tests
        run: pytest tests/test_python_bindings.py -v
```

## Troubleshooting

### Common Issues

**Problem:** Tests fail with "permission denied"

**Solution:** Run with sudo for tests that require root access:
```bash
sudo -E cargo test
```

**Problem:** Python tests can't import guestctl

**Solution:** Make sure Python bindings are built:
```bash
./build_python.sh
source .venv/bin/activate
```

**Problem:** Integration tests timeout

**Solution:** Increase timeout or run in release mode:
```bash
cargo test --release -- --test-threads=1
```

## Test Statistics

### Current Coverage

- **Rust Tests:** 13 integration test files
- **Python Tests:** 16 test cases (10 basic + 6 with disk image)
- **Total Functions:** 578 guestfs functions (97.4% implemented)
- **Python Bindings:** 58 methods exposed

### Test Execution Time

- Rust unit tests: ~2 seconds
- Rust integration tests: ~10 seconds
- Python basic tests: ~0.1 seconds
- Python full tests (with image): ~5 seconds

## Contributing Tests

When adding new features:

1. **Write Rust unit tests** for new functions
2. **Add integration tests** for complex workflows
3. **Update Python tests** if bindings are affected
4. **Document test requirements** in code comments
5. **Ensure tests pass** before submitting PR

### Test Naming Conventions

- Rust: `test_<feature>` or `test_<feature>_<scenario>`
- Python: `test_<feature>` in snake_case
- Files: `<feature>_test.rs` or `test_<feature>.py`

## References

- [Rust Testing Guide](https://doc.rust-lang.org/book/ch11-00-testing.html)
- [pytest Documentation](https://docs.pytest.org/)
- [PyO3 Testing](https://pyo3.rs/v0.22.0/testing)

---

**Happy Testing! 🧪**
