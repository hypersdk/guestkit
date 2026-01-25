# guestctl Architecture

Complete **Pure Rust** implementation for guest VM operations.

## Overview

**guestctl** is a modern Rust library providing:
- **Disk format conversion** (qemu-img wrapper)
- **Pure Rust disk image reading** (qcow2, raw, vmdk detection)
- **Pure Rust partition table parsing** (MBR, GPT)
- **Pure Rust filesystem detection** (ext4, NTFS, XFS, Btrfs, FAT32)
- **Guest OS detection and manipulation**
- **PyO3 Python bindings** for zero-overhead integration
- **Production-ready CLI tool**

## Key Design Principle

**Zero External C Dependencies** - All disk and filesystem operations are implemented in pure Rust without  or other C libraries (except qemu-img for conversion).

## Architecture Layers

```
┌─────────────────────────────────────────────────────────┐
│                    Applications                          │
│  ┌───────────────┐  ┌──────────────┐  ┌──────────────┐ │
│  │ guestctl CLI  │  │ hyper2kvm    │  │ Custom Apps  │ │
│  └───────────────┘  └──────────────┘  └──────────────┘ │
└─────────────────────────────────────────────────────────┘
                             │
                             ▼
┌─────────────────────────────────────────────────────────┐
│              High-Level Rust API                        │
│  ┌──────────────┐  ┌──────────────┐  ┌──────────────┐ │
│  │ DiskConverter│  │GuestDetector │  │ Pipeline     │ │
│  └──────────────┘  └──────────────┘  └──────────────┘ │
└─────────────────────────────────────────────────────────┘
                             │
                             ▼
┌─────────────────────────────────────────────────────────┐
│              Pure Rust Disk Layer                        │
│  ┌──────────────┐  ┌──────────────┐  ┌──────────────┐ │
│  │ DiskReader   │  │PartitionTable│  │ FileSystem   │ │
│  │ (qcow2, raw) │  │ (MBR, GPT)   │  │(ext4, NTFS)  │ │
│  └──────────────┘  └──────────────┘  └──────────────┘ │
└─────────────────────────────────────────────────────────┘
                             │
                             ▼
┌─────────────────────────────────────────────────────────┐
│              Core Utilities                              │
│  ┌──────────────┐  ┌──────────────┐  ┌──────────────┐ │
│  │ Error Types  │  │ Retry Logic  │  │ Type System  │ │
│  └──────────────┘  └──────────────┘  └──────────────┘ │
└─────────────────────────────────────────────────────────┘
                             │
                             ▼
┌─────────────────────────────────────────────────────────┐
│              External Tools (Optional)                   │
│  ┌──────────────┐                                       │
│  │ qemu-img     │  (for format conversion only)         │
│  └──────────────┘                                       │
└─────────────────────────────────────────────────────────┘
```

## Module Structure

### `src/core/` - Core Utilities

**Purpose:** Fundamental types and utilities used throughout guestctl

**Files:**
- `error.rs` - Error types using thiserror
- `retry.rs` - Exponential backoff retry logic
- `types.rs` - Common types (DiskFormat, GuestType, etc.)

**Key Types:**
```rust
pub enum Error {
    Io(std::io::Error),
    Conversion(String),
    Detection(String),
    CommandFailed(String),
    InvalidFormat(String),
    // ...
}

pub type Result<T> = std::result::Result<T, Error>;

pub enum DiskFormat {
    Qcow2,
    Raw,
    Vmdk,
    Vhd,
    Vhdx,
    Vdi,
    Unknown,
}

pub struct GuestIdentity {
    pub os_type: GuestType,
    pub os_name: String,
    pub os_version: String,
    pub architecture: String,
    pub firmware: Firmware,
    pub init_system: Option<String>,
    pub distro: Option<String>,
}
```

### `src/disk/` - Pure Rust Disk Operations

**Purpose:** Read and parse disk images, partition tables, and filesystems without C dependencies

#### `disk/reader.rs` - Disk Image Reader

**Features:**
- Auto-detect disk format from magic bytes
- Read from raw and qcow2 disk images
- Memory-efficient byte-level access

**Example:**
```rust
use guestctl::disk::DiskReader;

let mut reader = DiskReader::open("/path/to/disk.qcow2")?;
println!("Format: {:?}", reader.format());
println!("Size: {} bytes", reader.size());

let mut buffer = vec![0u8; 512];
reader.read_exact_at(0, &mut buffer)?; // Read MBR
```

#### `disk/partition.rs` - Partition Table Parser

**Features:**
- Parse MBR (Master Boot Record) partition tables
- Parse GPT (GUID Partition Table)
- Extract partition metadata (LBA, size, type)

**Example:**
```rust
use guestctl::disk::PartitionTable;

let partition_table = PartitionTable::parse(&mut reader)?;
for partition in partition_table.partitions() {
    println!("Partition {}: {} sectors at LBA {}",
        partition.number,
        partition.size_sectors,
        partition.start_lba
    );
}
```

**Supported Partition Schemes:**
- MBR (DOS partition table)
- GPT (GUID Partition Table)
- Automatic detection

#### `disk/filesystem.rs` - Filesystem Detection

**Features:**
- Detect ext2/ext3/ext4 filesystems
- Detect NTFS filesystems
- Detect FAT32 filesystems
- Detect XFS filesystems
- Detect Btrfs filesystems
- Extract filesystem labels and UUIDs

**Example:**
```rust
use guestctl::disk::FileSystem;

let fs = FileSystem::detect(&mut reader, &partition)?;
println!("Filesystem: {:?}", fs.fs_type());
if let Some(label) = fs.label() {
    println!("Label: {}", label);
}
```

**Detection Method:**
Each filesystem has a unique signature (magic bytes) at specific offsets:
- ext2/3/4: 0xEF53 at offset 1024+56
- NTFS: "NTFS    " at offset 3
- FAT32: "FAT32   " at offset 82
- XFS: "XFSB" at offset 0
- Btrfs: "_BHRfS_M" at offset 65536+64

#### `disk/loop_device.rs` - Loop Device Management (Primary)

**Purpose:** Mount disk images as block devices using Linux loop devices (built-in)

**Features:**
- Automatic loop device allocation (`losetup -f`)
- Partition scanning (`--partscan`)
- Read-only and read-write modes
- Automatic sudo handling
- Device cleanup on drop

**Supported Formats:**
- RAW - Raw disk images
- IMG - Disk image files
- ISO - ISO 9660 images
- Block devices

**Advantages:**
- ✅ Built into Linux kernel (no module loading)
- ✅ Faster setup (~100ms)
- ✅ More reliable
- ✅ No external dependencies

**Example:**
```rust
use guestctl::disk::LoopDevice;

let mut loop_dev = LoopDevice::new()?;
loop_dev.connect("/path/to/disk.raw", true)?; // read-only

if let Some(device_path) = loop_dev.device_path() {
    println!("Loop device: {}", device_path.display());
    // Access partitions: /dev/loop0p1, /dev/loop0p2, etc.
}
// Automatically disconnected on drop
```

#### `disk/nbd.rs` - NBD Device Management (Fallback)

**Purpose:** Mount advanced disk formats using qemu-nbd (Network Block Device)

**Features:**
- Automatic NBD module loading
- Support for compressed formats
- Snapshot/incremental disk support
- Format auto-detection
- Device cleanup on drop

**Supported Formats:**
- QCOW2 - QEMU Copy-On-Write v2
- VMDK - VMware Virtual Disk
- VDI - VirtualBox Disk Image
- VHD/VPC - Hyper-V Virtual Hard Disk

**Requirements:**
- NBD kernel module (auto-loaded)
- qemu-nbd tool

**Example:**
```rust
use guestctl::disk::NbdDevice;

let mut nbd = NbdDevice::new()?;
nbd.connect("/path/to/disk.qcow2", true)?; // read-only

println!("NBD device: {}", nbd.device_path().display());
// Access partitions: /dev/nbd0p1, /dev/nbd0p2, etc.
// Automatically disconnected on drop
```

**Automatic Strategy Selection:**

The `Guestfs::launch()` method automatically selects the optimal strategy:

```rust
// Strategy: Loop device first (fast path), NBD fallback (advanced formats)
if LoopDevice::is_format_supported(&drive.path) {
    // RAW/IMG/ISO → Use loop device (no modules needed)
    use_loop_device();
} else {
    // QCOW2/VMDK/VDI/VHD → Use NBD (auto-load module)
    use_nbd_device();
}
```

### `src/converters/` - Disk Format Conversion

**Purpose:** Convert disk images between formats using qemu-img

**Files:**
- `disk_converter.rs` - Main converter implementation

**Features:**
- Format auto-detection
- Compression support
- Snapshot flattening
- Progress reporting (planned)

**Example:**
```rust
use guestctl::converters::DiskConverter;

let converter = DiskConverter::new();
let result = converter.convert(
    Path::new("/path/to/source.vmdk"),
    Path::new("/path/to/output.qcow2"),
    "qcow2",
    true,  // compress
    true,  // flatten
)?;
```

### `src/detectors/` - Guest OS Detection

**Purpose:** High-level guest OS detection using pure Rust disk analysis

**Files:**
- `guest_detector.rs` - GuestDetector implementation

**Features:**
- OS type detection (Linux, Windows, BSD, etc.)
- Version detection
- Architecture detection
- Firmware detection (BIOS/UEFI)
- Distribution detection (Fedora, Ubuntu, RHEL, etc.)

**Detection Strategy:**
1. Open disk image
2. Parse partition table (MBR/GPT)
3. Detect filesystem type on each partition
4. Infer OS from filesystem patterns:
   - NTFS → Windows
   - ext4/XFS/Btrfs → Linux
   - Filesystem labels provide distribution hints
5. GPT → UEFI firmware, MBR → BIOS firmware

**Example:**
```rust
use guestctl::detectors::GuestDetector;

let detector = GuestDetector::new();
let guest = detector.detect_from_image("/path/to/disk.qcow2")?;

println!("OS: {} {}", guest.os_name, guest.os_version);
println!("Type: {:?}", guest.os_type);
println!("Arch: {}", guest.architecture);
println!("Firmware: {:?}", guest.firmware);
```

### `src/python.rs` - PyO3 Python Bindings

**Purpose:** Native Python module for zero-overhead integration

**Features:**
- **Zero subprocess overhead** - Direct function calls
- **Type-safe Python API** - Proper dictionaries, not strings
- **Error propagation** - Rust errors → Python exceptions

**Build:**
```bash
maturin develop --features python-bindings
```

**Usage:**
```python
import guestctl_py

converter = guestctl_py.DiskConverter()
result = converter.convert(
    source="/path/to/vm.vmdk",
    output="/path/to/vm.qcow2",
    format="qcow2",
    compress=True
)

if result["success"]:
    print(f"Size: {result['output_size']} bytes")
```

### `src/orchestrator/` - Pipeline Orchestration

**Purpose:** Multi-stage migration pipelines

**Stages:**
```
FETCH → FLATTEN → INSPECT → FIX → CONVERT → VALIDATE
```

**Features:**
- Sequential stage execution
- Context passing between stages
- Pre/post hooks
- Error recovery
- Progress tracking (planned)

## Build System

### Cargo Features

```toml
[features]
default = ["disk-ops", "guest-inspect"]
disk-ops = []                    # Disk operations (qemu-img)
guest-inspect = []               # Guest OS detection
python-bindings = ["pyo3"]       # PyO3 Python module
```

### Dependencies

**Core:**
- `anyhow`, `thiserror` - Error handling
- `tokio` - Async runtime
- `serde`, `serde_json` - Serialization
- `clap` - CLI parsing

**Disk Operations:**
- `memmap2` - Memory-mapped file I/O
- `byteorder` - Binary parsing
- `regex` - Pattern matching

**Python:**
- `pyo3` - Python bindings (optional)

**No C Dependencies** - No bindgen, no pkg-config, no !

## Error Handling

### Error Flow

```
Disk I/O → Rust Error → Application
 (std::io)  (Result<T>)  (user handling)
```

### Error Types

```rust
#[derive(Error, Debug)]
pub enum Error {
    #[error("I/O error: {0}")]
    Io(#[from] io::Error),

    #[error("Conversion error: {0}")]
    Conversion(String),

    #[error("Detection error: {0}")]
    Detection(String),

    // ...
}

pub type Result<T> = std::result::Result<T, Error>;
```

## Memory Management

### Safe Rust Patterns

All memory management uses safe Rust:
- No manual memory allocation
- No unsafe pointer arithmetic (except for reading disk bytes)
- RAII for file handles
- Automatic cleanup via Drop trait

### Minimal Unsafe Code

Unsafe code is limited to:
- Reading raw bytes from disk images
- Memory-mapped file I/O (via memmap2)

All unsafe blocks are carefully audited and documented.

## Testing Strategy

### Unit Tests

```rust
#[cfg(test)]
mod tests {
    #[test]
    fn test_retry_success_on_first_attempt() { ... }

    #[test]
    fn test_disk_format_conversion() { ... }

    #[test]
    fn test_partition_parsing() { ... }
}
```

### Integration Tests

```python
# integration/tests/test_integration.py
def test_version_command():
    result = subprocess.run([guestctl_path, "version"], ...)
    assert result.returncode == 0
```

### Doc Tests

```rust
/// Convert disk image
///
/// # Examples
///
/// ```no_run
/// use guestctl::DiskConverter;
/// let converter = DiskConverter::new();
/// let result = converter.convert(...)?;
/// ```
pub fn convert(...) { ... }
```

## Performance Considerations

### Zero-Cost Abstractions

- Rust wrappers compile to same code as manual implementations
- No runtime overhead for safety
- Inlining optimizations

### Memory Efficiency

- Stream-based reading (no loading entire disk into memory)
- Memory-mapped I/O for large files
- Efficient buffer reuse

### Async Support (Planned)

```rust
pub async fn convert_async(...) -> Result<ConversionResult> {
    tokio::task::spawn_blocking(move || {
        converter.convert(...)
    }).await?
}
```

## Integration Points

### For hyper2kvm

**Option 1: Python Subprocess**
```python
from guestctl_wrapper import GuestkitWrapper
wrapper = GuestkitWrapper()
result = wrapper.convert(source, output, compress=True)
```

**Option 2: PyO3 Native (Recommended)**
```python
import guestctl_py
converter = guestctl_py.DiskConverter()
result = converter.convert(source, output, "qcow2", compress=True)
```

**Option 3: Rust Library**
```rust
use guestctl::DiskConverter;
let result = converter.convert(source, output, "qcow2", true, true)?;
```

## Future Enhancements

### Short-term
- [ ] Complete qcow2 image format parser (currently detects, not fully parses)
- [ ] File reading from ext4 filesystems
- [ ] File reading from NTFS filesystems
- [ ] More accurate OS version detection
- [ ] Async disk operations
- [ ] Progress callbacks

### Long-term
- [ ] Cloud integration (AWS, Azure, GCP)
- [ ] Distributed operations
- [ ] Web UI
- [ ] gRPC API

## Design Principles

1. **Pure Rust** - No C dependencies (except qemu-img tool)
2. **Safety** - Leverage Rust's type system and ownership
3. **Zero-cost** - Abstractions with no runtime overhead
4. **Correctness** - Extensive testing and type checking
5. **Usability** - Ergonomic high-level APIs
6. **Performance** - Async, efficient, optimized
7. **Compatibility** - Works with existing tools
8. **Maintainability** - Clean, well-documented code

## Disk Format Support

### Read Support

| Format | Detection | Full Parsing |
|--------|-----------|--------------|
| Raw    | ✅        | ✅           |
| QCOW2  | ✅        | 🚧 Planned   |
| VMDK   | ✅        | 🚧 Planned   |
| VHD    | 🚧        | 🚧 Planned   |
| VHDX   | 🚧        | 🚧 Planned   |
| VDI    | 🚧        | 🚧 Planned   |

### Conversion Support

All formats supported via qemu-img wrapper

## Filesystem Support

| Filesystem | Detection | Read Files |
|------------|-----------|------------|
| ext2/3/4   | ✅        | 🚧 Planned |
| NTFS       | ✅        | 🚧 Planned |
| FAT32      | ✅        | ❌         |
| XFS        | ✅        | 🚧 Planned |
| Btrfs      | ✅        | 🚧 Planned |

## References

- **Rust std::io**: https://doc.rust-lang.org/std/io/
- **PyO3 docs**: https://pyo3.rs
- **Rust async book**: https://rust-lang.github.io/async-book/
- **MBR specification**: https://en.wikipedia.org/wiki/Master_boot_record
- **GPT specification**: https://en.wikipedia.org/wiki/GUID_Partition_Table

---

**Version:** 0.1.0
**License:** LGPL-3.0-or-later
**Author:** Susant Sahani <ssahani@gmail.com>
