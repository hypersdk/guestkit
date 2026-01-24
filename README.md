# guestkit

A pure Rust toolkit for disk image inspection and manipulation. Designed to work seamlessly with [hyper2kvm](https://github.com/ssahani/hyper2kvm).

[![License: LGPL v3](https://img.shields.io/badge/License-LGPL_v3-blue.svg)](https://www.gnu.org/licenses/lgpl-3.0)
[![Rust](https://img.shields.io/badge/rust-1.70%2B-orange.svg)](https://www.rust-lang.org/)

## Features

- 🦀 **Ergonomic Rust API** - Type-safe enums, builder patterns, and fluent interfaces for modern Rust idioms
- 🔍 **Comprehensive API** - 578 disk image manipulation functions (563 fully implemented, 15 API-defined) - **97.4% implementation coverage**
- 🦀 **Pure Rust** - No C dependencies for core library, memory safe, high performance
- ⚡ **Compile-Time Safety** - Type-safe filesystems, OS detection, and partition tables prevent runtime errors
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

### Advanced CLI Features (guestkit)

- 📊 **Multiple Output Formats** - JSON, YAML, CSV, and plain text for automation and scripting
- 🎯 **Inspection Profiles** - Specialized analysis modes:
  - **Security Profile** - SSH hardening, firewall status, user security, SELinux/AppArmor, risk scoring
  - **Migration Profile** - Complete inventory for VM migration planning
  - **Performance Profile** - System tuning opportunities and bottleneck detection
- 🔄 **VM Comparison** - Diff two VMs or compare multiple VMs against a baseline
- 📤 **Report Export** - HTML and Markdown report generation for documentation
- ⚡ **Result Caching** - SHA256-based caching for instant repeated inspections (60x speedup)
- 🚀 **Batch Processing** - Multi-threaded parallel inspection of multiple disk images

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

### CLI Tool (`guestctl`)

GuestCtl is a command-line tool for inspecting and manipulating disk images without mounting them.

```bash
# Inspect a disk image
sudo guestctl inspect ubuntu.qcow2

# List filesystems
sudo guestctl filesystems ubuntu.qcow2

# List installed packages
sudo guestctl packages ubuntu.qcow2

# Copy files
sudo guestctl cp ubuntu.qcow2:/etc/passwd ./passwd

# List directory
sudo guestctl ls ubuntu.qcow2 /etc

# Read file
sudo guestctl cat ubuntu.qcow2 /etc/hostname

# JSON output for scripting
sudo guestctl inspect --json ubuntu.qcow2 | jq '.operating_systems[0].distro'
```

**Available Commands:**
- `inspect` - OS detection and information
- `filesystems` - List devices, partitions, LVM
- `packages` - List installed software (dpkg, RPM, pacman)
- `cp` - Copy files from disk images
- `ls` - List directories
- `cat` - Read files

### CLI Tool (`guestkit`) - Advanced Features

GuestKit provides advanced VM inspection capabilities with profiles, caching, and batch processing.

```bash
# Basic inspection
guestkit inspect vm.qcow2

# JSON output for automation
guestkit inspect vm.qcow2 --output json | jq '.os.hostname'

# Security audit profile
guestkit inspect vm.qcow2 --profile security

# Migration planning profile
guestkit inspect vm.qcow2 --profile migration --output json

# Performance tuning profile
guestkit inspect vm.qcow2 --profile performance

# Compare two VMs
guestkit diff vm-before.qcow2 vm-after.qcow2

# Compare multiple VMs against baseline
guestkit compare baseline.qcow2 vm1.qcow2 vm2.qcow2 vm3.qcow2

# Export HTML report
guestkit inspect vm.qcow2 --export html --export-output report.html

# Export Markdown inventory
guestkit inspect vm.qcow2 --export markdown --export-output inventory.md

# Use caching for faster repeated inspections
guestkit inspect vm.qcow2 --cache  # First run: ~30s, subsequent: <0.5s

# Batch inspect multiple VMs in parallel
guestkit inspect-batch *.qcow2 --parallel 4 --cache

# Cache management
guestkit cache-stats
guestkit cache-clear
```

**Available Commands:**
- `inspect` - Comprehensive VM inspection with profiles
- `diff` - Compare two disk images
- `compare` - Compare multiple VMs against baseline
- `inspect-batch` - Parallel batch inspection
- `list` - List files in disk image
- `extract` - Extract files from disk image
- `execute` - Execute commands in guest
- `backup` - Create tar backup from guest
- `convert` - Convert disk image formats
- `create` - Create new disk image
- `check` - Filesystem check
- `usage` - Disk usage statistics
- `detect` - Detect disk image format
- `info` - Disk image information
- `cache-stats` - View cache statistics
- `cache-clear` - Clear inspection cache
- `version` - Show version information

**Full Documentation:** [`docs/CLI_GUIDE.md`](docs/CLI_GUIDE.md)

---

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

#### Ergonomic Rust API (Recommended)

GuestKit provides modern Rust patterns for better type safety and ergonomics:

```rust
use guestkit::guestfs::{Guestfs, FilesystemType, OsType, Distro};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Builder pattern for configuration
    let mut g = Guestfs::builder()
        .add_drive_ro("/path/to/disk.qcow2")
        .verbose(true)
        .build_and_launch()?;

    // Type-safe filesystem creation
    g.mkfs("/dev/sda1")
        .ext4()              // Type-safe enum, not string!
        .label("rootfs")
        .blocksize(4096)
        .create()?;

    // Type-safe OS detection with pattern matching
    for root in g.inspect_os()? {
        let os_type = OsType::from_str(&g.inspect_get_type(&root)?);

        match os_type {
            OsType::Linux => {
                let distro = Distro::from_str(&g.inspect_get_distro(&root)?);

                // Smart methods on enums
                if let Some(pkg_mgr) = distro.package_manager() {
                    println!("Package manager: {}", pkg_mgr);  // "deb", "rpm", "pacman", etc.
                }

                // Exhaustive pattern matching
                match distro {
                    Distro::Ubuntu | Distro::Debian => println!("Debian-based"),
                    Distro::Fedora | Distro::Rhel => println!("RPM-based"),
                    Distro::Archlinux => println!("Arch Linux"),
                    _ => println!("Other Linux"),
                }
            }
            OsType::Windows => println!("Windows detected"),
            _ => println!("Other OS"),
        }
    }

    g.shutdown()?;
    Ok(())
}
```

See [`docs/ERGONOMIC_API.md`](docs/ERGONOMIC_API.md) and [`docs/MIGRATION_GUIDE.md`](docs/MIGRATION_GUIDE.md) for details.

#### Python Bindings

GuestKit provides native Python bindings via PyO3 for seamless integration with Python workflows.

**Installation:**
```bash
# Quick build (recommended)
./build_python.sh

# Or manual build
python3 -m venv .venv
source .venv/bin/activate
pip install maturin
PYO3_USE_ABI3_FORWARD_COMPATIBILITY=1 maturin develop --features python-bindings

# Verify installation
python3 -c "import guestkit; print(guestkit.__version__)"
```

**Basic Example:**
```python
from guestkit import Guestfs

# Create handle and configure
g = Guestfs()
g.add_drive_ro("/path/to/disk.qcow2")
g.launch()

# Inspect operating system
roots = g.inspect_os()
for root in roots:
    print(f"OS Type: {g.inspect_get_type(root)}")
    print(f"Distro: {g.inspect_get_distro(root)}")
    print(f"Version: {g.inspect_get_major_version(root)}.{g.inspect_get_minor_version(root)}")
    print(f"Hostname: {g.inspect_get_hostname(root)}")

# Mount and read files
mountpoints = g.inspect_get_mountpoints(root)
for mount_path, device in sorted(mountpoints, key=lambda x: len(x[0])):
    g.mount_ro(device, mount_path)

if g.is_file("/etc/hostname"):
    content = g.read_file("/etc/hostname")
    print(f"Hostname from file: {content.decode('utf-8').strip()}")

# List installed packages
apps = g.inspect_list_applications(root)
print(f"Total packages: {len(apps)}")

# Cleanup
g.umount_all()
g.shutdown()
```

**More Examples:**
- [`examples/python/comprehensive_example.py`](examples/python/comprehensive_example.py) - Complete example demonstrating all features
- [`examples/python/extract_files.py`](examples/python/extract_files.py) - File extraction from disk images
- [`examples/python/archive_example.py`](examples/python/archive_example.py) - Archive operations (tar, tar.gz)
- [`examples/python/basic_inspection.py`](examples/python/basic_inspection.py) - OS detection and inspection
- [`examples/python/list_filesystems.py`](examples/python/list_filesystems.py) - Enumerate devices and partitions
- [`examples/python/mount_and_explore.py`](examples/python/mount_and_explore.py) - Mount and read files
- [`examples/python/package_inspection.py`](examples/python/package_inspection.py) - List installed packages
- [`examples/python/create_disk.py`](examples/python/create_disk.py) - Create new disk images
- [`examples/python/test_bindings.py`](examples/python/test_bindings.py) - Comprehensive test suite

**Full Documentation:**
- [`docs/guides/PYTHON_BINDINGS.md`](docs/guides/PYTHON_BINDINGS.md) - Comprehensive Python guide
- [`docs/api/PYTHON_API_REFERENCE.md`](docs/api/PYTHON_API_REFERENCE.md) - Complete API reference with 100+ methods
- [`docs/status/PYTHON_BINDINGS_STATUS.md`](docs/status/PYTHON_BINDINGS_STATUS.md) - Implementation status and build instructions

**Python API Coverage:**
- 58 Guestfs methods covering OS inspection, file operations, device management, LVM, archives, and more
- 3 DiskConverter methods for format conversion and detection
- Comprehensive error handling with Python exceptions
- Full type conversion between Rust and Python types

## Project Structure

```
guestkit/
├── Cargo.toml
├── README.md
├── ARCHITECTURE.md                    # Architecture documentation
├── GUESTFS_IMPLEMENTATION_STATUS.md   # Implementation status (578 functions)
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
│   │   ├── ufs_ops.rs                 # UFS operations (3 functions)
│   │   ├── inspect_enhanced.rs        # Enhanced inspection with profiles
│   │   └── windows_registry.rs        # Windows registry parsing
│   ├── python.rs                      # Python bindings (PyO3, 100+ methods)
│   ├── converters/                    # Disk format converters
│   │   └── disk_converter.rs          # qemu-img wrapper
│   └── cli/                           # CLI implementation
│       ├── commands.rs                # CLI commands
│       ├── profiles/                  # Inspection profiles
│       ├── exporters/                 # HTML/Markdown exporters
│       ├── formatters.rs              # Output formatters
│       ├── templates/                 # Askama templates
│       ├── diff.rs                    # VM comparison
│       └── cache.rs                   # Result caching
├── examples/                          # Example programs
│   ├── rust/
│   │   ├── inspect_disk.rs
│   │   └── list_partitions.rs
│   └── python/                        # Python examples
│       ├── test_bindings.py           # Comprehensive test suite
│       ├── comprehensive_example.py   # Full-featured example
│       ├── archive_example.py         # Archive operations
│       ├── extract_files.py           # File extraction
│       └── README.md                  # Python examples guide
├── docs/                              # Documentation (organized)
│   ├── README.md                      # Documentation index
│   ├── guides/                        # User guides
│   ├── api/                           # API documentation
│   ├── architecture/                  # Architecture docs
│   ├── development/                   # Contributor docs
│   ├── testing/                       # Testing docs
│   ├── status/                        # Implementation status
│   └── archive/                       # Historical docs
├── pyproject.toml                     # Python package config
├── build_python.sh                    # Python build script
└── tests/                             # Integration tests
```

## Documentation

📚 **Complete documentation is organized in [`docs/`](docs/)**

**Quick Links:**
- 🚀 **[Quick Start](docs/guides/QUICKSTART.md)** - Get started in minutes
- 📖 **[CLI Guide](docs/guides/CLI_GUIDE.md)** - Command-line usage
- 🐍 **[Python Guide](docs/guides/PYTHON_BINDINGS.md)** - Python API guide
- 🔍 **[API Reference](docs/api/PYTHON_API_REFERENCE.md)** - Complete Python API
- 🏗️ **[Architecture](docs/architecture/ARCHITECTURE.md)** - System architecture
- 🧪 **[Testing Guide](docs/testing/TESTING.md)** - How to test
- 📊 **[Project Status](docs/status/PROJECT_SUMMARY.md)** - Implementation status
- 🚀 **[Enhancement Roadmap](docs/development/ENHANCEMENT_ROADMAP.md)** - Future plans

See **[docs/README.md](docs/README.md)** for complete documentation index.

## Architecture

See [docs/architecture/ARCHITECTURE.md](docs/architecture/ARCHITECTURE.md) for detailed architecture documentation.

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

See [GUESTFS_IMPLEMENTATION_STATUS.md](GUESTFS_IMPLEMENTATION_STATUS.md) for complete function implementation status.

### Design Principles

1. **Pure Rust** - No C dependencies (except qemu-img tool)
2. **Memory Safety** - Leveraging Rust's ownership system
3. **Zero-cost Abstractions** - High-level APIs with no runtime overhead
4. **Clean API Design** - Intuitive function signatures and error handling
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

guestkit is designed to work seamlessly with [hyper2kvm](https://github.com/ssahani/hyper2kvm):

```rust
use guestkit::guestfs::Guestfs;

// Simple, intuitive API
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

Benefits for hyper2kvm:
- ✅ **No root required** for read-only operations
- ✅ **Faster** - Pure Rust implementation
- ✅ **Simpler** - No C dependencies
- ✅ **Safer** - Rust memory safety
- ✅ **Comprehensive** - 578 functions, 97.4% implementation coverage

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

**Note:** guestkit is a pure Rust implementation with no C library dependencies!

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

**Rust Tests:**
```bash
# Unit tests
cargo test

# Integration tests
cargo test --test '*'

# With coverage
cargo tarpaulin --out Html
```

**Python Tests:**
```bash
# Install pytest (if not already installed)
pip install pytest

# Run Python bindings tests
pytest tests/test_python_bindings.py -v

# Run with test disk image
export GUESTKIT_TEST_IMAGE=/path/to/test.qcow2
pytest tests/test_python_bindings.py -v
```

**Comprehensive Testing:**
```bash
# Run all tests (Rust + Python)
cargo test --all-features
pytest tests/test_python_bindings.py -v

# Example Python integration test
cd examples/python
sudo python3 test_bindings.py /path/to/disk.img
```

**Note on Permissions:** Some tests require root access for mounting disk images.

See [docs/testing/TESTING.md](docs/testing/TESTING.md) for complete testing documentation.

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

See [GUESTFS_IMPLEMENTATION_STATUS.md](GUESTFS_IMPLEMENTATION_STATUS.md) for detailed implementation status.

### Phase 0: Foundation (✅ COMPLETE)
- [x] Pure Rust architecture (no C dependencies)
- [x] Disk format detection (QCOW2, VMDK, RAW)
- [x] Partition table parsing (MBR, GPT)
- [x] Filesystem detection (ext4, NTFS, XFS, Btrfs, FAT32)
- [x] Comprehensive API structure (578 functions, 97.4% implemented)
- [x] OS inspection (30+ functions fully working)
- [x] Device operations (20+ functions fully working)
- [x] Partition operations (15+ functions fully working)
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

- **hyper2kvm** - Primary use case and integration target
- **QEMU** - Disk format conversion tools
- **Rust Community** - For excellent crates and tooling

## Support

- **GitHub Issues**: [Report bugs](https://github.com/ssahani/guestkit/issues)
- **Documentation**: [API docs](https://docs.rs/guestkit)
- **Examples**: See [`examples/`](examples/) directory

## Related Projects

- **[hyper2kvm](https://github.com/ssahani/hyper2kvm)** - Production-grade VM migration toolkit
- **[hypersdk](https://github.com/ssahani/hypersdk)** - High-performance hypervisor SDK

---

Made with ❤️ for reliable VM operations
