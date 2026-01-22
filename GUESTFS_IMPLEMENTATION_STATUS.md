# GuestFS Implementation Status

This document outlines the implementation status of libguestfs-compatible APIs in guestkit.

## Summary

**guestkit** provides a **pure Rust** implementation of libguestfs-compatible APIs. The implementation is structured in layers:

1. **Low-level disk access** (✅ COMPLETE) - Read disk images, parse partitions, detect filesystems
2. **High-level inspection** (🚧 PARTIAL) - OS detection, filesystem properties
3. **Mount operations** (⚠️ API ONLY) - Mount/unmount APIs defined but not functional
4. **File operations** (⚠️ API ONLY) - File I/O APIs defined but not functional
5. **LVM/LUKS** (⚠️ API ONLY) - Storage stack APIs defined but not functional

## hyper2kvm Required Functions (from analysis)

Total functions used by hyper2kvm: **80+**

### ✅ IMPLEMENTED (Working)

These functions are fully functional:

**Initialization/Management:**
- `Guestfs::new()` - Create GuestFS instance
- `add_drive_opts()` - Add disk with format and readonly options
- `add_drive_ro()` - Add read-only disk
- `launch()` - Initialize handle
- `close()` / `shutdown()` - Clean up
- `set_trace()` / `get_trace()` - Tracing
- `set_verbose()` / `get_verbose()` - Verbose output

**Device/Partition Operations:**
- `list_devices()` - List block devices
- `list_partitions()` - List partitions
- `list_filesystems()` - List filesystems with types
- `part_list()` - Get partition table details
- `part_get_parttype()` - Get partition table type (mbr/gpt)
- `part_get_bootable()` - Get bootable flag
- `part_get_mbr_id()` - Get MBR partition type ID
- `part_to_dev()` - Convert partition to device
- `part_to_partnum()` - Get partition number

**Filesystem Properties:**
- `vfs_type()` - Get filesystem type
- `vfs_label()` - Get filesystem label
- `vfs_uuid()` - Get filesystem UUID

**Device Properties:**
- `blockdev_getsize64()` - Get device size in bytes
- `blockdev_getsz()` - Get device size in sectors
- `canonical_device_name()` - Normalize device names
- `device_index()` - Get device index
- `is_whole_device()` - Check if whole device vs partition

**Basic Inspection:**
- `inspect_os()` - Detect operating systems
- `inspect_get_type()` - Get OS type (linux/windows/etc)
- `inspect_get_distro()` - Get distribution name
- `inspect_get_product_name()` - Get product name
- `inspect_get_arch()` - Get architecture
- `inspect_get_package_format()` - Get package format (rpm/deb)
- `inspect_get_mountpoints()` - Get mountpoints

### 🚧 PARTIAL IMPLEMENTATION

These APIs exist but have limitations:

**Inspection (needs filesystem reading):**
- `inspect_get_major_version()` - Returns 0 (needs /etc/os-release parser)
- `inspect_get_minor_version()` - Returns 0 (needs /etc/os-release parser)
- `inspect_get_hostname()` - Returns "localhost" (needs /etc/hostname parser)
- `inspect_list_applications()` - Returns empty (needs RPM/dpkg database parser)
- `inspect_is_live()` - Returns false (needs live CD indicators)

### ⚠️ API DEFINED (Not Functional)

These APIs are defined with correct signatures but return errors or stubs:

**Mount Operations:**
- `mount()` - Mount read-write
- `mount_ro()` - Mount read-only
- `mount_options()` - Mount with options
- `mount_vfs()` - Mount with VFS type
- `umount()` - Unmount specific mountpoint
- `umount_all()` - Unmount all
- `mounts()` - Get mounted devices
- `mountpoints()` - Get mountpoints map
- `mkmountpoint()` - Create mountpoint
- `rmmountpoint()` - Remove mountpoint
- `sync()` - Sync filesystems

**File Operations:**
- `is_file()` - Check if file (heuristic only)
- `is_dir()` - Check if directory (heuristic only)
- `exists()` - Check if path exists (heuristic only)
- `read_file()` - Read file content
- `cat()` - Read file as text
- `read_lines()` - Read file as lines
- `write()` - Write file
- `mkdir()` - Create directory
- `mkdir_p()` - Create directory with parents
- `ls()` - List directory
- `ll()` - List with long format
- `stat()` - Get file statistics
- `filesize()` - Get file size
- `rm()` - Remove file
- `rmdir()` - Remove directory
- `touch()` - Touch file
- `chmod()` - Change permissions
- `chown()` - Change ownership
- `realpath()` - Resolve symlinks

**LVM Operations:**
- `vgscan()` - Scan for volume groups
- `lvs()` - List logical volumes
- `lvs_full()` - List LVs with details
- `vgs()` - List volume groups
- `pvs()` - List physical volumes

### ❌ NOT IMPLEMENTED

These functions are used by hyper2kvm but not yet implemented:

**Advanced Mount:**
- `vgchange_activate_all()` - Activate LVM volumes

**LUKS/Encryption:**
- `cryptsetup_open()` - Open LUKS device

**Advanced Operations:**
- `command()` - Execute arbitrary commands

**File Operations (Extended):**
- `cp()` - Copy file
- `mv()` - Move/rename file
- `grep()` - Search file contents
- `find()` - Recursive file search
- `tar_out()` - Create tar archive
- `tar_in()` - Extract tar archive

**Other:**
- `statvfs()` - Get filesystem statistics
- `get_uuid()` / `set_uuid()` - UUID management
- `get_label()` / `set_label()` - Label management

**Advanced Partition:**
- `part_init()` - Initialize partition table
- `part_add()` - Add partition
- `part_del()` - Delete partition
- `part_disk()` - Create single partition
- `part_set_bootable()` - Set bootable flag
- `part_set_mbr_id()` - Set MBR type ID
- `part_set_gpt_type()` - Set GPT type GUID
- `part_get_gpt_type()` - Get GPT type GUID
- `part_resize()` - Resize partition

## Implementation Approaches

### Current Architecture: Pure Rust

**Pros:**
- ✅ No C dependencies
- ✅ Memory safe
- ✅ Cross-platform
- ✅ Easy to build

**Cons:**
- ❌ Complex to implement fully (filesystem parsers needed)
- ❌ Mount operations require kernel support

### Path to Full Implementation

To achieve 100% compatibility with hyper2kvm's needs:

#### Option 1: NBD + Kernel Mount (Recommended for Production)

```rust
// Use qemu-nbd to export qcow2 as block device
// Then use system mount for actual mounting

pub fn mount_ro(&mut self, mountable: &str, mountpoint: &str) -> Result<()> {
    // 1. Export via NBD
    Command::new("qemu-nbd")
        .args(&["-r", "-c", "/dev/nbd0", &self.disk_path])
        .status()?;

    // 2. Mount normally
    Command::new("mount")
        .args(&["-o", "ro", mountable, mountpoint])
        .status()?;

    Ok(())
}
```

**Pros:**
- Leverages kernel filesystems (ext4, NTFS, XFS, etc.)
- Full functionality immediately
- Production-tested code paths

**Cons:**
- Requires root/CAP_SYS_ADMIN
- Platform-specific (Linux)
- External dependencies (qemu-nbd)

#### Option 2: FUSE Filesystems

Use existing FUSE implementations:
- `ext4fuse` for ext2/3/4
- `ntfs-3g` for NTFS
- etc.

```rust
// Mount via FUSE
pub fn mount_ro(&mut self, mountable: &str, mountpoint: &str) -> Result<()> {
    Command::new("ext4fuse")
        .args(&[mountable, mountpoint, "-o", "ro"])
        .status()?;

    Ok(())
}
```

**Pros:**
- No root required
- Userspace implementation
- Cross-platform (FUSE available on Linux, macOS, BSD)

**Cons:**
- External dependencies
- Performance overhead
- Limited filesystem support

#### Option 3: Pure Rust Filesystem Parsers

Implement full filesystem parsers in Rust:

```rust
mod ext4 {
    pub struct Ext4FS {
        superblock: Superblock,
        block_groups: Vec<BlockGroup>,
        // ...
    }

    impl Ext4FS {
        pub fn read_file(&self, path: &str) -> Result<Vec<u8>> {
            // 1. Parse directory tree to find inode
            // 2. Read inode metadata
            // 3. Follow extent tree / block pointers
            // 4. Read data blocks
            // 5. Return file content
        }
    }
}
```

**Pros:**
- Pure Rust, no external dependencies
- Full control
- Cross-platform

**Cons:**
- **Extremely complex** (months/years of work)
- Each filesystem needs complete implementation
- Difficult to maintain compatibility

#### Option 4: Hybrid Approach (Recommended for guestkit)

Combine approaches based on use case:

1. **Inspection APIs**: Pure Rust (✅ Already done)
   - Read superblocks
   - Parse partition tables
   - Detect OS type

2. **File Reading**: Implement minimal parsers for common files
   - `/etc/os-release`
   - `/etc/hostname`
   - `/etc/fstab`
   - etc.

3. **Full Mount**: Delegate to NBD + kernel mount when available

4. **Python Wrapper**: Provide both APIs
   ```python
   # Option 1: Pure Rust inspection (fast, no root)
   identity = guestkit.inspect("/path/to/disk.qcow2")

   # Option 2: Full mount (requires root, full functionality)
   with guestkit.mount("/path/to/disk.qcow2") as mnt:
       content = mnt.read_file("/etc/os-release")
   ```

## Recommendations for hyper2kvm Integration

### Short Term: Use Existing libguestfs

For immediate hyper2kvm use, continue using python3-guestfs:

```python
import guestfs

g = guestfs.GuestFS(python_return_dict=True)
g.add_drive_opts(disk_path, readonly=True, format="qcow2")
g.launch()

roots = g.inspect_os()
# ... existing code works
```

### Medium Term: Hybrid Approach

Use guestkit for inspection (fast, no root), fall back to libguestfs for operations:

```python
import guestkit_py
import guestfs

# Fast inspection with guestkit (no libguestfs needed)
identity = guestkit_py.inspect(disk_path)

# If file operations needed, use libguestfs
if needs_file_operations:
    g = guestfs.GuestFS()
    # ... full operations
```

### Long Term: Full guestkit

Implement NBD-based mounting in guestkit for complete replacement:

```python
import guestkit_py

# All operations through guestkit
with guestkit_py.Guest(disk_path, backend="nbd") as g:
    roots = g.inspect_os()
    content = g.read_file(root, "/etc/os-release")
    g.write_file(root, "/etc/hostname", "newhost")
```

## Current Capabilities Summary

| Feature | Status | Notes |
|---------|--------|-------|
| Disk format detection | ✅ Full | qcow2, raw, vmdk |
| Partition parsing | ✅ Full | MBR, GPT |
| Filesystem detection | ✅ Full | ext4, NTFS, XFS, Btrfs, FAT32 |
| UUID/Label extraction | ✅ Full | From filesystem superblocks |
| OS type detection | ✅ Full | Linux, Windows heuristics |
| Distribution detection | 🚧 Partial | From FS labels (limited) |
| Version detection | ⚠️ Stub | Needs file reading |
| Mount operations | ⚠️ API only | Needs NBD or FUSE |
| File reading | ⚠️ API only | Needs FS parser or mount |
| File writing | ⚠️ API only | Needs FS parser or mount |
| LVM support | ⚠️ API only | Needs LVM metadata parser |
| LUKS support | ❌ None | Needs LUKS header parser |

## Testing Status

- ✅ Unit tests passing (20+ tests)
- ✅ API structure complete
- ⚠️ Integration tests limited (no real disk mounting)
- ⚠️ hyper2kvm compatibility untested

## Next Steps

To achieve full hyper2kvm compatibility:

1. **Implement NBD mounting** (1-2 weeks)
   - Export qcow2 via qemu-nbd
   - System mount for actual FS access
   - Python bindings for mount context manager

2. **Implement common file reading** (1-2 weeks)
   - Parse /etc/os-release
   - Parse /etc/fstab
   - Parse /etc/hostname
   - Parse simple text files

3. **Test with real hyper2kvm workflows** (1 week)
   - Run hyper2kvm test suite
   - Fix compatibility issues
   - Performance benchmarking

4. **Documentation and examples** (1 week)
   - Migration guide from libguestfs
   - Python API documentation
   - Example code for common operations

**Total estimated time for production-ready hyper2kvm integration: 5-6 weeks**

## Conclusion

guestkit provides a **solid foundation** for pure Rust disk inspection. The current implementation handles the most common hyper2kvm use cases:
- OS detection
- Filesystem identification
- Partition analysis

For full compatibility, NBD-based mounting is the recommended approach, providing:
- Complete functionality
- Production reliability
- Reasonable implementation effort

---

**Version:** 0.1.0
**Date:** 2026-01-23
**Status:** Development - Inspection Complete, Operations Planned
