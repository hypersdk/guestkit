# Enhanced Guest Inspection Features

This document describes all the comprehensive inspection enhancements added to GuestCtl, making it a powerful tool for deep VM and disk image analysis comparable to guestfish and virt-inspector.

## 🎯 Overview

The enhanced inspection system provides detailed information about guest operating systems across 14 major categories, extracting comprehensive system configuration, security settings, network details, user accounts, and much more.

## 📊 Inspection Categories

### 1. **Block Devices & Partitions**
- Block device information with sizes
- Read-only status
- Sector sizes
- Partition scheme detection (GPT, MBR, etc.)
- Partition details (number, start, size, end positions)

### 2. **Operating System Information**
- OS type (Linux, Windows, etc.)
- Distribution (Fedora, Ubuntu, RHEL, Photon, etc.)
- Product name and variant
- Architecture
- Version (major.minor)
- Hostname
- Package format (rpm, deb)
- Init system (systemd, upstart, sysvinit)
- Package manager (tdnf, dnf, yum, apt, dpkg)
- Installation format

### 3. **System Configuration**
- **Timezone** - System timezone setting
- **Locale** - Language and locale configuration
- **SELinux** - SELinux status and mode
- **Cloud-init** - Cloud-init presence detection
- **VM Guest Tools** - Detection of:
  - VMware Tools
  - QEMU Guest Agent
  - VirtualBox Guest Additions
  - Hyper-V tools

### 4. **Network Configuration**
- **Network Interfaces**:
  - Interface names
  - IP addresses (static/DHCP)
  - MAC addresses
  - DHCP vs static configuration
- **DNS Configuration**:
  - DNS servers from `/etc/resolv.conf`
- **Supported formats**:
  - Debian/Ubuntu (`/etc/network/interfaces`)
  - RHEL/Fedora (`/etc/sysconfig/network-scripts/`)
  - Netplan (modern Ubuntu)

### 5. **User Accounts & Security**
- **User Accounts**:
  - Regular users (UID ≥ 1000)
  - System users (UID < 1000)
  - User details (UID, GID, home directory, shell)
- **SSH Configuration**:
  - SSH port
  - PermitRootLogin setting
  - PasswordAuthentication setting
  - Other sshd_config parameters

### 6. **System Services**
- **Systemd Services**:
  - List of enabled services
  - Service count
  - Services from `/etc/systemd/system/`
- **Systemd Timers**:
  - Active timer units
  - Scheduled systemd tasks

### 7. **Storage Configuration**
- **LVM Detection**:
  - Physical Volumes (PVs)
  - Volume Groups (VGs)
  - Logical Volumes (LVs)
- **Swap Configuration**:
  - Swap devices from `/etc/fstab`
- **Filesystem Mounts**:
  - All mounts from `/etc/fstab`
  - Device, mountpoint, and filesystem type

### 8. **Boot Configuration**
- **Bootloader** - GRUB2 detection
- **Boot timeout** - Default boot timeout
- **Default entry** - Default boot option
- **Kernel information**:
  - Installed kernel versions
  - Kernel files in `/boot`

### 9. **Disk Usage & Filesystem Info**
- Total disk space
- Used space (GB and percentage)
- Free space
- Filesystem labels and UUIDs
- Package counts (RPM/DEB)

### 10. **Language Runtimes**
Detection of installed programming language runtimes:
- Python (python, python2, python3)
- Node.js
- Ruby
- Java
- Go
- Perl

### 11. **Container Runtimes**
Detection of container platforms:
- Docker
- Podman
- containerd
- CRI-O

### 12. **Scheduled Tasks**
- **Cron Jobs**:
  - System crontab entries
  - Jobs in `/etc/cron.d`, `/etc/cron.{daily,hourly,weekly,monthly}`
- **Systemd Timers**:
  - Enabled timer units

### 13. **SSL Certificates**
- Certificate discovery in:
  - `/etc/ssl/certs`
  - `/etc/pki/tls/certs`
  - `/etc/pki/ca-trust`
- Count and list of `.crt` and `.pem` files

### 14. **Kernel Parameters**
- Kernel tuning parameters from `/etc/sysctl.conf`
- Sorted parameter display
- Key-value pairs for system tuning

## 🔍 Verbose Mode

When running with `--verbose` or `-v` flag, the inspection provides extensive logging:

```bash
[VERBOSE] Adding drive: disk.qcow2
[VERBOSE] Launching QEMU appliance...
[VERBOSE] Enumerating block devices...
[VERBOSE] Analyzing partition table...
[VERBOSE] Detecting filesystems...
[VERBOSE] Running OS detection algorithms...
[VERBOSE] Gathering system configuration...
[VERBOSE] Analyzing network configuration...
[VERBOSE] Listing user accounts...
[VERBOSE] Checking SSH configuration...
[VERBOSE] Listing systemd services...
[VERBOSE] Detecting language runtimes...
[VERBOSE] Detecting container runtimes...
[VERBOSE] Analyzing storage configuration...
[VERBOSE] Analyzing boot configuration...
[VERBOSE] Checking scheduled tasks...
[VERBOSE] Scanning SSL certificates...
[VERBOSE] Reading kernel parameters...
[VERBOSE] Shutting down appliance...
[VERBOSE] Inspection complete
```

## 📝 Usage

```bash
# Basic inspection
guestctl inspect disk.qcow2

# Verbose inspection with detailed logging
guestctl inspect disk.qcow2 --verbose

# Short form
guestctl inspect disk.qcow2 -v
```

## 🔧 Example Output

```
=== Operating Systems ===
  Root: /dev/sda1
    Type:         linux
    Distribution: fedora
    Product:      Fedora Linux
    Architecture: x86_64
    Version:      39.0
    Hostname:     fedora-server
    Packages:     rpm
    Init system:  systemd
    Pkg Manager:  dnf
    Format:       installed

    Disk usage:
      Total: 20.00 GB
      Used:  8.50 GB (42.5%)
      Free:  11.50 GB

    Installed RPM packages: 1247

    Installed kernels:
      vmlinuz-6.5.6-300.fc39.x86_64
      vmlinuz-6.6.8-200.fc39.x86_64

    === System Configuration ===
      Timezone: America/New_York
      Locale:   en_US.UTF-8
      SELinux:  enforcing
      Cloud-init: yes
      VM Tools: qemu-guest-agent

    === Network Configuration ===
      Interface: eth0
        IP: 192.168.1.100
        MAC: 52:54:00:12:34:56
        DHCP: no
      DNS Servers: 8.8.8.8, 8.8.4.4

    === User Accounts ===
      Regular users: 2
        john (uid: 1000, home: /home/john)
        jane (uid: 1001, home: /home/jane)
      System users: 42

    === SSH Configuration ===
      Port: 22
      PermitRootLogin: no
      PasswordAuthentication: yes

    === Systemd Services ===
      Enabled services: 35
        sshd
        firewalld
        chronyd
        NetworkManager
        ... and 31 more

    === Language Runtimes ===
      python3: installed
      python: installed
      nodejs: installed
      java: installed

    === Container Runtimes ===
      podman
      docker

    === LVM Configuration ===
      Volume Groups: vg_main
      Logical Volumes: lv_root, lv_home

    === Swap Configuration ===
      /dev/vg_main/lv_swap

    === Filesystem Mounts (fstab) ===
      /dev/vg_main/lv_root on / type ext4
      /dev/vg_main/lv_home on /home type ext4
      /dev/sda1 on /boot type ext4

    === Boot Configuration ===
      Bootloader: GRUB2
      Timeout: 5
      Default: 0

    === Cron Jobs ===
      Total: 8
        0 2 * * * /usr/bin/backup.sh
        */15 * * * * /usr/bin/check-health.sh
        ... and 6 more

    === SSL Certificates ===
      Found: 156 certificates
        /etc/ssl/certs/ca-bundle.crt
        /etc/pki/tls/certs/localhost.crt
        ... and 154 more

    === Kernel Parameters (sysctl) ===
      Total: 42
        kernel.pid_max = 4194304
        net.ipv4.ip_forward = 1
        vm.swappiness = 60
        ... and 39 more
```

## 🎯 API Functions

All inspection functions are available programmatically:

```rust
use guestctl::guestfs::Guestfs;

let mut g = Guestfs::new()?;
g.add_drive_ro("/path/to/disk.qcow2")?;
g.launch()?;

let roots = g.inspect_os()?;
for root in &roots {
    // Basic info
    let os_type = g.inspect_get_type(root)?;
    let distro = g.inspect_get_distro(root)?;
    let hostname = g.inspect_get_hostname(root)?;

    // Enhanced info
    let timezone = g.inspect_timezone(root)?;
    let locale = g.inspect_locale(root)?;
    let selinux = g.inspect_selinux(root)?;

    // Network
    let interfaces = g.inspect_network(root)?;
    let dns_servers = g.inspect_dns(root)?;

    // Users
    let users = g.inspect_users(root)?;

    // SSH
    let ssh_config = g.inspect_ssh_config(root)?;

    // Services
    let services = g.inspect_systemd_services(root)?;

    // Runtimes
    let runtimes = g.inspect_runtimes(root)?;
    let containers = g.inspect_container_runtimes(root)?;

    // Storage
    let lvm_info = g.inspect_lvm(root)?;
    let swap = g.inspect_swap(root)?;
    let fstab = g.inspect_fstab(root)?;

    // Boot
    let boot_config = g.inspect_boot_config(root)?;

    // Scheduled tasks
    let cron_jobs = g.inspect_cron(root)?;
    let timers = g.inspect_systemd_timers(root)?;

    // Certificates
    let certs = g.inspect_certificates(root)?;

    // Kernel
    let kernel_params = g.inspect_kernel_params(root)?;

    // VM tools
    let vm_tools = g.inspect_vm_tools(root)?;

    // Cloud
    let has_cloud_init = g.inspect_cloud_init(root)?;
}

g.shutdown()?;
```

## 🔐 Security Analysis Use Cases

1. **Security Auditing**:
   - Check SSH configuration (root login, password auth)
   - Review SELinux status
   - Audit user accounts
   - Inspect SSL certificates

2. **Compliance Checking**:
   - Verify required services are enabled
   - Check kernel parameters meet requirements
   - Validate network configuration

3. **Migration Planning**:
   - Inventory installed packages and runtimes
   - Identify dependencies (LVM, network config)
   - Document system configuration

4. **Forensics**:
   - Analyze user accounts and activities
   - Review scheduled tasks
   - Examine network configuration

## 🚀 Performance

The enhanced inspection is designed to be efficient:
- Parallel information gathering where possible
- Minimal filesystem mounts
- Cached reads for related data
- Smart filtering to display only relevant information

## 📚 Compatibility

Works with:
- **Linux distributions**: Fedora, RHEL, CentOS, Ubuntu, Debian, Photon OS, and more
- **Disk formats**: QCOW2, RAW, VMDK, VDI
- **Filesystems**: ext2/3/4, XFS, Btrfs, NTFS
- **Partition schemes**: GPT, MBR

## 🔧 Future Enhancements

Potential additions:
- Windows Registry inspection
- Performance metrics extraction
- Log file analysis
- Package vulnerability scanning
- Configuration drift detection
- Cloud provider metadata extraction
