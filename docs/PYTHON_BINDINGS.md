# GuestKit Python Bindings

Complete guide to using GuestKit from Python for virtual machine disk image inspection and manipulation.

## Table of Contents

- [Installation](#installation)
- [Quick Start](#quick-start)
- [API Overview](#api-overview)
- [Complete Examples](#complete-examples)
- [Common Patterns](#common-patterns)
- [API Reference](#api-reference)
- [Comparison with Rust API](#comparison-with-rust-api)
- [Best Practices](#best-practices)
- [Troubleshooting](#troubleshooting)

---

## Installation

### Prerequisites

GuestKit Python bindings require:
- Python 3.6 or later
- libguestfs installed on your system
- Root/sudo access (for most operations)

### Install System Dependencies

**Ubuntu/Debian:**
```bash
sudo apt-get install libguestfs-dev libguestfs-tools python3-guestfs
```

**Fedora/RHEL/CentOS:**
```bash
sudo dnf install libguestfs-devel libguestfs-tools python3-libguestfs
```

**Arch Linux:**
```bash
sudo pacman -S libguestfs python-libguestfs
```

### Install GuestKit Python Bindings

```bash
pip install guestkit
```

Or from source:
```bash
git clone https://github.com/yourusername/guestkit
cd guestkit
pip install .
```

### Verify Installation

```python
from guestkit import Guestfs

g = Guestfs()
print(f"GuestKit version: {g.version()}")
```

---

## Quick Start

Here's a minimal example that inspects a disk image:

```python
from guestkit import Guestfs

# Create handle
g = Guestfs()

# Add disk image (read-only)
g.add_drive_ro("/path/to/disk.img")

# Launch the appliance
g.launch()

# Detect operating systems
roots = g.inspect_os()
for root in roots:
    print(f"Found OS: {g.inspect_get_type(root)}")
    print(f"Distribution: {g.inspect_get_distro(root)}")

# Cleanup
g.shutdown()
```

**Important:** Most operations require root/sudo:
```bash
sudo python3 your_script.py
```

---

## API Overview

### Core Workflow

The typical workflow for using GuestKit from Python:

```python
# 1. Create handle
g = Guestfs()

# 2. Configure (optional)
g.set_verbose(True)   # Enable verbose output
g.set_trace(True)     # Enable API tracing

# 3. Add disk images
g.add_drive_ro("/path/to/disk.img")  # Read-only
g.add_drive("/path/to/disk.img")     # Read-write

# 4. Launch appliance
g.launch()

# 5. Perform operations
roots = g.inspect_os()
# ... your operations here ...

# 6. Cleanup
g.shutdown()
```

### Context Manager Support

GuestKit supports Python context managers for automatic cleanup:

```python
with Guestfs() as g:
    g.add_drive_ro("/path/to/disk.img")
    g.launch()

    roots = g.inspect_os()
    # ... operations ...

# Automatic shutdown on exit
```

---

## Complete Examples

### Example 1: Basic OS Inspection

Inspect operating system information from a disk image.

**File:** `examples/python/basic_inspection.py`

```python
#!/usr/bin/env python3
from guestkit import Guestfs
import sys

def main():
    if len(sys.argv) < 2:
        print("Usage: {} <disk-image>".format(sys.argv[0]))
        sys.exit(1)

    disk_path = sys.argv[1]

    g = Guestfs()
    g.add_drive_ro(disk_path)
    g.launch()

    # Detect operating systems
    roots = g.inspect_os()

    if not roots:
        print("No operating systems detected.")
        return

    for i, root in enumerate(roots, 1):
        print(f"=== OS #{i} ===")
        print(f"Root device: {root}")

        # Get OS type
        os_type = g.inspect_get_type(root)
        print(f"Type: {os_type}")

        # Get distribution (for Linux)
        if os_type == "linux":
            distro = g.inspect_get_distro(root)
            print(f"Distribution: {distro}")

            major = g.inspect_get_major_version(root)
            minor = g.inspect_get_minor_version(root)
            print(f"Version: {major}.{minor}")

        # Get hostname
        hostname = g.inspect_get_hostname(root)
        print(f"Hostname: {hostname}")

    g.shutdown()

if __name__ == "__main__":
    main()
```

**Usage:**
```bash
sudo python3 basic_inspection.py /path/to/ubuntu.qcow2
```

**Output:**
```
=== OS #1 ===
Root device: /dev/sda2
Type: linux
Distribution: ubuntu
Version: 22.4
Hostname: ubuntu-server
```

---

### Example 2: List Filesystems and Partitions

Enumerate all filesystems, partitions, and storage devices.

**File:** `examples/python/list_filesystems.py`

```python
#!/usr/bin/env python3
from guestkit import Guestfs
import sys

def format_size(bytes_val):
    """Format bytes into human-readable size."""
    for unit in ['B', 'KB', 'MB', 'GB', 'TB']:
        if bytes_val < 1024.0:
            return f"{bytes_val:.2f} {unit}"
        bytes_val /= 1024.0
    return f"{bytes_val:.2f} PB"

def main():
    disk_path = sys.argv[1]

    g = Guestfs()
    g.add_drive_ro(disk_path)
    g.launch()

    # List devices
    print("--- Devices ---")
    devices = g.list_devices()
    for device in devices:
        print(f"Device: {device}")
        size = g.blockdev_getsize64(device)
        print(f"  Size: {format_size(size)}")

    # List partitions
    print("\n--- Partitions ---")
    partitions = g.list_partitions()
    for partition in partitions:
        print(f"Partition: {partition}")

        fstype = g.vfs_type(partition)
        print(f"  Filesystem: {fstype}")

        label = g.vfs_label(partition)
        if label:
            print(f"  Label: {label}")

    # List all filesystems
    print("\n--- All Filesystems ---")
    filesystems = g.list_filesystems()
    for device, fstype in filesystems.items():
        print(f"{device}: {fstype}")

    # LVM information
    print("\n--- LVM Information ---")
    vgs = g.vgs()
    if vgs:
        print("Volume Groups:", ", ".join(vgs))

    lvs = g.lvs()
    if lvs:
        print("Logical Volumes:", ", ".join(lvs))

    g.shutdown()

if __name__ == "__main__":
    main()
```

**Usage:**
```bash
sudo python3 list_filesystems.py /path/to/disk.img
```

---

### Example 3: Mount and Explore Files

Mount filesystems and read files from the disk image.

**File:** `examples/python/mount_and_explore.py`

```python
#!/usr/bin/env python3
from guestkit import Guestfs
import sys

def main():
    disk_path = sys.argv[1]

    g = Guestfs()
    g.add_drive_ro(disk_path)
    g.launch()

    # Detect and mount
    roots = g.inspect_os()
    root = roots[0]

    mountpoints = g.inspect_get_mountpoints(root)
    mountpoints_sorted = sorted(mountpoints, key=lambda x: len(x[0]))

    for mount_path, device in mountpoints_sorted:
        g.mount_ro(device, mount_path)
        print(f"Mounted {device} at {mount_path}")

    # List root directory
    print("\n--- Root Directory ---")
    entries = g.ls("/")
    for entry in entries:
        print(f"  {entry}")

    # Read system files
    print("\n--- /etc/os-release ---")
    if g.is_file("/etc/os-release"):
        content = g.read_file("/etc/os-release")
        print(content.decode('utf-8'))

    # Disk usage
    print("\n--- Disk Usage ---")
    statvfs = g.statvfs("/")
    total = statvfs['blocks'] * statvfs['bsize']
    free = statvfs['bfree'] * statvfs['bsize']

    print(f"Total: {total / (1024**3):.2f} GB")
    print(f"Free:  {free / (1024**3):.2f} GB")

    g.umount_all()
    g.shutdown()

if __name__ == "__main__":
    main()
```

**Usage:**
```bash
sudo python3 mount_and_explore.py /path/to/ubuntu.qcow2
```

---

### Example 4: Package Inspection

List installed packages on different Linux distributions.

**File:** `examples/python/package_inspection.py`

```python
#!/usr/bin/env python3
from guestkit import Guestfs
import sys

def main():
    disk_path = sys.argv[1]

    g = Guestfs()
    g.add_drive_ro(disk_path)
    g.launch()

    # Detect OS
    roots = g.inspect_os()
    root = roots[0]

    distro = g.inspect_get_distro(root)
    print(f"Distribution: {distro}")

    # Mount filesystems
    mountpoints = g.inspect_get_mountpoints(root)
    for mount_path, device in sorted(mountpoints, key=lambda x: len(x[0])):
        g.mount_ro(device, mount_path)

    # Get installed packages
    apps = g.inspect_list_applications(root)

    print(f"\nFound {len(apps)} installed packages\n")
    print("Package Name                     Version")
    print("-" * 70)

    for app in apps[:20]:  # Show first 20
        name = app.get('app_name', 'unknown')[:30]
        version = app.get('app_version', '')[:30]
        print(f"{name:<32} {version}")

    if len(apps) > 20:
        print(f"\n... and {len(apps) - 20} more packages")

    g.umount_all()
    g.shutdown()

if __name__ == "__main__":
    main()
```

**Usage:**
```bash
sudo python3 package_inspection.py /path/to/fedora.qcow2
```

---

### Example 5: Create New Disk Image

Create and partition a new disk image from scratch.

**File:** `examples/python/create_disk.py`

```python
#!/usr/bin/env python3
from guestkit import Guestfs
import sys

def create_disk(output_path, size_mb=1024):
    print(f"Creating {size_mb}MB disk at {output_path}")

    g = Guestfs()

    # Create empty disk
    g.disk_create(output_path, "raw", size_mb * 1024 * 1024)

    # Add and launch
    g.add_drive(output_path)
    g.launch()

    # Create GPT partition table
    device = "/dev/sda"
    g.part_init(device, "gpt")

    # Create EFI partition (512 MB)
    g.part_add(device, "primary", 2048, 1050623)
    efi_part = "/dev/sda1"

    # Create root partition (remaining space)
    g.part_add(device, "primary", 1050624, -2048)
    root_part = "/dev/sda2"

    # Format filesystems
    g.mkfs("vfat", efi_part)
    g.set_label(efi_part, "EFI")

    g.mkfs("ext4", root_part)
    g.set_label(root_part, "rootfs")

    # Mount and create directory structure
    g.mount(root_part, "/")
    g.mkdir("/boot")
    g.mount(efi_part, "/boot")

    for directory in ["/etc", "/home", "/root", "/var", "/tmp", "/usr"]:
        g.mkdir(directory)

    # Create fstab
    fstab = """LABEL=rootfs  /      ext4  defaults  0  1
LABEL=EFI     /boot  vfat  defaults  0  2
"""
    g.write("/etc/fstab", fstab.encode())
    g.write("/etc/hostname", b"test-system\n")

    # Sync and cleanup
    g.sync()
    g.umount_all()
    g.shutdown()

    print(f"✓ Disk created successfully")

if __name__ == "__main__":
    output_path = sys.argv[1] if len(sys.argv) > 1 else "/tmp/test-disk.img"
    create_disk(output_path)
```

**Usage:**
```bash
sudo python3 create_disk.py /tmp/new-disk.img
```

---

## Common Patterns

### Error Handling

Always wrap operations in try-except blocks:

```python
from guestkit import Guestfs

g = Guestfs()

try:
    g.add_drive_ro("/path/to/disk.img")
    g.launch()

    roots = g.inspect_os()
    for root in roots:
        try:
            os_type = g.inspect_get_type(root)
            print(f"OS Type: {os_type}")
        except Exception as e:
            print(f"Could not get OS type: {e}")

except Exception as e:
    print(f"Error: {e}")

finally:
    try:
        g.shutdown()
    except:
        pass
```

### Using Context Managers

Cleaner resource management with `with`:

```python
with Guestfs() as g:
    g.add_drive_ro("/path/to/disk.img")
    g.launch()

    # Operations here

# Automatic cleanup
```

### Working with Multiple Disk Images

```python
g = Guestfs()

# Add multiple disks
g.add_drive_ro("/path/to/disk1.img")
g.add_drive_ro("/path/to/disk2.img")
g.add_drive("/path/to/writable.img")

g.launch()

# Devices will be /dev/sda, /dev/sdb, /dev/sdc
devices = g.list_devices()
print(devices)  # ['/dev/sda', '/dev/sdb', '/dev/sdc']
```

### Mounting Read-Only vs Read-Write

```python
# Read-only (safe for inspection)
g.mount_ro("/dev/sda1", "/")

# Read-write (for modifications)
g.mount("/dev/sda1", "/")

# Mount with options
g.mount_options("subvol=@,compress=zstd", "/dev/sda2", "/")
```

### File Operations

```python
# Check file existence
if g.is_file("/etc/passwd"):
    # Read file
    content = g.read_file("/etc/passwd")
    print(content.decode('utf-8'))

# Write file
g.write("/etc/hostname", b"new-hostname\n")

# Check directory
if g.is_dir("/home"):
    # List directory
    entries = g.ls("/home")
    for entry in entries:
        print(entry)

# Get file metadata
stat = g.statns("/etc/passwd")
print(f"Size: {stat['st_size']} bytes")
print(f"Mode: {oct(stat['st_mode'])}")
```

### Working with LVM

```python
g = Guestfs()
g.add_drive_ro("/path/to/lvm-disk.img")
g.launch()

# List volume groups
vgs = g.vgs()
print(f"Volume groups: {vgs}")

# List logical volumes
lvs = g.lvs()
print(f"Logical volumes: {lvs}")

# Activate logical volumes
g.vg_activate_all(True)

# Mount LV
g.mount_ro("/dev/mapper/vg0-root", "/")
```

### Working with BTRFS Subvolumes

```python
g = Guestfs()
g.add_drive_ro("/path/to/btrfs-disk.img")
g.launch()

# Mount with subvolume
g.mount_options("subvol=@", "/dev/sda2", "/")

# List subvolumes
subvols = g.btrfs_subvolume_list("/dev/sda2")
for subvol in subvols:
    print(f"Subvolume: {subvol['path']}")
```

---

## API Reference

### Handle Creation and Configuration

```python
g = Guestfs()                    # Create handle
g.set_verbose(True)              # Enable verbose output
g.set_trace(True)                # Enable API call tracing
g.set_autosync(True)             # Auto-sync on shutdown
g.set_network(True)              # Enable networking
```

### Disk Management

```python
g.add_drive(path)                # Add read-write disk
g.add_drive_ro(path)             # Add read-only disk
g.add_drive_opts(path, **opts)   # Add disk with options
g.disk_create(path, fmt, size)   # Create new disk image
g.launch()                       # Launch appliance
g.shutdown()                     # Shutdown appliance
```

### OS Inspection

```python
roots = g.inspect_os()                       # Detect operating systems
os_type = g.inspect_get_type(root)           # Get OS type (linux, windows, etc.)
distro = g.inspect_get_distro(root)          # Get distribution (ubuntu, fedora, etc.)
major = g.inspect_get_major_version(root)    # Get major version
minor = g.inspect_get_minor_version(root)    # Get minor version
arch = g.inspect_get_arch(root)              # Get architecture (x86_64, aarch64, etc.)
hostname = g.inspect_get_hostname(root)      # Get hostname
product = g.inspect_get_product_name(root)   # Get product name
mountpoints = g.inspect_get_mountpoints(root) # Get mountpoints [(path, device), ...]
```

### Device and Partition Management

```python
devices = g.list_devices()                   # List block devices
partitions = g.list_partitions()             # List partitions
filesystems = g.list_filesystems()           # List filesystems {device: type, ...}

g.part_init(device, "gpt")                   # Create partition table (gpt/mbr/msdos)
g.part_add(device, "primary", start, end)    # Add partition
g.part_get_parttype(device)                  # Get partition table type
```

### Filesystem Operations

```python
g.mkfs(fstype, device)                       # Create filesystem
g.set_label(device, label)                   # Set filesystem label
g.vfs_type(device)                           # Get filesystem type
g.vfs_label(device)                          # Get filesystem label
g.vfs_uuid(device)                           # Get filesystem UUID

g.mount(device, mountpoint)                  # Mount filesystem
g.mount_ro(device, mountpoint)               # Mount read-only
g.mount_options(opts, device, mountpoint)    # Mount with options
g.umount(mountpoint)                         # Unmount filesystem
g.umount_all()                               # Unmount all
```

### File and Directory Operations

```python
g.is_file(path)                              # Check if file exists
g.is_dir(path)                               # Check if directory exists
g.exists(path)                               # Check if path exists

g.read_file(path)                            # Read entire file (returns bytes)
g.write(path, content)                       # Write file (content as bytes)
g.cat(path)                                  # Read file (returns string)

g.ls(directory)                              # List directory (names only)
g.ll(directory)                              # List directory (detailed)
g.find(directory)                            # Recursive file listing

g.mkdir(path)                                # Create directory
g.mkdir_p(path)                              # Create directory with parents
g.rm(path)                                   # Remove file
g.rm_rf(path)                                # Remove recursively
g.rmdir(path)                                # Remove empty directory
```

### File Metadata

```python
stat = g.statns(path)                        # Get file stats
# Returns dict with: st_size, st_mode, st_uid, st_gid, st_atime, st_mtime, etc.

size = g.filesize(path)                      # Get file size
g.touch(path)                                # Touch file
g.chmod(mode, path)                          # Change permissions
g.chown(uid, gid, path)                      # Change ownership
```

### Package Management

```python
apps = g.inspect_list_applications(root)     # List installed packages
# Returns list of dicts with: app_name, app_version, app_release, etc.

pkg_format = g.inspect_get_package_format(root)  # Get package format (deb, rpm, etc.)
pkg_mgmt = g.inspect_get_package_management(root) # Get package manager (apt, yum, etc.)
```

### LVM Operations

```python
pvs = g.pvs()                                # List physical volumes
vgs = g.vgs()                                # List volume groups
lvs = g.lvs()                                # List logical volumes

g.vg_activate_all(True)                      # Activate volume groups
g.vg_activate(True, ["vg0"])                 # Activate specific VG
```

### BTRFS Operations

```python
g.btrfs_subvolume_create(path)               # Create subvolume
g.btrfs_subvolume_list(device)               # List subvolumes
g.btrfs_subvolume_delete(path)               # Delete subvolume
```

### Archive Operations

```python
g.tar_out(directory, tarfile)                # Create tar archive
g.tar_in(tarfile, directory)                 # Extract tar archive
g.tgz_out(directory, tarfile)                # Create compressed tar.gz
g.tgz_in(tarfile, directory)                 # Extract tar.gz
```

### System Information

```python
size = g.blockdev_getsize64(device)          # Get block device size in bytes
g.sync()                                     # Sync filesystem
statvfs = g.statvfs(path)                    # Get filesystem stats
# Returns dict with: blocks, bfree, bavail, bsize, etc.
```

---

## Comparison with Rust API

GuestKit provides both Rust and Python APIs. Here's how they compare:

### Creating and Launching

**Python:**
```python
g = Guestfs()
g.add_drive_ro("/path/to/disk.img")
g.set_verbose(True)
g.launch()
```

**Rust (Old API):**
```rust
let mut g = Guestfs::new()?;
g.add_drive_ro("/path/to/disk.img")?;
g.set_verbose(true)?;
g.launch()?;
```

**Rust (New Ergonomic API):**
```rust
let mut g = Guestfs::builder()
    .add_drive_ro("/path/to/disk.img")
    .verbose(true)
    .build_and_launch()?;
```

### Creating Filesystems

**Python:**
```python
g.mkfs("ext4", "/dev/sda1")
g.set_label("/dev/sda1", "rootfs")
```

**Rust (Old API):**
```rust
g.mkfs("ext4", "/dev/sda1", None, Some("rootfs"), None, None)?;
```

**Rust (New Ergonomic API):**
```rust
g.mkfs("/dev/sda1")
    .ext4()
    .label("rootfs")
    .create()?;
```

### OS Detection

**Python:**
```python
roots = g.inspect_os()
for root in roots:
    os_type = g.inspect_get_type(root)
    if os_type == "linux":
        distro = g.inspect_get_distro(root)
        if distro in ["ubuntu", "debian"]:
            # Handle Debian-based
```

**Rust (New Ergonomic API):**
```rust
let roots = g.inspect_os()?;
for root in &roots {
    let os_type = OsType::from_str(&g.inspect_get_type(root)?);

    match os_type {
        OsType::Linux => {
            let distro = Distro::from_str(&g.inspect_get_distro(root)?);
            match distro {
                Distro::Ubuntu | Distro::Debian => {
                    // Handle Debian-based
                }
                _ => {}
            }
        }
        _ => {}
    }
}
```

**Key Differences:**

1. **Error Handling:**
   - Python: Exceptions (try/except)
   - Rust: Result types (? operator, match, if let)

2. **Type Safety:**
   - Python: Duck typing, runtime checks
   - Rust: Static typing, compile-time checks, enums

3. **Resource Management:**
   - Python: Manual cleanup or context managers
   - Rust: RAII, automatic Drop

4. **Performance:**
   - Python: Interpreted, GC overhead
   - Rust: Compiled, zero-cost abstractions

5. **API Style:**
   - Python: Direct C API mapping
   - Rust: Ergonomic builders and type-safe enums (new API)

---

## Best Practices

### 1. Always Use Read-Only Mode for Inspection

```python
# Good - safe inspection
g.add_drive_ro("/path/to/disk.img")

# Bad - unnecessary write access
g.add_drive("/path/to/disk.img")
```

### 2. Run with Appropriate Permissions

```bash
# Most operations require root
sudo python3 your_script.py

# Or use sudo inside script for specific operations
```

### 3. Handle Errors Gracefully

```python
try:
    hostname = g.inspect_get_hostname(root)
except Exception:
    hostname = "unknown"
```

### 4. Clean Up Resources

```python
# Always shutdown, even on error
try:
    g.add_drive_ro(disk_path)
    g.launch()
    # ... operations ...
finally:
    try:
        g.shutdown()
    except:
        pass

# Or use context manager
with Guestfs() as g:
    # ... operations ...
    pass  # Automatic cleanup
```

### 5. Mount Filesystems in Correct Order

```python
# Get mountpoints and sort by length
mountpoints = g.inspect_get_mountpoints(root)
mountpoints_sorted = sorted(mountpoints, key=lambda x: len(x[0]))

# Mount in order (/ before /usr, /var, etc.)
for mount_path, device in mountpoints_sorted:
    g.mount_ro(device, mount_path)
```

### 6. Use Appropriate Filesystem Operations

```python
# For small files
content = g.read_file("/etc/hostname")

# For large files or binary data
g.download("/var/log/syslog", "/tmp/syslog.copy")

# For archives
g.tar_out("/home", "/tmp/home-backup.tar")
```

### 7. Check File Types Before Operations

```python
if g.is_file(path):
    content = g.read_file(path)
elif g.is_dir(path):
    entries = g.ls(path)
else:
    print(f"{path} is not a regular file or directory")
```

### 8. Limit Output for Large Datasets

```python
# Don't print all files in /usr/bin
entries = g.ls("/usr/bin")[:20]  # First 20 only

# Use find with caution on large filesystems
# Consider using find0() and processing incrementally
```

---

## Troubleshooting

### Permission Denied Errors

**Problem:** `RuntimeError: permission denied`

**Solution:** Run with sudo/root:
```bash
sudo python3 your_script.py
```

### Appliance Launch Failures

**Problem:** `RuntimeError: appliance failed to start`

**Solutions:**
1. Check libguestfs is properly installed
2. Ensure KVM/virtualization is available
3. Set verbose mode to see detailed errors:
   ```python
   g.set_verbose(True)
   g.set_trace(True)
   ```

### Filesystem Mount Errors

**Problem:** `RuntimeError: mount: /dev/sda1: can't read superblock`

**Solutions:**
1. Check filesystem type is correct: `g.vfs_type(device)`
2. Device might be encrypted (LUKS)
3. Filesystem might be corrupted

### Missing Operating Systems

**Problem:** `inspect_os()` returns empty list

**Solutions:**
1. Image might not have a recognizable OS
2. Try mounting manually and exploring
3. Check partition table: `g.part_get_parttype(device)`

### Package List Empty

**Problem:** `inspect_list_applications()` returns empty list

**Solutions:**
1. Ensure filesystem is mounted
2. Some minimal systems don't have package databases
3. Check package database exists:
   ```python
   # For Debian/Ubuntu
   g.is_file("/var/lib/dpkg/status")

   # For RPM
   g.is_dir("/var/lib/rpm")
   ```

### Memory Issues

**Problem:** High memory usage or OOM errors

**Solutions:**
1. Limit recursion depth when exploring directories
2. Process large files in chunks
3. Use `find0()` instead of `find()` for large directory trees
4. Set memory limit: `g.set_memsize(2048)`  # 2GB

---

## Additional Resources

- **GitHub Repository:** https://github.com/yourusername/guestkit
- **Rust API Documentation:** `docs/ERGONOMIC_API.md`
- **Migration Guide:** `docs/MIGRATION_GUIDE.md`
- **Examples Directory:** `examples/python/`
- **libguestfs Documentation:** https://libguestfs.org/

---

## Complete Example: Comprehensive Disk Analysis

Here's a complete script that demonstrates many features:

```python
#!/usr/bin/env python3
"""
Comprehensive Disk Image Analysis
Demonstrates multiple GuestKit features in one script.
"""

from guestkit import Guestfs
import sys

def format_size(bytes_val):
    for unit in ['B', 'KB', 'MB', 'GB', 'TB']:
        if bytes_val < 1024.0:
            return f"{bytes_val:.2f} {unit}"
        bytes_val /= 1024.0
    return f"{bytes_val:.2f} PB"

def analyze_disk(disk_path):
    print(f"=== Comprehensive Disk Analysis ===")
    print(f"Image: {disk_path}\n")

    g = Guestfs()
    g.set_verbose(False)

    try:
        # Add and launch
        print("[1/6] Launching appliance...")
        g.add_drive_ro(disk_path)
        g.launch()

        # Device information
        print("\n[2/6] Device Information")
        devices = g.list_devices()
        for device in devices:
            size = g.blockdev_getsize64(device)
            print(f"  {device}: {format_size(size)}")

        # Partition information
        print("\n[3/6] Partition Information")
        partitions = g.list_partitions()
        for partition in partitions:
            print(f"  {partition}")
            try:
                size = g.blockdev_getsize64(partition)
                fstype = g.vfs_type(partition)
                label = g.vfs_label(partition)
                print(f"    Size: {format_size(size)}")
                print(f"    Type: {fstype}")
                if label:
                    print(f"    Label: {label}")
            except Exception as e:
                print(f"    Error: {e}")

        # OS detection
        print("\n[4/6] Operating System Detection")
        roots = g.inspect_os()

        if not roots:
            print("  No operating systems detected")
            return

        for root in roots:
            print(f"  Root: {root}")

            os_type = g.inspect_get_type(root)
            print(f"    Type: {os_type}")

            if os_type == "linux":
                distro = g.inspect_get_distro(root)
                major = g.inspect_get_major_version(root)
                minor = g.inspect_get_minor_version(root)
                print(f"    Distribution: {distro} {major}.{minor}")

            hostname = g.inspect_get_hostname(root)
            print(f"    Hostname: {hostname}")

        # Mount filesystems
        print("\n[5/6] Mounting and Exploring")
        root = roots[0]
        mountpoints = g.inspect_get_mountpoints(root)

        for mount_path, device in sorted(mountpoints, key=lambda x: len(x[0])):
            try:
                g.mount_ro(device, mount_path)
                print(f"  Mounted {device} -> {mount_path}")
            except Exception as e:
                print(f"  Failed to mount {device}: {e}")

        # Read key files
        key_files = {
            "/etc/os-release": "OS Release",
            "/etc/hostname": "Hostname",
            "/etc/fstab": "Filesystem Table",
        }

        for path, description in key_files.items():
            if g.is_file(path):
                print(f"\n  --- {description} ({path}) ---")
                try:
                    content = g.read_file(path).decode('utf-8')
                    lines = content.split('\n')[:5]
                    for line in lines:
                        if line.strip():
                            print(f"  {line}")
                except Exception as e:
                    print(f"  Error reading: {e}")

        # Package information
        print("\n[6/6] Package Information")
        try:
            apps = g.inspect_list_applications(root)
            print(f"  Total packages: {len(apps)}")

            if apps:
                print("\n  Sample packages:")
                for app in apps[:5]:
                    name = app.get('app_name', 'unknown')
                    version = app.get('app_version', '')
                    print(f"    {name} {version}")
        except Exception as e:
            print(f"  Package inspection failed: {e}")

        # Cleanup
        g.umount_all()
        g.shutdown()

        print("\n✓ Analysis complete!")

    except Exception as e:
        print(f"\n✗ Error: {e}")
        try:
            g.shutdown()
        except:
            pass
        sys.exit(1)

if __name__ == "__main__":
    if len(sys.argv) < 2:
        print(f"Usage: {sys.argv[0]} <disk-image>")
        sys.exit(1)

    analyze_disk(sys.argv[1])
```

Save as `analyze.py` and run:
```bash
sudo python3 analyze.py /path/to/disk.img
```

---

**Happy disk image hacking with GuestKit! 🚀**
