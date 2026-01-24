# GuestCtl Inspection Quick Reference

## 🚀 Quick Start

```bash
# Basic inspection
guestctl inspect disk.qcow2

# Verbose mode (detailed logging)
guestctl inspect disk.qcow2 --verbose
guestctl inspect disk.qcow2 -v
```

## 📋 What Gets Inspected

| Category | Information Extracted |
|----------|----------------------|
| **Hardware** | Block devices, partitions, sector sizes, partition schemes |
| **OS** | Type, distribution, version, hostname, architecture |
| **Disk** | Usage, free space, filesystems, labels, UUIDs |
| **Packages** | Package format, manager, installed count |
| **Kernels** | Installed kernel versions in /boot |
| **Network** | Interfaces, IPs, MAC addresses, DHCP, DNS servers |
| **Users** | Regular users, system users, shells, home dirs |
| **SSH** | Port, root login, password auth settings |
| **Security** | SELinux status and mode |
| **Services** | Enabled systemd services, timers |
| **Boot** | Bootloader (GRUB2), timeout, default entry |
| **Storage** | LVM (PVs/VGs/LVs), swap, fstab mounts |
| **Runtimes** | Python, Node.js, Ruby, Java, Go, Perl |
| **Containers** | Docker, Podman, containerd, CRI-O |
| **Tasks** | Cron jobs, systemd timers |
| **Certs** | SSL/TLS certificates in standard locations |
| **Tuning** | Kernel parameters (sysctl.conf) |
| **Cloud** | Cloud-init detection |
| **VM Tools** | VMware Tools, QEMU GA, VirtualBox, Hyper-V |
| **Config** | Timezone, locale |

## 🎯 Common Commands

```bash
# Inspect a QCOW2 image
guestctl inspect vm-disk.qcow2

# Inspect with verbose logging
guestctl inspect vm-disk.qcow2 -v 2>verbose.log

# Inspect a RAW disk
guestctl inspect disk.img

# Inspect and save output
guestctl inspect disk.qcow2 > inspection-report.txt

# Inspect with verbose to separate files
guestctl inspect disk.qcow2 -v >report.txt 2>debug.log
```

## 📊 Sample Output Sections

```
=== Block Devices ===
  /dev/sda: 21474836480 bytes (21.47 GB)
    Read-only: no
    Sector size: 512 bytes

=== Partitions ===
  /dev/sda1
    Number: 1
    Start:  1048576 bytes
    Size:   21473787904 bytes (21.47 GB)

=== Operating Systems ===
  Root: /dev/sda1
    Type:         linux
    Distribution: fedora
    Product:      Fedora Linux
    Version:      39.0
    Hostname:     fedora-server
    Init system:  systemd
    Pkg Manager:  dnf

    Disk usage:
      Total: 20.00 GB
      Used:  8.50 GB (42.5%)
      Free:  11.50 GB

    === Network Configuration ===
      Interface: eth0
        IP: 192.168.1.100
        DHCP: no

    === User Accounts ===
      Regular users: 2
        john (uid: 1000)
        jane (uid: 1001)

    === Language Runtimes ===
      python3: installed
      nodejs: installed

    === Container Runtimes ===
      docker
      podman
```

## 🔍 Verbose Output Examples

```bash
$ guestctl inspect disk.qcow2 -v

[VERBOSE] Adding drive: disk.qcow2
[VERBOSE] Launching QEMU appliance...
[VERBOSE] Enumerating block devices...
[VERBOSE] Found device: /dev/sda (21474836480 bytes)
[VERBOSE] Analyzing partition table...
[VERBOSE] Examining partition: /dev/sda1
[VERBOSE] Partition scheme: gpt
[VERBOSE] Detecting filesystems...
[VERBOSE] Filesystem on /dev/sda1: ext4
[VERBOSE] Running OS detection algorithms...
[VERBOSE] Inspecting OS at root: /dev/sda1
[VERBOSE] OS type detected: linux
[VERBOSE] Distribution: fedora
[VERBOSE] Gathering system configuration...
[VERBOSE] Analyzing network configuration...
[VERBOSE] Listing user accounts...
[VERBOSE] Detecting language runtimes...
[VERBOSE] Shutting down appliance...
[VERBOSE] Inspection complete
```

## 💡 Pro Tips

### 1. Filter Specific Information
```bash
# Get only network info
guestctl inspect disk.qcow2 | grep -A 20 "Network Configuration"

# Get only user accounts
guestctl inspect disk.qcow2 | grep -A 30 "User Accounts"

# Get OS summary
guestctl inspect disk.qcow2 | grep -A 15 "Operating Systems"
```

### 2. Compare Two VMs
```bash
# Inspect both and compare
guestctl inspect vm1.qcow2 > vm1-report.txt
guestctl inspect vm2.qcow2 > vm2-report.txt
diff vm1-report.txt vm2-report.txt
```

### 3. Extract Specific Data
```bash
# Get hostname
guestctl inspect disk.qcow2 | grep "Hostname:"

# Get installed kernels
guestctl inspect disk.qcow2 | grep -A 5 "Installed kernels"

# Get enabled services
guestctl inspect disk.qcow2 | grep -A 20 "Systemd Services"
```

### 4. Debugging Issues
```bash
# Full verbose output for troubleshooting
guestctl inspect problematic.qcow2 -v 2>&1 | tee full-debug.log

# Check what failed
guestctl inspect disk.qcow2 -v 2>&1 | grep -i "error\|failed"
```

### 5. Automation
```bash
# Inspect all QCOW2 files in directory
for img in *.qcow2; do
    echo "=== $img ==="
    guestctl inspect "$img"
    echo ""
done > all-vms-report.txt
```

## 🎨 Output Formatting Tips

### Create Summary Report
```bash
#!/bin/bash
DISK=$1
echo "VM Inspection Report"
echo "===================="
echo "Date: $(date)"
echo "Disk: $DISK"
echo ""

guestctl inspect "$DISK" | grep -E "(Root:|Type:|Distribution:|Product:|Version:|Hostname:|Disk usage:)"
```

### Extract JSON-like Data (with jq-style parsing)
```bash
# Get OS info as key-value pairs
guestctl inspect disk.qcow2 | grep -A 10 "Operating Systems" | grep ":" | sed 's/^[[:space:]]*//'
```

## 🔧 Programmatic Usage (Rust)

```rust
use guestctl::guestfs::Guestfs;

fn inspect_vm(path: &str) -> Result<(), Box<dyn std::error::Error>> {
    let mut g = Guestfs::new()?;
    g.add_drive_ro(path)?;
    g.launch()?;

    let roots = g.inspect_os()?;
    for root in &roots {
        // Basic info
        println!("OS: {}", g.inspect_get_product_name(root)?);
        println!("Hostname: {}", g.inspect_get_hostname(root)?);

        // Network
        let interfaces = g.inspect_network(root)?;
        for iface in &interfaces {
            println!("Interface {}: {:?}", iface.name, iface.ip_address);
        }

        // Users
        let users = g.inspect_users(root)?;
        println!("User count: {}", users.len());

        // Services
        let services = g.inspect_systemd_services(root)?;
        println!("Enabled services: {}", services.len());

        // Runtimes
        let runtimes = g.inspect_runtimes(root)?;
        for (name, version) in runtimes {
            println!("Runtime: {} ({})", name, version);
        }
    }

    g.shutdown()?;
    Ok(())
}
```

## 📚 Related Commands

```bash
# List files in VM
guestctl list disk.qcow2 /etc

# Extract file from VM
guestctl extract disk.qcow2 /etc/hostname hostname.txt

# Execute command in VM (if supported)
guestctl exec disk.qcow2 cat /etc/os-release

# Check filesystem
guestctl fsck disk.qcow2

# Show disk usage
guestctl df disk.qcow2
```

## ⚡ Performance Tips

1. **Use SSD**: Store disk images on SSD for faster inspection
2. **Verbose mode**: Only use when debugging (adds overhead)
3. **Local files**: Inspect local files rather than network-mounted
4. **Read-only**: Inspection is always read-only and safe

## 🐛 Troubleshooting

| Issue | Solution |
|-------|----------|
| "No OS found" | Check if disk has a bootable OS partition |
| "Permission denied" | Run with appropriate permissions or use sudo |
| "Unsupported format" | Check if disk format is supported (QCOW2, RAW, etc.) |
| Missing info | Some info requires OS-specific files; may not exist |
| Slow performance | Check disk I/O, use SSD, ensure enough memory |

## 🎓 Learn More

- Full documentation: `ENHANCED_INSPECTION.md`
- Implementation details: `ENHANCEMENTS_SUMMARY.md`
- API reference: `cargo doc --open`

## ✨ What's New

All these features are brand new in the enhanced inspection:
- ✅ Network configuration analysis
- ✅ User account enumeration
- ✅ SSH configuration inspection
- ✅ SELinux status
- ✅ Language runtime detection
- ✅ Container runtime detection
- ✅ LVM analysis
- ✅ Boot configuration
- ✅ Scheduled tasks (cron, timers)
- ✅ SSL certificate discovery
- ✅ Kernel parameter inspection
- ✅ VM tools detection
- ✅ Cloud-init detection
- ✅ Comprehensive verbose logging

Happy inspecting! 🎉
