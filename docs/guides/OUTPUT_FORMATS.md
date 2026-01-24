# Output Format Support

GuestKit supports multiple output formats for inspection results, enabling integration with automation tools, scripts, and infrastructure-as-code systems.

## Supported Formats

### JSON (Machine-Readable)
```bash
guestkit inspect disk.qcow2 --output json
# or
guestkit inspect disk.qcow2 -o json
```

**Use cases:**
- Automation scripts
- CI/CD pipelines
- Integration with monitoring tools
- Processing with `jq`

**Example:**
```bash
# Get hostname from JSON output
guestkit inspect disk.qcow2 -o json | jq '.os.hostname'

# Extract all network interfaces
guestkit inspect disk.qcow2 -o json | jq '.network.interfaces[]'

# Check if cloud-init is present
guestkit inspect disk.qcow2 -o json | jq '.system_config.cloud_init'
```

### YAML (Configuration-Friendly)
```bash
guestkit inspect disk.qcow2 --output yaml
# or
guestkit inspect disk.qcow2 -o yaml
```

**Use cases:**
- Infrastructure-as-code configurations
- Ansible playbooks
- Human-readable structured data
- Processing with `yq`

**Example:**
```bash
# Get OS type from YAML
guestkit inspect disk.qcow2 -o yaml | yq '.os.os_type'

# Extract network configuration
guestkit inspect disk.qcow2 -o yaml | yq '.network'
```

### Text (Human-Readable, Default)
```bash
guestkit inspect disk.qcow2
# or explicitly:
guestkit inspect disk.qcow2 --output text
```

**Use cases:**
- Interactive terminal use
- Visual inspection
- Reports and documentation

### CSV (Tabular Data)
```bash
guestkit inspect disk.qcow2 --output csv
# or
guestkit inspect disk.qcow2 -o csv
```

**Use cases:**
- Bulk comparison of multiple VMs
- Spreadsheet import
- Database loading
- Compliance reporting

**Example:**
```bash
# Extract user accounts as CSV
guestkit inspect disk.qcow2 -o csv > users.csv

# Compare packages across multiple VMs
for vm in *.qcow2; do
  echo "=== $vm ===" >> all-vms.csv
  guestkit inspect "$vm" -o csv >> all-vms.csv
done
```

**Note:** CSV output format is optimized for tabular data like users, services, and packages. For full inspection data, use JSON or YAML formats.

## JSON/YAML Output Structure

The JSON and YAML outputs follow this structure:

```yaml
image_path: /path/to/disk.qcow2
os:
  root: /dev/sda1
  os_type: linux
  distribution: fedora
  product_name: Fedora Linux
  architecture: x86_64
  version:
    major: 39
    minor: 0
  hostname: fedora-server
  package_format: rpm
  init_system: systemd
  package_manager: dnf
  format: installed

system_config:
  timezone: America/New_York
  locale: en_US.UTF-8
  selinux: enforcing
  cloud_init: true
  vm_tools:
    - qemu-guest-agent

network:
  interfaces:
    - name: eth0
      ip_address:
        - 192.168.1.100
      mac_address: "52:54:00:12:34:56"
      dhcp: false
  dns_servers:
    - 8.8.8.8
    - 8.8.4.4

users:
  regular_users:
    - username: john
      uid: "1000"
      gid: "1000"
      home: /home/john
      shell: /bin/bash
  system_users_count: 42
  total_users: 43

ssh:
  config:
    Port: "22"
    PermitRootLogin: "no"
    PasswordAuthentication: "yes"

services:
  enabled_services:
    - name: sshd
      enabled: true
      state: enabled
  timers:
    - backup.timer

runtimes:
  language_runtimes:
    python3: installed
    nodejs: installed
  container_runtimes:
    - docker
    - podman

storage:
  lvm:
    physical_volumes:
      - /dev/sda2
    volume_groups:
      - vg_main
    logical_volumes:
      - lv_root
      - lv_home
  swap_devices:
    - /dev/vg_main/lv_swap
  fstab_mounts:
    - device: /dev/vg_main/lv_root
      mountpoint: /
      fstype: ext4

boot:
  bootloader: GRUB2
  default_entry: "0"
  timeout: "5"
  kernel_cmdline: ""

scheduled_tasks:
  cron_jobs:
    - "0 2 * * * /usr/bin/backup.sh"
  systemd_timers:
    - backup.timer

security:
  certificates_count: 156
  certificate_paths:
    - /etc/ssl/certs/ca-bundle.crt
  kernel_parameters_count: 42

packages:
  format: rpm
  count: 1247
  kernels:
    - vmlinuz-6.6.8-200.fc39.x86_64

disk_usage:
  total_bytes: 21474836480
  used_bytes: 9126805504
  free_bytes: 12348030976
  used_percent: 42.5

windows:  # Only present for Windows systems
  software:
    - name: "Microsoft Office Professional Plus"
      version: "16.0.5345.1002"
      publisher: "Microsoft Corporation"
      install_date: "2024-01-15"
  services:
    - name: "wuauserv"
      display_name: "Windows Update"
      start_type: "Manual"
      status: "Stopped"
    - name: "Dhcp"
      display_name: "DHCP Client"
      start_type: "Automatic"
      status: "Running"
  network_adapters:
    - name: "Ethernet"
      description: "Intel(R) PRO/1000 MT Network Connection"
      mac_address: "00:0C:29:12:34:56"
      ip_address:
        - "192.168.1.100"
      dns_servers:
        - "8.8.8.8"
        - "8.8.4.4"
      dhcp_enabled: true
  updates:
    - kb: "KB5034441"
      title: "2024-01 Cumulative Update for Windows 11"
      installed_date: "2024-01-15"
      update_type: "Security Update"
  event_logs:
    - event_id: 6005
      level: "Information"
      source: "EventLog"
      message: "The Event log service was started"
      time_created: "2024-01-15T10:30:00Z"
```

## Integration Examples

### Ansible Integration
```yaml
- name: Inspect VM disk
  command: guestkit inspect /path/to/vm.qcow2 --output json
  register: vm_inspection

- name: Parse inspection results
  set_fact:
    vm_hostname: "{{ (vm_inspection.stdout | from_json).os.hostname }}"
    vm_distro: "{{ (vm_inspection.stdout | from_json).os.distribution }}"

- name: Verify OS distribution
  assert:
    that:
      - vm_distro == "fedora"
    fail_msg: "Expected Fedora, got {{ vm_distro }}"
```

### Terraform Data Source
```hcl
data "external" "vm_inspection" {
  program = ["guestkit", "inspect", var.disk_image, "--output", "json"]
}

output "vm_hostname" {
  value = data.external.vm_inspection.result.os.hostname
}
```

### Shell Script Processing
```bash
#!/bin/bash

# Inspect disk and extract key information
INSPECTION=$(guestkit inspect disk.qcow2 --output json)

HOSTNAME=$(echo "$INSPECTION" | jq -r '.os.hostname')
OS_TYPE=$(echo "$INSPECTION" | jq -r '.os.os_type')
HAS_CLOUD_INIT=$(echo "$INSPECTION" | jq -r '.system_config.cloud_init')

echo "VM Hostname: $HOSTNAME"
echo "OS Type: $OS_TYPE"
echo "Cloud-init: $HAS_CLOUD_INIT"

# Check if SSH permits root login
SSH_ROOT=$(echo "$INSPECTION" | jq -r '.ssh.config.PermitRootLogin // "unknown"')
if [ "$SSH_ROOT" = "yes" ]; then
    echo "WARNING: SSH root login is enabled!"
fi
```

### Windows VM Analysis
```bash
#!/bin/bash

# Inspect Windows VM and extract security information
INSPECTION=$(guestkit inspect windows-server.qcow2 --output json)

# Check installed updates
UPDATES=$(echo "$INSPECTION" | jq -r '.windows.updates[].kb')
echo "Installed updates: $UPDATES"

# Check running services
SERVICES=$(echo "$INSPECTION" | jq -r '.windows.services[] | select(.status == "Running") | .name')
echo "Running services: $SERVICES"

# Security audit: Check for specific software
HAS_DEFENDER=$(echo "$INSPECTION" | jq '.windows.software[] | select(.name | contains("Defender"))')
if [ -z "$HAS_DEFENDER" ]; then
    echo "WARNING: Windows Defender not detected!"
fi
```

### Python Integration
```python
import subprocess
import json

# Run inspection
result = subprocess.run(
    ["guestkit", "inspect", "disk.qcow2", "--output", "json"],
    capture_output=True,
    text=True
)

# Parse JSON
inspection = json.loads(result.stdout)

# Access data
print(f"Hostname: {inspection['os']['hostname']}")
print(f"Distribution: {inspection['os']['distribution']}")
print(f"Package count: {inspection.get('packages', {}).get('count', 0)}")

# Check security settings
if inspection.get('ssh', {}).get('config', {}).get('PermitRootLogin') == 'yes':
    print("WARNING: SSH root login enabled")

# List container runtimes
runtimes = inspection.get('runtimes', {}).get('container_runtimes', [])
if runtimes:
    print(f"Container runtimes: {', '.join(runtimes)}")
```

## Field Reference

### Always Present
- `os.root` - Root device path
- `os.os_type` - Operating system type (linux, windows, unknown)

### Optional Fields (may be `null` or absent)
All other fields are optional and only present if the information could be detected:
- Network configuration may be absent if network files are not readable
- User information may be limited if `/etc/passwd` is not accessible
- Storage details depend on filesystem mount success
- Package counts require successful filesystem mounting

### Skip Serialization
Fields with `null` or empty values are automatically omitted from JSON/YAML output to reduce noise and output size.

## CSV Output Examples

CSV output is currently optimized for users data. The format provides tabular data suitable for spreadsheets and databases:

### User Accounts CSV
```csv
username,uid,gid,home,shell
john,1000,1000,/home/john,/bin/bash
jane,1001,1001,/home/jane,/bin/zsh
admin,1002,1002,/home/admin,/bin/bash
```

### Services CSV
```csv
name,enabled,state
sshd,true,enabled
nginx,true,enabled
postgresql,false,disabled
```

### Packages CSV
```csv
kernel_version,format,total_count
vmlinuz-6.6.8-200.fc39.x86_64,rpm,1247
vmlinuz-6.5.6-200.fc39.x86_64,rpm,1247
```

## Future Enhancements

- Markdown output for documentation generation
- HTML report generation
- Custom output templates
- Extended CSV formats (network interfaces, storage details)

## See Also

- [Inspection Quick Reference](../INSPECTION_QUICK_REF.md)
- [Enhanced Inspection Guide](../ENHANCED_INSPECTION.md)
- [CLI Guide](CLI_GUIDE.md)
