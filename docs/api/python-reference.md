# GuestCtl Python API Reference

Complete API reference for GuestCtl Python bindings.

## Table of Contents

- [Installation](#installation)
- [Guestfs Class](#guestfs-class)
- [DiskConverter Class](#diskconverter-class)
- [Quick Start](#quick-start)
- [Complete Examples](#complete-examples)

## Installation

### Build and Install

```bash
# Install maturin (PyO3 build tool)
pip install maturin

# Build and install in development mode
cd /path/to/guestctl
PYO3_USE_ABI3_FORWARD_COMPATIBILITY=1 maturin develop --features python-bindings

# Or build wheel for distribution
PYO3_USE_ABI3_FORWARD_COMPATIBILITY=1 maturin build --release --features python-bindings
pip install target/wheels/guestctl-*.whl
```

### Verify Installation

```python
import guestctl
print(guestctl.__version__)  # Should print "0.3.0"
```

## Guestfs Class

Main class for VM disk image inspection and manipulation.

### Constructor

#### `Guestfs()`

Create a new GuestFS handle.

**Returns:** New Guestfs instance

**Raises:** RuntimeError if handle creation fails

**Example:**
```python
from guestctl import Guestfs

g = Guestfs()
```

---

### Handle Management

#### `shutdown()`

Shutdown the appliance and close the handle.

**Example:**
```python
g.shutdown()
```

#### `set_verbose(verbose: bool)`

Enable or disable verbose output for debugging.

**Parameters:**
- `verbose` (bool): True to enable, False to disable

**Example:**
```python
g.set_verbose(True)  # See detailed operations
```

---

### Drive Operations

#### `add_drive_ro(filename: str)`

Add a disk image in read-only mode (safe for inspection).

**Parameters:**
- `filename` (str): Path to disk image file

**Raises:** RuntimeError if file doesn't exist or cannot be added

**Example:**
```python
g.add_drive_ro("/path/to/vm.qcow2")
```

#### `add_drive(filename: str)`

Add a disk image in read-write mode.

**Parameters:**
- `filename` (str): Path to disk image file

**Raises:** RuntimeError if file doesn't exist or cannot be added

**Warning:** Use with caution. Always backup before using read-write mode.

**Example:**
```python
g.add_drive("/path/to/vm.qcow2")
```

#### `launch()`

Launch the appliance. Must be called after adding drives and before inspection.

**Raises:** RuntimeError if appliance fails to launch

**Example:**
```python
g.add_drive_ro("vm.qcow2")
g.launch()  # Start the appliance
```

---

### OS Inspection

#### `inspect_os() -> List[str]`

Detect operating systems in the disk image.

**Returns:** List of root device paths (e.g., `["/dev/sda1"]`)

**Example:**
```python
roots = g.inspect_os()
if roots:
    print(f"Found {len(roots)} operating system(s)")
    for root in roots:
        print(f"  Root: {root}")
```

#### `inspect_get_type(root: str) -> str`

Get operating system type.

**Parameters:**
- `root` (str): Root device from `inspect_os()`

**Returns:** OS type: `"linux"`, `"windows"`, `"freebsd"`, etc.

**Example:**
```python
os_type = g.inspect_get_type("/dev/sda1")
if os_type == "linux":
    print("Linux system detected")
```

#### `inspect_get_distro(root: str) -> str`

Get Linux distribution name.

**Parameters:**
- `root` (str): Root device from `inspect_os()`

**Returns:** Distribution: `"ubuntu"`, `"fedora"`, `"debian"`, `"centos"`, etc.

**Example:**
```python
distro = g.inspect_get_distro("/dev/sda1")
print(f"Distribution: {distro}")
```

#### `inspect_get_major_version(root: str) -> int`

Get OS major version number.

**Parameters:**
- `root` (str): Root device from `inspect_os()`

**Returns:** Major version number (e.g., 22 for Ubuntu 22.04)

**Example:**
```python
major = g.inspect_get_major_version("/dev/sda1")
```

#### `inspect_get_minor_version(root: str) -> int`

Get OS minor version number.

**Parameters:**
- `root` (str): Root device from `inspect_os()`

**Returns:** Minor version number (e.g., 4 for Ubuntu 22.04)

**Example:**
```python
minor = g.inspect_get_minor_version("/dev/sda1")
version = f"{major}.{minor}"
```

#### `inspect_get_hostname(root: str) -> str`

Get system hostname.

**Parameters:**
- `root` (str): Root device from `inspect_os()`

**Returns:** Hostname string

**Example:**
```python
hostname = g.inspect_get_hostname("/dev/sda1")
print(f"Hostname: {hostname}")
```

#### `inspect_get_arch(root: str) -> str`

Get system architecture.

**Parameters:**
- `root` (str): Root device from `inspect_os()`

**Returns:** Architecture: `"x86_64"`, `"aarch64"`, `"i686"`, etc.

**Example:**
```python
arch = g.inspect_get_arch("/dev/sda1")
print(f"Architecture: {arch}")
```

#### `inspect_get_product_name(root: str) -> str`

Get full product name.

**Parameters:**
- `root` (str): Root device from `inspect_os()`

**Returns:** Product name (e.g., "Ubuntu 22.04 LTS")

**Example:**
```python
product = g.inspect_get_product_name("/dev/sda1")
```

#### `inspect_get_package_format(root: str) -> str`

Get package format used by the OS.

**Parameters:**
- `root` (str): Root device from `inspect_os()`

**Returns:** Package format: `"rpm"`, `"deb"`, `"pacman"`, etc.

**Example:**
```python
pkg_format = g.inspect_get_package_format("/dev/sda1")
if pkg_format == "deb":
    print("Debian-based system")
```

#### `inspect_get_package_management(root: str) -> str`

Get package management tool.

**Parameters:**
- `root` (str): Root device from `inspect_os()`

**Returns:** Tool name: `"apt"`, `"dnf"`, `"yum"`, `"pacman"`, etc.

**Example:**
```python
pkg_mgmt = g.inspect_get_package_management("/dev/sda1")
```

#### `inspect_get_mountpoints(root: str) -> dict`

Get filesystem mountpoints.

**Parameters:**
- `root` (str): Root device from `inspect_os()`

**Returns:** Dictionary mapping mountpoint paths to device paths

**Example:**
```python
mountpoints = g.inspect_get_mountpoints("/dev/sda1")
# Example: {"/": "/dev/sda2", "/boot": "/dev/sda1"}

# Mount in correct order (shortest path first)
for mp, dev in sorted(mountpoints.items(), key=lambda x: len(x[0])):
    g.mount_ro(dev, mp)
```

#### `inspect_list_applications(root: str) -> List[dict]`

List all installed packages.

**Parameters:**
- `root` (str): Root device from `inspect_os()`

**Returns:** List of dictionaries with package information

**Dictionary keys:**
- `app_name` - Package name
- `app_display_name` - Display name
- `app_epoch` - Epoch number
- `app_version` - Version string
- `app_release` - Release string
- `app_install_path` - Installation path
- `app_publisher` - Publisher/maintainer
- `app_url` - Homepage URL
- `app_description` - Description

**Example:**
```python
apps = g.inspect_list_applications("/dev/sda1")
print(f"Total packages: {len(apps)}")

for app in apps[:10]:  # First 10
    name = app['app_name']
    version = app['app_version']
    print(f"{name}-{version}")

# Find specific packages
nginx_packages = [a for a in apps if 'nginx' in a['app_name'].lower()]
```

---

### Device Operations

#### `list_devices() -> List[str]`

List all block devices.

**Returns:** List of device paths (e.g., `["/dev/sda", "/dev/sdb"]`)

**Example:**
```python
devices = g.list_devices()
for dev in devices:
    print(f"Device: {dev}")
```

#### `list_partitions() -> List[str]`

List all partitions.

**Returns:** List of partition paths (e.g., `["/dev/sda1", "/dev/sda2"]`)

**Example:**
```python
partitions = g.list_partitions()
for part in partitions:
    size = g.blockdev_getsize64(part)
    size_gb = size / (1024**3)
    print(f"{part}: {size_gb:.2f} GB")
```

#### `blockdev_getsize64(device: str) -> int`

Get device or partition size in bytes.

**Parameters:**
- `device` (str): Device or partition path

**Returns:** Size in bytes

**Example:**
```python
size = g.blockdev_getsize64("/dev/sda")
size_gb = size / (1024**3)
print(f"Disk size: {size_gb:.2f} GB")
```

---

### Filesystem Operations

#### `vfs_type(device: str) -> str`

Get filesystem type.

**Parameters:**
- `device` (str): Device or partition path

**Returns:** Filesystem type: `"ext4"`, `"xfs"`, `"vfat"`, `"btrfs"`, etc.

**Example:**
```python
fs_type = g.vfs_type("/dev/sda1")
print(f"Filesystem: {fs_type}")
```

#### `vfs_label(device: str) -> str`

Get filesystem label.

**Parameters:**
- `device` (str): Device or partition path

**Returns:** Filesystem label string (may be empty)

**Example:**
```python
label = g.vfs_label("/dev/sda1")
if label:
    print(f"Label: {label}")
```

#### `vfs_uuid(device: str) -> str`

Get filesystem UUID.

**Parameters:**
- `device` (str): Device or partition path

**Returns:** UUID string

**Example:**
```python
uuid = g.vfs_uuid("/dev/sda1")
print(f"UUID: {uuid}")
```

#### `mount(device: str, mountpoint: str)`

Mount a filesystem in read-write mode.

**Parameters:**
- `device` (str): Device to mount
- `mountpoint` (str): Mount point path

**Raises:** RuntimeError if mount fails

**Example:**
```python
g.mount("/dev/sda2", "/")
```

#### `mount_ro(device: str, mountpoint: str)`

Mount a filesystem in read-only mode (recommended for inspection).

**Parameters:**
- `device` (str): Device to mount
- `mountpoint` (str): Mount point path

**Raises:** RuntimeError if mount fails

**Example:**
```python
g.mount_ro("/dev/sda2", "/")
```

#### `umount(mountpoint: str)`

Unmount a filesystem.

**Parameters:**
- `mountpoint` (str): Mount point path

**Example:**
```python
g.umount("/")
```

#### `umount_all()`

Unmount all mounted filesystems.

**Example:**
```python
g.umount_all()
```

#### `sync()`

Synchronize filesystem (flush write buffers).

**Example:**
```python
g.sync()  # Ensure all writes are complete
```

---

### File Operations

#### `read_file(path: str) -> bytes`

Read file contents as bytes.

**Parameters:**
- `path` (str): File path in guest

**Returns:** File contents as bytes

**Raises:** RuntimeError if file doesn't exist or cannot be read

**Example:**
```python
content = g.read_file("/etc/passwd")
text = content.decode('utf-8')
```

#### `cat(path: str) -> str`

Read file contents as string.

**Parameters:**
- `path` (str): File path in guest

**Returns:** File contents as string

**Example:**
```python
hostname = g.cat("/etc/hostname")
print(f"Hostname: {hostname.strip()}")
```

#### `write(path: str, content: bytes)`

Write bytes to file.

**Parameters:**
- `path` (str): File path in guest
- `content` (bytes): Content to write

**Example:**
```python
g.write("/etc/motd", b"Welcome!\\n")
```

#### `exists(path: str) -> bool`

Check if path exists.

**Parameters:**
- `path` (str): Path in guest

**Returns:** True if exists, False otherwise

**Example:**
```python
if g.exists("/etc/passwd"):
    content = g.read_file("/etc/passwd")
```

#### `is_file(path: str) -> bool`

Check if path is a regular file.

**Parameters:**
- `path` (str): Path in guest

**Returns:** True if file, False otherwise

**Example:**
```python
if g.is_file("/etc/hostname"):
    hostname = g.cat("/etc/hostname")
```

#### `is_dir(path: str) -> bool`

Check if path is a directory.

**Parameters:**
- `path` (str): Path in guest

**Returns:** True if directory, False otherwise

**Example:**
```python
if g.is_dir("/home"):
    users = g.ls("/home")
```

#### `ls(directory: str) -> List[str]`

List directory contents.

**Parameters:**
- `directory` (str): Directory path in guest

**Returns:** List of filenames (without paths)

**Example:**
```python
files = g.ls("/etc")
for f in files:
    print(f)
```

#### `download(remotefilename: str, filename: str)`

Download file from guest to host.

**Parameters:**
- `remotefilename` (str): Path in guest
- `filename` (str): Path on host

**Example:**
```python
g.download("/etc/passwd", "./passwd.txt")
```

#### `upload(filename: str, remotefilename: str)`

Upload file from host to guest.

**Parameters:**
- `filename` (str): Path on host
- `remotefilename` (str): Path in guest

**Example:**
```python
g.upload("./config.txt", "/etc/myapp/config.txt")
```

#### `mkdir(path: str)`

Create directory.

**Parameters:**
- `path` (str): Directory path in guest

**Example:**
```python
g.mkdir("/tmp/mydir")
```

#### `mkdir_p(path: str)`

Create directory with parents (like `mkdir -p`).

**Parameters:**
- `path` (str): Directory path in guest

**Example:**
```python
g.mkdir_p("/tmp/parent/child/grandchild")
```

#### `rm(path: str)`

Remove file.

**Parameters:**
- `path` (str): File path in guest

**Example:**
```python
g.rm("/tmp/tempfile.txt")
```

#### `rmdir(path: str)`

Remove empty directory.

**Parameters:**
- `path` (str): Directory path in guest

**Example:**
```python
g.rmdir("/tmp/emptydir")
```

#### `rm_rf(path: str)`

Remove directory recursively (like `rm -rf`).

**Parameters:**
- `path` (str): Directory path in guest

**Warning:** Dangerous operation. Use with caution.

**Example:**
```python
g.rm_rf("/tmp/old_data")
```

#### `chmod(mode: int, path: str)`

Change file permissions.

**Parameters:**
- `mode` (int): Permission mode (octal)
- `path` (str): File path in guest

**Example:**
```python
g.chmod(0o644, "/etc/myconfig")  # rw-r--r--
g.chmod(0o755, "/usr/local/bin/myscript")  # rwxr-xr-x
```

#### `chown(owner: int, group: int, path: str)`

Change file owner and group.

**Parameters:**
- `owner` (int): User ID
- `group` (int): Group ID
- `path` (str): File path in guest

**Example:**
```python
g.chown(1000, 1000, "/home/user/file.txt")
```

#### `stat(path: str) -> dict`

Get file status information.

**Parameters:**
- `path` (str): File path in guest

**Returns:** Dictionary with stat fields

**Dictionary keys:**
- `dev` - Device ID
- `ino` - Inode number
- `mode` - File mode
- `nlink` - Number of hard links
- `uid` - Owner user ID
- `gid` - Owner group ID
- `rdev` - Device ID (if special file)
- `size` - File size in bytes
- `blksize` - Block size
- `blocks` - Number of blocks
- `atime` - Access time
- `mtime` - Modification time
- `ctime` - Status change time

**Example:**
```python
stat = g.stat("/etc/passwd")
print(f"Size: {stat['size']} bytes")
print(f"Owner: UID {stat['uid']}, GID {stat['gid']}")
print(f"Permissions: {oct(stat['mode'])}")
```

#### `statvfs(path: str) -> dict`

Get filesystem statistics.

**Parameters:**
- `path` (str): Path in guest

**Returns:** Dictionary with filesystem stats

**Dictionary keys:**
- `bsize` - Block size
- `frsize` - Fragment size
- `blocks` - Total blocks
- `bfree` - Free blocks
- `bavail` - Available blocks
- `files` - Total inodes
- `ffree` - Free inodes
- `favail` - Available inodes
- `fsid` - Filesystem ID
- `flag` - Mount flags
- `namemax` - Maximum filename length

**Example:**
```python
statvfs = g.statvfs("/")
total_bytes = statvfs['blocks'] * statvfs['frsize']
free_bytes = statvfs['bfree'] * statvfs['frsize']
avail_bytes = statvfs['bavail'] * statvfs['frsize']

total_gb = total_bytes / (1024**3)
free_gb = free_bytes / (1024**3)
used_gb = total_gb - free_gb
used_percent = (used_gb / total_gb * 100) if total_gb > 0 else 0

print(f"Total: {total_gb:.2f} GB")
print(f"Used:  {used_gb:.2f} GB ({used_percent:.1f}%)")
print(f"Free:  {free_gb:.2f} GB")
```

---

### Command Execution

#### `command(arguments: List[str]) -> str`

Execute a command in the guest.

**Parameters:**
- `arguments` (List[str]): Command and arguments

**Returns:** Command output as string

**Requires:** Mounted filesystem

**Example:**
```python
output = g.command(["/bin/ls", "-la", "/etc"])
print(output)
```

#### `sh(command: str) -> str`

Execute shell command.

**Parameters:**
- `command` (str): Shell command string

**Returns:** Command output as string

**Example:**
```python
output = g.sh("cat /etc/os-release | grep VERSION_ID")
```

#### `sh_lines(command: str) -> List[str]`

Execute shell command and return lines.

**Parameters:**
- `command` (str): Shell command string

**Returns:** List of output lines

**Example:**
```python
lines = g.sh_lines("ls -1 /etc")
for line in lines:
    print(line)
```

---

### LVM Operations

#### `vgscan()`

Scan for LVM volume groups.

**Example:**
```python
g.vgscan()
```

#### `vgs() -> List[str]`

List volume groups.

**Returns:** List of volume group names

**Example:**
```python
vgs = g.vgs()
if vgs:
    print(f"Volume groups: {', '.join(vgs)}")
```

#### `pvs() -> List[str]`

List physical volumes.

**Returns:** List of physical volume paths

**Example:**
```python
pvs = g.pvs()
for pv in pvs:
    print(f"Physical volume: {pv}")
```

#### `lvs() -> List[str]`

List logical volumes.

**Returns:** List of logical volume paths

**Example:**
```python
lvs = g.lvs()
for lv in lvs:
    print(f"Logical volume: {lv}")
    # Mount LV
    g.mount_ro(lv, "/")
```

---

### Archive Operations

#### `tar_in(tarfile: str, directory: str)`

Extract tar archive into guest directory.

**Parameters:**
- `tarfile` (str): Path to tar file on host
- `directory` (str): Directory in guest to extract to

**Example:**
```python
g.tar_in("/path/to/backup.tar", "/restore")
```

#### `tar_out(directory: str, tarfile: str)`

Create tar archive from guest directory.

**Parameters:**
- `directory` (str): Directory in guest to archive
- `tarfile` (str): Path to tar file on host

**Example:**
```python
g.tar_out("/etc", "/backups/etc-backup.tar")
```

#### `tgz_in(tarfile: str, directory: str)`

Extract compressed tar archive (tar.gz) into guest directory.

**Parameters:**
- `tarfile` (str): Path to tar.gz file on host
- `directory` (str): Directory in guest to extract to

**Example:**
```python
g.tgz_in("/path/to/data.tar.gz", "/restore")
```

#### `tgz_out(directory: str, tarfile: str)`

Create compressed tar archive (tar.gz) from guest directory.

**Parameters:**
- `directory` (str): Directory in guest to archive
- `tarfile` (str): Path to tar.gz file on host

**Example:**
```python
g.tgz_out("/var/log", "/backups/logs.tar.gz")
```

---

### Checksum Operations

#### `checksum(csumtype: str, path: str) -> str`

Calculate file checksum.

**Parameters:**
- `csumtype` (str): Checksum type - `"md5"`, `"sha1"`, `"sha224"`, `"sha256"`, `"sha384"`, `"sha512"`
- `path` (str): File path in guest

**Returns:** Checksum as hexadecimal string

**Example:**
```python
md5 = g.checksum("md5", "/etc/passwd")
sha256 = g.checksum("sha256", "/etc/passwd")

print(f"MD5:    {md5}")
print(f"SHA256: {sha256}")
```

---

## DiskConverter Class

Class for converting disk image formats.

### Constructor

#### `DiskConverter()`

Create a new disk converter instance.

**Example:**
```python
from guestctl import DiskConverter

converter = DiskConverter()
```

### Methods

#### `convert(source: str, output: str, format: str = "qcow2", compress: bool = False, flatten: bool = True) -> dict`

Convert disk image format.

**Parameters:**
- `source` (str): Source disk image path
- `output` (str): Output disk image path
- `format` (str): Output format - `"qcow2"`, `"raw"`, `"vmdk"`, `"vdi"` (default: `"qcow2"`)
- `compress` (bool): Enable compression (default: `False`)
- `flatten` (bool): Flatten snapshot chains (default: `True`)

**Returns:** Dictionary with conversion results

**Dictionary keys:**
- `source_path` - Source file path
- `output_path` - Output file path
- `source_format` - Detected source format
- `output_format` - Output format
- `output_size` - Output file size in bytes
- `duration_secs` - Conversion duration
- `success` - True if successful
- `error` - Error message (if failed)

**Example:**
```python
converter = DiskConverter()

result = converter.convert(
    "/path/to/source.vmdk",
    "/path/to/output.qcow2",
    format="qcow2",
    compress=True
)

if result['success']:
    size_gb = result['output_size'] / (1024**3)
    print(f"✓ Converted successfully")
    print(f"  Output: {result['output_path']}")
    print(f"  Size: {size_gb:.2f} GB")
    print(f"  Duration: {result['duration_secs']:.1f}s")
else:
    print(f"✗ Conversion failed: {result['error']}")
```

#### `detect_format(image: str) -> str`

Detect disk image format.

**Parameters:**
- `image` (str): Disk image path

**Returns:** Format string (`"qcow2"`, `"raw"`, `"vmdk"`, etc.)

**Example:**
```python
format = converter.detect_format("/path/to/disk.img")
print(f"Format: {format}")
```

#### `get_info(image: str) -> dict`

Get disk image metadata.

**Parameters:**
- `image` (str): Disk image path

**Returns:** Dictionary with image information (parsed from qemu-img info)

**Example:**
```python
info = converter.get_info("/path/to/disk.qcow2")
print(f"Format: {info.get('format')}")
print(f"Virtual size: {info.get('virtual-size')}")
```

---

## Complete Examples

### Example 1: Basic Inspection

```python
#!/usr/bin/env python3
from guestctl import Guestfs
import sys

def main():
    if len(sys.argv) < 2:
        print("Usage: script.py <disk-image>")
        sys.exit(1)

    disk_path = sys.argv[1]
    g = Guestfs()

    try:
        # Setup
        g.add_drive_ro(disk_path)
        g.launch()

        # Inspect
        roots = g.inspect_os()
        if not roots:
            print("No OS detected")
            return

        root = roots[0]

        # Get OS info
        print(f"OS Type: {g.inspect_get_type(root)}")
        print(f"Distribution: {g.inspect_get_distro(root)}")

        major = g.inspect_get_major_version(root)
        minor = g.inspect_get_minor_version(root)
        print(f"Version: {major}.{minor}")

        print(f"Hostname: {g.inspect_get_hostname(root)}")
        print(f"Architecture: {g.inspect_get_arch(root)}")

    finally:
        g.shutdown()

if __name__ == "__main__":
    main()
```

### Example 2: File Extraction

```python
#!/usr/bin/env python3
from guestctl import Guestfs

def extract_configs(disk_path, output_dir):
    import os
    os.makedirs(output_dir, exist_ok=True)

    g = Guestfs()

    try:
        g.add_drive_ro(disk_path)
        g.launch()

        roots = g.inspect_os()
        if not roots:
            return

        # Mount filesystems
        mountpoints = g.inspect_get_mountpoints(roots[0])
        for mp, dev in sorted(mountpoints.items(), key=lambda x: len(x[0])):
            g.mount_ro(dev, mp)

        # Extract files
        files = [
            "/etc/passwd",
            "/etc/hostname",
            "/etc/fstab",
        ]

        for file_path in files:
            if g.is_file(file_path):
                basename = os.path.basename(file_path)
                output_path = os.path.join(output_dir, basename)
                g.download(file_path, output_path)
                print(f"Extracted: {file_path}")

    finally:
        g.umount_all()
        g.shutdown()

extract_configs("vm.qcow2", "./configs")
```

### Example 3: Package Listing

```python
#!/usr/bin/env python3
from guestctl import Guestfs

def list_packages(disk_path):
    g = Guestfs()

    try:
        g.add_drive_ro(disk_path)
        g.launch()

        roots = g.inspect_os()
        if not roots:
            return

        root = roots[0]

        # Mount
        mountpoints = g.inspect_get_mountpoints(root)
        for mp, dev in sorted(mountpoints.items(), key=lambda x: len(x[0])):
            g.mount_ro(dev, mp)

        # List packages
        apps = g.inspect_list_applications(root)

        print(f"Total packages: {len(apps)}")
        print("\nPackage listing:")
        print(f"{'Name':<30} {'Version':<20}")
        print("-" * 50)

        for app in sorted(apps, key=lambda x: x['app_name'])[:20]:
            name = app['app_name'][:29]
            version = app['app_version'][:19]
            print(f"{name:<30} {version:<20}")

    finally:
        g.umount_all()
        g.shutdown()

list_packages("ubuntu.qcow2")
```

---

**GuestCtl Python Bindings** - Powerful VM disk image manipulation from Python 🐍
