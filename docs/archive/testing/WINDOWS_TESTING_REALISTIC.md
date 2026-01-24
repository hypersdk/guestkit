# Windows Realistic Disk Image Testing

## Overview

This document describes the comprehensive Windows disk image testing infrastructure for GuestCtl. These tests create production-quality Windows images with complete system configurations including registry simulation, Windows services, and authentic directory structures for multiple Windows versions.

## What Makes These Tests "Realistic"

Unlike minimal test images, these Windows images include:

1. **Authentic Windows Directory Structure**
   - Complete `/Windows/System32` hierarchy
   - `/Program Files` and `/Program Files (x86)`
   - `/ProgramData` for application data
   - `/Users` with proper user profiles
   - System directories (`/Windows/Temp`, `/Recovery`, etc.)

2. **Windows Registry Simulation**
   - SYSTEM hive with CurrentVersion keys
   - SOFTWARE hive with installed applications
   - SAM (Security Account Manager)
   - SECURITY hive
   - DEFAULT user profile hive
   - User hives (NTUSER.DAT)

3. **Boot Configuration**
   - Boot Configuration Data (BCD) for both BIOS and UEFI
   - EFI boot entries (for GPT)
   - System Reserved partition (MBR) or MSR partition (GPT)
   - Boot manager configuration

4. **Windows Services**
   - Windows Update (wuauserv)
   - DHCP Client
   - DNS Client (Dnscache)
   - Windows Event Log
   - Service configuration files

5. **System Files**
   - Network configuration (`hosts` file)
   - Unattended installation XML
   - Version information
   - Event logs (System, Application, Security)
   - Pagefile, hibernation file, swap file

6. **Windows Binaries**
   - Fake PE (Portable Executable) headers
   - Common system binaries (cmd.exe, powershell.exe, explorer.exe, etc.)
   - System services (svchost.exe, services.exe, lsass.exe, etc.)

7. **User Profiles**
   - Administrator account
   - User registry hives
   - Desktop, Documents, Downloads folders
   - Public user profile

8. **Windows Update Integration**
   - Software Distribution directory
   - Update database
   - Download cache

## Windows Versions Supported

| Version | Product Name | Build | Edition | Boot Layout | Test Function |
|---------|--------------|-------|---------|-------------|---------------|
| **10** | Windows 10 Pro | 19045 | Professional | MBR/BIOS | `test_windows_10_mbr` |
| **11** | Windows 11 Pro | 22631 | Professional | EFI/GPT | `test_windows_11_efi` |
| **Server 2022** | Windows Server 2022 | 20348 | ServerStandard | EFI/GPT | `test_windows_server_2022_efi` |

## Disk Layout

### MBR/BIOS Layout (Windows 10)

```
MBR Partition Table
├── Partition 1: System Reserved
│   ├── Start: Sector 2048
│   ├── Size: 100 MB
│   ├── Filesystem: NTFS
│   ├── Label: SYSTEM
│   ├── Bootable: Yes
│   └── Contents: Boot files, BCD
│
└── Partition 2: Windows
    ├── Start: After partition 1
    ├── Size: ~924 MB (remainder)
    ├── Filesystem: NTFS
    ├── Label: Windows
    └── Contents: Full Windows installation
```

### EFI/GPT Layout (Windows 11, Server 2022)

```
GPT Partition Table
├── Partition 1: EFI System Partition (ESP)
│   ├── Type: c12a7328-f81f-11d2-ba4b-00a0c93ec93b (EFI)
│   ├── Start: Sector 2048
│   ├── Size: 100 MB
│   ├── Filesystem: VFAT
│   ├── Label: SYSTEM
│   └── Contents: EFI boot files, BCD
│
├── Partition 2: Microsoft Reserved (MSR)
│   ├── Type: e3c9e316-0b5c-4db8-817d-f92df00215ae (MSR)
│   ├── Size: 16 MB
│   ├── No filesystem
│   └── Purpose: Reserved for Windows
│
└── Partition 3: Windows
    ├── Type: Linux filesystem (for testing)
    ├── Size: ~908 MB (remainder)
    ├── Filesystem: NTFS
    ├── Label: Windows
    └── Contents: Full Windows installation
```

### Windows Directory Structure

```
C:\
├── Windows\
│   ├── System32\
│   │   ├── config\
│   │   │   ├── SYSTEM (registry)
│   │   │   ├── SOFTWARE (registry)
│   │   │   ├── SAM (registry)
│   │   │   ├── SECURITY (registry)
│   │   │   ├── DEFAULT (registry)
│   │   │   └── services\
│   │   │       ├── wuauserv.ini
│   │   │       ├── Dhcp.ini
│   │   │       ├── Dnscache.ini
│   │   │       └── EventLog.ini
│   │   ├── drivers\
│   │   │   └── etc\
│   │   │       └── hosts
│   │   ├── winevt\
│   │   │   └── Logs\
│   │   │       ├── System.evtx
│   │   │       ├── Application.evtx
│   │   │       └── Security.evtx
│   │   ├── cmd.exe
│   │   ├── powershell.exe
│   │   ├── notepad.exe
│   │   ├── explorer.exe
│   │   ├── svchost.exe
│   │   ├── services.exe
│   │   ├── lsass.exe
│   │   └── winlogon.exe
│   ├── SysWOW64\
│   ├── Boot\
│   │   ├── EFI\
│   │   │   └── BCD
│   │   └── PCAT\
│   │       └── BCD
│   ├── Temp\
│   ├── Logs\
│   │   └── CBS\
│   ├── Panther\
│   │   └── unattend.xml
│   └── SoftwareDistribution\
│       ├── Download\
│       └── DataStore\
│           └── DataStore.edb
│
├── Program Files\
│   ├── Common Files\
│   └── Windows Defender\
│       └── platform.ini
│
├── Program Files (x86)\
│
├── ProgramData\
│   └── Microsoft\
│       └── Windows\
│           └── Windows Defender\
│               └── platform.ini
│
├── Users\
│   ├── Administrator\
│   │   ├── Desktop\
│   │   │   └── desktop.ini
│   │   ├── Documents\
│   │   ├── Downloads\
│   │   └── NTUSER.DAT (user registry)
│   └── Public\
│
├── EFI\                   # Only on GPT
│   └── Microsoft\
│       └── Boot\
│           └── BCD
│
├── Temp\
├── Recovery\
├── System Volume Information\
├── pagefile.sys
├── hiberfil.sys
└── swapfile.sys
```

## Tests Included

### Test 1: Windows 10 MBR/BIOS (`test_windows_10_mbr`)

Creates a realistic Windows 10 Professional disk image with:
- Legacy MBR partition table
- System Reserved partition
- NTFS filesystems
- Complete Windows structure
- 16-step creation process with validation

### Test 2: Windows 11 EFI/GPT (`test_windows_11_efi`)

Creates a realistic Windows 11 Professional disk image with:
- GPT partition table with ESP and MSR
- Modern UEFI boot configuration
- Latest Windows 11 build (22631)
- Complete registry simulation

### Test 3: Windows Server 2022 EFI/GPT (`test_windows_server_2022_efi`)

Creates a realistic Windows Server 2022 disk image with:
- Server edition configuration
- Server-specific build (20348)
- Enterprise features

### Test 4: OS Inspection (`test_windows_inspection`)

Validates guestfs inspection APIs on Windows 11:
- `inspect_os()` - Detects Windows installation
- `inspect_get_type()` - Returns "windows"
- `inspect_get_distro()` - Returns "windows"
- `inspect_get_major_version()` - Returns 10

### Test 5: NTFS Features (`test_windows_ntfs_features`)

Validates NTFS-specific functionality:
- Directory listing in System32
- File existence checks
- Directory detection
- NTFS filesystem operations

## Phase 3 APIs Tested

Each Windows image creation tests these Phase 3 APIs:

| API | Usage | Validation |
|-----|-------|------------|
| `create()` | Create guestfs handle | Handle created successfully |
| `add_drive()` | Attach disk image | Drive attached in read-write mode |
| `add_drive_ro()` | Attach for inspection | Drive attached read-only |
| `disk_create()` | Create sparse disk image | Image file created |
| `part_init()` | Initialize partition table | MBR or GPT created |
| `part_add()` | Add partition | Partitions created |
| `part_set_gpt_type()` | Set partition GUID | ESP and MSR marked correctly |
| `part_set_name()` | Set partition name | Names set correctly |
| `part_set_mbr_id()` | Set MBR partition type | NTFS type (0x07) set |
| `part_set_bootable()` | Set bootable flag | Boot partition marked |
| `mkfs()` | Create filesystem | NTFS and vfat created |
| `mount()` | Mount filesystem | Windows partition mounted |
| `mkdir()` / `mkdir_p()` | Create directories | Complete Windows tree created |
| `write()` | Write file | All Windows files written |
| `stat()` | Get file metadata | Returns correct file sizes |
| `lstat()` | Get symlink metadata | File metadata retrieved |
| `rm()` | Remove single file | File deleted successfully |
| `rm_rf()` | Remove directory tree | Directory tree deleted |
| `ls()` | List directory | System32 files listed |
| `is_file()` | Check if path is file | Binaries detected |
| `is_dir()` | Check if path is directory | Directories detected |
| `sync()` | Sync filesystem | Data flushed to disk |
| `umount_all()` | Unmount all | All filesystems unmounted |
| `shutdown()` | Shutdown appliance | Clean shutdown |

## Running the Tests

### Quick Start

```bash
# Run all Windows tests
./scripts/run_windows_realistic_tests.sh

# Run specific version
cargo test --test windows_realistic test_windows_11_efi -- --nocapture

# Run OS inspection test
cargo test --test windows_realistic test_windows_inspection -- --nocapture

# Run NTFS validation test
cargo test --test windows_realistic test_windows_ntfs_features -- --nocapture
```

### Prerequisites

1. **System Tools:**
   ```bash
   # Fedora/RHEL
   sudo dnf install qemu-img parted gdisk ntfs-3g

   # Ubuntu/Debian
   sudo apt-get install qemu-utils parted gdisk ntfs-3g
   ```

2. **NBD Module:**
   ```bash
   sudo modprobe nbd max_part=8
   ```

3. **Disk Space:**
   - Minimum: 1.5 GB free in `/tmp`
   - Recommended: 3 GB for multiple test runs

4. **Permissions:**
   ```bash
   # Setup test environment
   sudo ./scripts/setup_test_env.sh
   ```

### Test Output Example

```
=== Creating Realistic Windows 11 Pro Image (EFI/GPT) ===

[1/16] Creating Guestfs handle and disk image...
  ✓ Disk image created and guestfs launched

[2/16] Creating GPT partition table for UEFI...
  ✓ GPT with UEFI partitions created

[3/16] Creating filesystems...
  ✓ Filesystems: vfat (ESP) + NTFS (Windows)

[4/16] Mounting filesystems...
  ✓ Windows partition mounted

[5/16] Creating Windows directory structure...
  ✓ Directory structure created

[6/16] Creating Windows registry files...
  ✓ Registry hives created

[7/16] Creating boot configuration...
  ✓ Boot configuration created

[8/16] Creating Windows system files...
  ✓ Windows system files created

[9/16] Creating Windows services...
  ✓ Windows services configured

[10/16] Creating fake Windows binaries...
  ✓ Fake Windows binaries created

[11/16] Creating event logs...
  ✓ Event logs created

[12/16] Creating user profiles...
  ✓ User profiles created

[13/16] Creating Windows Update metadata...
  ✓ Windows Update metadata created

[14/16] Creating system files...
  ✓ System files created

[15/16] Testing Phase 3 APIs on Windows image...
  ✓ stat(/Windows/System32/version.txt): size=45 bytes
  ✓ rm() test passed
  ✓ rm_rf() test passed

[16/16] Creating Windows Defender configuration...
  ✓ Windows Defender configured

[Finalizing] Syncing and unmounting...

=== Windows 11 Pro Image Created Successfully! ===
  Image: /tmp/windows-test.img
  Size: 1024 MB
  Filesystem: NTFS
  Boot mode: EFI/GPT
  Edition: Professional
  Build: 22631
```

## Benefits of These Tests

1. **Multiple Windows Versions:** Tests Windows 10, 11, and Server 2022
2. **Dual Boot Support:** Tests both legacy BIOS and modern UEFI boot
3. **Complete Registry:** Simulates Windows registry hives
4. **Authentic Structure:** Matches real Windows installations
5. **Service Configuration:** Tests Windows service setup
6. **Cross-Platform:** Works on Linux distributions for Windows image testing

## Key Features

1. **Registry Simulation:**
   - SYSTEM hive with version information
   - SOFTWARE hive with application data
   - User profile hives (NTUSER.DAT)
   - Security hives (SAM, SECURITY)

2. **Boot Configuration:**
   - Boot Configuration Data (BCD)
   - EFI boot entries for GPT
   - BIOS boot configuration for MBR
   - System Reserved / MSR partitions

3. **Windows Services:**
   - Windows Update service
   - Network services (DHCP, DNS)
   - Event Log service
   - Service configuration files

4. **Binaries with PE Headers:**
   - Minimal PE (Portable Executable) headers
   - Common Windows executables
   - System services

## Comparison with Previous Windows Tests

| Feature | phase3_windows.rs | windows_realistic.rs | Status |
|---------|-------------------|----------------------|--------|
| Multiple versions | ✗ | ✓ (3 versions) | ✅ Enhanced |
| Registry simulation | ✗ | ✓ (5 hives) | ✅ Enhanced |
| Windows services | ✗ | ✓ (4 services) | ✅ Enhanced |
| Event logs | ✗ | ✓ (3 logs) | ✅ Enhanced |
| User profiles | ✗ | ✓ (Complete) | ✅ Enhanced |
| Boot configuration | Basic | ✓ (BCD) | ✅ Enhanced |
| PE binaries | ✗ | ✓ (8 binaries) | ✅ Enhanced |
| Windows Update | ✗ | ✓ | ✅ Enhanced |
| Unattend XML | ✗ | ✓ | ✅ Enhanced |

**The new Windows tests are significantly more comprehensive!**

## Future Enhancements

- [ ] Add Windows 8.1 support
- [ ] Add Windows Server 2019
- [ ] Add Active Directory simulation
- [ ] Add IIS configuration
- [ ] Add more Windows services
- [ ] Add Windows Features simulation
- [ ] Add encrypted volumes (BitLocker simulation)
- [ ] Add Windows Subsystem for Linux detection

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

### "NTFS not supported"

```bash
# Install NTFS support
sudo dnf install ntfs-3g        # Fedora
sudo apt-get install ntfs-3g    # Ubuntu
```

### Tests timeout or hang

```bash
# Clean up stale NBD devices
for i in {0..15}; do
    sudo qemu-nbd --disconnect /dev/nbd$i
done

# Remove test images
rm -f /tmp/windows-test.img
```

## References

- [Windows Partitioning](https://docs.microsoft.com/en-us/windows-hardware/manufacture/desktop/configure-biosmbr-based-hard-drive-partitions)
- [UEFI/GPT Partitioning](https://docs.microsoft.com/en-us/windows-hardware/manufacture/desktop/configure-uefigpt-based-hard-drive-partitions)
- [Windows Registry](https://docs.microsoft.com/en-us/windows/win32/sysinfo/registry)
- [BCD Reference](https://docs.microsoft.com/en-us/windows-hardware/drivers/devtest/bcd-boot-options-reference)
- [NTFS Documentation](https://docs.microsoft.com/en-us/windows/win32/fileio/filesystem-functionality-comparison)

## Statistics

- **Test File:** `tests/windows_realistic.rs`
- **Lines of Code:** 1,100+
- **Windows Versions:** 3 (10, 11, Server 2022)
- **Test Functions:** 5
- **APIs Tested:** 25+ Phase 3 APIs
- **Directory Structure:** 35+ directories
- **System Files:** 30+ files
- **Registry Hives:** 5 hives + 1 user hive
- **Windows Services:** 4 services
- **Event Logs:** 3 logs
- **Binaries:** 8 fake PE executables
- **Boot Layouts:** 2 (MBR/BIOS and EFI/GPT)
