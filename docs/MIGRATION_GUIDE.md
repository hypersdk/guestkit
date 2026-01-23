# Migration Guide: Adopting the Ergonomic API

This guide shows how to migrate from the old string-based API to the new type-safe, fluent API.

## Quick Start: Most Common Migrations

### 1. Guest Creation and Launch

**Before:**
```rust
let mut g = Guestfs::new()?;
g.add_drive("/path/to/disk.img")?;
g.add_drive_ro("/path/to/template.img")?;
g.set_verbose(true)?;
g.launch()?;
```

**After:**
```rust
let mut g = Guestfs::builder()
    .add_drive("/path/to/disk.img")
    .add_drive_ro("/path/to/template.img")
    .verbose(true)
    .build_and_launch()?;
```

**Benefits:**
- ✅ Fewer lines of code
- ✅ Can't forget to call `launch()`
- ✅ Method chaining is more readable
- ✅ All configuration in one place

---

### 2. Creating Filesystems

**Before:**
```rust
g.mkfs("ext4", "/dev/sda1", Some(4096), Some("rootfs"), None, None)?;
g.mkfs("vfat", "/dev/sda2", None, Some("EFI"), None, None)?;
g.mkfs("xfs", "/dev/sda3", None, Some("data"), None, None)?;
```

**After:**
```rust
use guestkit::guestfs::FilesystemType;

g.mkfs("/dev/sda1")
    .ext4()
    .blocksize(4096)
    .label("rootfs")
    .create()?;

g.mkfs("/dev/sda2")
    .vfat()
    .label("EFI")
    .create()?;

g.mkfs("/dev/sda3")
    .xfs()
    .label("data")
    .create()?;
```

**Benefits:**
- ✅ Type-safe filesystem selection (no typos!)
- ✅ Self-documenting code
- ✅ Optional parameters are clearly named
- ✅ IDE autocomplete shows available options

---

### 3. Partition Table Creation

**Before:**
```rust
g.part_init("/dev/sda", "gpt")?;  // Could typo as "GPT", "Gpt", etc.
```

**After:**
```rust
use guestkit::guestfs::PartitionTableType;

g.part_init("/dev/sda", PartitionTableType::Gpt.as_str())?;
```

**Benefits:**
- ✅ Compile-time validation
- ✅ No string typos
- ✅ IDE shows available options (Gpt, Mbr)

---

### 4. OS Detection with Type Safety

**Before:**
```rust
let roots = g.inspect_os()?;
for root in &roots {
    let ostype = g.inspect_get_type(root)?;
    if ostype == "linux" {
        let distro = g.inspect_get_distro(root)?;
        if distro == "ubuntu" || distro == "debian" {
            // Handle Debian-based
        } else if distro == "fedora" || distro == "rhel" {
            // Handle Red Hat-based
        }
    }
}
```

**After:**
```rust
use guestkit::guestfs::{OsType, Distro};

let roots = g.inspect_os()?;
for root in &roots {
    let ostype = OsType::from_str(&g.inspect_get_type(root)?);

    match ostype {
        OsType::Linux => {
            let distro = Distro::from_str(&g.inspect_get_distro(root)?);

            match distro {
                Distro::Ubuntu | Distro::Debian => {
                    let pkg_mgr = distro.package_manager(); // PackageManager::Dpkg
                    // Handle Debian-based
                }
                Distro::Fedora | Distro::Rhel | Distro::CentOs => {
                    let pkg_mgr = distro.package_manager(); // PackageManager::Rpm
                    // Handle Red Hat-based
                }
                Distro::Archlinux => {
                    let pkg_mgr = distro.package_manager(); // PackageManager::Pacman
                    // Handle Arch
                }
                _ => {
                    // Compiler ensures we handle all cases!
                }
            }
        }
        OsType::Windows => {
            // Handle Windows
        }
        _ => {}
    }
}
```

**Benefits:**
- ✅ Pattern matching with exhaustiveness checking
- ✅ Compiler catches missing cases
- ✅ Smart methods like `distro.package_manager()`
- ✅ No string comparison errors

---

## Advanced Migrations

### 5. Mount with Options (Planned - Future)

**Before:**
```rust
g.mount("/dev/sda1", "/", Some("subvol=@,compress=zstd,ro"))?;
```

**After (Future API):**
```rust
g.mount_with("/dev/sda1", "/")
    .subvolume("@")
    .compress("zstd")
    .readonly()
    .perform()?;
```

**Note:** The fluent mount API is planned but not yet implemented. Use the old API for now.

---

## Type Reference

### Available Filesystem Types

```rust
pub enum FilesystemType {
    Ext2,       // ext2 filesystem
    Ext3,       // ext3 filesystem
    Ext4,       // ext4 filesystem (most common)
    Xfs,        // XFS filesystem
    Btrfs,      // BTRFS filesystem
    Vfat,       // VFAT/FAT32
    Ntfs,       // NTFS (Windows)
    Exfat,      // exFAT
    F2fs,       // Flash-optimized
    Jfs,        // IBM JFS
    Reiserfs,   // ReiserFS
    Minix,      // MINIX filesystem
}

// Usage:
FilesystemType::Ext4.as_str()           // "ext4"
FilesystemType::Ext4.supports_labels()  // true
FilesystemType::Ext4.supports_uuid()    // true
```

### Available OS Types

```rust
pub enum OsType {
    Linux,
    Windows,
    Hurd,
    FreeBsd,
    NetBsd,
    OpenBsd,
    Minix,
    Unknown,
}

// Usage:
OsType::from_str("linux")  // OsType::Linux
```

### Available Distributions

```rust
pub enum Distro {
    Ubuntu,
    Debian,
    Fedora,
    Rhel,
    CentOs,
    Archlinux,
    Gentoo,
    Opensuse,
    Suse,
    Alpine,
    Void,
    Nixos,
    Unknown,
}

// Usage:
let distro = Distro::Ubuntu;
distro.package_manager()  // Some(PackageManager::Dpkg)
```

### Available Package Managers

```rust
pub enum PackageManager {
    Dpkg,     // Debian/Ubuntu
    Rpm,      // Red Hat/Fedora/SUSE
    Pacman,   // Arch Linux
    Portage,  // Gentoo
    Apk,      // Alpine
    Nix,      // NixOS
    Xbps,     // Void Linux
}
```

---

## Migration Checklist

### Phase 1: Low-Hanging Fruit
- [ ] Replace `Guestfs::new()` → `Guestfs::builder()`
- [ ] Add imports for `FilesystemType`, `PartitionTableType`
- [ ] Replace string literals in `mkfs()` calls
- [ ] Replace string literals in `part_init()` calls

### Phase 2: OS Detection
- [ ] Add imports for `OsType`, `Distro`, `PackageManager`
- [ ] Replace string comparisons with enum matching
- [ ] Use `.package_manager()` instead of manual mapping

### Phase 3: Testing
- [ ] Run tests to ensure no regressions
- [ ] Check for compile-time errors (good - they catch bugs!)
- [ ] Update error handling if needed

---

## Examples to Study

See these examples for complete usage patterns:

1. **`examples/fluent_api.rs`** - Showcase of all new patterns
2. **`examples/create_disk_fluent.rs`** - Disk creation with builders
3. **`examples/inspect_os_typed.rs`** - Type-safe OS detection
4. **`examples/system_info.rs`** - Updated to use builder pattern

Run them with:
```bash
cargo run --example fluent_api
cargo run --example inspect_os_typed <disk-image>
```

---

## Backward Compatibility

**Important:** The old API still works! You can:

1. **Mix old and new styles** in the same code
2. **Migrate incrementally** - no need to update everything at once
3. **Keep existing code working** while adopting new features

Example of mixing:
```rust
// New builder
let mut g = Guestfs::builder()
    .add_drive("disk.img")
    .build()?;

g.launch()?;  // Old style still works

// New fluent API
g.mkfs("/dev/sda1").ext4().create()?;

// Old API still works
g.mount("/dev/sda1", "/", None)?;
```

---

## Common Questions

### Q: Do I have to migrate everything?
**A:** No! The old API still works. Migrate when convenient.

### Q: Will this make my code slower?
**A:** No! These are zero-cost abstractions that compile away.

### Q: What if I find a bug in the new API?
**A:** You can always fallback to the old API. Report the issue on GitHub.

### Q: Can I use this in production?
**A:** Yes! The new API is built on top of the existing, tested code.

### Q: Will more fluent APIs be added?
**A:** Yes! Planned additions include mount options, BTRFS subvolumes, and more.

---

## Need Help?

- Check `docs/ERGONOMIC_API.md` for complete API documentation
- See `examples/` directory for working code
- File an issue on GitHub for questions or problems

---

## Summary

The new ergonomic API provides:

✅ **Type safety** - Catch errors at compile time
✅ **Better IDE support** - Full autocomplete
✅ **Self-documenting code** - Clear intent
✅ **Zero runtime cost** - Just as fast
✅ **Backward compatible** - Old code still works
✅ **Incremental adoption** - Migrate at your own pace

Start with the builder pattern and type-safe filesystems - you'll immediately see the benefits!
