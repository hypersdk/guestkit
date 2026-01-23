# GuestCtl CLI Guide

`guestctl` is a command-line tool for inspecting and manipulating virtual machine disk images without mounting them.

## Installation

```bash
# Build from source
cargo build --release

# Binary location
./target/release/guestctl

# Install globally (optional)
cargo install --path .
```

## Quick Start

```bash
# Inspect a disk image
sudo guestctl inspect ubuntu.qcow2

# List filesystems
sudo guestctl filesystems ubuntu.qcow2

# List packages
sudo guestctl packages ubuntu.qcow2

# Copy a file
sudo guestctl cp ubuntu.qcow2:/etc/passwd ./passwd

# List directory
sudo guestctl ls ubuntu.qcow2 /etc

# Read file
sudo guestctl cat ubuntu.qcow2 /etc/hostname
```

## Commands

### `inspect` - OS Information

Detect and display operating system information from a disk image.

**Usage:**
```bash
guestctl inspect [OPTIONS] <DISK>
```

**Options:**
- `-j, --json` - Output as JSON for scripting

**Examples:**
```bash
# Human-readable output
sudo guestctl inspect ubuntu.qcow2

# JSON output for jq processing
sudo guestctl inspect --json ubuntu.qcow2 | jq '.operating_systems[0].distro'
```

**Output:**
```
=== Disk Image: ubuntu.qcow2 ===

Found 1 operating system(s):

OS #1
  Root device: /dev/sda2
  Type: linux
  Distribution: ubuntu
  Version: 22.4
  Product: Ubuntu 22.04 LTS
  Hostname: webserver-01
  Architecture: x86_64
  Package format: deb
  Package management: apt
```

**JSON Output:**
```json
{
  "disk": "ubuntu.qcow2",
  "os_count": 1,
  "operating_systems": [
    {
      "root": "/dev/sda2",
      "type": "linux",
      "distro": "ubuntu",
      "major_version": 22,
      "minor_version": 4,
      "product_name": "Ubuntu 22.04 LTS",
      "hostname": "webserver-01",
      "arch": "x86_64",
      "package_format": "deb",
      "package_management": "apt"
    }
  ]
}
```

---

### `filesystems` - List Storage

List all devices, partitions, and filesystems in a disk image.

**Usage:**
```bash
guestctl filesystems [OPTIONS] <DISK>
```

**Options:**
- `-d, --detailed` - Show detailed information (UUIDs, partition numbers)

**Examples:**
```bash
# Basic listing
sudo guestctl filesystems ubuntu.qcow2

# Detailed information
sudo guestctl filesystems --detailed ubuntu.qcow2
```

**Output:**
```
=== Devices ===
/dev/sda
  Size: 21474836480 bytes (21.47 GB)
  Partition table: gpt

=== Partitions ===
/dev/sda1
  Size: 536870912 bytes (0.54 GB)
  Filesystem: vfat
  Label: EFI

/dev/sda2
  Size: 20937965568 bytes (20.94 GB)
  Filesystem: ext4
  Label: rootfs
  UUID: a1b2c3d4-e5f6-7890-abcd-ef1234567890

=== LVM Volume Groups ===
(none)

=== LVM Logical Volumes ===
(none)
```

---

### `packages` - List Installed Software

List all installed packages from a disk image.

**Usage:**
```bash
guestctl packages [OPTIONS] <DISK>
```

**Options:**
- `-f, --filter <NAME>` - Filter packages by name
- `-l, --limit <N>` - Limit number of results
- `-j, --json` - Output as JSON

**Examples:**
```bash
# List all packages
sudo guestctl packages ubuntu.qcow2

# Find nginx packages
sudo guestctl packages --filter nginx ubuntu.qcow2

# Show first 20 packages
sudo guestctl packages --limit 20 ubuntu.qcow2

# JSON output
sudo guestctl packages --json ubuntu.qcow2 | jq '.packages[] | select(.name | contains("kernel"))'
```

**Output:**
```
Found 1847 package(s)

Package                                  Version              Release
----------------------------------------------------------------------------------
accountsservice                          0.6.55               0ubuntu12
adduser                                  3.118                ubuntu2
apache2                                  2.4.52               1ubuntu4
...
```

---

### `cp` - Copy Files

Copy files from a disk image to the local filesystem.

**Usage:**
```bash
guestctl cp <SOURCE> <DEST>
```

**Source Format:** `disk.img:/path/to/file`

**Examples:**
```bash
# Copy passwd file
sudo guestctl cp ubuntu.qcow2:/etc/passwd ./passwd

# Copy nginx config
sudo guestctl cp ubuntu.qcow2:/etc/nginx/nginx.conf ./nginx.conf

# Copy log file
sudo guestctl cp ubuntu.qcow2:/var/log/syslog ./syslog
```

**Output:**
```
✓ Copied ubuntu.qcow2:/etc/passwd -> ./passwd
```

---

### `ls` - List Directory

List files and directories inside a disk image.

**Usage:**
```bash
guestctl ls [OPTIONS] <DISK> [PATH]
```

**Options:**
- `-l, --long` - Long listing format (like `ls -l`)

**Examples:**
```bash
# List root directory
sudo guestctl ls ubuntu.qcow2 /

# List /etc
sudo guestctl ls ubuntu.qcow2 /etc

# Long format
sudo guestctl ls --long ubuntu.qcow2 /etc
```

**Output (basic):**
```
bin
boot
dev
etc
home
lib
...
```

**Output (long format):**
```
drwxr-xr-x   2 root root     4096 Jan 15 10:30 bin
drwxr-xr-x   3 root root     4096 Jan 15 10:31 boot
drwxr-xr-x  16 root root     3200 Jan 22 08:15 dev
drwxr-xr-x  95 root root     4096 Jan 22 09:22 etc
...
```

---

### `cat` - Read Files

Display the contents of a file from a disk image.

**Usage:**
```bash
guestctl cat <DISK> <PATH>
```

**Examples:**
```bash
# Read hostname
sudo guestctl cat ubuntu.qcow2 /etc/hostname

# Read OS release
sudo guestctl cat ubuntu.qcow2 /etc/os-release

# Read systemd service
sudo guestctl cat ubuntu.qcow2 /etc/systemd/system/myapp.service
```

**Output:**
```
$ sudo guestctl cat ubuntu.qcow2 /etc/hostname
webserver-01

$ sudo guestctl cat ubuntu.qcow2 /etc/os-release
NAME="Ubuntu"
VERSION="22.04 LTS (Jammy Jellyfish)"
ID=ubuntu
ID_LIKE=debian
...
```

---

## Global Options

### Verbose Mode

Enable detailed logging for debugging.

```bash
# Any command with verbose output
sudo guestctl -v inspect ubuntu.qcow2
sudo guestctl --verbose filesystems ubuntu.qcow2
```

**Verbose Output:**
```
Launching appliance...
Mounting filesystems...
  Mounted /dev/sda2 at /
  Mounted /dev/sda1 at /boot
...
```

---

## Scripting with JSON

Many commands support `--json` output for easy parsing with `jq`:

### Extract specific information:

```bash
# Get OS type
sudo guestctl inspect --json disk.img | jq -r '.operating_systems[0].type'

# Get hostname
sudo guestctl inspect --json disk.img | jq -r '.operating_systems[0].hostname'

# Count packages
sudo guestctl packages --json disk.img | jq '.total'

# Filter packages by name
sudo guestctl packages --json disk.img | jq '.packages[] | select(.name | contains("python"))'
```

### Batch processing:

```bash
# Inspect multiple disks
for disk in *.qcow2; do
    echo "=== $disk ==="
    sudo guestctl inspect --json "$disk" | jq -r '.operating_systems[0].distro'
done

# Generate report
sudo guestctl inspect --json disk.img | jq '{
    hostname: .operating_systems[0].hostname,
    os: .operating_systems[0].distro,
    version: .operating_systems[0].version
}'
```

---

## Requirements

- **Root/sudo access** - Most operations require root privileges
- **libguestfs** - System dependencies (qemu-img, etc.)
- **Disk formats** - Supports QCOW2, VMDK, RAW, VDI, VHD, and more

---

## Common Use Cases

### 1. Pre-deployment Verification

```bash
# Check OS version before deploying VM
sudo guestctl inspect --json template.qcow2 | jq -r '.operating_systems[0].version'

# Verify hostname is set correctly
sudo guestctl cat template.qcow2 /etc/hostname
```

### 2. Security Audit

```bash
# List all installed packages
sudo guestctl packages disk.img > installed-packages.txt

# Check for specific vulnerable packages
sudo guestctl packages --filter openssl disk.img

# Extract SSH keys
sudo guestctl cp disk.img:/root/.ssh/authorized_keys ./authorized_keys
```

### 3. Troubleshooting

```bash
# Check system logs without booting VM
sudo guestctl cat disk.img /var/log/syslog > syslog.txt

# List running services configuration
sudo guestctl ls --long disk.img /etc/systemd/system

# Extract config files for analysis
sudo guestctl cp disk.img:/etc/nginx/nginx.conf ./nginx.conf
```

### 4. Inventory Management

```bash
# Generate inventory report
for vm in /var/lib/libvirt/images/*.qcow2; do
    hostname=$(sudo guestctl inspect --json "$vm" | jq -r '.operating_systems[0].hostname')
    distro=$(sudo guestctl inspect --json "$vm" | jq -r '.operating_systems[0].distro')
    version=$(sudo guestctl inspect --json "$vm" | jq -r '.operating_systems[0].version')
    echo "$vm,$hostname,$distro,$version"
done > vm-inventory.csv
```

### 5. Backup Verification

```bash
# Verify backup contains expected files
sudo guestctl ls backup.img /home/user/important-data

# Extract specific files from backup
sudo guestctl cp backup.img:/home/user/document.pdf ./recovered-document.pdf
```

---

## Tips & Best Practices

### 1. Always Use Read-Only Operations

The CLI automatically uses read-only mode for all operations except when explicitly writing. This prevents accidental modifications.

### 2. Use JSON for Automation

When scripting, always use `--json` output and parse with `jq`:

```bash
# Good: Robust parsing
distro=$(sudo guestctl inspect --json disk.img | jq -r '.operating_systems[0].distro')

# Bad: Fragile text parsing
distro=$(sudo guestctl inspect disk.img | grep "Distribution:" | cut -d: -f2)
```

### 3. Filter Before Limiting

When working with packages, filter first, then limit:

```bash
# Good: Get all kernel packages
sudo guestctl packages --filter kernel disk.img

# Less useful: Random 20 packages
sudo guestctl packages --limit 20 disk.img
```

### 4. Check for Errors

Always check exit codes in scripts:

```bash
if sudo guestctl inspect disk.img > /dev/null 2>&1; then
    echo "Disk is valid"
else
    echo "Error: Invalid disk image"
    exit 1
fi
```

---

## Troubleshooting

### Permission Denied

**Problem:** `Error: permission denied`

**Solution:** Run with sudo:
```bash
sudo guestctl inspect disk.img
```

### Appliance Failed to Launch

**Problem:** `Error: Failed to launch appliance`

**Solutions:**
1. Check KVM is available:
   ```bash
   ls -l /dev/kvm
   sudo usermod -aG kvm $USER
   ```

2. Enable verbose mode to see details:
   ```bash
   sudo guestctl -v inspect disk.img
   ```

3. Ensure qemu-img is installed:
   ```bash
   sudo apt-get install qemu-utils
   ```

### No OS Detected

**Problem:** `⚠️  No operating systems detected`

**Possible causes:**
- Disk is not bootable
- Disk is encrypted (LUKS)
- Unsupported OS type
- Corrupted disk image

**Check:**
```bash
# Check filesystem types
sudo guestctl filesystems disk.img

# Verify disk is readable
qemu-img info disk.img
```

### File Not Found

**Problem:** `Error: File not found: /path/to/file`

**Solutions:**
1. Verify file exists:
   ```bash
   sudo guestctl ls disk.img /path/to
   ```

2. Check filesystem is mounted:
   ```bash
   sudo guestctl -v cat disk.img /path/to/file
   ```

---

## Performance Tips

### 1. Use JSON Output for Large Results

JSON parsing is faster than human-readable output for large datasets:

```bash
# Faster for 1000+ packages
sudo guestctl packages --json disk.img | jq '.total'
```

### 2. Limit Results Early

Use `--limit` to avoid processing unnecessary data:

```bash
# Only need 10 results
sudo guestctl packages --limit 10 disk.img
```

### 3. Filter Efficiently

Combine filter and limit for best performance:

```bash
sudo guestctl packages --filter nginx --limit 5 disk.img
```

---

## Integration Examples

### Ansible Playbook

```yaml
---
- name: Inspect VM disk image
  hosts: localhost
  tasks:
    - name: Get OS information
      shell: guestctl inspect --json /var/lib/libvirt/images/vm.qcow2
      register: vm_info
      become: yes

    - name: Parse OS details
      set_fact:
        os_type: "{{ (vm_info.stdout | from_json).operating_systems[0].type }}"
        hostname: "{{ (vm_info.stdout | from_json).operating_systems[0].hostname }}"

    - name: Display info
      debug:
        msg: "VM {{ hostname }} is running {{ os_type }}"
```

### Shell Script

```bash
#!/bin/bash
# vm-audit.sh - Audit all VM disks

for disk in /var/lib/libvirt/images/*.qcow2; do
    echo "Auditing: $disk"

    # Get OS info
    info=$(sudo guestctl inspect --json "$disk")

    hostname=$(echo "$info" | jq -r '.operating_systems[0].hostname // "unknown"')
    distro=$(echo "$info" | jq -r '.operating_systems[0].distro // "unknown"')

    # Count packages
    pkg_count=$(sudo guestctl packages --json "$disk" | jq '.total')

    echo "  Hostname: $hostname"
    echo "  OS: $distro"
    echo "  Packages: $pkg_count"
    echo
done
```

---

## See Also

- **[Python Bindings](PYTHON_BINDINGS.md)** - Use GuestKit from Python
- **[Ergonomic API](ERGONOMIC_API.md)** - Type-safe Rust API
- **[Quick Wins](QUICK_WINS.md)** - Implementation guide
- **[Enhancement Roadmap](ENHANCEMENT_ROADMAP.md)** - Future features

---

**Built with GuestKit** 🚀
