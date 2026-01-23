# guestkit

A pure Rust implementation of libguestfs-compatible API for disk image inspection and manipulation. Designed to work seamlessly with [hyper2kvm](https://github.com/ssahani/hyper2kvm).

[![License: LGPL v3](https://img.shields.io/badge/License-LGPL_v3-blue.svg)](https://www.gnu.org/licenses/lgpl-3.0)
[![Rust](https://img.shields.io/badge/rust-1.70%2B-orange.svg)](https://www.rust-lang.org/)

## Features

- 🔍 **GuestFS-Compatible API** - 578 functions compatible with libguestfs (563 fully working, 15 API-defined) - **97.4% coverage, 76.8% of libguestfs**
- 🦀 **Pure Rust** - No C dependencies for core library, memory safe, high performance
- 💿 **Disk Format Support** - QCOW2, VMDK, RAW detection via magic bytes
- 📊 **Partition Tables** - MBR and GPT parsing, partition creation/deletion/resizing
- 🗂️ **Filesystem Operations** - Mount/unmount, create (mkfs), check (fsck), tune, trim, resize
- 🔎 **OS Inspection** - Detect OS type, distro, version, architecture, hostname
- 📦 **Package Management** - List and inspect dpkg/rpm packages
- 🌐 **Network Configuration** - Read hostname, DNS, interface config
- 👤 **System Configuration** - Timezone, locale, users, groups, systemd units
- 🔐 **Encryption Support** - LUKS encrypted volumes
- 📚 **LVM Support** - Logical volume management
- 🗜️ **Archive Operations** - tar, tgz, cpio creation and extraction
- 🔑 **Checksums** - MD5, SHA1, SHA256, SHA384, SHA512
- 🛡️ **Security Operations** - SELinux, AppArmor, capabilities, ACLs
- 🥾 **Boot Configuration** - Bootloader detection, kernel management, UEFI support
- 💾 **Advanced Disk Operations** - Swap management, hexdump, strings, secure scrubbing
- 🔧 **Service Management** - systemd/sysvinit service detection, cron jobs
- 🔑 **SSH Operations** - SSH key management, certificates, authorized_keys
- ⚙️ **Configuration Editing** - Augeas-based config file editing
- 🪟 **Windows Support** - Registry hive access, Windows-specific inspection
- 🌳 **Btrfs Advanced** - Subvolumes, snapshots, balance, scrub operations
- 📊 **File Metadata** - Detailed stat operations, inode info, permissions
- 🛠️ **Utility Functions** - Feature detection, settings management, debug tools
- 🔷 **XFS Support** - XFS repair, administration, info, database operations
- 💿 **ISO Operations** - ISO creation, inspection, mounting
- 📤 **Advanced Transfer** - Offset-based downloads/uploads, device copying
- 💾 **Disk Image Management** - Create, resize, convert, sparsify, snapshot disk images
- 🔧 **Internal API** - State management, environment parsing, debug functions
- 💿 **NTFS Operations** - ntfsclone, ntfsfix, label management
- 🔷 **Extended Filesystem** - ext2/3/4 UUID, label, dump/restore operations
- 🔍 **Glob Operations** - Pattern matching, ls0, find0, case-insensitive search
- 🔧 **Node Operations** - mknod, mkfifo, mktemp, truncate, utimens
- 💾 **MD/RAID** - Software RAID creation, management, inspection
- 🛡️ **SELinux Extended** - SELinux inspection, restorecon
- 🔐 **Capabilities** - Linux capabilities management
- 🔒 **ACL Operations** - POSIX ACL management
- 🪟 **Hivex** - Windows registry hive manipulation (16 functions)
- 🔄 **Rsync** - rsync-based file synchronization
- 🥾 **Syslinux** - syslinux/extlinux bootloader installation
- 📔 **Journal** - systemd journal reading, export, verification
- 👁️ **Inotify** - file monitoring with inotify
- 🗜️ **SquashFS** - SquashFS creation and extraction
- 🦠 **YARA** - malware scanning with YARA rules
- 🔬 **TSK** - forensics with The Sleuth Kit (deleted file recovery)
- 💽 **ZFS** - ZFS filesystem management (10 functions)
- 🪟 **LDM** - Windows dynamic disk support (8 functions)
- 🔀 **Multipath** - multipath device management
- 🥾 **GRUB** - GRUB bootloader installation and configuration
- ⚡ **F2FS** - Flash-Friendly File System support
- 💾 **Bcache** - block cache management
- 📁 **DOSFS** - FAT12/16/32 filesystem tools
- 📦 **CPIO** - CPIO archive format support
- 🗂️ **NILFS** - log-structured filesystem support
- 🔧 **UFS** - Unix File System support
- 🌲 **ReiserFS** - ReiserFS filesystem management
- 📝 **JFS** - Journaled File System support
- 🔹 **Minix** - Minix filesystem support
- 🩺 **SMART** - disk health monitoring with smartctl
- 🧹 **SysPrep** - VM preparation operations (remove unique data)
- 🛠️ **Utilities** - version info, QEMU detection, umask, device stats
- 🔧 **Block Device Ops** - setro/setrw, flush, reread partition table, block/sector size
- 📝 **Base64** - Base64 encoding/decoding for file content
- 🔄 **Extended Swap** - swap label/UUID management operations
- 💾 **DD Operations** - dd-style copy, zero device operations
- 📍 **Positional I/O** - pread/pwrite with offset support
- 🔍 **Virt Tools** - virt-inspector, virt-convert, virt-resize, virt-sparsify info
- 🗜️ **Compression** - gzip, bzip2, xz compression/decompression for files and devices
- 🏷️ **Label Operations** - generic filesystem label/UUID management (auto-detect fs type)
- 🔄 **Sync Operations** - sync, drop_caches, flush for data consistency
- 🔖 **Attributes** - extended attributes (xattr) and file flags management
- 🧩 **Partition Types** - GPT type GUID, attributes, expand partition tables
- 🔗 **Link Management** - symbolic and hard link operations
- 🐍 **Python Bindings** - PyO3-based native Python bindings
- ⚡ **Retry Logic** - Built-in exponential backoff for reliable operations
- 🔌 **Extensible** - Modular architecture for easy extension

## Quick Start

### Installation

```bash
# Install system dependencies (Fedora/RHEL)
sudo dnf install qemu-img

# From source
git clone https://github.com/ssahani/guestkit
cd guestkit
cargo build --release
cargo install --path .
```

### Basic Usage

#### Library (GuestFS API)

```rust
use guestkit::guestfs::Guestfs;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Create handle
    let mut g = Guestfs::new()?;

    // Add disk image (read-only)
    g.add_drive_ro("/path/to/disk.qcow2")?;

    // Launch (analyzes disk)
    g.launch()?;

    // Inspect OS
    let roots = g.inspect_os()?;
    for root in roots {
        println!("Found OS root: {}", root);
        println!("  Type: {}", g.inspect_get_type(&root)?);
        println!("  Distro: {}", g.inspect_get_distro(&root)?);
        println!("  Version: {}.{}",
            g.inspect_get_major_version(&root)?,
            g.inspect_get_minor_version(&root)?);
        println!("  Hostname: {}", g.inspect_get_hostname(&root)?);
    }

    // List partitions
    let partitions = g.list_partitions()?;
    for part in partitions {
        println!("Partition: {}", part);
        println!("  Filesystem: {}", g.vfs_type(&part)?);
        println!("  Label: {}", g.vfs_label(&part).unwrap_or_default());
    }

    // Cleanup
    g.shutdown()?;

    Ok(())
}
```

#### Python Bindings

```python
from guestkit import Guestfs

g = Guestfs()
g.add_drive_ro("/path/to/disk.qcow2")
g.launch()

# Inspect OS
roots = g.inspect_os()
for root in roots:
    print(f"OS Type: {g.inspect_get_type(root)}")
    print(f"Distro: {g.inspect_get_distro(root)}")
    print(f"Version: {g.inspect_get_major_version(root)}.{g.inspect_get_minor_version(root)}")

# List filesystems
filesystems = g.list_filesystems()
for device, fstype in filesystems.items():
    print(f"{device}: {fstype}")

g.shutdown()
```

## Project Structure

```
guestkit/
├── Cargo.toml
├── README.md
├── ARCHITECTURE.md                    # Architecture documentation
├── LIBGUESTFS_COMPARISON.md           # Comparison with libguestfs (733 functions)
├── GUESTFS_IMPLEMENTATION_STATUS.md   # Implementation status
├── src/
│   ├── lib.rs                         # Library entry point
│   ├── core/                          # Core utilities
│   │   ├── error.rs                   # Error types
│   │   ├── retry.rs                   # Retry logic
│   │   └── types.rs                   # Common types
│   ├── disk/                          # Disk operations (Pure Rust)
│   │   ├── reader.rs                  # Disk image reader (magic byte detection)
│   │   ├── partition.rs               # MBR/GPT parser
│   │   └── filesystem.rs              # Filesystem detection (ext4, NTFS, etc.)
│   ├── guestfs/                       # GuestFS-compatible API (486 functions)
│   │   ├── handle.rs                  # Main handle (new/launch/shutdown)
│   │   ├── inspect.rs                 # OS inspection (12 functions)
│   │   ├── device.rs                  # Device operations (9 functions)
│   │   ├── partition.rs               # Partition operations (6 functions)
│   │   ├── mount.rs                   # Mount operations (11 functions)
│   │   ├── file_ops.rs                # File operations (35+ functions)
│   │   ├── lvm.rs                     # LVM operations (9 functions)
│   │   ├── command.rs                 # Command execution (4 functions)
│   │   ├── archive.rs                 # Archive operations (7 functions)
│   │   ├── luks.rs                    # LUKS encryption (6 functions)
│   │   ├── checksum.rs                # Checksums and file content (9 functions)
│   │   ├── filesystem.rs              # Filesystem operations (8 functions)
│   │   ├── utils.rs                   # File utilities (11 functions)
│   │   ├── network.rs                 # Network configuration (7 functions)
│   │   ├── package.rs                 # Package management (5 functions)
│   │   ├── system.rs                  # System configuration (13 functions)
│   │   ├── security.rs                # Security operations (10 functions)
│   │   ├── boot.rs                    # Boot configuration (10 functions)
│   │   ├── disk_ops.rs                # Advanced disk operations (12 functions)
│   │   ├── service.rs                 # Service management (8 functions)
│   │   ├── ssh.rs                     # SSH operations (10 functions)
│   │   ├── part_mgmt.rs               # Partition management (9 functions)
│   │   ├── augeas.rs                  # Configuration editing (11 functions)
│   │   ├── resize.rs                  # Filesystem resize (7 functions)
│   │   ├── windows.rs                 # Windows operations (12 functions)
│   │   ├── btrfs.rs                   # Btrfs operations (12 functions)
│   │   ├── metadata.rs                # File metadata (17 functions)
│   │   ├── misc.rs                    # Miscellaneous utilities (22 functions)
│   │   ├── xfs.rs                     # XFS operations (4 functions)
│   │   ├── iso.rs                     # ISO operations (4 functions)
│   │   ├── transfer.rs                # Advanced file transfer (8 functions)
│   │   ├── disk_mgmt.rs               # Disk image management (10 functions)
│   │   ├── internal.rs                # Internal operations (16 functions)
│   │   ├── ntfs.rs                    # NTFS operations (5 functions)
│   │   ├── ext_ops.rs                 # Extended filesystem ops (11 functions)
│   │   ├── glob_ops.rs                # Glob operations (7 functions)
│   │   ├── node_ops.rs                # Node operations (10 functions)
│   │   ├── md_ops.rs                  # MD/RAID operations (5 functions)
│   │   ├── selinux_ops.rs             # SELinux extended (4 functions)
│   │   ├── cap_ops.rs                 # Capabilities (4 functions)
│   │   ├── acl_ops.rs                 # ACL operations (8 functions)
│   │   ├── hivex_ops.rs               # Hivex operations (16 functions)
│   │   ├── rsync_ops.rs               # Rsync operations (2 functions)
│   │   ├── syslinux_ops.rs            # Syslinux operations (2 functions)
│   │   ├── journal_ops.rs             # Journal operations (11 functions)
│   │   ├── inotify_ops.rs             # Inotify operations (6 functions)
│   │   ├── squashfs_ops.rs            # SquashFS operations (3 functions)
│   │   ├── yara_ops.rs                # YARA operations (4 functions)
│   │   ├── tsk_ops.rs                 # TSK operations (4 functions)
│   │   ├── zfs_ops.rs                 # ZFS operations (10 functions)
│   │   ├── ldm_ops.rs                 # LDM operations (8 functions)
│   │   ├── mpath_ops.rs               # Multipath operations (5 functions)
│   │   ├── grub_ops.rs                # GRUB operations (7 functions)
│   │   ├── f2fs_ops.rs                # F2FS operations (4 functions)
│   │   ├── bcache_ops.rs              # Bcache operations (5 functions)
│   │   ├── dosfs_ops.rs               # DOSFS operations (5 functions)
│   │   ├── cpio_ops.rs                # CPIO operations (3 functions)
│   │   ├── nilfs_ops.rs               # NILFS operations (4 functions)
│   │   └── ufs_ops.rs                 # UFS operations (3 functions)
│   ├── python/                        # Python bindings (PyO3)
│   │   └── bindings.rs
│   └── converters/                    # Disk format converters
│       └── disk_converter.rs          # qemu-img wrapper
├── examples/                          # Example programs
│   ├── inspect_disk.rs
│   └── list_partitions.rs
└── tests/                             # Integration tests (24 unit, 9 doc tests)
```

## Architecture

See [ARCHITECTURE.md](ARCHITECTURE.md) for detailed architecture documentation.

### Core Modules

#### `guestfs` - GuestFS-Compatible API (115 functions)
- **handle.rs** - Main GuestFS handle (lifecycle management)
- **inspect.rs** - OS inspection (12 functions, fully working)
- **device.rs** - Device operations (9 functions, fully working)
- **partition.rs** - Partition operations (6 functions, fully working)
- **mount.rs** - Mount operations (11 functions, API-defined, needs NBD)
- **file_ops.rs** - File operations (35+ functions, API-defined, needs FS parser or NBD)
- **lvm.rs** - LVM operations (5 functions, API-defined)
- **command.rs** - Command execution (4 functions, API-defined)
- **archive.rs** - Archive operations (8 functions, API-defined)

#### `disk` - Pure Rust Disk Operations
- **reader.rs** - Disk image reader with format detection via magic bytes
- **partition.rs** - MBR and GPT partition table parser
- **filesystem.rs** - Filesystem detection (ext4, NTFS, XFS, Btrfs, FAT32)

#### `core` - Core Utilities
- **error.rs** - Error types using thiserror
- **retry.rs** - Exponential backoff retry logic
- **types.rs** - Common types (DiskFormat, GuestType, etc.)

### Implementation Status

| Category | Functions | Status |
|----------|-----------|--------|
| **Total APIs** | 115 | 35 working, 80 API-defined |
| **Lifecycle** | 8 | ✅ Fully working |
| **Inspection** | 12 | ✅ Fully working |
| **Device Ops** | 9 | ✅ Fully working |
| **Partition Ops** | 6 | ✅ Fully working |
| **Mount Ops** | 11 | ⚠️ API-only (needs NBD) |
| **File Ops** | 35+ | ⚠️ API-only (needs FS parser) |
| **LVM** | 5 | ⚠️ API-only |
| **Commands** | 4 | ⚠️ API-only |
| **Archives** | 8 | ⚠️ API-only |

See [LIBGUESTFS_COMPARISON.md](LIBGUESTFS_COMPARISON.md) for comparison with all 733 libguestfs functions.

### Design Principles

1. **Pure Rust** - No C dependencies (except qemu-img tool)
2. **Memory Safety** - Leveraging Rust's ownership system
3. **Zero-cost Abstractions** - High-level APIs with no runtime overhead
4. **API Compatibility** - GuestFS-compatible function signatures
5. **Modularity** - Clean separation of concerns
6. **Testability** - Comprehensive test coverage (33 tests passing)

## Examples

See the [`examples/`](examples/) directory for complete examples:

- **convert_disk.rs** - Convert disk image formats
- **detect_format.rs** - Detect and inspect disk images
- **retry_example.rs** - Using retry logic with exponential backoff

Run examples with:

```bash
cargo run --example convert_disk
cargo run --example detect_format
```

## API Coverage

### Statistics

| Metric | Count | Percentage |
|--------|-------|------------|
| **LibGuestFS functions** | 733 | 100% |
| **GuestKit APIs defined** | 364 | 49.7% |
| **Fully working** | 349 | 47.6% |
| **API-only (needs impl)** | 15 | 2.0% |

### Comparison with LibGuestFS

See [LIBGUESTFS_COMPARISON.md](LIBGUESTFS_COMPARISON.md) for:
- Complete function-by-function comparison
- What's implemented vs what's missing
- Implementation phases and timeline
- Recommendations for full compatibility

### Implementation Strategy

**Current**: Pure Rust implementation without C dependencies

**Working**:
- ✅ Disk format detection (magic bytes)
- ✅ Partition table parsing (MBR, GPT)
- ✅ Filesystem detection (superblock analysis)
- ✅ OS inspection (35 functions)

**Planned (Phase 1)**:
- 🔄 NBD mounting (qemu-nbd integration)
- 🔄 File I/O via NBD mount
- 🔄 Command execution via chroot
- 🔄 Archive operations (tar, tgz)

## Integration with hyper2kvm

guestkit is designed as a drop-in replacement for libguestfs in [hyper2kvm](https://github.com/ssahani/hyper2kvm):

```rust
use guestkit::guestfs::Guestfs;

// Same API as libguestfs!
let mut g = Guestfs::new()?;
g.add_drive_ro("/path/to/disk.qcow2")?;
g.launch()?;

// Inspect VM
let roots = g.inspect_os()?;
for root in &roots {
    println!("OS: {}", g.inspect_get_distro(root)?);
    println!("Version: {}.{}",
        g.inspect_get_major_version(root)?,
        g.inspect_get_minor_version(root)?);
}
```

Benefits over libguestfs for hyper2kvm:
- ✅ **No root required** for read-only operations
- ✅ **Faster** - No VM launch overhead
- ✅ **Simpler** - No C dependencies
- ✅ **Safer** - Rust memory safety
- ⚠️ **Limited** - Not all functions implemented yet (Phase 1 in progress)

## Dependencies

### System Dependencies

- **qemu-img** - Disk image manipulation (QEMU tools) - Optional, for format conversion

```bash
# Fedora/RHEL
sudo dnf install qemu-img

# Ubuntu/Debian
sudo apt install qemu-utils

# Arch Linux
sudo pacman -S qemu
```

**Note:** Unlike libguestfs, guestkit does NOT require libguestfs.so or any C library dependencies. It's pure Rust!

### Rust Dependencies

See [`Cargo.toml`](Cargo.toml) for complete list:

- **thiserror** - Custom error types
- **byteorder** - Binary parsing
- **memmap2** - Memory-mapped I/O
- **regex** - Pattern matching
- **pyo3** (optional) - Python bindings
- **serde** / **serde_json** - Serialization

## Development

### Building

```bash
# Debug build
cargo build

# Release build (optimized)
cargo build --release

# Run tests
cargo test

# Run with logging
RUST_LOG=debug cargo run -- convert --source test.vmdk --output test.qcow2
```

### Running Tests

```bash
# Unit tests
cargo test

# Integration tests
cargo test --test '*'

# With coverage
cargo tarpaulin --out Html
```

### Code Quality

```bash
# Format code
cargo fmt

# Lint
cargo clippy

# Check documentation
cargo doc --no-deps --open
```

## Cargo Features

guestkit uses Cargo features for optional functionality:

- **`disk-ops`** (default) - Disk operation utilities
- **`guest-inspect`** (default) - Guest OS inspection
- **`python-bindings`** (optional) - PyO3 Python bindings

```toml
[dependencies]
guestkit = { version = "0.1", features = ["guest-inspect"] }

# With Python bindings
guestkit = { version = "0.1", features = ["python-bindings"] }
```

Build with Python bindings:

```bash
cargo build --features python-bindings
```

## Roadmap

See [LIBGUESTFS_COMPARISON.md](LIBGUESTFS_COMPARISON.md) for detailed implementation timeline.

### Phase 0: Foundation (✅ COMPLETE)
- [x] Pure Rust architecture (no libguestfs.so dependency)
- [x] Disk format detection (QCOW2, VMDK, RAW)
- [x] Partition table parsing (MBR, GPT)
- [x] Filesystem detection (ext4, NTFS, XFS, Btrfs, FAT32)
- [x] GuestFS-compatible API structure (115 functions)
- [x] OS inspection (12 functions fully working)
- [x] Device operations (9 functions fully working)
- [x] Partition operations (6 functions fully working)
- [x] Python bindings foundation (PyO3)

### Phase 1: Essential Operations (🔄 PLANNED - 3 weeks)
Implement for 90% hyper2kvm compatibility:

- [ ] **NBD mounting** - qemu-nbd integration for filesystem access
- [ ] **Command execution** (4 functions) - command, sh, sh_lines
- [ ] **Archive operations** (8 functions) - tar_in, tar_out, tgz_in, tgz_out
- [ ] **File operations** (10 functions) - cp, mv, download, upload, grep, find
- [ ] **LUKS operations** (6 functions) - luks_open, luks_close, luks_format
- [ ] **LVM activation** (4 functions) - vg_activate_all, lvcreate, lvremove

**Total: 30+ critical functions**

### Phase 2: Filesystem Operations (📅 FUTURE - 2 weeks)
- [ ] Filesystem creation (mkfs, mke2fs, mkfs_btrfs)
- [ ] Filesystem repair (fsck, e2fsck, ntfsfix, xfs_repair)
- [ ] Extended attributes (getxattr, setxattr)
- [ ] Resize operations (resize2fs, ntfsresize)

### Phase 3: Advanced Features (📅 FUTURE - 4 weeks)
- [ ] Augeas (config file editing)
- [ ] Windows registry (Hivex operations)
- [ ] Partition management (add/delete/resize)
- [ ] SELinux relabeling

### Phase 4: Specialized (📅 OPTIONAL)
- [ ] Btrfs advanced features (subvolumes, snapshots)
- [ ] ZFS support
- [ ] YARA malware scanning
- [ ] File recovery (TSK)

## Contributing

Contributions are welcome! Please:

1. Fork the repository
2. Create a feature branch
3. Write tests for new functionality
4. Ensure all tests pass
5. Submit a pull request

See [CONTRIBUTING.md](CONTRIBUTING.md) for detailed guidelines.

## License

This project is licensed under the **GNU Lesser General Public License v3.0 (LGPL-3.0)**.

See [LICENSE](LICENSE) for full license text.

## Acknowledgments

- **libguestfs** - Inspiration and design patterns
- **hyper2kvm** - Primary use case and integration target
- **QEMU** - Disk format conversion tools

## Support

- **GitHub Issues**: [Report bugs](https://github.com/ssahani/guestkit/issues)
- **Documentation**: [API docs](https://docs.rs/guestkit)
- **Examples**: See [`examples/`](examples/) directory

## Related Projects

- **[hyper2kvm](https://github.com/ssahani/hyper2kvm)** - Production-grade VM migration toolkit
- **[hypersdk](https://github.com/ssahani/hypersdk)** - High-performance hypervisor SDK
- **[libguestfs](https://libguestfs.org/)** - Guest filesystem inspection library

---

Made with ❤️ for reliable VM operations
