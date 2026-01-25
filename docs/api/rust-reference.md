# GuestCtl API Reference

Complete API reference for guestctl v0.2.0 - a pure Rust implementation of -compatible APIs.

## Table of Contents

- [Initialization and Configuration](#initialization-and-configuration)
- [Disk and Device Operations](#disk-and-device-operations)
- [Partition Management](#partition-management)
- [Filesystem Operations](#filesystem-operations)
- [File Operations](#file-operations)
- [Mount Operations](#mount-operations)
- [Archive and Compression](#archive-and-compression)
- [Encryption (LUKS)](#encryption-luks)
- [LVM Operations](#lvm-operations)
- [Inspection and Detection](#inspection-and-detection)
- [Command Execution](#command-execution)
- [Network Configuration](#network-configuration)
- [Package Management](#package-management)
- [System Configuration](#system-configuration)
- [Security Operations](#security-operations)
- [Advanced Topics](#advanced-topics)

---

## Initialization and Configuration

### Creating a GuestFS Handle

```rust
use guestctl::Guestfs;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut g = Guestfs::new()?;

    // Configure verbosity
    g.set_verbose(true);
    g.set_trace(true);

    // Add disk images
    g.add_drive_ro("/path/to/disk.qcow2")?;

    // Launch and analyze
    g.launch()?;

    // ... perform operations ...

    // Cleanup
    g.shutdown()?;
    Ok(())
}
```

### Core Initialization Functions

- **`Guestfs::new()`** - Create a new GuestFS handle
  ```rust
  let mut g = Guestfs::new()?;
  ```

- **`add_drive_ro(path)`** - Add a disk image in read-only mode
  ```rust
  g.add_drive_ro("/path/to/disk.img")?;
  ```

- **`add_drive_opts(path, format, readonly)`** - Add disk with options
  ```rust
  g.add_drive_opts("/path/to/disk.qcow2", Some("qcow2"), false)?;
  ```

- **`launch()`** - Initialize the handle and analyze the disk
  ```rust
  g.launch()?;
  ```

- **`shutdown()`** - Clean up resources
  ```rust
  g.shutdown()?;
  ```

### Configuration Methods

- **`set_verbose(bool)`** / **`get_verbose()`** - Control verbose output
- **`set_trace(bool)`** / **`get_trace()`** - Control operation tracing

---

## Disk and Device Operations

### Listing Devices

- **`list_devices()`** - List all block devices
  ```rust
  let devices = g.list_devices()?;
  for device in devices {
      println!("Device: {}", device);
  }
  ```

- **`list_partitions()`** - List all partitions
  ```rust
  let partitions = g.list_partitions()?;
  ```

- **`list_filesystems()`** - List filesystems with their types
  ```rust
  let filesystems = g.list_filesystems()?;
  for (device, fstype) in filesystems {
      println!("{}: {}", device, fstype);
  }
  ```

### Device Properties

- **`blockdev_getsize64(device)`** - Get device size in bytes
  ```rust
  let size = g.blockdev_getsize64("/dev/sda")?;
  println!("Size: {} bytes", size);
  ```

- **`blockdev_getsz(device)`** - Get device size in 512-byte sectors

- **`blockdev_getss(device)`** - Get logical block (sector) size

- **`blockdev_getbsz(device)`** - Get block size

### Disk Image Management

- **`disk_create(path, format, size)`** - Create a new disk image
  ```rust
  g.disk_create("/tmp/new.img", "raw", 1024 * 1024 * 1024)?; // 1GB
  ```

- **`disk_format(device)`** - Detect disk format
  ```rust
  let format = g.disk_format("/dev/sda")?;
  ```

- **`disk_virtual_size(path)`** - Get virtual size of disk image

---

## Partition Management

### Partition Table Operations

- **`part_init(device, parttype)`** - Initialize partition table
  ```rust
  g.part_init("/dev/sda", "gpt")?; // or "mbr"
  ```

- **`part_get_parttype(device)`** - Get partition table type
  ```rust
  let pttype = g.part_get_parttype("/dev/sda")?;
  ```

### Creating and Managing Partitions

- **`part_add(device, prlogex, startsect, endsect)`** - Add partition
  ```rust
  g.part_add("/dev/sda", "primary", 2048, -2048)?;
  ```

- **`part_del(device, partnum)`** - Delete partition
  ```rust
  g.part_del("/dev/sda", 1)?;
  ```

- **`part_list(device)`** - List partitions with details
  ```rust
  let parts = g.part_list("/dev/sda")?;
  for part in parts {
      println!("Partition {}: start={}, size={}",
               part.part_num, part.part_start, part.part_size);
  }
  ```

### Partition Properties

- **`part_get_bootable(device, partnum)`** - Check if partition is bootable

- **`part_set_bootable(device, partnum, bootable)`** - Set bootable flag

- **`part_get_name(device, partnum)`** - Get GPT partition name

- **`part_set_name(device, partnum, name)`** - Set GPT partition name
  ```rust
  g.part_set_name("/dev/sda", 1, "Boot")?;
  ```

- **`part_get_mbr_id(device, partnum)`** - Get MBR partition type ID

- **`part_set_mbr_id(device, partnum, id)`** - Set MBR partition type ID

---

## Filesystem Operations

### Creating Filesystems

- **`mkfs(fstype, device)`** - Create a filesystem
  ```rust
  g.mkfs("ext4", "/dev/sda1")?;
  g.mkfs("xfs", "/dev/sda2")?;
  g.mkfs("btrfs", "/dev/sda3")?;
  ```

- **`mkfs_opts(fstype, device, options)`** - Create filesystem with options
  ```rust
  g.mkfs_opts("ext4", "/dev/sda1", Some("blocksize=4096,label=MyDisk"))?;
  ```

### Filesystem Maintenance

- **`fsck(fstype, device)`** - Check and repair filesystem
  ```rust
  g.fsck("ext4", "/dev/sda1")?;
  ```

- **`tune2fs(device, options)`** - Tune ext2/3/4 filesystem
  ```rust
  g.tune2fs("/dev/sda1", "-L NewLabel")?;
  ```

- **`zerofree(device)`** - Zero free space on filesystem
  ```rust
  g.zerofree("/dev/sda1")?;
  ```

- **`fstrim(mountpoint)`** - Discard unused blocks (SSD optimization)
  ```rust
  g.fstrim("/")?;
  ```

### Filesystem Properties

- **`vfs_type(device)`** - Get filesystem type
  ```rust
  let fstype = g.vfs_type("/dev/sda1")?;
  ```

- **`vfs_label(device)`** - Get filesystem label
  ```rust
  let label = g.vfs_label("/dev/sda1")?;
  ```

- **`vfs_uuid(device)`** - Get filesystem UUID
  ```rust
  let uuid = g.vfs_uuid("/dev/sda1")?;
  ```

- **`set_label(device, label)`** - Set filesystem label
  ```rust
  g.set_label("/dev/sda1", "MyData")?;
  ```

### Filesystem Statistics

- **`df()`** - Get filesystem statistics
  ```rust
  let stats = g.df()?;
  ```

- **`df_h()`** - Get human-readable filesystem statistics

- **`statvfs(path)`** - Get detailed filesystem statistics
  ```rust
  let statvfs = g.statvfs("/")?;
  println!("Total: {} bytes", statvfs["blocks"] * statvfs["bsize"]);
  ```

---

## File Operations

### Basic File Operations

- **`exists(path)`** - Check if path exists
  ```rust
  if g.exists("/etc/passwd")? {
      println!("File exists");
  }
  ```

- **`is_file(path)`** - Check if path is a regular file

- **`is_dir(path)`** - Check if path is a directory

- **`is_symlink(path)`** - Check if path is a symbolic link

### Reading Files

- **`cat(path)`** - Read file as text
  ```rust
  let content = g.cat("/etc/hosts")?;
  println!("{}", content);
  ```

- **`read_file(path)`** - Read file as bytes
  ```rust
  let bytes = g.read_file("/bin/ls")?;
  ```

- **`read_lines(path)`** - Read file as lines
  ```rust
  let lines = g.read_lines("/etc/passwd")?;
  for line in lines {
      println!("{}", line);
  }
  ```

- **`head(path)`** / **`head_n(path, n)`** - Read first N lines
  ```rust
  let first_10 = g.head("/var/log/messages")?;
  let first_20 = g.head_n("/var/log/messages", 20)?;
  ```

- **`tail(path)`** / **`tail_n(path, n)`** - Read last N lines
  ```rust
  let last_10 = g.tail("/var/log/messages")?;
  ```

### Writing Files

- **`write(path, content)`** - Write content to file
  ```rust
  g.write("/tmp/test.txt", b"Hello, World!")?;
  ```

- **`write_append(path, content)`** - Append to file
  ```rust
  g.write_append("/tmp/test.txt", b"\nNew line")?;
  ```

### File Metadata

- **`stat(path)`** - Get file statistics
  ```rust
  let stat = g.stat("/etc/passwd")?;
  println!("Size: {}, Mode: {:o}", stat.size, stat.mode);
  ```

- **`filesize(path)`** - Get file size
  ```rust
  let size = g.filesize("/etc/passwd")?;
  ```

- **`chmod(mode, path)`** - Change file permissions
  ```rust
  g.chmod(0o644, "/tmp/file.txt")?;
  ```

- **`chown(owner, group, path)`** - Change file ownership
  ```rust
  g.chown(1000, 1000, "/tmp/file.txt")?;
  ```

### Directory Operations

- **`mkdir(path)`** - Create directory
  ```rust
  g.mkdir("/tmp/newdir")?;
  ```

- **`mkdir_p(path)`** - Create directory with parents
  ```rust
  g.mkdir_p("/tmp/path/to/dir")?;
  ```

- **`ls(directory)`** - List directory contents
  ```rust
  let files = g.ls("/")?;
  for file in files {
      println!("{}", file);
  }
  ```

- **`ll(directory)`** - Long listing format

- **`find(directory)`** - Find files recursively
  ```rust
  let files = g.find("/etc")?;
  ```

- **`du(path)`** - Calculate disk usage

### File Operations

- **`rm(path)`** - Remove file
  ```rust
  g.rm("/tmp/file.txt")?;
  ```

- **`rmdir(path)`** - Remove empty directory

- **`rm_rf(path)`** - Remove recursively (files and directories)

- **`touch(path)`** - Create empty file or update timestamp
  ```rust
  g.touch("/tmp/newfile")?;
  ```

- **`cp(src, dest)`** - Copy file
  ```rust
  g.cp("/etc/passwd", "/tmp/passwd.bak")?;
  ```

- **`cp_a(src, dest)`** - Copy with attributes

- **`cp_r(src, dest)`** - Copy recursively

- **`mv(src, dest)`** - Move/rename file
  ```rust
  g.mv("/tmp/old.txt", "/tmp/new.txt")?;
  ```

### File Transfer

- **`upload(local, remote)`** - Upload file from host to guest
  ```rust
  g.upload("/home/user/file.txt", "/tmp/file.txt")?;
  ```

- **`download(remote, local)`** - Download file from guest to host
  ```rust
  g.download("/etc/passwd", "/tmp/passwd")?;
  ```

### Search Operations

- **`grep(pattern, path)`** - Search for pattern in file
  ```rust
  let matches = g.grep("root", "/etc/passwd")?;
  ```

- **`egrep(pattern, path)`** - Extended regex grep

- **`fgrep(pattern, path)`** - Fixed string grep

- **`zgrep(pattern, path)`** - Search in compressed file

---

## Mount Operations

### Mounting Filesystems

- **`mount(device, mountpoint)`** - Mount filesystem read-write
  ```rust
  g.mount("/dev/sda1", "/")?;
  ```

- **`mount_ro(device, mountpoint)`** - Mount filesystem read-only
  ```rust
  g.mount_ro("/dev/sda1", "/")?;
  ```

- **`mount_options(options, device, mountpoint)`** - Mount with options
  ```rust
  g.mount_options("noatime,nodiratime", "/dev/sda1", "/")?;
  ```

- **`mount_vfs(vfstype, device, mountpoint)`** - Mount with explicit VFS type

### Unmounting

- **`umount(mountpoint)`** - Unmount filesystem
  ```rust
  g.umount("/")?;
  ```

- **`umount_all()`** - Unmount all filesystems
  ```rust
  g.umount_all()?;
  ```

### Mount Information

- **`mounts()`** - Get list of mounted filesystems
  ```rust
  let mounts = g.mounts()?;
  for mount in mounts {
      println!("{}", mount);
  }
  ```

- **`mountpoints()`** - Get mountpoint mapping

### Mountpoint Management

- **`mkmountpoint(path)`** - Create mountpoint directory

- **`rmmountpoint(path)`** - Remove mountpoint directory

---

## Archive and Compression

### TAR Operations

- **`tar_in(tarfile, directory)`** - Extract tar archive
  ```rust
  g.tar_in("/tmp/backup.tar", "/")?;
  ```

- **`tar_out(directory, tarfile)`** - Create tar archive
  ```rust
  g.tar_out("/home", "/tmp/home.tar")?;
  ```

- **`tgz_in(tarfile, directory)`** - Extract gzipped tar
  ```rust
  g.tgz_in("/tmp/backup.tar.gz", "/")?;
  ```

- **`tgz_out(directory, tarfile)`** - Create gzipped tar
  ```rust
  g.tgz_out("/home", "/tmp/home.tar.gz")?;
  ```

- **`tar_in_opts(tarfile, directory, compress)`** - Extract with options
  ```rust
  g.tar_in_opts("/tmp/backup.tar.xz", "/", Some("xz"))?;
  ```

- **`tar_out_opts(directory, tarfile, compress)`** - Create with options
  ```rust
  g.tar_out_opts("/home", "/tmp/home.tar.bz2", Some("bzip2"))?;
  ```

### CPIO Operations

- **`cpio_out(directory, cpiofile)`** - Create CPIO archive
  ```rust
  g.cpio_out("/etc", "/tmp/etc.cpio")?;
  ```

- **`cpio_in(cpiofile, directory)`** - Extract CPIO archive

### Compression Operations

- **`compress_out(format, file, compressed)`** - Compress file
  ```rust
  g.compress_out("gzip", "/large.txt", "/large.txt.gz")?;
  // Supported: gzip, bzip2, xz
  ```

- **`compress_in(format, compressed, file)`** - Decompress file
  ```rust
  g.compress_in("gzip", "/file.gz", "/file")?;
  ```

---

## Encryption (LUKS)

### Opening Encrypted Volumes

- **`luks_open(device, key, mapname)`** - Open LUKS device
  ```rust
  g.luks_open("/dev/sda1", "mypassword", "encrypted_data")?;
  // Now accessible as /dev/mapper/encrypted_data
  ```

- **`luks_open_ro(device, key, mapname)`** - Open LUKS device read-only

- **`luks_close(device)`** - Close LUKS device
  ```rust
  g.luks_close("/dev/mapper/encrypted_data")?;
  ```

### LUKS Management

- **`luks_format(device, key)`** - Format device as LUKS
  ```rust
  g.luks_format("/dev/sda1", "newpassword")?;
  ```

- **`luks_add_key(device, key, newkey)`** - Add encryption key
  ```rust
  g.luks_add_key("/dev/sda1", "oldpassword", "newpassword")?;
  ```

- **`luks_uuid(device)`** - Get LUKS UUID
  ```rust
  let uuid = g.luks_uuid("/dev/sda1")?;
  ```

---

## LVM Operations

### Volume Group Operations

- **`vgscan()`** - Scan for volume groups
  ```rust
  g.vgscan()?;
  ```

- **`vg_activate_all(activate)`** - Activate/deactivate all volume groups
  ```rust
  g.vg_activate_all(true)?;
  ```

- **`vg_activate(activate, vgnames)`** - Activate specific volume groups
  ```rust
  g.vg_activate(true, &["vg0"])?;
  ```

- **`vgs()`** - List volume groups
  ```rust
  let vgs = g.vgs()?;
  for vg in vgs {
      println!("VG: {}", vg);
  }
  ```

### Logical Volume Operations

- **`lvcreate(logvol, volgroup, mbytes)`** - Create logical volume
  ```rust
  g.lvcreate("data", "vg0", 1000)?; // 1000 MB
  ```

- **`lvremove(device)`** - Remove logical volume
  ```rust
  g.lvremove("/dev/vg0/data")?;
  ```

- **`lvs()`** - List logical volumes (simple)
  ```rust
  let lvs = g.lvs()?;
  ```

- **`lvs_full()`** - List logical volumes with details

### Physical Volume Operations

- **`pvs()`** - List physical volumes
  ```rust
  let pvs = g.pvs()?;
  ```

---

## Inspection and Detection

### OS Detection

- **`inspect_os()`** - Detect operating systems
  ```rust
  let roots = g.inspect_os()?;
  for root in roots {
      println!("Found OS at: {}", root);
  }
  ```

### OS Properties

- **`inspect_get_type(root)`** - Get OS type (linux/windows/freebsd/etc)
  ```rust
  let ostype = g.inspect_get_type("/dev/sda1")?;
  ```

- **`inspect_get_distro(root)`** - Get Linux distribution
  ```rust
  let distro = g.inspect_get_distro("/dev/sda1")?;
  // Returns: fedora, rhel, ubuntu, debian, etc.
  ```

- **`inspect_get_product_name(root)`** - Get OS product name

- **`inspect_get_arch(root)`** - Get architecture
  ```rust
  let arch = g.inspect_get_arch("/dev/sda1")?;
  // Returns: x86_64, i386, aarch64, etc.
  ```

- **`inspect_get_major_version(root)`** - Get OS major version

- **`inspect_get_minor_version(root)`** - Get OS minor version

- **`inspect_get_hostname(root)`** - Get hostname

- **`inspect_get_package_format(root)`** - Get package format (rpm/deb/etc)

### Mount Points and Applications

- **`inspect_get_mountpoints(root)`** - Get suggested mountpoints
  ```rust
  let mountpoints = g.inspect_get_mountpoints("/dev/sda1")?;
  for (mp, device) in mountpoints {
      g.mount(&device, &mp)?;
  }
  ```

- **`inspect_list_applications(root)`** - List installed applications
  ```rust
  let apps = g.inspect_list_applications("/dev/sda1")?;
  for app in apps {
      println!("{} {}", app.name, app.version);
  }
  ```

- **`inspect_is_live(root)`** - Check if live CD/DVD

---

## Command Execution

### Running Commands

- **`command(args)`** - Execute command in guest
  ```rust
  let output = g.command(&["ls", "-la", "/"])?;
  println!("{}", output);
  ```

- **`command_lines(args)`** - Execute command, return lines
  ```rust
  let lines = g.command_lines(&["cat", "/etc/passwd"])?;
  for line in lines {
      println!("{}", line);
  }
  ```

### Shell Execution

- **`sh(command)`** - Execute shell command
  ```rust
  let output = g.sh("df -h | grep /dev/sda")?;
  ```

- **`sh_lines(command)`** - Execute shell command, return lines
  ```rust
  let lines = g.sh_lines("ps aux | grep nginx")?;
  ```

---

## Network Configuration

- **`get_hostname()`** - Get system hostname
  ```rust
  let hostname = g.get_hostname()?;
  ```

- **`set_hostname(hostname)`** - Set system hostname
  ```rust
  g.set_hostname("myserver")?;
  ```

- **`list_network_interfaces()`** - List network interfaces
  ```rust
  let ifaces = g.list_network_interfaces()?;
  ```

- **`get_network_config(interface)`** - Get interface configuration

- **`read_etc_hosts()`** - Read /etc/hosts file

- **`get_dns()`** - Get DNS servers

---

## Package Management

- **`dpkg_list()`** - List Debian packages
  ```rust
  let packages = g.dpkg_list()?;
  for pkg in packages {
      println!("{}: {}", pkg.name, pkg.version);
  }
  ```

- **`rpm_list()`** - List RPM packages
  ```rust
  let packages = g.rpm_list()?;
  ```

- **`get_package_info(package)`** - Get package information

- **`is_package_installed(package)`** - Check if package is installed
  ```rust
  if g.is_package_installed("nginx")? {
      println!("nginx is installed");
  }
  ```

- **`package_files(package)`** - List files in package

---

## System Configuration

### Time and Locale

- **`get_timezone()`** - Get system timezone

- **`set_timezone(timezone)`** - Set system timezone
  ```rust
  g.set_timezone("America/New_York")?;
  ```

- **`get_locale()`** - Get system locale

- **`set_locale(locale)`** - Set system locale

### System Information

- **`get_osinfo()`** - Get OS information

- **`get_kernel_version()`** - Get kernel version

- **`get_uptime()`** - Get system uptime

- **`get_machine_id()`** - Get machine ID

---

## Security Operations

### SELinux

- **`get_selinux_enabled()`** - Check if SELinux is enabled

- **`selinux_relabel(force)`** - Relabel filesystem

### Extended Attributes and ACLs

- **`getxattr(path, name)`** - Get extended attribute
  ```rust
  let value = g.getxattr("/file", "user.comment")?;
  ```

- **`setxattr(name, val, path)`** - Set extended attribute
  ```rust
  g.setxattr("user.comment", "Important file", "/file")?;
  ```

- **`listxattr(path)`** - List extended attributes

- **`removexattr(path, name)`** - Remove extended attribute

---

## Advanced Topics

### Checksums

```rust
let md5 = g.checksum("md5", "/file")?;
let sha256 = g.checksum("sha256", "/file")?;
```

### Links

```rust
// Symbolic link
g.ln_s("/target", "/link")?;
let target = g.readlink("/link")?;

// Hard link
g.ln("/original", "/hardlink")?;
```

### File Type Detection

```rust
let filetype = g.file("/bin/ls")?;
let arch = g.file_architecture("/bin/ls")?;
```

---

## Error Handling

All functions return `Result<T, Error>`. Use `?` operator for propagation:

```rust
fn process_vm(path: &str) -> Result<(), Box<dyn std::error::Error>> {
    let mut g = Guestfs::new()?;
    g.add_drive_ro(path)?;
    g.launch()?;

    let roots = g.inspect_os()?;
    if roots.is_empty() {
        return Err("No OS found".into());
    }

    let hostname = g.inspect_get_hostname(&roots[0])?;
    println!("Hostname: {}", hostname);

    g.shutdown()?;
    Ok(())
}
```

---

## Complete Example

```rust
use guestctl::Guestfs;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize
    let mut g = Guestfs::new()?;
    g.set_verbose(true);

    // Add disk
    g.add_drive_ro("/path/to/vm.qcow2")?;
    g.launch()?;

    // Inspect OS
    let roots = g.inspect_os()?;
    let root = &roots[0];

    println!("OS Type: {}", g.inspect_get_type(root)?);
    println!("Distro: {}", g.inspect_get_distro(root)?);
    println!("Hostname: {}", g.inspect_get_hostname(root)?);

    // Mount filesystem
    let mountpoints = g.inspect_get_mountpoints(root)?;
    for (mp, dev) in mountpoints {
        g.mount(&dev, &mp)?;
    }

    // Read files
    let passwd = g.cat("/etc/passwd")?;
    println!("Users: {}", passwd.lines().count());

    // List installed apps
    let apps = g.inspect_list_applications(root)?;
    println!("Installed packages: {}", apps.len());

    // Cleanup
    g.umount_all()?;
    g.shutdown()?;

    Ok(())
}
```

---

## See Also

- [GUESTFS_IMPLEMENTATION_STATUS.md](GUESTFS_IMPLEMENTATION_STATUS.md) - Implementation status
- [CHANGELOG.md](CHANGELOG.md) - Version history
- [README.md](README.md) - Project overview
- [examples/](examples/) - Example programs
