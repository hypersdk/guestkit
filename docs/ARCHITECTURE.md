# Architecture Documentation

This document provides a deep dive into the guestkit architecture, design decisions, and internal workings.

## Table of Contents

- [Overview](#overview)
- [Core Concepts](#core-concepts)
- [Module Architecture](#module-architecture)
- [Data Flow](#data-flow)
- [Design Decisions](#design-decisions)
- [Comparison with libguestfs](#comparison-with-libguestfs)
- [Future Architecture](#future-architecture)

---

## Overview

guestkit is a pure Rust implementation of guestfs APIs for disk image inspection and manipulation. It follows a layered architecture designed for:

- **Safety**: Memory-safe Rust prevents common vulnerabilities
- **Performance**: Zero-cost abstractions and efficient I/O
- **Modularity**: Clear separation of concerns
- **Extensibility**: Easy to add new operations

### High-Level Architecture

```
┌─────────────────────────────────────────────────────┐
│                  CLI / Applications                 │
│              (src/main.rs, examples/)               │
└──────────────────────┬──────────────────────────────┘
                       │
┌──────────────────────▼──────────────────────────────┐
│              Public API (src/lib.rs)                │
│         Guestfs, DiskConverter, GuestDetector       │
└──────────┬────────────────────────────┬─────────────┘
           │                            │
           │                            │
┌──────────▼────────┐      ┌───────────▼────────────┐
│   GuestFS Module  │      │   Converters Module    │
│   (src/guestfs/)  │      │  (src/converters/)     │
│                   │      │                        │
│  84 operation     │      │  Format conversion     │
│  modules          │      │  qemu-img wrapper      │
└──────────┬────────┘      └────────────────────────┘
           │
┌──────────▼────────────────────────────────────────┐
│           Disk Access Layer (src/disk/)           │
│  DiskReader, PartitionTable, FileSystem parsing   │
└──────────┬────────────────────────────────────────┘
           │
┌──────────▼────────────────────────────────────────┐
│        Core Layer (src/core/)                     │
│  Error types, Retry logic, Common types          │
└──────────┬────────────────────────────────────────┘
           │
┌──────────▼────────────────────────────────────────┐
│           External Dependencies                    │
│  qemu-img, qemu-nbd, cryptsetup, lvm2            │
└───────────────────────────────────────────────────┘
```

---

## Core Concepts

### 1. The Guestfs Handle

The central abstraction is the `Guestfs` struct:

```rust
pub struct Guestfs {
    // Disk access
    drives: Vec<DriveConfig>,
    reader: Option<DiskReader>,

    // NBD state
    nbd_device: Option<String>,
    nbd_pid: Option<u32>,

    // Mount state
    mountpoints: HashMap<String, String>,
    mount_root: Option<String>,

    // Configuration
    verbose: bool,
    trace: bool,

    // Lifecycle state
    launched: bool,
}
```

**Lifecycle:**

```
new() → add_drive*() → launch() → operations... → shutdown()
```

### 2. Lazy Initialization

Guestfs uses lazy initialization:

```rust
impl Guestfs {
    pub fn new() -> Result<Self> {
        // Minimal initialization
        Ok(Self {
            drives: Vec::new(),
            launched: false,
            // ... minimal setup
        })
    }

    pub fn launch(&mut self) -> Result<()> {
        if self.launched {
            return Ok(());  // Idempotent
        }

        // Heavy initialization happens here:
        // 1. Analyze disk images
        // 2. Setup NBD
        // 3. Scan partitions/filesystems

        self.launched = true;
        Ok(())
    }
}
```

**Benefits:**
- Fast handle creation
- Deferred resource allocation
- Better error handling (errors happen at launch, not construction)

### 3. Error Handling Strategy

Comprehensive error types:

```rust
#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Command failed: {0}")]
    CommandFailed(String),

    #[error("Not ready: {0}")]
    NotReady(String),

    #[error("Filesystem error: {0}")]
    FileSystem(String),

    #[error("Not implemented: {0}")]
    NotImplemented(String),

    // ... more variants
}
```

**Strategy:**
- Use `Result<T, Error>` for all fallible operations
- Provide context with error messages
- Allow caller to handle or propagate with `?`

---

## Module Architecture

### Core Module (`src/core/`)

**Purpose:** Foundation types and utilities

**Components:**
- `error.rs` - Error types and Result alias
- `retry.rs` - Exponential backoff retry logic
- `types.rs` - Common types (StatInfo, PartitionInfo, etc.)

**Example - Retry Logic:**

```rust
pub struct RetryConfig {
    pub max_attempts: u32,
    pub initial_delay_ms: u64,
    pub max_delay_ms: u64,
    pub backoff_factor: f64,
}

impl RetryConfig {
    pub fn retry<F, T>(&self, mut f: F) -> Result<T>
    where
        F: FnMut() -> Result<T>,
    {
        let mut delay = self.initial_delay_ms;

        for attempt in 1..=self.max_attempts {
            match f() {
                Ok(val) => return Ok(val),
                Err(e) if attempt == self.max_attempts => return Err(e),
                Err(_) => {
                    thread::sleep(Duration::from_millis(delay));
                    delay = (delay as f64 * self.backoff_factor) as u64;
                    delay = delay.min(self.max_delay_ms);
                }
            }
        }

        unreachable!()
    }
}
```

### Disk Module (`src/disk/`)

**Purpose:** Low-level disk access and parsing

**Components:**
- `reader.rs` - DiskReader for reading disk images
- `format.rs` - Disk format detection (QCOW2, VMDK, RAW)
- `partition.rs` - Partition table parsing (MBR, GPT)
- `filesystem.rs` - Filesystem detection

**Architecture:**

```rust
pub struct DiskReader {
    file: File,
    mmap: Option<Mmap>,
    format: DiskFormat,
}

impl DiskReader {
    pub fn open(path: &Path) -> Result<Self> {
        let file = File::open(path)?;
        let format = detect_format(&file)?;

        Ok(Self {
            file,
            mmap: None,
            format,
        })
    }

    pub fn read_at(&mut self, offset: u64, size: usize) -> Result<Vec<u8>> {
        // Handle different formats
        match self.format {
            DiskFormat::Raw => self.read_raw(offset, size),
            DiskFormat::Qcow2 => self.read_qcow2(offset, size),
            // ... other formats
        }
    }
}
```

**Design Decision:** Use memory-mapped I/O for large files:

```rust
// Memory-map for efficient random access
if file_size > MMAP_THRESHOLD {
    self.mmap = Some(unsafe { Mmap::map(&self.file)? });
}
```

### GuestFS Module (`src/guestfs/`)

**Purpose:** guestfs API implementation

**Structure:** 84 specialized modules, each focused on specific functionality:

```
src/guestfs/
├── handle.rs        # Core Guestfs struct
├── mount.rs         # Mount operations
├── file_ops.rs      # File I/O
├── archive.rs       # Tar/cpio operations
├── luks.rs          # LUKS encryption
├── lvm.rs           # LVM management
├── filesystem.rs    # Filesystem operations
├── partition.rs     # Partition management
├── ... (76 more modules)
└── mod.rs           # Module exports
```

**Design Pattern:** Extension trait pattern

```rust
// Each module extends Guestfs
impl Guestfs {
    pub fn mount(&mut self, device: &str, mountpoint: &str) -> Result<()> {
        self.ensure_ready()?;  // Check state

        // Implementation
        // ...

        self.mountpoints.insert(mountpoint.to_string(), device.to_string());
        Ok(())
    }
}
```

### CLI Module (`src/cli/`)

**Purpose:** Command-line interface

**Components:**
- `commands.rs` - Command implementations
- `output.rs` - Formatting utilities

**Architecture:**

```rust
// Separate command logic from CLI parsing
pub fn inspect_image(image: &PathBuf, verbose: bool) -> Result<()> {
    let mut g = Guestfs::new()?;
    g.set_verbose(verbose);

    // Implementation
    // ...
}

// CLI layer just parses and dispatches
match cli.command {
    Commands::Inspect { image } => {
        inspect_image(&image, cli.verbose)?;
    }
    // ...
}
```

---

## Data Flow

### Typical Operation Flow

Example: Reading a file from a disk image

```
1. User calls: g.cat("/etc/passwd")
   ↓
2. cat() checks: ensure_ready()?
   ↓
3. cat() resolves path: resolve_guest_path("/etc/passwd")?
   ↓
4. cat() uses mount point: mount_root + "/etc/passwd"
   ↓
5. cat() reads file via NBD: std::fs::read(host_path)?
   ↓
6. cat() returns: String (UTF-8 converted)
```

### Mount Operation Flow

```
1. User calls: g.mount("/dev/sda1", "/")
   ↓
2. mount() connects NBD:
   - Start qemu-nbd process
   - Connect to /dev/nbd0
   ↓
3. mount() creates partitions on NBD:
   - qemu-nbd exposes partitions as /dev/nbd0p1
   ↓
4. mount() mounts to host path:
   - mkdir -p /tmp/guestfs.XXXXX/
   - mount /dev/nbd0p1 /tmp/guestfs.XXXXX/
   ↓
5. mount() records state:
   - mountpoints["/"] = "/dev/sda1"
   - mount_root = Some("/tmp/guestfs.XXXXX")
```

### Command Execution Flow

```
1. User calls: g.command(&["ls", "-la", "/"])
   ↓
2. command() prepares chroot:
   - mount_root must exist
   - Construct full command
   ↓
3. command() executes via chroot:
   - chroot /tmp/guestfs.XXXXX ls -la /
   ↓
4. command() captures output:
   - stdout → return value
   - stderr → error if non-zero exit
```

---

## Design Decisions

### Why Pure Rust?

**Advantages:**
- Memory safety without garbage collection
- No C/FFI overhead for core operations
- Better integration with Rust ecosystem
- Modern error handling
- Fearless concurrency

**Trade-offs:**
- Some operations still call external tools (qemu-img, cryptsetup)
- Not 100% API 
- Smaller ecosystem than C

### Why Not FFI to libguestfs?

Considered but rejected:
- Complex C API with manual memory management
- Difficult to provide safe Rust wrappers
- FFI overhead
- Loss of Rust's safety guarantees

### NBD vs. Direct Filesystem Parsing

**Current:** Use qemu-nbd for mounting

**Advantages:**
- Leverages kernel filesystem drivers
- Supports all Linux filesystems
- Mature and well-tested

**Disadvantages:**
- Requires elevated privileges
- Platform-dependent (Linux-only)
- External dependency

**Future:** Consider pure Rust filesystem parsing for common formats (ext4, NTFS)

### Module Organization

**Decision:** 84 small, focused modules vs. few large modules

**Rationale:**
- Clear separation of concerns
- Easy to find specific functionality
- Parallel development
- Independent testing
- Optional features possible

**Trade-off:** More files, but better organization

### Error Handling Philosophy

**Decision:** Return Result, not panic

**Rationale:**
- Library code should never panic
- Caller decides how to handle errors
- Composable with `?` operator
- Clear error types

```rust
// Good
pub fn mount(&mut self, device: &str, mp: &str) -> Result<()> {
    if !self.launched {
        return Err(Error::NotReady("Must call launch() first".into()));
    }
    // ...
}

// Bad
pub fn mount(&mut self, device: &str, mp: &str) {
    assert!(self.launched, "Must call launch() first");  // ✗ Panics!
}
```

---

## Comparison with libguestfs

### Similarities

- API names and semantics
- Operation categories
- Workflow patterns
- Use cases

### Differences

| Aspect | libguestfs | guestkit |
|--------|-----------|----------|
| **Language** | C | Rust |
| **Memory Safety** | Manual | Automatic |
| **Backend** | Custom daemon | qemu-nbd + commands |
| **API Coverage** | 733 functions | 578 functions (76.8%) |
| **Platform** | Linux, Windows, macOS | Linux (future: multi-platform) |
| **Dependencies** | Large (libvirt, etc.) | Minimal (qemu-img, nbd) |
| **Bindings** | Many languages | Rust native, Python planned |

### Performance Comparison

**guestkit advantages:**
- Faster for simple operations (less overhead)
- Better memory usage (Rust's ownership)
- Parallel operations safe by default

**libguestfs advantages:**
- More mature optimizations
- More format support
- Broader platform support

---

## Future Architecture

### Planned Improvements

#### 1. Async I/O

```rust
// Future API
pub async fn cat_async(&mut self, path: &str) -> Result<String> {
    // Async file I/O
    tokio::fs::read_to_string(self.resolve_guest_path(path)?).await
}
```

#### 2. Pure Rust Filesystem Parsing

```rust
// Future: No qemu-nbd dependency
pub struct Ext4Parser {
    disk: DiskReader,
    superblock: Ext4Superblock,
}

impl Ext4Parser {
    pub fn read_file(&mut self, inode: u32) -> Result<Vec<u8>> {
        // Direct ext4 parsing
    }
}
```

#### 3. Plugin Architecture

```rust
// Future: Extensible operations
pub trait GuestfsPlugin {
    fn name(&self) -> &str;
    fn operations(&self) -> Vec<Operation>;
}

impl Guestfs {
    pub fn register_plugin(&mut self, plugin: Box<dyn GuestfsPlugin>) {
        // Dynamic operation loading
    }
}
```

#### 4. Remote Operations

```rust
// Future: Remote disk access
pub struct RemoteDisk {
    url: String,  // http://server/disk.img
    auth: Option<Auth>,
}

impl Guestfs {
    pub fn add_remote_drive(&mut self, disk: RemoteDisk) -> Result<()> {
        // Download or stream remote disk
    }
}
```

### Roadmap Phases

See [ROADMAP.md](../ROADMAP.md) for detailed timeline:

- **Phase 3** (Q1 2026): Stabilization ← Current
- **Phase 4** (Q2 2026): Python bindings
- **Phase 5** (Q2-Q3 2026): Performance optimization
- **Phase 6** (Q3-Q4 2026): Advanced features
- **Phase 7** (2027): Ecosystem integration

---

## Implementation Guidelines

When adding new operations:

### 1. Follow the Pattern

```rust
impl Guestfs {
    /// Brief description
    ///
    /// Detailed documentation
    ///
    /// # Arguments
    /// * `param` - Parameter description
    ///
    /// # Returns
    /// Return value description
    ///
    /// # Errors
    /// Error conditions
    ///
    /// # Example
    /// ```no_run
    /// # use guestkit::Guestfs;
    /// let mut g = Guestfs::new()?;
    /// g.operation(param)?;
    /// # Ok::<(), Box<dyn std::error::Error>>(())
    /// ```
    pub fn operation(&mut self, param: &str) -> Result<ReturnType> {
        self.ensure_ready()?;  // ← Always check state

        if self.verbose {
            eprintln!("guestfs: operation {}", param);
        }

        // Implementation

        Ok(result)
    }
}
```

### 2. Write Tests

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_operation() {
        let mut g = Guestfs::new().unwrap();
        // Test implementation
    }
}
```

### 3. Update Documentation

- Add to `GUESTFS_IMPLEMENTATION_STATUS.md`
- Update `API_REFERENCE.md` if major feature
- Add example if complex

---

## Additional Resources

- [Contributing Guide](../CONTRIBUTING.md)
- [API Reference](../API_REFERENCE.md)
- [Performance Guide](PERFORMANCE.md)
- [Troubleshooting Guide](TROUBLESHOOTING.md)
