# Arch Linux Realistic Disk Image Testing

## Overview

This document describes the realistic Arch Linux disk image testing infrastructure for GuestCtl. These tests create production-quality Arch Linux images with BTRFS subvolumes and modern systemd-boot configuration to validate guestfs APIs in realistic scenarios.

## What Makes These Tests "Realistic"

Unlike minimal test images, these Arch Linux images include:

1. **Modern BTRFS Layout**
   - BTRFS filesystem with subvolumes
   - @ (root) subvolume
   - @home subvolume for /home
   - @var subvolume for /var
   - @snapshots subvolume for snapshots
   - Modern Arch Linux recommended layout

2. **EFI Boot Configuration**
   - GPT partition table
   - EFI System Partition (ESP) with correct GUID
   - systemd-boot bootloader (Arch default)
   - Boot loader entries
   - Fake kernel and initramfs

3. **Complete Systemd Integration**
   - SSH daemon (`sshd.service`)
   - NetworkManager service
   - Journal service (`systemd-journald.service`)
   - Proper service symlinks in `multi-user.target.wants`
   - Default target configuration

4. **Arch-Specific Metadata**
   - `/etc/os-release` - Arch identification
   - `/etc/lsb-release` - LSB compatibility
   - `/etc/hostname` - System hostname
   - `/etc/fstab` - Filesystem mount table with BTRFS subvolumes
   - `/etc/locale.conf` - Locale configuration
   - `/etc/vconsole.conf` - Console configuration
   - `/etc/machine-id` - Systemd machine ID

5. **Package Management**
   - pacman configuration (`/etc/pacman.conf`)
   - Mirror list (`/etc/pacman.d/mirrorlist`)
   - Local package database (`/var/lib/pacman/local/`)
   - ALPM database version marker

6. **Rolling Release**
   - No fixed version numbers
   - Rolling release metadata
   - Current kernel version (6.7.0-arch1-1)

7. **Complete Directory Structure**
   - FHS-compliant layout
   - `/usr/lib/systemd/system` for units
   - `/var/lib/pacman` for package database
   - `/var/log` for logs
   - User home directories

8. **User Accounts**
   - root user
   - arch user (wheel group for sudo)
   - System accounts (dbus, systemd-*, etc.)
   - Proper `/etc/passwd`, `/etc/group`, `/etc/shadow`

## Arch Linux Snapshot

| Property | Value | Description |
|----------|-------|-------------|
| **Distribution** | Arch Linux | Rolling release |
| **Snapshot** | 2024.01 | January 2024 snapshot |
| **Kernel** | 6.7.0-arch1-1 | Latest Arch kernel |
| **Init** | systemd | Default init system |
| **Bootloader** | systemd-boot | UEFI bootloader |
| **Filesystem** | BTRFS | With subvolumes |
| **Package Manager** | pacman | Arch package manager |

## Disk Layout

### Partitions

```
GPT Partition Table
├── Partition 1: EFI System Partition (ESP)
│   ├── Type: c12a7328-f81f-11d2-ba4b-00a0c93ec93b (EFI)
│   ├── Start: Sector 2048
│   ├── Size: 512 MB
│   ├── Filesystem: VFAT
│   ├── Label: BOOT
│   ├── Mount: /boot
│   └── Contents: systemd-boot, kernel, initramfs
│
└── Partition 2: Root Filesystem
    ├── Type: Linux filesystem
    ├── Size: ~512 MB (remainder)
    ├── Filesystem: BTRFS
    ├── Label: ArchRoot
    └── Subvolumes:
        ├── @ → mounted at /
        ├── @home → mounted at /home
        ├── @var → mounted at /var
        └── @snapshots → mounted at /.snapshots
```

### BTRFS Subvolume Layout

```
/dev/sda2 (BTRFS)
├── @ (subvol=/@)
│   ├── /bin → /usr/bin
│   ├── /sbin → /usr/sbin
│   ├── /usr/
│   ├── /etc/
│   ├── /root/
│   └── /tmp/
│
├── @home (subvol=/@home)
│   └── /home/
│       └── arch/
│
├── @var (subvol=/@var)
│   └── /var/
│       ├── lib/pacman/
│       ├── log/
│       └── cache/
│
└── @snapshots (subvol=/@snapshots)
    └── /.snapshots/
```

### File System Contents

```
/
├── boot/
│   ├── loader/
│   │   ├── loader.conf
│   │   └── entries/
│   │       └── arch.conf
│   ├── vmlinuz-linux
│   ├── initramfs-linux.img
│   └── initramfs-linux-fallback.img
│
├── etc/
│   ├── os-release
│   ├── lsb-release
│   ├── hostname
│   ├── fstab
│   ├── locale.conf
│   ├── vconsole.conf
│   ├── machine-id
│   ├── passwd, group, shadow
│   ├── pacman.conf
│   ├── pacman.d/
│   │   └── mirrorlist
│   └── systemd/
│       └── system/
│           ├── default.target → /usr/lib/systemd/system/multi-user.target
│           └── multi-user.target.wants/
│               ├── sshd.service
│               └── NetworkManager.service
│
├── usr/
│   ├── bin/
│   │   ├── bash
│   │   ├── pacman
│   │   └── systemctl
│   ├── sbin/ → usr/bin
│   ├── lib/
│   │   └── systemd/
│   │       └── system/
│   │           ├── sshd.service
│   │           ├── NetworkManager.service
│   │           ├── systemd-journald.service
│   │           └── multi-user.target
│   └── share/
│       ├── locale/
│       └── zoneinfo/
│
├── var/
│   ├── lib/
│   │   └── pacman/
│   │       ├── local/
│   │       │   └── base-3-2/
│   │       │       └── desc
│   │       └── ALPM_DB_VERSION
│   ├── log/
│   │   ├── pacman.log
│   │   └── journal/
│   └── cache/
│       └── pacman/
│           └── pkg/
│
├── home/
│   └── arch/
│
└── root/
```

## Tests Included

### Test 1: Arch Linux with BTRFS (`test_arch_realistic`)

Creates a realistic Arch Linux disk image with:
- BTRFS filesystem with 4 subvolumes
- systemd-boot bootloader
- Complete systemd configuration
- pacman package manager setup
- 18-step creation process with validation

### Test 2: OS Inspection (`test_arch_inspection`)

Validates guestfs inspection APIs on Arch Linux:
- `inspect_os()` - Detects Arch installation
- `inspect_get_type()` - Returns "linux"
- `inspect_get_distro()` - Returns "archlinux"
- `inspect_get_package_format()` - Returns "pacman"

### Test 3: BTRFS Subvolume Validation (`test_arch_btrfs_subvolumes`)

Validates BTRFS-specific APIs:
- `btrfs_subvolume_list()` - Lists all subvolumes
- Verifies all 4 subvolumes exist (@, @home, @var, @snapshots)
- Validates subvolume paths

## Phase 3 APIs Tested

Each Arch image creation tests these Phase 3 APIs:

| API | Usage | Validation |
|-----|-------|------------|
| `create()` | Create guestfs handle | Handle created successfully |
| `add_drive()` | Attach disk image | Drive attached in read-write mode |
| `add_drive_ro()` | Attach for inspection | Drive attached read-only |
| `disk_create()` | Create sparse disk image | Image file created |
| `part_init()` | Initialize partition table | GPT created |
| `part_add()` | Add partition | Partitions created |
| `part_set_gpt_type()` | Set EFI partition GUID | ESP marked correctly |
| `part_set_name()` | Set partition name | Names set correctly |
| `mkfs()` | Create filesystem | BTRFS and vfat created |
| `mount()` | Mount filesystem | Filesystems mounted with options |
| `btrfs_subvolume_create()` | Create BTRFS subvolume | 4 subvolumes created |
| `btrfs_subvolume_list()` | List BTRFS subvolumes | All subvolumes found |
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
# Run all Arch tests
./scripts/run_arch_tests.sh

# Run specific test
cargo test --test arch_realistic test_arch_realistic -- --nocapture

# Run OS inspection test
cargo test --test arch_realistic test_arch_inspection -- --nocapture

# Run BTRFS validation test
cargo test --test arch_realistic test_arch_btrfs_subvolumes -- --nocapture
```

### Prerequisites

1. **System Tools:**
   ```bash
   # Fedora/RHEL
   sudo dnf install qemu-img parted gdisk btrfs-progs

   # Ubuntu/Debian
   sudo apt-get install qemu-utils parted gdisk btrfs-progs
   ```

2. **NBD Module:**
   ```bash
   sudo modprobe nbd max_part=8
   ```

3. **Disk Space:**
   - Minimum: 1.5 GB free in `/tmp`
   - Recommended: 2 GB for multiple test runs

4. **Permissions:**
   ```bash
   # Setup test environment
   sudo ./scripts/setup_test_env.sh
   ```

### Test Output Example

```
=== Creating Realistic Arch Linux Image ===

[1/18] Creating Guestfs handle and disk image...
  ✓ Disk image created and guestfs launched

[2/18] Creating GPT partition table for EFI...
  ✓ GPT with EFI System Partition created

[3/18] Creating filesystems...
  ✓ Filesystems: vfat (ESP) + btrfs (root)

[4/18] Creating BTRFS subvolumes...
  ✓ BTRFS subvolumes created: @, @home, @var, @snapshots

[5/18] Mounting BTRFS subvolumes...
  ✓ All BTRFS subvolumes mounted

[6/18] Creating Arch directory structure...
  ✓ Directory structure created

[7/18] Writing Arch metadata files...
  ✓ Arch metadata files written

[8/18] Creating pacman configuration...
  ✓ pacman configuration created

[9/18] Creating systemd units...
  ✓ Systemd units created (sshd, NetworkManager, journald)

[10/18] Creating systemd-boot configuration...
  ✓ systemd-boot configuration created

[11/18] Creating fake kernel and initramfs...
  ✓ Fake kernel and initramfs created

[12/18] Creating user accounts...
  ✓ User accounts created (root, arch)

[13/18] Creating log files...
  ✓ Log files created

[14/18] Creating locale and timezone configuration...
  ✓ Locale and timezone configured

[15/18] Creating fake binaries...
  ✓ Fake binaries created

[16/18] Creating machine-id...
  ✓ machine-id created

[17/18] Testing Phase 3 APIs on Arch image...
  ✓ stat(/etc/hostname): size=10 bytes
  ✓ lstat(/bin/bash): size=13 (symlink)
  ✓ rm() test passed
  ✓ rm_rf() test passed

[18/18] Testing BTRFS operations...
  ✓ Found 4 BTRFS subvolumes
    - @
    - @home
    - @var
    - @snapshots

[Finalizing] Syncing and unmounting...

=== Arch Linux Image Created Successfully! ===
  Image: /tmp/arch-test.img
  Size: 1024 MB
  Filesystem: BTRFS with subvolumes
  Boot: systemd-boot (EFI)
  Subvolumes: @, @home, @var, @snapshots
```

## Benefits of These Tests

1. **Modern Filesystem:** Tests BTRFS with subvolumes (modern Arch default)
2. **Rolling Release:** Validates package management for rolling distros
3. **systemd-boot:** Tests UEFI bootloader configuration
4. **BTRFS Features:** Comprehensive subvolume testing
5. **Realistic Layout:** Matches actual Arch Linux installations
6. **Cross-Platform:** Works on any Linux distribution

## Key Differences from Other Distribution Tests

1. **Rolling Release**: No version numbers, always latest snapshot
2. **BTRFS Subvolumes**: Advanced filesystem layout testing
3. **systemd-boot**: Modern UEFI bootloader (not GRUB)
4. **pacman**: Different package manager from apt/dpkg/rpm
5. **Simplified Structure**: `/bin` and `/sbin` are symlinks to `/usr/bin`

## Future Enhancements

- [ ] Add ZFS filesystem option
- [ ] Add encrypted BTRFS volumes (LUKS)
- [ ] Add systemd-homed integration
- [ ] Add more pacman packages in database
- [ ] Add AUR helper configuration
- [ ] Add snapshots in @snapshots subvolume
- [ ] Add kernel module configuration

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

### "BTRFS not supported"

```bash
# Install BTRFS tools
sudo dnf install btrfs-progs    # Fedora
sudo apt-get install btrfs-progs  # Ubuntu
```

### Tests timeout or hang

```bash
# Clean up stale NBD devices
for i in {0..15}; do
    sudo qemu-nbd --disconnect /dev/nbd$i
done

# Remove test images
rm -f /tmp/arch-test.img
```

## References

- [Arch Linux Wiki](https://wiki.archlinux.org/)
- [BTRFS Documentation](https://btrfs.readthedocs.io/)
- [systemd-boot](https://www.freedesktop.org/software/systemd/man/systemd-boot.html)
- [pacman](https://wiki.archlinux.org/title/Pacman)
- [Arch Installation Guide](https://wiki.archlinux.org/title/Installation_guide)

## Statistics

- **Test File:** `tests/arch_realistic.rs`
- **Lines of Code:** 900+
- **Test Functions:** 3
- **APIs Tested:** 25+ Phase 3 APIs including BTRFS operations
- **Directory Structure:** 30+ directories
- **System Files:** 20+ files
- **Systemd Units:** 4 services
- **BTRFS Subvolumes:** 4 subvolumes
- **Bootloader:** systemd-boot
