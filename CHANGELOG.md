# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased] - Phase 3 In Progress

### Added - Testing and Quality Infrastructure

#### Integration Tests (2 test suites)
- **integration_basic.rs** (10 comprehensive tests)
  - Disk creation and inspection workflow
  - Partition creation and management
  - Filesystem creation, mount, and operations
  - File I/O operations (read, write, copy, delete)
  - Archive operations (tar creation and extraction)
  - Checksum verification (md5, sha256)
  - File stat and metadata operations
  - Guest command execution
  - Multiple partition scenarios
  - Recursive copy operations

- **integration_lvm_luks.rs** (10 advanced tests)
  - LUKS encryption basic workflow
  - LVM volume management
  - Combined LUKS + LVM scenarios
  - LUKS key management and rotation
  - Volume group operations
  - VG scan and activation
  - Read-only encrypted volumes
  - LVM volume inspection

#### Performance Benchmarks
- **benchmarks.rs** - Criterion-based benchmarks
  - Disk creation (10MB to 500MB sizes)
  - Partition operations performance
  - Filesystem creation (ext4, xfs)
  - File operations (1KB to 1MB writes/reads)
  - Mount/unmount cycle overhead
  - Checksum algorithms (md5, sha256 on various file sizes)
  - Archive operations (tar in/out)
  - Launch/shutdown overhead measurement

#### Project Documentation
- **ROADMAP.md** - Comprehensive project roadmap
  - Phase 3: Stabilization and Integration (Q1 2026)
  - Phase 4: Python Bindings (Q2 2026)
  - Phase 5: Performance Optimization (Q2-Q3 2026)
  - Phase 6: Advanced Features (Q3-Q4 2026)
  - Phase 7: Ecosystem Integration (2027)
  - Success metrics and version milestones

- **API_REFERENCE.md** - Complete API documentation with examples
- **CONTRIBUTING.md** - Developer contribution guidelines
- **SECURITY.md** - Security policy and vulnerability reporting

#### CI/CD Infrastructure
- **ci.yml** - Comprehensive continuous integration
  - Multi-platform testing (Ubuntu, stable/beta Rust)
  - Code formatting and clippy checks
  - Code coverage with Codecov
  - Security audits with cargo-audit
  - Multi-target release builds
  - Documentation verification

- **release.yml** - Automated release workflow
  - Multi-architecture builds (x86_64, aarch64, musl)
  - Changelog extraction
  - Binary packaging with checksums
  - Automated crates.io publishing

### Infrastructure
- Benchmark harness configuration in Cargo.toml
- Criterion dependency for performance testing
- Test organization structure

## [0.2.0] - 2026-01-23 - Phase 2 Complete

### Added - Phase 2 Implementation

This massive update adds 73 new modules implementing 463 additional libguestfs-compatible APIs, bringing total coverage from 22.6% to 76.8% of libguestfs functionality.

#### Core Utilities (10 modules)
- **checksum**: File checksum operations (md5, sha1, sha256, sha384, sha512)
- **utils**: File type detection, readlink, symlink checking
- **misc**: Version info, available features, settings management
- **util_ops**: Device stats, umask, QEMU detection
- **glob_ops**: Pattern matching, find0, ls0, grep, case-insensitive search
- **base64_ops**: Base64 encoding/decoding for file content
- **dd_ops**: dd-style copy, zero device operations
- **pread_ops**: Positional read/write with offset support
- **sync_ops**: sync, drop_caches, flush operations
- **label_ops**: Generic filesystem label/UUID management

#### Filesystem Support (14 modules)
- **filesystem**: Generic mkfs, fsck, tune2fs, zerofree, fstrim
- **btrfs**: Btrfs subvolumes, snapshots, balance, scrub
- **xfs**: XFS repair, info, admin, db operations
- **ntfs**: ntfsclone, ntfsfix, label management
- **ext_ops**: ext2/3/4 UUID, label, dump/restore
- **f2fs_ops**: Flash-Friendly File System support
- **dosfs_ops**: FAT12/16/32 filesystem management
- **nilfs_ops**: Log-structured filesystem support
- **ufs_ops**: Unix File System support
- **reiserfs_ops**: ReiserFS filesystem management
- **jfs_ops**: Journaled File System support
- **minix_ops**: Minix filesystem support
- **zfs_ops**: ZFS filesystem management (10 functions)
- **squashfs_ops**: SquashFS creation and extraction

#### Disk & Partition Management (12 modules)
- **disk_ops**: Advanced disk operations (swap, hexdump, strings, scrubbing)
- **disk_mgmt**: Disk image creation, resize, convert, snapshot
- **part_mgmt**: Partition creation, deletion, resizing
- **part_type_ops**: GPT type GUID, attributes, expand
- **blockdev_ops**: setro/setrw, flush, reread partition table
- **resize**: resize2fs, ntfsresize, xfs_growfs
- **md_ops**: Software RAID creation, management, inspection
- **bcache_ops**: Block cache management
- **ldm_ops**: Windows dynamic disk support (8 functions)
- **mpath_ops**: Multipath device management
- **smart_ops**: Disk health monitoring with smartctl
- **swap_ops**: Swap label/UUID management

#### Security Operations (4 modules)
- **security**: SELinux and AppArmor management
- **selinux_ops**: SELinux inspection, restorecon
- **cap_ops**: Linux capabilities management
- **acl_ops**: POSIX ACL management

#### System Management (5 modules)
- **system**: Timezone, locale, users, groups, systemd configuration
- **boot**: Bootloader, kernels, UEFI, fstab management
- **service**: systemd, sysvinit, cron job management
- **network**: Hostname, interfaces, DNS settings
- **package**: dpkg/rpm package inspection

#### Bootloader Configuration (2 modules)
- **grub_ops**: GRUB bootloader installation and configuration
- **syslinux_ops**: syslinux/extlinux bootloader installation

#### File Metadata & Attributes (6 modules)
- **metadata**: Stat operations, inode, times, permissions
- **node_ops**: mknod, mkfifo, mktemp, truncate, utimens
- **link_ops**: Symbolic and hard link management
- **attr_ops**: Extended attributes (xattr) management
- **owner_ops**: File ownership operations
- **time_ops**: File timestamp operations

#### File Transfer & Archives (5 modules)
- **transfer**: Advanced file transfer with offset downloads/uploads
- **cpio_ops**: CPIO archive support
- **compress_ops**: gzip, bzip2, xz compression/decompression
- **rsync_ops**: rsync-based file synchronization
- **backup_ops**: Backup operations

#### Specialized Tools Integration (6 modules)
- **augeas**: Augeas configuration file editing
- **hivex_ops**: Windows registry hive manipulation (16 functions)
- **journal_ops**: systemd journal reading, export, verification
- **inotify_ops**: File monitoring with inotify
- **yara_ops**: Malware scanning with YARA rules
- **tsk_ops**: Forensics with The Sleuth Kit (deleted file recovery)

#### Windows, SSH & ISO (3 modules)
- **windows**: Windows registry hives and Windows-specific inspection
- **ssh**: SSH keys, certificates, authorized_keys management
- **iso**: ISO creation, inspection, mounting

#### Virtualization & Inspection (3 modules)
- **sysprep_ops**: VM preparation (removing unique data)
- **virt_ops**: virt-* tool equivalents (inspector, convert info)
- **inspect_ext_ops**: Extended inspection operations

#### Internal & Text Processing (3 modules)
- **internal**: State management, environment, debug operations
- **sed_ops**: sed-style text editing operations
- **template_ops**: Template processing and VM cloning operations

### Enhanced

#### Existing Modules (5 modules)
- **archive**: Added cpio support and additional tar operations
- **file_ops**: Added extended file operations (head, tail, grep, cat, etc.)
- **handle**: Added config and state management methods
- **lvm**: Added extended LVM operations
- **mount**: Added mount option handling improvements

### Fixed
- Type mismatches in template_ops.rs (String to bytes conversion)
- Type casting for chown_recursive parameters (u32 to i32)
- Removed unused imports in multiple modules
- Compilation errors across all new modules

### Documentation
- Updated GUESTFS_IMPLEMENTATION_STATUS.md with comprehensive Phase 2 coverage
- Updated implementation statistics: 578 APIs total, 563 working (97.4%)
- Documented coverage increase from 22.6% to 76.8% of libguestfs
- Added detailed function listings for all 76 operation categories

### Testing
- All 97 unit tests passing
- API structure tests for all new modules
- Successful compilation with zero errors

### Project Statistics
- **Total Modules**: 84 Rust source files
- **Total APIs**: 578 functions
- **Working APIs**: 563 (97.4% functional)
- **libguestfs Coverage**: 76.8% (563 of 733 total libguestfs APIs)
- **Lines of Code**: ~15,000+ lines of implementation
- **Test Coverage**: 97 unit tests

## [0.1.0] - Phase 1 Complete

### Initial Implementation
- Core disk access and inspection
- NBD device management via qemu-nbd
- Mount/unmount operations
- File I/O operations
- Command execution in guest
- Archive operations (tar, tgz)
- LUKS encryption support
- LVM support
- Basic partition management
- OS detection and inspection
