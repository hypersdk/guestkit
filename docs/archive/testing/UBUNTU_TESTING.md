# Ubuntu Realistic Disk Image Testing

## Overview

This document describes the realistic Ubuntu disk image testing infrastructure for GuestCtl. These tests create production-quality Ubuntu images with complete system configurations to validate guestfs APIs in realistic scenarios.

## What Makes These Tests "Realistic"

Unlike minimal test images, these Ubuntu images include:

1. **Proper EFI Boot Configuration**
   - GPT partition table
   - EFI System Partition (ESP) with correct GUID
   - GRUB configuration for both BIOS and EFI
   - Fake kernel and initrd

2. **Complete Systemd Integration**
   - SSH service (`ssh.service`)
   - Networking service (`networking.service`)
   - Journal service (`systemd-journald.service`)
   - Proper service symlinks in `multi-user.target.wants`
   - Default target configuration

3. **Ubuntu-Specific Metadata**
   - `/etc/lsb-release` - Ubuntu version identification
   - `/etc/os-release` - Standard OS identification
   - `/etc/debian_version` - Debian compatibility layer
   - `/etc/hostname` - System hostname

4. **Package Management**
   - dpkg database (`/var/lib/dpkg/status`)
   - APT sources list for Ubuntu mirrors
   - Realistic package entries (base-files, bash, coreutils, systemd, linux-image, etc.)

5. **Filesystem Variety**
   - Ubuntu 10.10: ext2 (legacy)
   - Ubuntu 20.04: ext4
   - Ubuntu 22.04: ext4
   - Ubuntu 24.04: XFS (latest)

6. **Complete Directory Structure**
   - FHS-compliant layout (`/bin`, `/sbin`, `/usr`, `/var`, `/etc`, etc.)
   - Log directories (`/var/log`)
   - Runtime directories (`/run`, `/run/lock`)
   - User home directories

7. **Network Configuration**
   - `/etc/network/interfaces`
   - `/etc/resolv.conf`
   - DHCP configuration

8. **User Accounts**
   - root user
   - ubuntu user (sudo group)
   - System accounts (daemon, bin, sys)
   - Proper `/etc/passwd`, `/etc/group`, `/etc/shadow`

## Ubuntu Versions Supported

| Version | Codename | Description | Root FS | Test Function |
|---------|----------|-------------|---------|---------------|
| **10.10** | maverick | Ubuntu 10.10 (Maverick Meerkat) | ext2 | N/A (legacy) |
| **20.04** | focal | Ubuntu 20.04 LTS (Focal Fossa) | ext4 | `test_ubuntu_2004_realistic` |
| **22.04** | jammy | Ubuntu 22.04 LTS (Jammy Jellyfish) | ext4 | `test_ubuntu_2204_realistic` |
| **24.04** | noble | Ubuntu 24.04 LTS (Noble Numbat) | xfs | `test_ubuntu_2404_realistic` |

## Disk Layout

### Partitions

```
GPT Partition Table
├── Partition 1: EFI System Partition (ESP)
│   ├── Type: c12a7328-f81f-11d2-ba4b-00a0c93ec93b (EFI)
│   ├── Size: 200 MB
│   ├── Filesystem: VFAT
│   ├── Mount: /boot/efi
│   └── Contents: GRUB EFI configuration
│
└── Partition 2: Root Filesystem
    ├── Type: Linux filesystem
    ├── Size: ~1.8 GB (remainder)
    ├── Filesystem: ext4/xfs (version-dependent)
    ├── Mount: /
    └── Contents: Full Ubuntu system
```

### File System Contents

```
/
├── boot/
│   ├── efi/
│   │   └── EFI/
│   │       └── ubuntu/
│   │           └── grub.cfg
│   ├── grub/
│   │   └── grub.cfg
│   ├── vmlinuz-6.5.0-35-generic
│   ├── initrd.img-6.5.0-35-generic
│   ├── vmlinuz -> vmlinuz-6.5.0-35-generic
│   └── initrd.img -> initrd.img-6.5.0-35-generic
│
├── etc/
│   ├── lsb-release
│   ├── os-release
│   ├── hostname
│   ├── fstab
│   ├── debian_version
│   ├── passwd, group, shadow
│   ├── network/
│   │   └── interfaces
│   ├── apt/
│   │   └── sources.list
│   ├── default/
│   └── systemd/
│       └── system/
│           ├── default.target -> /lib/systemd/system/multi-user.target
│           └── multi-user.target.wants/
│               ├── ssh.service -> ../../../lib/systemd/system/ssh.service
│               └── networking.service -> ../../../lib/systemd/system/networking.service
│
├── lib/systemd/system/
│   ├── ssh.service
│   ├── networking.service
│   ├── systemd-journald.service
│   └── multi-user.target
│
├── var/
│   ├── lib/
│   │   └── dpkg/
│   │       ├── status (7 packages)
│   │       └── available
│   └── log/
│       ├── dpkg.log
│       ├── syslog
│       └── apt/
│           └── history.log
│
├── home/
│   └── ubuntu/
│
└── root/
```

## Tests Included

### Test 1: Ubuntu 22.04 LTS Creation (`test_ubuntu_2204_realistic`)

Creates a realistic Ubuntu 22.04 (Jammy Jellyfish) disk image with:
- ext4 root filesystem
- Complete systemd configuration
- All metadata files
- 15-step creation process with validation

### Test 2: Ubuntu 20.04 LTS Creation (`test_ubuntu_2004_realistic`)

Creates a realistic Ubuntu 20.04 (Focal Fossa) disk image with:
- ext4 root filesystem
- LTS-specific configurations
- Package database

### Test 3: Ubuntu 24.04 LTS Creation (`test_ubuntu_2404_realistic`)

Creates a realistic Ubuntu 24.04 (Noble Numbat) disk image with:
- **XFS root filesystem** (Ubuntu's latest default)
- Modern systemd units
- Latest package versions

### Test 4: OS Inspection (`test_ubuntu_inspection`)

Validates guestfs inspection APIs on Ubuntu 22.04:
- `inspect_os()` - Detects Ubuntu installation
- `inspect_get_type()` - Returns "linux"
- `inspect_get_distro()` - Returns "ubuntu"
- `inspect_get_major_version()` - Returns 22
- `inspect_get_minor_version()` - Returns 04
- `inspect_get_package_format()` - Returns "deb"

## Phase 3 APIs Tested

Each Ubuntu image creation tests these Phase 3 APIs:

| API | Usage | Validation |
|-----|-------|------------|
| `create()` | Create guestfs handle | Handle created successfully |
| `add_drive()` | Attach disk image | Drive attached in read-write mode |
| `add_drive_ro()` | Attach for inspection | Drive attached read-only |
| `stat()` | Get file metadata | Returns correct file sizes |
| `lstat()` | Get symlink metadata | Doesn't follow symlinks |
| `rm()` | Remove single file | File deleted successfully |
| `rm_rf()` | Remove directory tree | Directory tree deleted |
| `part_set_gpt_type()` | Set EFI partition GUID | ESP marked correctly |

## Running the Tests

### Quick Start

```bash
# Run all Ubuntu tests
./scripts/run_ubuntu_tests.sh

# Run specific version
cargo test --test ubuntu_realistic test_ubuntu_2204_realistic -- --nocapture

# Run OS inspection test
cargo test --test ubuntu_realistic test_ubuntu_inspection -- --nocapture
```

### Prerequisites

1. **System Tools:**
   ```bash
   # Fedora/RHEL
   sudo dnf install qemu-img parted gdisk

   # Ubuntu/Debian
   sudo apt-get install qemu-utils parted gdisk
   ```

2. **NBD Module:**
   ```bash
   sudo modprobe nbd max_part=8
   ```

3. **Disk Space:**
   - Minimum: 2 GB free in `/tmp`
   - Recommended: 5 GB for multiple test runs

4. **Permissions:**
   ```bash
   # Setup test environment
   sudo ./scripts/setup_test_env.sh
   ```

### Test Output Example

```
=== Creating Realistic Ubuntu 22.04 EFI Disk Image ===

[1/15] Creating Guestfs handle and disk image...
  ✓ Disk image created and guestfs launched

[2/15] Creating GPT partition table with EFI System Partition...
  ✓ GPT with EFI System Partition created

[3/15] Creating filesystems...
  ✓ Filesystems: vfat (ESP) + ext4 (root)

[4/15] Mounting filesystems...
  ✓ Root and EFI partitions mounted

[5/15] Creating Ubuntu directory structure...
  ✓ Directory structure created

[6/15] Writing Ubuntu metadata files...
  ✓ Ubuntu metadata files written

[7/15] Creating dpkg package database...
  ✓ dpkg database created with 7 packages

[8/15] Creating systemd units...
  ✓ Systemd units created (ssh, networking, journald)

[9/15] Creating APT sources list...
  ✓ APT sources configured for jammy

[10/15] Creating GRUB configuration...
  ✓ GRUB configuration created

[11/15] Creating fake kernel and initrd...
  ✓ Fake kernel files created

[12/15] Creating network configuration...
  ✓ Network configuration created

[13/15] Creating user accounts...
  ✓ User accounts created (root, ubuntu)

[14/15] Creating log files and runtime directories...
  ✓ Log files and runtime directories created

[15/15] Testing Phase 3 APIs on Ubuntu image...
  ✓ stat(/etc/hostname): size=26 bytes
  ✓ lstat(/boot/vmlinuz): size=28 (symlink)
  ✓ rm() test passed
  ✓ rm_rf() test passed

[Finalizing] Syncing and unmounting...

=== Ubuntu 22.04 Image Created Successfully! ===
  Image: /tmp/ubuntu-efi-test.img
  Size: 2.00 GB
  Filesystem: ext4
  Codename: jammy
  Description: Ubuntu 22.04 LTS (Jammy Jellyfish)
```

## Comparison with Python Reference

This Rust implementation matches the libguestfs Python test for creating fake Ubuntu images:

| Feature | Python Script | Rust Test | Status |
|---------|--------------|-----------|--------|
| GPT + EFI | ✓ | ✓ | ✅ Matching |
| Multiple versions | ✓ | ✓ | ✅ Matching |
| Filesystem variety | ✓ | ✓ | ✅ Matching |
| Systemd units | ✓ | ✓ | ✅ Enhanced |
| dpkg database | ✓ | ✓ | ✅ Enhanced |
| GRUB config | ✓ | ✓ | ✅ Matching |
| APT sources | ✗ | ✓ | ✅ Enhanced |
| Network config | ✗ | ✓ | ✅ Enhanced |
| User accounts | ✗ | ✓ | ✅ Enhanced |

**The Rust implementation is more comprehensive than the Python reference!**

## Benefits of These Tests

1. **Realistic Validation:** Tests APIs against real-world Ubuntu configurations
2. **Multiple Versions:** Validates across different Ubuntu LTS releases
3. **Filesystem Coverage:** Tests ext2, ext4, and XFS
4. **EFI Testing:** Validates GPT and EFI partition handling
5. **Systemd Integration:** Tests realistic service configurations
6. **Package Management:** Validates dpkg database handling
7. **Cross-Platform:** Same tests work on Fedora, Ubuntu, and other Linux distros

## Future Enhancements

- [ ] Add Ubuntu 18.04 LTS (Bionic)
- [ ] Add kernel module loading configuration
- [ ] Add cloud-init configuration
- [ ] Add snap package database
- [ ] Add netplan configuration (Ubuntu 18.04+)
- [ ] Add encrypted root partition tests
- [ ] Add LVM configuration
- [ ] Add RAID configuration

## Troubleshooting

### "No available NBD devices"

```bash
sudo modprobe nbd max_part=8
lsmod | grep nbd
```

### "Permission denied on /dev/nbd*"

```bash
sudo ./scripts/setup_test_env.sh
```

### "Filesystem XFS not supported"

```bash
# Install xfsprogs
sudo dnf install xfsprogs  # Fedora
sudo apt-get install xfsprogs  # Ubuntu
```

### Tests timeout or hang

```bash
# Clean up stale NBD devices
for i in {0..15}; do
    sudo qemu-nbd --disconnect /dev/nbd$i
done

# Remove test images
rm -f /tmp/ubuntu-efi-test.img
```

## References

- [Ubuntu Releases](https://wiki.ubuntu.com/Releases)
- [Filesystem Hierarchy Standard](https://refspecs.linuxfoundation.org/FHS_3.0/fhs-3.0.pdf)
- [systemd.unit(5)](https://www.freedesktop.org/software/systemd/man/systemd.unit.html)
- [UEFI Specification](https://uefi.org/specifications)
- [GPT Partition Table](https://en.wikipedia.org/wiki/GUID_Partition_Table)
- [dpkg Database Format](https://manpages.debian.org/dpkg)

## Statistics

- **Test File:** `tests/ubuntu_realistic.rs`
- **Lines of Code:** 700+
- **Ubuntu Versions:** 4 (10.10, 20.04, 22.04, 24.04)
- **Test Functions:** 4
- **APIs Tested:** 8 Phase 3 APIs + 6 inspection APIs
- **Directory Structure:** 30+ directories
- **System Files:** 25+ files
- **Systemd Units:** 4 services
- **Packages:** 7 in dpkg database
