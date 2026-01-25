## Ergonomic, Type-Safe Rust API for GuestCtl

This document describes the improved GuestCtl API that leverages Rust's type system and ergonomic patterns to provide a safer, more pleasant development experience.

## Design Philosophy

###  **Why Improve?**

The original  C API was designed in 2009 for C programs. While functional, it doesn't take advantage of modern Rust features:

**Problems with C-style API:**
- ❌ String-based everything (easy to make typos)
- ❌ No compile-time validation
- ❌ Verbose configuration
- ❌ No IDE support for options
- ❌ Easy to forget required steps

**Modern Rust API solves this:**
- ✅ Type-safe enums instead of strings
- ✅ Builder patterns for configuration
- ✅ Fluent interfaces for complex operations
- ✅ Compile-time validation
- ✅ Excellent IDE support
- ✅ Hard to misuse

## Key Improvements

### 1. Builder Pattern for Guest Creation

**Old API** (manual configuration):
```rust
let mut g = Guestfs::new()?;
g.set_verbose(true)?;
g.set_trace(true)?;
g.add_drive("/path/to/disk.img")?;
g.add_drive_ro("/path/to/template.img")?;
g.launch()?;
```

**Fluent API** (fluent builder):
```rust
let mut guest = Guestfs::builder()
    .verbose(true)
    .trace(true)
    .add_drive("/path/to/disk.img")
    .add_drive_ro("/path/to/template.img")
    .build_and_launch()?;
```

**Benefits:**
- Shorter, more readable code
- Method chaining
- Can't forget to call `launch()`
- IDE autocomplete guides you

### 2. Type-Safe Filesystem Types

**Old API** (string-based):
```rust
// Easy to make typos!
g.mkfs("ext4", "/dev/sda1", Some(4096), Some("rootfs"), None, None)?;
g.mkfs("etx4", "/dev/sda2", None, None, None, None)?;  // Typo! Won't catch until runtime
```

**Fluent API** (enum-based):
```rust
use guestctl::guestfs::FilesystemType;

// Type-safe - compiler catches typos!
g.mkfs("/dev/sda1")
    .ext4()
    .blocksize(4096)
    .label("rootfs")
    .create()?;

// This won't compile:
// g.mkfs("/dev/sda2").etx4()  // Compile error!
```

**Available Filesystem Types:**
```rust
pub enum FilesystemType {
    Ext2, Ext3, Ext4,    // ext family
    Xfs, Btrfs,          // Modern Linux filesystems
    Vfat, Ntfs, Exfat,   // FAT/Windows filesystems
    F2fs,                // Flash-optimized
    Jfs, Reiserfs,       // Legacy filesystems
    Minix,               // Classic Unix
}
```

**Smart Features:**
```rust
let fs = FilesystemType::Ext4;
println!("Supports labels: {}", fs.supports_labels());  // true
println!("Supports UUID: {}", fs.supports_uuid());      // true

let vfat = FilesystemType::Vfat;
println!("Supports labels: {}", vfat.supports_labels());  // false!
```

### 3. Fluent Filesystem Operations

**Old API** (positional parameters):
```rust
// What does each parameter mean? Have to check docs!
g.mkfs("btrfs", "/dev/sda2", None, Some("data"), None, None)?;
```

**Fluent API** (named builder methods):
```rust
// Self-documenting, clear intent
g.mkfs("/dev/sda2")
    .btrfs()              // Filesystem type
    .label("data")        // Filesystem label
    .create()?;           // Execute

// Or use generic fstype:
g.mkfs("/dev/sda1")
    .fstype(FilesystemType::Xfs)
    .label("boot")
    .blocksize(4096)
    .create()?;
```

### 4. Fluent Mount Operations

**Old API** (comma-separated option strings):
```rust
// Options are just strings - no validation
g.mount("/dev/sda1", "/", Some("subvol=@,compress=zstd,ro"))?;
```

**Fluent API** (builder with methods):
```rust
// Type-safe, self-documenting
g.mount_with("/dev/sda1", "/")
    .subvolume("@")          // Clear what this does
    .compress("zstd")        // Clear option
    .readonly()              // Boolean flag, not string
    .perform()?;

// Each option is a method - IDE autocompletes!
```

### 5. Type-Safe OS Detection

**Old API** (string matching):
```rust
let roots = g.inspect_os()?;
for root in &roots {
    let ostype = g.inspect_get_type(root)?;
    if ostype == "linux" {  // String comparison
        let distro = g.inspect_get_distro(root)?;
        if distro == "ubuntu" || distro == "debian" {  // More strings
            // Handle Debian-based
        }
    }
}
```

**Fluent API** (enums):
```rust
use guestctl::guestfs::{OsType, Distro, PackageManager};

let roots = g.inspect_os()?;
for root in &roots {
    let ostype = OsType::from_str(&g.inspect_get_type(root)?);

    match ostype {
        OsType::Linux => {
            let distro = Distro::from_str(&g.inspect_get_distro(root)?);

            match distro {
                Distro::Ubuntu | Distro::Debian => {
                    // Compile-time exhaustiveness checking!
                    let pkg_mgr = distro.package_manager();  // Returns PackageManager::Dpkg
                }
                Distro::Fedora | Distro::Rhel => {
                    // Handle RPM-based
                }
                _ => {}
            }
        }
        OsType::Windows => {
            // Handle Windows
        }
        _ => {}
    }
}
```

**Smart Distro Features:**
```rust
let distro = Distro::Ubuntu;
println!("Package manager: {:?}", distro.package_manager());  // Some(Dpkg)

let arch = Distro::Archlinux;
println!("Package manager: {:?}", arch.package_manager());    // Some(Pacman)
```

### 6. Partition Table Types

**Old API**:
```rust
g.part_init("/dev/sda", "gpt")?;  // String - could type "GPT", "Gpt", etc.
```

**Fluent API**:
```rust
use guestctl::guestfs::PartitionTableType;

g.part_init("/dev/sda", PartitionTableType::Gpt.as_str())?;
// Or:
g.part_init("/dev/sda", PartitionTableType::Mbr.as_str())?;
```

## Migration Guide

### Step 1: Update Imports

**Old:**
```rust
use guestctl::Guestfs;
```

**New:**
```rust
use guestctl::guestfs::{
    Guestfs,              // Main handle
    FilesystemType,       // Type-safe filesystems
    PartitionTableType,   // Type-safe partition tables
    OsType, Distro,       // Type-safe OS detection
};
```

### Step 2: Convert to Builder Pattern

**Old:**
```rust
let mut g = Guestfs::new()?;
g.add_drive("disk.img")?;
g.set_verbose(true)?;
g.launch()?;
```

**New:**
```rust
let mut g = Guestfs::builder()
    .add_drive("disk.img")
    .verbose(true)
    .build_and_launch()?;
```

### Step 3: Use Type-Safe Filesystem Operations

**Old:**
```rust
g.mkfs("ext4", "/dev/sda1", Some(4096), Some("rootfs"), None, None)?;
```

**New:**
```rust
g.mkfs("/dev/sda1")
    .ext4()
    .blocksize(4096)
    .label("rootfs")
    .create()?;
```

### Step 4: Use Fluent Mount API

**Old:**
```rust
g.mount("/dev/sda1", "/", Some("subvol=@,compress=zstd"))?;
```

**New:**
```rust
g.mount_with("/dev/sda1", "/")
    .subvolume("@")
    .compress("zstd")
    .perform()?;
```

### Step 5: Use Type-Safe Enums for OS Detection

**Old:**
```rust
if g.inspect_get_distro(root)? == "ubuntu" {
    // ...
}
```

**New:**
```rust
let distro = Distro::from_str(&g.inspect_get_distro(root)?);
if distro == Distro::Ubuntu {
    // Compile-time checked!
}

// Or use match for exhaustiveness:
match distro {
    Distro::Ubuntu => { /* ... */ }
    Distro::Debian => { /* ... */ }
    Distro::Fedora => { /* ... */ }
    _ => { /* ... */ }
}
```

## Backward Compatibility

**Important:** The old API still works! You can mix old and new styles:

```rust
let mut g = Guestfs::builder()   // New builder
    .add_drive("disk.img")
    .build()?;

g.launch()?;                      // Old style

g.mkfs("/dev/sda1")               // New fluent API
    .ext4()
    .create()?;

g.mount("/dev/sda1", "/", None)?; // Old API still works
```

This allows gradual migration without breaking existing code.

## Best Practices

### 1. Use Builder Pattern for Configuration

```rust
// ✅ Good - clear, concise
let guest = Guestfs::builder()
    .add_drive("disk.img")
    .verbose(true)
    .build()?;

// ❌ Avoid - verbose, easy to forget steps
let mut g = Guestfs::new()?;
g.add_drive("disk.img")?;
g.set_verbose(true)?;
```

### 2. Use Type-Safe Enums

```rust
// ✅ Good - compile-time checked
g.mkfs("/dev/sda1").ext4().create()?;

// ❌ Avoid - runtime errors possible
g.mkfs("ext4", "/dev/sda1", None, None, None, None)?;
```

### 3. Use Fluent APIs for Complex Operations

```rust
// ✅ Good - self-documenting
g.mount_with("/dev/sda1", "/")
    .subvolume("@")
    .compress("zstd")
    .readonly()
    .perform()?;

// ❌ Avoid - hard to understand
g.mount("/dev/sda1", "/", Some("subvol=@,compress=zstd,ro"))?;
```

### 4. Use Match for Exhaustive Handling

```rust
// ✅ Good - compiler ensures all cases handled
match distro {
    Distro::Ubuntu | Distro::Debian => { /* dpkg */ }
    Distro::Fedora | Distro::Rhel => { /* rpm */ }
    Distro::Archlinux => { /* pacman */ }
    _ => { /* others */ }
}

// ❌ Avoid - can miss cases
if distro == Distro::Ubuntu || distro == Distro::Debian {
    // What about Arch? Fedora? Easy to forget!
}
```

## Examples

See the comprehensive examples in:
- `examples/modern_api.rs` - Fluent API showcase
- `examples/migration_guide.rs` - Step-by-step migration

Run them with:
```bash
cargo run --example modern_api
```

## Future Enhancements

Planned improvements for future versions:

1. **Async/Await Support**
   ```rust
   let guest = Guestfs::builder()
       .add_drive("disk.img")
       .build_async()
       .await?;
   ```

2. **RAII Mount Guards**
   ```rust
   {
       let _mount = guest.mount_scoped("/dev/sda1", "/")?;
       // Auto-unmount when _mount drops
   }
   ```

3. **Iterator-Based File Listing**
   ```rust
   for entry in guest.list_dir("/etc")? {
       println!("{}", entry);
   }
   ```

4. **Trait-Based Extensibility**
   ```rust
   trait FilesystemOps {
       fn create_fs(&mut self) -> Result<()>;
   }
   ```

## Conclusion

The modern GuestCtl API provides:

- ✅ **Type Safety**: Catch errors at compile time
- ✅ **Ergonomics**: Fluent, self-documenting APIs
- ✅ **IDE Support**: Excellent autocomplete and documentation
- ✅ **Backward Compatible**: Old API still works
- ✅ **Gradual Migration**: Adopt new features incrementally

Start using the ergonomic API today for a better development experience!
