# Testing Permissions and Requirements

## Overview

GuestKit's comprehensive tests require privileged operations because they:
1. Connect disk images to NBD (Network Block Device) devices using `qemu-nbd`
2. Create partition tables on block devices
3. Mount filesystems

This document explains permission requirements and provides solutions for different testing scenarios.

## Permission Requirements

###  Why Root/Sudo is Required

The comprehensive Phase 3 tests perform operations that require elevated privileges:

| Operation | Command | Why Privileged |
|-----------|---------|----------------|
| **NBD Connection** | `qemu-nbd --connect /dev/nbd0 disk.img` | Requires CAP_SYS_ADMIN to bind NBD device |
| **Partition Creation** | `parted /dev/nbd0 mklabel gpt` | Requires write access to block device |
| **Filesystem Creation** | `mkfs.ext4 /dev/nbd0p1` | Requires write access to block device |
| **Mounting** | `mount /dev/nbd0p1 /mnt` | Requires CAP_SYS_ADMIN capability |

Even with `chmod 666 /dev/nbd*`, the `qemu-nbd --connect` operation itself requires root privileges to set up the NBD socket and bind the device.

## Test Environment Setup

### Automated Setup (Recommended)

Run the automated setup script to configure your system:

```bash
sudo ./scripts/setup_test_env.sh
```

This script:
- Loads the NBD kernel module with proper parameters
- Sets NBD device permissions
- Configures the qemu-nbd lock directory
- Verifies all required tools are installed

### Manual Setup

If you prefer manual setup:

```bash
# 1. Load NBD kernel module
sudo modprobe nbd max_part=8

# 2. Set NBD device permissions
sudo chmod 666 /dev/nbd*

# 3. Configure lock directory
sudo chmod 1777 /run/lock

# 4. Verify tools are installed
which qemu-nbd parted sgdisk cpio
```

## Running Tests

### Option 1: Run Tests with Sudo (Easiest)

```bash
# Run all comprehensive tests
sudo -E ./scripts/run_all_phase3_tests.sh

# Run individual test suites
sudo -E ./scripts/run_phase3_tests.sh        # Fedora tests
sudo -E ./scripts/run_phase3_windows_tests.sh  # Windows tests

# Run specific test
sudo -E cargo test --test phase3_comprehensive test_phase3_comprehensive -- --nocapture
```

**Note:** The `-E` flag preserves environment variables (important for Rust/Cargo paths).

### Option 2: Use Sudo Wrapper in Code (Future Enhancement)

We could modify `src/disk/nbd.rs` to automatically use `sudo` when needed:

```rust
// Check if running as root
let needs_sudo = !nix::unistd::Uid::effective().is_root();

let mut cmd = if needs_sudo {
    let mut sudo_cmd = Command::new("sudo");
    sudo_cmd.arg("qemu-nbd");
    sudo_cmd
} else {
    Command::new("qemu-nbd")
};
```

**Pros:** Tests can run without sudo
**Cons:** Requires passwordless sudo configuration, security implications

### Option 3: Unit Tests (No Privileges Required)

For basic API validation without NBD:

```bash
# Run unit tests only (no NBD required)
cargo test --lib

# Run doc tests
cargo test --doc
```

These tests validate:
- API signatures
- Data structures
- Error handling
- Pure Rust disk reading (no mounting)

## CI/CD Integration

### GitHub Actions Example

```yaml
name: Tests

on: [push, pull_request]

jobs:
  test:
    runs-on: ubuntu-latest

    steps:
      - uses: actions/checkout@v3

      - name: Install dependencies
        run: |
          sudo apt-get update
          sudo apt-get install -y qemu-utils parted gdisk cpio

      - name: Setup test environment
        run: sudo ./scripts/setup_test_env.sh

      - name: Run unit tests (no privileges)
        run: cargo test --lib

      - name: Run comprehensive tests (with sudo)
        run: sudo -E ./scripts/run_all_phase3_tests.sh
```

### GitLab CI Example

```yaml
test:
  image: rust:latest

  before_script:
    - apt-get update && apt-get install -y qemu-utils parted gdisk cpio
    - ./scripts/setup_test_env.sh

  script:
    - cargo test --lib  # Unit tests
    - ./scripts/run_all_phase3_tests.sh  # Comprehensive (runs as root in container)
```

## Alternative: Using Containers

Run tests in a privileged container where root operations are isolated:

```bash
# Docker
docker run --privileged -v $(pwd):/workspace -w /workspace rust:latest bash -c "
  apt-get update && apt-get install -y qemu-utils parted gdisk cpio &&
  ./scripts/setup_test_env.sh &&
  ./scripts/run_all_phase3_tests.sh
"

# Podman
podman run --privileged -v $(pwd):/workspace:Z -w /workspace rust:latest bash -c "
  dnf install -y qemu-img parted gdisk cpio &&
  ./scripts/setup_test_env.sh &&
  ./scripts/run_all_phase3_tests.sh
"
```

## Security Considerations

### Why Not Always Use Sudo?

Adding automatic `sudo` to qemu-nbd calls in library code is **not recommended** because:

1. **Security Risk:** Library code shouldn't escalate privileges automatically
2. **User Consent:** Users should explicitly choose to run privileged operations
3. **Audit Trail:** Explicit sudo provides clear audit logs
4. **Configuration:** Passwordless sudo is a security concern on multi-user systems

### Best Practices

1. **Development:** Use `sudo` explicitly when running tests
2. **CI/CD:** Run in containers or VMs where root access is isolated
3. **Production:** Library never requires sudo; only tests do

## Troubleshooting

### "Failed to set NBD socket"

**Problem:** qemu-nbd can't connect to NBD device
**Solution:** Run test with `sudo` or check if NBD device is already in use

```bash
# Check for existing qemu-nbd processes
pgrep -a qemu-nbd

# Kill stale processes
sudo pkill qemu-nbd

# Disconnect all NBD devices
for i in {0..15}; do
    sudo qemu-nbd --disconnect /dev/nbd$i 2>/dev/null
done
```

### "No available NBD devices found"

**Problem:** NBD kernel module not loaded
**Solution:** Load the module

```bash
sudo modprobe nbd max_part=8
lsmod | grep nbd  # Verify loaded
```

### "Permission denied" on /dev/nbd*

**Problem:** NBD devices not readable/writable
**Solution:** Set permissions

```bash
sudo chmod 666 /dev/nbd*
ls -l /dev/nbd*  # Verify permissions
```

### "parted: command not found"

**Problem:** Required tools not installed
**Solution:** Install dependencies

```bash
# Fedora/RHEL
sudo dnf install qemu-img parted gdisk cpio

# Ubuntu/Debian
sudo apt-get install qemu-utils parted gdisk cpio

# Arch
sudo pacman -S qemu-img parted gptfdisk cpio
```

## Summary

| Test Type | Privileges Required | Command |
|-----------|-------------------|---------|
| **Unit Tests** | None | `cargo test --lib` |
| **Doc Tests** | None | `cargo test --doc` |
| **Integration (Comprehensive)** | Root/Sudo | `sudo -E ./scripts/run_all_phase3_tests.sh` |
| **CI/CD** | Container root | See examples above |

**Recommendation for Development:**
1. Run `sudo ./scripts/setup_test_env.sh` once after boot
2. Use `sudo -E ./scripts/run_all_phase3_tests.sh` for comprehensive testing
3. Use `cargo test --lib` for quick iteration without privileges

**Recommendation for CI/CD:**
Run in privileged containers or VMs where root access is isolated from the host system.
