# guestkit

A pure Rust implementation of libguestfs-compatible API for disk image inspection and manipulation. Designed to work seamlessly with [hyper2kvm](https://github.com/ssahani/hyper2kvm).

[![License: LGPL v3](https://img.shields.io/badge/License-LGPL_v3-blue.svg)](https://www.gnu.org/licenses/lgpl-3.0)
[![Rust](https://img.shields.io/badge/rust-1.70%2B-orange.svg)](https://www.rust-lang.org/)

## Features

- 🔍 **GuestFS-Compatible API** - 115 functions compatible with libguestfs (35 fully working, 80 API-defined)
- 🦀 **Pure Rust** - No C dependencies (except qemu-img tool), memory safe, high performance
- 💿 **Disk Format Support** - QCOW2, VMDK, RAW detection via magic bytes
- 📊 **Partition Tables** - MBR and GPT parsing
- 🗂️ **Filesystem Detection** - ext4, NTFS, XFS, Btrfs, FAT32 via superblock analysis
- 🔎 **OS Inspection** - Detect OS type, distro, version, architecture
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
│   ├── guestfs/                       # GuestFS-compatible API (115 functions)
│   │   ├── handle.rs                  # Main handle (new/launch/shutdown)
│   │   ├── inspect.rs                 # OS inspection (12 functions)
│   │   ├── device.rs                  # Device operations (9 functions)
│   │   ├── partition.rs               # Partition operations (6 functions)
│   │   ├── mount.rs                   # Mount operations (11 functions, API-only)
│   │   ├── file_ops.rs                # File operations (35+ functions, API-only)
│   │   ├── lvm.rs                     # LVM operations (5 functions, API-only)
│   │   ├── command.rs                 # Command execution (4 functions, API-only)
│   │   └── archive.rs                 # Archive operations (8 functions, API-only)
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
| **GuestKit APIs defined** | 115 | 15.7% |
| **Fully working** | 35 | 4.8% |
| **API-only (needs impl)** | 80 | 10.9% |

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
