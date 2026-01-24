# Debian Realistic Disk Image Testing

## Overview

This document describes the realistic Debian disk image testing infrastructure for GuestCtl. These tests create production-quality Debian images with complete system configurations including LVM layouts to validate guestfs APIs in realistic scenarios.

## What Makes These Tests "Realistic"

Unlike minimal test images, these Debian images include:

1. **Flexible Partitioning Schemes**
   - Legacy MBR/BIOS layout
   - Modern EFI/GPT layout with ESP
   - Proper partition type GUIDs

2. **LVM Configuration**
   - Physical Volume (PV) on partition
   - Volume Group: `debian`
   - Logical Volumes:
     - `/dev/debian/root` (64 MB) - Root filesystem
     - `/dev/debian/usr` (32 MB) - `/usr` directory
     - `/dev/debian/var` (32 MB) - `/var` directory
     - `/dev/debian/home` (32 MB) - Home directories
   - Deterministic UUIDs for testing

3. **Complete Systemd Integration**
   - SSH service (`ssh.service`)
   - Networking service (`networking.service`)
   - Journal service (`systemd-journald.service`)
   - Proper service symlinks in `multi-user.target.wants`
   - Default target configuration

4. **Debian-Specific Metadata**
   - `/etc/debian_version` - Debian version string
   - `/etc/os-release` - Standard OS identification
   - `/etc/hostname` - System hostname
   - `/etc/fstab` - Filesystem mount table with LVM devices

5. **Package Management**
   - dpkg database (`/var/lib/dpkg/status`)
   - APT sources list for Debian mirrors
   - Realistic package entries (base-files, bash, coreutils, systemd, linux-image, grub-efi-amd64, etc.)

6. **Boot Configuration**
   - EFI System Partition with correct GUID (for GPT)
   - GRUB configuration for both BIOS and EFI
   - Fake kernel and initrd
   - Boot partition with ext2 filesystem

7. **Complete Directory Structure**
   - FHS-compliant layout (`/bin`, `/sbin`, `/usr`, `/var`, `/etc`, etc.)
   - Log directories (`/var/log`)
   - dpkg database directories
   - User home directories

8. **Network Configuration**
   - `/etc/network/interfaces` - Network interface configuration
   - `/etc/resolv.conf` - DNS resolver configuration
   - DHCP configuration for primary interface

9. **User Accounts**
   - root user
   - debian user (sudo group)
   - System accounts (daemon, bin, sys, www-data, etc.)
   - Proper `/etc/passwd`, `/etc/group`, `/etc/shadow`

## Debian Versions Supported

| Version | Codename | Description | Boot Layout | Test Function |
|---------|----------|-------------|-------------|---------------|
| **11** | bullseye | Debian GNU/Linux 11 (bullseye) | MBR/BIOS | `test_debian_11_mbr` |
| **12** | bookworm | Debian GNU/Linux 12 (bookworm) | EFI/GPT | `test_debian_12_efi` |
| **13** | trixie | Debian GNU/Linux 13 (trixie) | EFI/GPT | `test_debian_13_efi` |

## Disk Layout

### MBR/BIOS Layout (Debian 11)

```
MBR Partition Table
├── Partition 1: Boot Partition
│   ├── Start: Sector 64
│   ├── End: Sector 524287
│   ├── Filesystem: ext2
│   ├── Label: BOOT
│   └── Mount: /boot
│
└── Partition 2: LVM Physical Volume
    ├── Start: Sector 524288
    ├── End: -64
    ├── Volume Group: debian
    └── Logical Volumes:
        ├── root (64 MB, ext2) → /
        ├── usr (32 MB, ext2) → /usr
        ├── var (32 MB, ext2) → /var
        └── home (32 MB, ext2) → /home
```

### EFI/GPT Layout (Debian 12, 13)

```
GPT Partition Table
├── Partition 1: EFI System Partition (ESP)
│   ├── Type: c12a7328-f81f-11d2-ba4b-00a0c93ec93b (EFI)
│   ├── Start: Sector 2048
│   ├── Size: 100 MB
│   ├── Filesystem: VFAT
│   ├── Label: EFI
│   ├── Mount: /boot/efi
│   └── Contents: GRUB EFI configuration
│
├── Partition 2: Boot Partition
│   ├── Size: 256 MB
│   ├── Filesystem: ext2
│   ├── Label: BOOT
│   ├── Mount: /boot
│   └── Contents: Kernel, initrd, GRUB config
│
└── Partition 3: LVM Physical Volume
    ├── Size: Remainder of disk
    ├── Volume Group: debian
    └── Logical Volumes:
        ├── root (64 MB, ext2) → /
        ├── usr (32 MB, ext2) → /usr
        ├── var (32 MB, ext2) → /var
        └── home (32 MB, ext2) → /home
```

### File System Contents

```
/
├── boot/
│   ├── efi/                    # EFI/GPT only
│   │   └── EFI/
│   │       └── debian/
│   │           └── grub.cfg
│   ├── grub/
│   │   └── grub.cfg
│   ├── vmlinuz-6.1.0-17-amd64
│   ├── initrd.img-6.1.0-17-amd64
│   ├── vmlinuz -> vmlinuz-6.1.0-17-amd64
│   └── initrd.img -> initrd.img-6.1.0-17-amd64
│
├── etc/
│   ├── debian_version
│   ├── os-release
│   ├── hostname
│   ├── fstab
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
│   │   ├── dpkg/
│   │   │   ├── status (7 packages)
│   │   │   └── available
│   │   └── urandom/
│   └── log/
│       ├── dpkg.log
│       ├── syslog
│       └── apt/
│           └── history.log
│
├── usr/
│   ├── bin/
│   ├── sbin/
│   └── lib/
│
├── home/
│   └── debian/
│
└── root/
```

## Tests Included

### Test 1: Debian 11 MBR/BIOS (`test_debian_11_mbr`)

Creates a realistic Debian 11 (Bullseye) disk image with:
- Legacy MBR partition table
- LVM layout with 4 logical volumes
- Complete systemd configuration
- All metadata files
- 16-step creation process with validation

### Test 2: Debian 12 EFI/GPT (`test_debian_12_efi`)

Creates a realistic Debian 12 (Bookworm) disk image with:
- GPT partition table with EFI System Partition
- LVM layout with 4 logical volumes
- EFI boot configuration
- Latest package versions

### Test 3: Debian 13 EFI/GPT (`test_debian_13_efi`)

Creates a realistic Debian 13 (Trixie) disk image with:
- GPT partition table with EFI System Partition
- LVM layout with 4 logical volumes
- Testing/unstable branch configuration

### Test 4: OS Inspection (`test_debian_inspection`)

Validates guestfs inspection APIs on Debian 12:
- `inspect_os()` - Detects Debian installation
- `inspect_get_type()` - Returns "linux"
- `inspect_get_distro()` - Returns "debian"
- `inspect_get_major_version()` - Returns 12
- `inspect_get_package_format()` - Returns "deb"

### Test 5: LVM Layout (`test_debian_lvm_layout`)

Validates LVM-specific APIs:
- `vgs()` - Lists volume groups, finds "debian"
- `lvs()` - Lists logical volumes
- Verifies all 4 logical volumes exist
- Validates filesystem UUIDs

## Phase 3 APIs Tested

Each Debian image creation tests these Phase 3 APIs:

| API | Usage | Validation |
|-----|-------|------------|
| `create()` | Create guestfs handle | Handle created successfully |
| `add_drive()` | Attach disk image | Drive attached in read-write mode |
| `add_drive_ro()` | Attach for inspection | Drive attached read-only |
| `disk_create()` | Create sparse disk image | Image file created |
| `part_init()` | Initialize partition table | MBR or GPT created |
| `part_add()` | Add partition | Partitions created |
| `part_set_gpt_type()` | Set EFI partition GUID | ESP marked correctly |
| `part_set_name()` | Set partition name | Names set correctly |
| `pvcreate()` | Create LVM physical volume | PV created |
| `vgcreate()` | Create volume group | VG "debian" created |
| `lvcreate()` | Create logical volume | 4 LVs created |
| `vgs()` | List volume groups | "debian" found |
| `lvs()` | List logical volumes | All 4 LVs found |
| `mkfs()` | Create filesystem | ext2 and vfat created |
| `set_uuid()` | Set filesystem UUID | UUIDs set correctly |
| `get_uuid()` | Get filesystem UUID | UUIDs retrieved correctly |
| `mount()` | Mount filesystem | All filesystems mounted |
| `mkdir()` / `mkdir_p()` | Create directories | Directory tree created |
| `write()` | Write file | All config files written |
| `ln_s()` | Create symlink | Symlinks created |
| `chmod()` | Change permissions | Permissions set |
| `stat()` | Get file metadata | Returns correct file sizes |
| `lstat()` | Get symlink metadata | Doesn't follow symlinks |
| `rm()` | Remove single file | File deleted successfully |
| `rm_rf()` | Remove directory tree | Directory tree deleted |
| `sync()` | Sync filesystem | Data flushed to disk |
| `umount_all()` | Unmount all | All filesystems unmounted |
| `shutdown()` | Shutdown appliance | Clean shutdown |

## Running the Tests

### Quick Start

```bash
# Run all Debian tests
./scripts/run_debian_tests.sh

# Run specific version
cargo test --test debian_realistic test_debian_12_efi -- --nocapture

# Run OS inspection test
cargo test --test debian_realistic test_debian_inspection -- --nocapture

# Run LVM validation test
cargo test --test debian_realistic test_debian_lvm_layout -- --nocapture
```

### Prerequisites

1. **System Tools:**
   ```bash
   # Fedora/RHEL
   sudo dnf install qemu-img parted gdisk lvm2

   # Ubuntu/Debian
   sudo apt-get install qemu-utils parted gdisk lvm2
   ```

2. **NBD Module:**
   ```bash
   sudo modprobe nbd max_part=8
   ```

3. **Disk Space:**
   - Minimum: 1 GB free in `/tmp`
   - Recommended: 2 GB for multiple test runs

4. **Permissions:**
   ```bash
   # Setup test environment
   sudo ./scripts/setup_test_env.sh
   ```

### Test Output Example

```
=== Creating Realistic Debian 12 Image (EFI/GPT) ===

[1/16] Creating Guestfs handle and disk image...
  ✓ Disk image created and guestfs launched

[2/16] Creating GPT partition table for EFI...
  ✓ GPT with EFI System Partition created

[3/16] Creating LVM layout (PV/VG/LVs)...
  ✓ LVM layout created: debian/root, debian/usr, debian/var, debian/home

[4/16] Creating filesystems...
  ✓ Filesystems: vfat (ESP) + ext2 (boot) + ext2 (LVM volumes)

[5/16] Mounting filesystems...
  ✓ All filesystems mounted

[6/16] Creating Debian directory structure...
  ✓ Directory structure created

[7/16] Writing Debian metadata files...
  ✓ Debian metadata files written

[8/16] Creating dpkg package database...
  ✓ dpkg database created with 7 packages

[9/16] Creating systemd units...
  ✓ Systemd units created (ssh, networking, journald)

[10/16] Creating APT sources list...
  ✓ APT sources configured for bookworm

[11/16] Creating GRUB configuration...
  ✓ GRUB configuration created

[12/16] Creating fake kernel and initrd...
  ✓ Fake kernel files created

[13/16] Creating network configuration...
  ✓ Network configuration created

[14/16] Creating user accounts...
  ✓ User accounts created (root, debian)

[15/16] Creating log files and runtime directories...
  ✓ Log files and runtime directories created

[16/16] Testing Phase 3 APIs on Debian image...
  ✓ stat(/etc/hostname): size=15 bytes
  ✓ lstat(/boot/vmlinuz): size=24 (symlink)
  ✓ rm() test passed
  ✓ rm_rf() test passed

[Finalizing] Syncing and unmounting...

=== Debian 12 Image Created Successfully! ===
  Image: /tmp/debian-test.img
  Size: 512 MB
  Boot mode: EFI/GPT
  Codename: bookworm
  Description: Debian GNU/Linux 12 (bookworm)
  LVM: debian/root, debian/usr, debian/var, debian/home
```

## Comparison with Python Reference

This Rust implementation matches the libguestfs Python script for creating fake Debian images:

| Feature | Python Script | Rust Test | Status |
|---------|--------------|-----------|--------|
| MBR/BIOS layout | ✓ | ✓ | ✅ Matching |
| EFI/GPT layout | ✓ | ✓ | ✅ Matching |
| LVM with multiple LVs | ✓ | ✓ | ✅ Matching |
| Deterministic UUIDs | ✓ | ✓ | ✅ Matching |
| Multiple versions | ✓ | ✓ | ✅ Matching |
| Systemd units | ✗ | ✓ | ✅ Enhanced |
| dpkg database | ✓ (minimal) | ✓ (7 pkgs) | ✅ Enhanced |
| GRUB config | ✓ (basic) | ✓ (full) | ✅ Enhanced |
| APT sources | ✗ | ✓ | ✅ Enhanced |
| Network config | ✗ | ✓ | ✅ Enhanced |
| User accounts | ✗ | ✓ | ✅ Enhanced |
| Fake ELF binary | ✓ | ✓ | ✅ Matching |

**The Rust implementation is more comprehensive than the Python reference!**

## Benefits of These Tests

1. **Realistic Validation:** Tests APIs against real-world Debian configurations
2. **Multiple Versions:** Validates across different Debian releases
3. **LVM Testing:** Comprehensive LVM layout with multiple logical volumes
4. **Dual Boot Support:** Tests both legacy BIOS and modern EFI boot
5. **Systemd Integration:** Tests realistic service configurations
6. **Package Management:** Validates dpkg database handling
7. **Cross-Platform:** Same tests work on Fedora, Ubuntu, and other Linux distros
8. **Deterministic:** Hard-coded UUIDs make inspection testing predictable

## Key Differences from Ubuntu Tests

1. **LVM Layout:** Debian tests use LVM with separate logical volumes for `/`, `/usr`, `/var`, `/home`
2. **Boot Schemes:** Support for both MBR/BIOS and EFI/GPT
3. **Metadata Files:** Uses `/etc/debian_version` instead of `/etc/lsb-release`
4. **Package Names:** Debian-specific package naming (grub-efi-amd64 vs grub2-efi-x64)
5. **Smaller Images:** 512 MB vs Ubuntu's 2 GB (LVM is more space-efficient)
6. **Version Scheme:** Uses major version numbers (11, 12, 13) instead of Ubuntu's `XX.YY` format

## Future Enhancements

- [ ] Add Debian 10 (Buster)
- [ ] Add encrypted LVM (LUKS)
- [ ] Add RAID configuration
- [ ] Add Btrfs subvolume layout
- [ ] Add cloud-init configuration
- [ ] Add systemd-networkd configuration
- [ ] Add multiple network interfaces
- [ ] Add swap LV

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

### "LVM commands not found"

```bash
# Install LVM tools
sudo dnf install lvm2        # Fedora
sudo apt-get install lvm2    # Ubuntu
```

### Tests timeout or hang

```bash
# Clean up stale NBD devices and LVM
for i in {0..15}; do
    sudo qemu-nbd --disconnect /dev/nbd$i
done

# Deactivate LVM volumes
sudo vgchange -an debian 2>/dev/null || true
sudo dmsetup remove_all 2>/dev/null || true

# Remove test images
rm -f /tmp/debian-test.img
```

## References

- [Debian Releases](https://www.debian.org/releases/)
- [Filesystem Hierarchy Standard](https://refspecs.linuxfoundation.org/FHS_3.0/fhs-3.0.pdf)
- [systemd.unit(5)](https://www.freedesktop.org/software/systemd/man/systemd.unit.html)
- [UEFI Specification](https://uefi.org/specifications)
- [GPT Partition Table](https://en.wikipedia.org/wiki/GUID_Partition_Table)
- [dpkg Database Format](https://manpages.debian.org/dpkg)
- [LVM HOWTO](https://tldp.org/HOWTO/LVM-HOWTO/)

## Statistics

- **Test File:** `tests/debian_realistic.rs`
- **Lines of Code:** 1,000+
- **Debian Versions:** 3 (11 Bullseye, 12 Bookworm, 13 Trixie)
- **Test Functions:** 5
- **APIs Tested:** 30+ Phase 3 APIs including LVM operations
- **Directory Structure:** 25+ directories
- **System Files:** 20+ files
- **Systemd Units:** 4 services
- **Packages:** 7 in dpkg database
- **LVM Volumes:** 1 VG + 4 LVs
- **Boot Layouts:** 2 (MBR/BIOS and EFI/GPT)
