# guestkit Architecture

Complete Rust implementation for guest VM operations with libguestfs integration.

## Overview

**guestkit** is a modern Rust library providing:
- **Disk format conversion** (qemu-img wrapper)
- **Complete libguestfs FFI bindings** (auto-generated via bindgen)
- **Guest OS detection and manipulation**
- **PyO3 Python bindings** for zero-overhead integration
- **Production-ready CLI tool**

## Architecture Layers

```
┌─────────────────────────────────────────────────────────┐
│                    Applications                          │
│  ┌───────────────┐  ┌──────────────┐  ┌──────────────┐ │
│  │ guestkit CLI  │  │ hyper2kvm    │  │ Custom Apps  │ │
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
│              Core Utilities                              │
│  ┌──────────────┐  ┌──────────────┐  ┌──────────────┐ │
│  │ Error Types  │  │ Retry Logic  │  │ Type System  │ │
│  └──────────────┘  └──────────────┘  └──────────────┘ │
└─────────────────────────────────────────────────────────┘
                             │
                             ▼
┌─────────────────────────────────────────────────────────┐
│           FFI Layer (Bindgen Auto-Generated)            │
│  ┌──────────────┐  ┌──────────────┐  ┌──────────────┐ │
│  │ libguestfs   │  │ qemu-img     │  │ System Libs  │ │
│  └──────────────┘  └──────────────┘  └──────────────┘ │
└─────────────────────────────────────────────────────────┘
                             │
                             ▼
┌─────────────────────────────────────────────────────────┐
│                 C Libraries                              │
│  ┌──────────────┐  ┌──────────────┐  ┌──────────────┐ │
│  │ libguestfs.so│  │ qemu tooling │  │ libc         │ │
│  └──────────────┘  └──────────────┘  └──────────────┘ │
└─────────────────────────────────────────────────────────┘
```

## Module Structure

### `src/core/` - Core Utilities

**Purpose:** Fundamental types and utilities used throughout guestkit

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
    Ffi(String),
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
use guestkit::converters::DiskConverter;

let converter = DiskConverter::new();
let result = converter.convert(
    Path::new("/path/to/source.vmdk"),
    Path::new("/path/to/output.qcow2"),
    "qcow2",
    true,  // compress
    true,  // flatten
)?;
```

### `src/ffi/` - libguestfs FFI Bindings

**Purpose:** Safe Rust bindings to libguestfs C library

**Files:**
- `bindings.rs` - Auto-generated FFI (via bindgen)
- `guestfs.rs` - Safe Rust wrapper
- `mod.rs` - Module exports

**Auto-Generation:**
```
build.rs → wrapper.h → bindgen → bindings.rs (OUT_DIR)
```

**Architecture:**
```
Raw C FFI (bindgen) → Safe Wrapper (Guestfs) → High-Level API
```

**Key Features:**
- **Automatic binding generation** - Always up-to-date with libguestfs
- **Complete API coverage** - 500+ functions from libguestfs
- **Type safety** - Rust ownership prevents memory leaks
- **Error handling** - All C errors converted to Rust Results
- **RAII cleanup** - Handles freed on Drop

**Example:**
```rust
use guestkit::ffi::Guestfs;

let g = Guestfs::new()?;
g.add_drive_ro("/path/to/disk.qcow2")?;
g.launch()?;

let roots = g.inspect_os()?;
for root in roots {
    let os_type = g.inspect_get_type(&root)?;
    println!("Found: {}", os_type);
}
```

### `src/detectors/` - Guest OS Detection

**Purpose:** High-level guest OS detection API

**Files:**
- `guest_detector.rs` - GuestDetector implementation

**Features:**
- OS type detection (Linux, Windows, BSD, etc.)
- Version detection
- Architecture detection
- Firmware detection (BIOS/UEFI)
- Distribution detection

**Example:**
```rust
use guestkit::detectors::GuestDetector;

let detector = GuestDetector::new();
let guest = detector.detect_from_image("/path/to/disk.qcow2")?;

println!("OS: {} {}", guest.os_name, guest.os_version);
println!("Type: {:?}", guest.os_type);
println!("Arch: {}", guest.architecture);
```

### `src/python.rs` - PyO3 Python Bindings

**Purpose:** Native Python module for zero-overhead integration

**Features:**
- **Zero subprocess overhead** - Direct FFI calls
- **Type-safe Python API** - Proper dictionaries, not strings
- **Error propagation** - C errors → Rust errors → Python exceptions

**Build:**
```bash
maturin develop --features python-bindings
```

**Usage:**
```python
import guestkit_py

converter = guestkit_py.DiskConverter()
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
default = ["disk-ops"]
disk-ops = []                    # Disk operations (qemu-img)
guest-inspect = []               # Guest OS detection
ffi-bindings = []                # libguestfs FFI bindings
python-bindings = ["pyo3"]       # PyO3 Python module
```

### Build Script (`build.rs`)

**Purpose:** Auto-generate FFI bindings at compile time

**Process:**
1. Check for `ffi-bindings` feature
2. Use pkg-config to find libguestfs
3. Run bindgen on `wrapper.h`
4. Generate `bindings.rs` in `$OUT_DIR`
5. Fallback to manual bindings if not available

**Benefits:**
- Always up-to-date with installed libguestfs version
- Type-safe bindings guaranteed to match C API
- No manual maintenance of 500+ function declarations
- Automatic documentation from C headers

## Error Handling

### Error Flow

```
C Error → FFI Error → Rust Error → Application
         (errno)     (Result<T>)    (user handling)
```

### Error Types

```rust
#[derive(Error, Debug)]
pub enum Error {
    #[error("I/O error: {0}")]
    Io(#[from] io::Error),

    #[error("Conversion error: {0}")]
    Conversion(String),

    #[error("FFI error: {0}")]
    Ffi(String),

    // ...
}

pub type Result<T> = std::result::Result<T, Error>;
```

### Error Conversion

```rust
// From C
last_error() → String → Error::Ffi

// From std
io::Error → Error::Io (automatic via #[from])

// Custom
"message" → Error::Conversion
```

## Memory Management

### RAII Pattern

```rust
pub struct Guestfs {
    handle: *mut guestfs_h,
}

impl Drop for Guestfs {
    fn drop(&mut self) {
        unsafe {
            if !self.handle.is_null() {
                guestfs_close(self.handle);
            }
        }
    }
}
```

**Benefits:**
- Automatic cleanup
- No memory leaks
- Exception-safe
- No manual free() calls

### String Management

```rust
// C string → Rust String (with automatic free)
unsafe fn c_str_to_string(ptr: *mut c_char) -> Option<String> {
    if ptr.is_null() {
        None
    } else {
        let c_str = CStr::from_ptr(ptr);
        let result = c_str.to_string_lossy().into_owned();
        libc::free(ptr as *mut c_void);  // Automatic free
        Some(result)
    }
}
```

## Testing Strategy

### Unit Tests

```rust
#[cfg(test)]
mod tests {
    #[test]
    fn test_retry_success_on_first_attempt() { ... }

    #[test]
    fn test_disk_format_conversion() { ... }
}
```

### Integration Tests

```python
# integration/tests/test_integration.py
def test_version_command():
    result = subprocess.run([guestkit_path, "version"], ...)
    assert result.returncode == 0
```

### Doc Tests

```rust
/// Convert disk image
///
/// # Examples
///
/// ```no_run
/// use guestkit::DiskConverter;
/// let converter = DiskConverter::new();
/// let result = converter.convert(...)?;
/// ```
pub fn convert(...) { ... }
```

## Performance Considerations

### Zero-Cost Abstractions

- Rust wrappers compile to same code as raw C calls
- No runtime overhead for safety
- Inlining optimizations

### Async Support (Planned)

```rust
pub async fn convert_async(...) -> Result<ConversionResult> {
    tokio::task::spawn_blocking(move || {
        converter.convert(...)
    }).await?
}
```

### Parallel Operations (Planned)

```rust
// Convert multiple disks concurrently
let results = futures::future::join_all(
    disks.into_iter().map(|disk| convert_async(disk))
).await;
```

## Integration Points

### For hyper2kvm

**Option 1: Python Subprocess**
```python
from guestkit_wrapper import GuestkitWrapper
wrapper = GuestkitWrapper()
result = wrapper.convert(source, output, compress=True)
```

**Option 2: PyO3 Native (Recommended)**
```python
import guestkit_py
converter = guestkit_py.DiskConverter()
result = converter.convert(source, output, "qcow2", compress=True)
```

**Option 3: Rust Library**
```rust
use guestkit::DiskConverter;
let result = converter.convert(source, output, "qcow2", true, true)?;
```

## Future Enhancements

### Short-term
- [ ] Async disk operations
- [ ] Progress callbacks
- [ ] More libguestfs wrappers (networking, etc.)
- [ ] Comprehensive benchmarks

### Long-term
- [ ] Cloud integration (AWS, Azure, GCP)
- [ ] Distributed operations
- [ ] Web UI
- [ ] gRPC API

## Design Principles

1. **Safety** - Leverage Rust's type system and ownership
2. **Zero-cost** - Abstractions with no runtime overhead
3. **Correctness** - Extensive testing and type checking
4. **Usability** - Ergonomic high-level APIs
5. **Performance** - Async, parallel, optimized
6. **Compatibility** - Works with existing tools
7. **Maintainability** - Auto-generated, well-documented

## References

- **libguestfs docs**: https://libguestfs.org
- **bindgen docs**: https://rust-lang.github.io/rust-bindgen/
- **PyO3 docs**: https://pyo3.rs
- **Rust async book**: https://rust-lang.github.io/async-book/

---

**Version:** 0.1.0
**License:** LGPL-3.0-or-later
**Author:** Susant Sahani <ssahani@gmail.com>
