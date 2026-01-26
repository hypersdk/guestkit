# Windows Support Guide

Comprehensive guide for working with Windows VMs in guestctl.

## Overview

guestctl v0.3.1+ provides extensive Windows support including:
- **Registry Parsing** - Direct Windows registry access for version detection
- **Full Version Detection** - Accurate Windows edition and build identification
- **Driver Management** - VirtIO driver injection for KVM migration
- **Configuration Management** - Modify Windows settings without booting
- **User Account Management** - List and inspect Windows users
- **Service Detection** - Identify Windows services and startup programs

## Windows Version Detection

### Automatic Detection

guestctl automatically detects Windows versions through registry parsing:

```bash
# Inspect Windows VM
sudo guestctl inspect windows.qcow2
```

**Output:**
```
┌────────────────────────────────────────────────────────┐
│ Windows 11 Pro                                          │
│ Type: windows | Arch: x86_64 | Hostname: WIN-DESKTOP   │
│ Build: 22621.963                                       │
└────────────────────────────────────────────────────────┘

🖥️  Operating Systems
────────────────────────────────────────────────────────────
    🪟 Type:         windows
    📦 Distribution: windows
    🏷️ Product:      Windows 11 Pro
    🏠 Hostname:     WIN-DESKTOP
    🔢 Version:      11.0
    🏗️ Build:        22621.963
    🔧 Edition:      Professional
```

### Supported Windows Versions

| Windows Version | Detection | Registry Parsing | Full Support |
|----------------|-----------|------------------|--------------|
| Windows 11 | ✅ | ✅ | ✅ |
| Windows 10 | ✅ | ✅ | ✅ |
| Windows Server 2022 | ✅ | ✅ | ✅ |
| Windows Server 2019 | ✅ | ✅ | ✅ |
| Windows Server 2016 | ✅ | ✅ | ✅ |
| Windows 8.1 | ✅ | ✅ | ✅ |
| Windows 8 | ✅ | ✅ | ✅ |
| Windows 7 | ✅ | ✅ | ✅ |
| Windows Vista | ✅ | ⚠️ Limited | ⚠️ Limited |
| Windows XP | ✅ | ⚠️ Limited | ⚠️ Limited |

### Registry-Based Detection

guestctl reads Windows registry hives to extract version information:

```rust
use guestctl::guestfs::Guestfs;

fn detect_windows_version() -> Result<(), Box<dyn std::error::Error>> {
    let mut g = Guestfs::new()?;
    g.add_drive_ro("windows.qcow2")?;
    g.launch()?;

    let roots = g.inspect_os()?;
    for root in &roots {
        // Get Windows product name from registry
        let product_name = g.inspect_get_product_name(root)?;
        println!("Product: {}", product_name);

        // Get build number
        let major = g.inspect_get_major_version(root)?;
        let minor = g.inspect_get_minor_version(root)?;
        println!("Version: {}.{}", major, minor);

        // Get Windows-specific information
        let build = g.windows_get_build_number(root)?;
        let edition = g.windows_get_edition(root)?;
        println!("Build: {}", build);
        println!("Edition: {}", edition);
    }

    g.shutdown()?;
    Ok(())
}
```

## Registry Access

### Reading Registry Keys

Access Windows registry directly from disk image:

```rust
use guestctl::guestfs::Guestfs;

fn read_registry() -> Result<(), Box<dyn std::error::Error>> {
    let mut g = Guestfs::new()?;
    g.add_drive_ro("windows.qcow2")?;
    g.launch()?;

    // Mount Windows filesystem
    let roots = g.inspect_os()?;
    let root = &roots[0];
    g.inspect_mount_root(root)?;

    // Read registry key
    let value = g.registry_get_value(
        "HKLM\\SOFTWARE\\Microsoft\\Windows NT\\CurrentVersion",
        "ProductName"
    )?;
    println!("Windows Product: {}", value);

    // Read installed programs
    let programs = g.registry_list_subkeys(
        "HKLM\\SOFTWARE\\Microsoft\\Windows\\CurrentVersion\\Uninstall"
    )?;
    println!("Installed programs: {}", programs.len());

    g.shutdown()?;
    Ok(())
}
```

### Registry Hives

Windows registry hives locations:

| Hive | Path | Content |
|------|------|---------|
| SYSTEM | `C:\Windows\System32\config\SYSTEM` | System configuration |
| SOFTWARE | `C:\Windows\System32\config\SOFTWARE` | Installed software |
| SAM | `C:\Windows\System32\config\SAM` | User accounts (secured) |
| SECURITY | `C:\Windows\System32\config\SECURITY` | Security policies |
| DEFAULT | `C:\Windows\System32\config\DEFAULT` | Default user profile |
| NTUSER.DAT | `C:\Users\<username>\NTUSER.DAT` | User-specific settings |

### Modifying Registry

Modify Windows registry before boot:

```python
from guestctl import Guestfs

g = Guestfs()
g.add_drive("windows.qcow2")
g.launch()

roots = g.inspect_os()
root = roots[0]
g.inspect_mount_root(root)

# Disable Windows Firewall
g.registry_set_value(
    "HKLM\\SYSTEM\\CurrentControlSet\\Services\\SharedAccess\\Parameters\\FirewallPolicy\\StandardProfile",
    "EnableFirewall",
    0  # DWORD value
)

# Set computer name
g.registry_set_value(
    "HKLM\\SYSTEM\\CurrentControlSet\\Control\\ComputerName\\ComputerName",
    "ComputerName",
    "NEW-HOSTNAME"  # String value
)

g.shutdown()
```

## Windows User Management

### List Users

```bash
# List all Windows users
guestctl interactive windows.qcow2
> mount /
> inspect-users
```

**Output:**
```
👥 Windows Users
────────────────────────────────────────
  Administrator (Disabled)
  Guest (Disabled)
  john.doe (Active)
  service_account (Active)
```

### User Account Details

```rust
fn inspect_windows_users() -> Result<(), Box<dyn std::error::Error>> {
    let mut g = Guestfs::new()?;
    g.add_drive_ro("windows.qcow2")?;
    g.launch()?;

    let roots = g.inspect_os()?;
    for root in &roots {
        let users = g.inspect_users(root)?;

        for user in users {
            println!("User: {}", user.username);
            println!("  UID: {}", user.uid);
            println!("  Home: {}", user.home_dir);
            println!("  Shell: {}", user.shell);
        }
    }

    g.shutdown()?;
    Ok(())
}
```

## VirtIO Driver Injection

For Hyper-V or VMware to KVM migrations, Windows needs VirtIO drivers.

### Download VirtIO Drivers

```bash
# Download latest VirtIO driver ISO
wget https://fedorapeople.org/groups/virt/virtio-win/direct-downloads/latest-virtio/virtio-win.iso

# Or specific version
wget https://fedorapeople.org/groups/virt/virtio-win/direct-downloads/archive-virtio/virtio-win-0.1.240/virtio-win-0.1.240.iso
```

### Inject Drivers (Method 1: Registry)

```bash
# Create registry file for driver injection
cat > virtio-drivers.reg <<'EOF'
Windows Registry Editor Version 5.00

; VirtIO SCSI Driver
[HKEY_LOCAL_MACHINE\SYSTEM\CurrentControlSet\Services\vioscsi]
"Start"=dword:00000000
"Type"=dword:00000001
"ErrorControl"=dword:00000001
"ImagePath"="system32\\drivers\\vioscsi.sys"
"DisplayName"="Red Hat VirtIO SCSI pass-through controller"

; VirtIO Block Driver
[HKEY_LOCAL_MACHINE\SYSTEM\CurrentControlSet\Services\viostor]
"Start"=dword:00000000
"Type"=dword:00000001
"ErrorControl"=dword:00000001
"ImagePath"="system32\\drivers\\viostor.sys"
"DisplayName"="Red Hat VirtIO SCSI controller"

; VirtIO Network Driver
[HKEY_LOCAL_MACHINE\SYSTEM\CurrentControlSet\Services\netkvm]
"Start"=dword:00000003
"Type"=dword:00000001
"ErrorControl"=dword:00000001
"ImagePath"="system32\\drivers\\netkvm.sys"
"DisplayName"="Red Hat VirtIO Ethernet Adapter"

; VirtIO Balloon Driver
[HKEY_LOCAL_MACHINE\SYSTEM\CurrentControlSet\Services\balloon]
"Start"=dword:00000003
"Type"=dword:00000001
"ErrorControl"=dword:00000001
"ImagePath"="system32\\drivers\\balloon.sys"
"DisplayName"="VirtIO Balloon Driver"
EOF

# Inject into Windows registry
virt-win-reg windows.qcow2 --merge virtio-drivers.reg
```

### Inject Drivers (Method 2: DISM)

```bash
# Mount Windows image
guestctl interactive windows.qcow2
> mount C:

# Use DISM to inject drivers
> command "dism /Image:C:\\ /Add-Driver /Driver:D:\\NetKVM\\w11\\amd64 /Recurse"
> command "dism /Image:C:\\ /Add-Driver /Driver:D:\\vioscsi\\w11\\amd64 /Recurse"
> command "dism /Image:C:\\ /Add-Driver /Driver:D:\\viostor\\w11\\amd64 /Recurse"
> exit
```

### Verify Driver Injection

```bash
# Check drivers were added
virt-win-reg windows.qcow2 --list 'HKLM\SYSTEM\CurrentControlSet\Services' | grep -i virtio
```

## Windows Services

### List Services

```bash
# Inspect Windows services
guestctl inspect windows.qcow2 --profile security --output json | jq '.services'
```

### Detect Running Services

```rust
fn list_windows_services() -> Result<(), Box<dyn std::error::Error>> {
    let mut g = Guestfs::new()?;
    g.add_drive_ro("windows.qcow2")?;
    g.launch()?;

    let roots = g.inspect_os()?;
    for root in &roots {
        // List Windows services from registry
        let services = g.registry_list_subkeys(
            "HKLM\\SYSTEM\\CurrentControlSet\\Services"
        )?;

        println!("Windows Services: {}", services.len());

        for service in services.iter().take(10) {
            println!("  - {}", service);
        }
    }

    g.shutdown()?;
    Ok(())
}
```

### Disable Windows Services

```python
from guestctl import Guestfs

def disable_windows_update(image_path):
    g = Guestfs()
    g.add_drive(image_path)
    g.launch()

    roots = g.inspect_os()
    g.inspect_mount_root(roots[0])

    # Disable Windows Update service
    g.registry_set_value(
        "HKLM\\SYSTEM\\CurrentControlSet\\Services\\wuauserv",
        "Start",
        4  # Disabled
    )

    g.shutdown()
```

## Windows System Configuration

### Hostname Change

```bash
# Change Windows computer name
guestctl interactive windows.qcow2
> mount C:
> registry-set "HKLM\\SYSTEM\\CurrentControlSet\\Control\\ComputerName\\ComputerName" "ComputerName" "NEW-HOSTNAME"
> registry-set "HKLM\\SYSTEM\\CurrentControlSet\\Services\\Tcpip\\Parameters" "Hostname" "NEW-HOSTNAME"
> exit
```

### Network Configuration

```python
from guestctl import Guestfs

def configure_windows_network(image_path):
    g = Guestfs()
    g.add_drive(image_path)
    g.launch()

    roots = g.inspect_os()
    g.inspect_mount_root(roots[0])

    # Set static IP (Windows 10/11/Server 2016+)
    interface_guid = "{12345678-1234-1234-1234-123456789012}"  # Get from registry

    g.registry_set_value(
        f"HKLM\\SYSTEM\\CurrentControlSet\\Services\\Tcpip\\Parameters\\Interfaces\\{interface_guid}",
        "EnableDHCP",
        0
    )

    g.registry_set_value(
        f"HKLM\\SYSTEM\\CurrentControlSet\\Services\\Tcpip\\Parameters\\Interfaces\\{interface_guid}",
        "IPAddress",
        ["192.168.1.100"]  # REG_MULTI_SZ
    )

    g.shutdown()
```

### Timezone Configuration

```bash
# Set Windows timezone
guestctl interactive windows.qcow2
> mount C:
> registry-set "HKLM\\SYSTEM\\CurrentControlSet\\Control\\TimeZoneInformation" "TimeZoneKeyName" "Pacific Standard Time"
> exit
```

## Windows Activation

### Check Activation Status

```rust
fn check_windows_activation() -> Result<(), Box<dyn std::error::Error>> {
    let mut g = Guestfs::new()?;
    g.add_drive_ro("windows.qcow2")?;
    g.launch()?;

    let roots = g.inspect_os()?;
    g.inspect_mount_root(&roots[0])?;

    // Read activation status from registry
    let license_status = g.registry_get_value(
        "HKLM\\SOFTWARE\\Microsoft\\Windows NT\\CurrentVersion\\SoftwareProtectionPlatform",
        "ActivationStatus"
    )?;

    println!("Activation Status: {}", license_status);

    g.shutdown()?;
    Ok(())
}
```

### Product Key Management

**Note:** guestctl can read but not activate Windows. Use official Microsoft tools for activation.

```bash
# Read current product key (partial)
guestctl interactive windows.qcow2
> mount C:
> registry-get "HKLM\\SOFTWARE\\Microsoft\\Windows NT\\CurrentVersion" "DigitalProductId"
> exit
```

## Windows-Specific Inspection

### Installed Applications

```bash
# List installed applications
guestctl packages windows.qcow2
```

**Output:**
```
📦 Installed Applications (Windows)
────────────────────────────────────────
  7-Zip 23.01                    (x64)
  Google Chrome 120.0.6099.130   (x64)
  Microsoft Office Professional Plus 2021
  Mozilla Firefox 121.0          (x64)
  VLC media player 3.0.20        (x64)
  WinRAR 6.24                    (x64)

Total: 6 applications
```

### System Information

```rust
fn get_windows_system_info() -> Result<(), Box<dyn std::error::Error>> {
    let mut g = Guestfs::new()?;
    g.add_drive_ro("windows.qcow2")?;
    g.launch()?;

    let roots = g.inspect_os()?;
    let root = &roots[0];
    g.inspect_mount_root(root)?;

    // Computer manufacturer and model
    let manufacturer = g.registry_get_value(
        "HKLM\\SYSTEM\\CurrentControlSet\\Control\\SystemInformation",
        "SystemManufacturer"
    )?;

    let model = g.registry_get_value(
        "HKLM\\SYSTEM\\CurrentControlSet\\Control\\SystemInformation",
        "SystemProductName"
    )?;

    println!("Manufacturer: {}", manufacturer);
    println!("Model: {}", model);

    // Installation date
    let install_date = g.registry_get_value(
        "HKLM\\SOFTWARE\\Microsoft\\Windows NT\\CurrentVersion",
        "InstallDate"
    )?;

    println!("Install Date: {}", install_date);

    g.shutdown()?;
    Ok(())
}
```

## Sysprep and Generalization

### Detect Sysprep Status

```bash
# Check if Windows has been generalized
guestctl inspect windows.qcow2 --profile migration | jq '.sysprep_status'
```

### Prepare for Cloning

```python
from guestctl import Guestfs

def sysprep_windows(image_path):
    """Prepare Windows for cloning (manual sysprep required before this)"""
    g = Guestfs()
    g.add_drive(image_path)
    g.launch()

    roots = g.inspect_os()
    g.inspect_mount_root(roots[0])

    # Remove unique identifiers
    # SID regeneration happens on first boot after sysprep

    # Clear event logs
    g.command(["wevtutil", "cl", "System"])
    g.command(["wevtutil", "cl", "Application"])
    g.command(["wevtutil", "cl", "Security"])

    # Remove machine-specific data
    g.rm_rf("C:\\Windows\\Panther\\Unattend.xml")
    g.rm_rf("C:\\Windows\\System32\\Sysprep\\Panther")

    g.shutdown()
```

## Windows Event Logs

### Read Event Logs

```bash
# Extract Windows event logs
guestctl interactive windows.qcow2
> mount C:
> download "C:\\Windows\\System32\\winevt\\Logs\\System.evtx" ./system.evtx
> download "C:\\Windows\\System32\\winevt\\Logs\\Application.evtx" ./application.evtx
> exit

# Parse event logs on host (requires evtx parser)
evtx_dump ./system.evtx > system-events.xml
```

## Troubleshooting Windows VMs

### Boot Failure: "INACCESSIBLE_BOOT_DEVICE"

**Cause:** Storage driver missing (common in Hyper-V → KVM migration)

**Solution:**
```bash
# Inject VirtIO storage drivers before first boot
virt-win-reg windows.qcow2 --merge virtio-drivers.reg

# Or use Safe Mode to install drivers after migration
```

### Network Not Working

**Cause:** VirtIO network driver not installed

**Solution:**
```bash
# Install VirtIO network driver
# Attach virtio-win.iso to VM
# Install drivers from Device Manager after boot
```

### Windows Activation Lost

**Cause:** Hardware changed significantly during migration

**Solution:**
```powershell
# Reactivate Windows (in VM after boot)
slmgr /rearm
slmgr /ato

# Or use phone activation if automated activation fails
```

### Blue Screen on Boot

**Cause:** Incompatible drivers for new hardware

**Solution:**
```bash
# Boot into Safe Mode
# F8 during boot → Safe Mode

# Or modify registry to boot Safe Mode
guestctl interactive windows.qcow2
> mount C:
> registry-set "HKLM\\SYSTEM\\CurrentControlSet\\Control\\SafeBoot\\Minimal" "Enabled" 1
> exit
```

## Best Practices

1. **Always backup before modifications** - Windows registry corruption can brick the VM
2. **Use read-only mode for inspection** - Prevent accidental changes
3. **Test registry changes** - Verify on test VM before production
4. **Keep driver ISOs** - Maintain VirtIO driver ISO for migrations
5. **Document registry paths** - Windows registry paths are case-insensitive but complex
6. **Use UUIDs not drive letters** - Drive letters can change
7. **Verify activation** - Check Windows activation status after migration

## API Reference

### Windows-Specific Functions

```rust
// Get Windows build number
pub fn windows_get_build_number(&mut self, root: &str) -> Result<String>

// Get Windows edition (Home, Pro, Enterprise)
pub fn windows_get_edition(&mut self, root: &str) -> Result<String>

// Registry operations
pub fn registry_get_value(&mut self, key: &str, value_name: &str) -> Result<String>
pub fn registry_set_value(&mut self, key: &str, value_name: &str, value: &str) -> Result<()>
pub fn registry_list_subkeys(&mut self, key: &str) -> Result<Vec<String>>
pub fn registry_delete_value(&mut self, key: &str, value_name: &str) -> Result<()>

// Driver injection
pub fn inject_virtio_drivers(&mut self, driver_path: &str) -> Result<()>

// Sysprep detection
pub fn windows_is_sysprepped(&mut self, root: &str) -> Result<bool>
```

## Further Reading

- [Windows Registry Documentation](https://docs.microsoft.com/en-us/windows/win32/sysinfo/registry)
- [VirtIO Drivers for Windows](https://docs.fedoraproject.org/en-US/quick-docs/creating-windows-virtual-machines-using-virtio-drivers/)
- [Windows Sysprep Guide](https://docs.microsoft.com/en-us/windows-hardware/manufacture/desktop/sysprep--system-preparation--overview)
- [VM Migration Guide](vm-migration.md) - Hyper-V to KVM migration

## Support

For Windows-related issues:
- GitHub Issues: https://github.com/ssahani/guestkit/issues
- Tag with: `windows`, `registry`, `virtio`
- Include: Windows version, error messages, registry paths
