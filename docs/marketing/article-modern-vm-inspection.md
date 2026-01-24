# GuestKit: Modern VM Disk Inspection Without the Boot Wait

## The 5-Second OS Inspection That Changes Everything

**TL;DR:** GuestKit is a Rust-powered VM inspection tool that analyzes disk images in seconds without booting them. Perfect for security audits, cloud migrations, and DevOps automation.

---

## The Problem We've All Faced

It's 2 AM. Your security monitoring just flagged suspicious activity on a production VM. You need to inspect it immediately, but booting it could:

- Execute potentially malicious code
- Alert the attacker
- Modify evidence
- Take 5+ minutes just to boot
- Require credentials you might not have

Or consider this: Your team is migrating 200 VMs to the cloud. Before you can plan the migration, you need to know:

- What OS is running on each VM?
- Which kernel versions are in use?
- What packages are installed?
- Which services are enabled?
- What's the configuration state?

Traditional approach? Boot each one, log in, run audit scripts. **Total time: Days.**

There has to be a better way.

## Enter GuestKit

GuestKit is a command-line tool that inspects VM disk images **without booting them**. Written in Rust for performance and safety, it gives you instant access to:

- OS type and distribution
- Kernel version
- Package inventory
- Service status
- Configuration files
- User accounts
- Network settings
- Security posture

All in seconds. All read-only. All safe.

## See It In Action

Let's inspect a VMware Photon OS disk:

```bash
$ sudo guestkit inspect photon.vmdk
```

**Output (in ~5 seconds):**

```
══════════════════════════════════════════════════════════════════════
📀 Disk Image: photon.vmdk
══════════════════════════════════════════════════════════════════════

✓ Found 1 operating system(s)

OS #1 (/dev/sda3)
──────────────────────────────────────────────────────────────────────
  Operating System
    Type: 🐧  linux
    Distribution: photon
    Version: 5.0
    Product Name: VMware Photon OS/Linux
    Hostname: photon-2e2948360ed5
    Architecture: ⚙️  x86_64

  Package Management
    Format: 🔴  rpm
    Tool: rpm

  System Information
    Machine ID: 🆔  56d8a0baf2ea44ceaac9c5e3e787b6ad
    Kernel: 🐧  6.18.5-200.fc43.x86_64
    Init system: ⚡  systemd

  Hardware Information
    Chassis: 💻  unknown
══════════════════════════════════════════════════════════════════════
```

**What just happened?**

In under 5 seconds, without booting the VM, we extracted:
- Complete OS profile
- Kernel version
- Package manager information
- System identifiers
- Hardware configuration

## Beyond Basic Inspection

GuestKit isn't just about OS detection. It's a complete VM forensics and automation toolkit.

### 1. Filesystem Exploration

```bash
# List all filesystems and partitions
$ guestkit filesystems vm.qcow2

Block Devices
──────────────────────────────────────
  ▪ /dev/sda
    Size: 21474836480 (20.00 GiB)
    Partition table: gpt

Partitions
──────────────────────────────────────
  📁 /dev/sda1 (ext4) 1.0 GiB
    Label: boot
    UUID: 7f8b4c3e-2a1d-4e9f-b3c2-5d6e7f8a9b0c

  🗄 /dev/sda2 (xfs) 19.0 GiB
    Label: root

LVM Logical Volumes
──────────────────────────────────────
  ▸ /dev/mapper/vg0-lv_data 50.0 GiB
```

### 2. Package Inventory

```bash
# List all installed packages
$ guestkit packages ubuntu.qcow2 --json

{
  "total": 847,
  "packages": [
    {
      "name": "openssh-server",
      "version": "1:8.9p1-3ubuntu0.6",
      "release": "ubuntu0.6"
    },
    {
      "name": "nginx",
      "version": "1.18.0-6ubuntu14.4",
      "release": "ubuntu14.4"
    }
    ...
  ]
}
```

Perfect for:
- Security vulnerability scanning
- License compliance audits
- Migration dependency analysis

### 3. Configuration Extraction

```bash
# Read any file from the disk
$ guestkit cat vm.qcow2 /etc/ssh/sshd_config

# Output is exactly what's in the file
PermitRootLogin no
PasswordAuthentication no
PubkeyAuthentication yes
...
```

No mounting. No boot. Just instant access.

### 4. Interactive Mode

For deeper exploration:

```bash
$ guestkit interactive vm.qcow2

guestkit> filesystems
[Lists filesystems...]

guestkit> mount /dev/sda1 /
Mounted /dev/sda1 at /

guestkit> ls /etc
[Lists /etc directory...]

guestkit> cat /etc/hostname
prod-web-01

guestkit> packages | grep apache
apache2 2.4.52-1ubuntu4.7

guestkit> services --enabled
sshd
nginx
docker
```

Full REPL with:
- Tab completion
- Command history (persists across sessions!)
- 27+ commands
- Intuitive interface

### 5. Batch Automation

Create script files for repeatable audits:

```bash
# security-audit.gk
mount /dev/sda1 /
packages > packages.txt
services > services.txt
cat /etc/ssh/sshd_config > sshd-config.txt
cat /etc/sudoers > sudoers.txt
find /home > user-files.txt
umount /
```

Execute:

```bash
$ guestkit script vm.qcow2 security-audit.gk

[1] mount /dev/sda1 /
  ✓ Success

[2] packages > packages.txt
  ✓ Success
  → Wrote output to packages.txt

[3] services > services.txt
  ✓ Success
  → Wrote output to services.txt

...

═══════════════════════════════════════
Batch Execution Report
═══════════════════════════════════════

Script: security-audit.gk
Total Commands: 6
Successful: 6
Failed: 0

✓ All commands executed successfully!
```

Perfect for:
- CI/CD pipelines
- Compliance checking at scale
- Automated security audits

### 6. Advanced Profiles

Built-in inspection profiles for specific use cases:

**Security Profile:**
```bash
$ guestkit inspect vm.qcow2 --profile security

Security Audit Report
═══════════════════════════════════════

SSH Configuration
  ✓ Root login: disabled
  ✓ Password auth: disabled
  ✓ Key-only auth: enabled

User Security
  ⚠ 3 users with UID 0
  ✓ No users with empty passwords
  ⚠ sudo access: 5 users

Firewall & Network
  ✓ Firewall: active (iptables)
  ⚠ 15 listening ports

SELinux/AppArmor
  ⚠ SELinux: permissive mode

Services Security
  ⚠ 12 enabled services
  ✓ SSH running on non-default port

Certificates
  ⚠ 2 certificates expiring within 30 days
```

**Migration Profile:**
```bash
$ guestkit inspect vm.qcow2 --profile migration

Migration Planning Report
═══════════════════════════════════════

Operating System
  Type: Ubuntu 22.04 LTS
  Kernel: 5.15.0-91-generic
  Architecture: x86_64
  ✓ Cloud-init installed

Package Inventory
  Total packages: 847
  Package format: deb
  Manager: apt

  Key packages:
    • docker-ce 24.0.7
    • nginx 1.18.0
    • postgresql 14.10

Storage Layout
  Root: 20 GiB (ext4)
  Data: 50 GiB (xfs, LVM)
  Swap: 2 GiB

  ⚠ Custom mount points:
    • /data (requires migration planning)

Network Configuration
  Hostname: prod-db-01
  Static IPs: 2 configured
  DNS: 8.8.8.8, 8.8.4.4

  ⚠ Network config requires updates for cloud

Custom Services
  ✓ 5 systemd units
  ✓ 2 cron jobs

  Migration considerations:
    • backup.service (custom systemd unit)
    • database-cleanup.timer
```

**Performance Profile:**
```bash
$ guestkit inspect vm.qcow2 --profile performance

Performance Tuning Report
═══════════════════════════════════════

Kernel Parameters
  ⚠ vm.swappiness = 60 (consider 10)
  ✓ net.core.somaxconn = 1024
  ⚠ fs.file-max = 65536 (consider increasing)

Swap Configuration
  Size: 2 GiB
  ⚠ Swappiness high for production workload

Disk I/O
  Scheduler: mq-deadline
  ✓ Appropriate for SSD

Services & Resources
  ⚠ 25 enabled services
  Notable: docker, nginx, postgresql

  Tuning opportunities:
    • Reduce enabled services
    • Optimize kernel parameters
    • Consider swap reduction
```

### 7. Beautiful HTML Reports

```bash
$ guestkit inspect vm.qcow2 --export html --export-output report.html
```

Generates an interactive HTML report with:
- 🌓 Dark/light mode toggle
- 🔍 Real-time search across all data
- 📊 Interactive charts (Chart.js)
- 📱 Fully responsive design
- 💾 Theme persistence
- 🖨️ Print-optimized styles

Perfect for:
- Executive summaries
- Compliance documentation
- Security audit reports
- Migration planning documents

## Real-World Use Cases

### 1. Incident Response

**Scenario:** Ransomware detected on a production VM.

**Traditional Approach:**
- Shut down VM (lose volatile memory)
- Wait for forensics team
- Create disk snapshot (time-consuming)
- Boot in isolated environment (risky)
- Manual analysis (hours/days)

**With GuestKit:**
```bash
# Immediate analysis (VM can stay running)
$ guestkit inspect compromised-vm.qcow2 --profile security

# Extract suspicious files for analysis
$ guestkit cat compromised-vm.qcow2 /var/log/auth.log > auth.log
$ guestkit cat compromised-vm.qcow2 /etc/crontab > crontab

# Get package changes
$ guestkit packages compromised-vm.qcow2 > current-packages.txt

# Check running services
$ guestkit interactive compromised-vm.qcow2
guestkit> services > services.txt
guestkit> find /tmp > temp-files.txt
```

**Time saved:** Hours to days
**Safety:** No execution of potentially malicious code
**Evidence:** Complete, unmodified disk analysis

### 2. Cloud Migration Planning

**Scenario:** Migrating 200 VMs from on-premise to AWS.

**Challenge:** Need complete inventory before migration planning.

**Solution:**
```bash
# Batch inspect all VMs
$ guestkit inspect-batch vm-images/*.qcow2 --parallel 8 --output json > inventory.json

# Generate migration reports
$ for vm in vm-images/*.qcow2; do
    guestkit inspect "$vm" --profile migration --export html --export-output "reports/$(basename $vm).html"
done
```

**Results:**
- Complete OS inventory: ✓
- Package dependencies mapped: ✓
- Network configurations documented: ✓
- Custom services identified: ✓
- Migration risks flagged: ✓

**Time:** 200 VMs analyzed in ~30 minutes (vs. days of manual work)

### 3. Security Compliance

**Scenario:** Quarterly security audit across 50 production VMs.

**Requirements:**
- SSH configuration review
- User account audit
- Service inventory
- Certificate expiry check
- Firewall status
- SELinux/AppArmor status

**Solution:**
```bash
# Automated compliance script
#!/bin/bash

for vm in production-vms/*.qcow2; do
    echo "Auditing $(basename $vm)..."

    guestkit inspect "$vm" --profile security \
        --export html \
        --export-output "compliance/$(basename $vm).html"

    guestkit script "$vm" compliance-check.gk
done

# Generate summary report
guestkit inspect-batch production-vms/*.qcow2 \
    --output json > compliance-summary.json
```

**Outcome:**
- 100% coverage (vs. random sampling)
- Consistent auditing (no manual errors)
- Automated reporting
- Historical tracking (version control the reports)

### 4. DevOps Automation

**Scenario:** CI/CD pipeline needs to verify VM images before deployment.

**Integration:**
```yaml
# .github/workflows/vm-validation.yml
name: Validate VM Image

on:
  pull_request:
    paths:
      - 'vm-images/**'

jobs:
  validate:
    runs-on: ubuntu-latest

    steps:
      - uses: actions/checkout@v3

      - name: Install GuestKit
        run: |
          cargo install guestkit

      - name: Validate VM Image
        run: |
          guestkit inspect vm-images/app-server.qcow2 --output json > inspection.json

          # Check for required packages
          guestkit packages vm-images/app-server.qcow2 | grep -q "docker-ce" || exit 1
          guestkit packages vm-images/app-server.qcow2 | grep -q "nginx" || exit 1

          # Verify SSH is disabled for root
          guestkit cat vm-images/app-server.qcow2 /etc/ssh/sshd_config | grep -q "PermitRootLogin no" || exit 1

          # Check kernel version
          KERNEL=$(guestkit inspect vm-images/app-server.qcow2 --output json | jq -r '.kernel_version')
          [[ "$KERNEL" =~ ^5\. ]] || exit 1

      - name: Generate Validation Report
        run: |
          guestkit inspect vm-images/app-server.qcow2 \
            --profile security \
            --export html \
            --export-output validation-report.html

      - name: Upload Report
        uses: actions/upload-artifact@v3
        with:
          name: vm-validation-report
          path: validation-report.html
```

**Benefits:**
- Automated quality gates
- Consistent validation
- Early problem detection
- Audit trail in CI/CD

## How It Works: The Technology

### Architecture

GuestKit uses a sophisticated but elegant approach:

1. **Direct Disk Access via NBD (Network Block Device)**
   - Mounts disk images as block devices
   - No filesystem mounting required
   - Read-only by default (safe)

2. **Filesystem Detection**
   - Reads partition tables (GPT, MBR)
   - Detects filesystem types (ext4, xfs, btrfs, ntfs, etc.)
   - Identifies LVM, RAID, encryption

3. **OS Inspection**
   - Analyzes boot configuration
   - Reads system metadata
   - Parses package databases
   - Extracts service configurations

4. **Safe File Access**
   - Virtual filesystem mounting
   - No actual system mounting
   - Zero risk of modification
   - Works on corrupted/partial images

### Why Rust?

**Performance:**
- Native speed (comparable to C)
- Zero-cost abstractions
- Efficient memory usage
- Parallel processing

**Safety:**
- Memory safety without garbage collection
- No null pointer dereferences
- No buffer overflows
- Thread safety

**Reliability:**
- Comprehensive error handling
- Graceful failure modes
- Predictable resource usage
- No runtime panics in production

### Supported Formats

- **QCOW2** (KVM, OpenStack)
- **VMDK** (VMware)
- **VHD/VHDX** (Hyper-V, Azure)
- **VDI** (VirtualBox)
- **RAW** (Universal)

### Supported Operating Systems

**Linux Distributions:**
- Ubuntu, Debian
- RHEL, CentOS, Fedora, Rocky, Alma
- SUSE, openSUSE
- Arch Linux
- Alpine Linux
- VMware Photon OS
- And many more...

**Other OS:**
- Windows (NT 6.0+)
- FreeBSD, OpenBSD
- Solaris/Illumos

## Installation

### From Source (Rust/Cargo)

```bash
# Install Rust
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# Clone and build
git clone https://github.com/ssahani/guestkit.git
cd guestkit
cargo build --release

# Binary at: target/release/guestkit
cargo install --path .
```

### From PyPI (Coming Soon)

```bash
pip install guestkit
```

### From Package Managers (Planned)

```bash
# Homebrew (macOS/Linux)
brew install guestkit

# apt (Debian/Ubuntu)
apt install guestkit

# dnf (Fedora/RHEL)
dnf install guestkit
```

## Performance Benchmarks

Tested on a laptop with Core i7, 16GB RAM, SSD:

| Operation | Time | Notes |
|-----------|------|-------|
| Appliance launch | ~5s | One-time startup |
| OS detection | <1s | After appliance ready |
| Package list (1000 pkgs) | ~3s | Including mount |
| File extraction (1MB) | <1s | Direct read |
| Batch (10 VMs) | ~45s | Parallel processing |
| HTML report generation | ~2s | With charts |

**Memory usage:** ~150MB typical
**Disk I/O:** Minimal (direct NBD access)
**CPU:** Scales with available cores

## Comparison with Alternatives

### vs. Manual VM Boot

| Aspect | Manual Boot | GuestKit |
|--------|-------------|----------|
| Time | 5-10 minutes | 5-10 seconds |
| Safety | Risky (executes code) | Safe (read-only) |
| Credentials | Required | Not required |
| Automation | Difficult | Built-in |
| Broken VMs | Can't boot | No problem |

### vs. Mounting Filesystems

| Aspect | Manual Mount | GuestKit |
|--------|--------------|----------|
| Complexity | High | Single command |
| Safety | Risky (write access) | Read-only always |
| LVM/RAID | Manual assembly | Automatic |
| Multiple OS | Manual per-OS | Automatic |
| Cleanup | Manual unmount | Automatic |

### vs. Other Tools

**Advantages:**
- Modern, clean CLI with beautiful output
- Built-in batch processing
- Multiple output formats (JSON, HTML, CSV)
- Interactive REPL mode
- Inspection profiles
- Active development
- Rust performance and safety
- No legacy baggage

## Python Bindings

For Python integration:

```python
from guestkit import Guestfs

# Context manager for automatic cleanup
with Guestfs() as g:
    # Add disk image
    g.add_drive_ro("vm.qcow2")

    # Launch appliance
    g.launch()

    # Inspect OS
    roots = g.inspect_os()
    if roots:
        root = roots[0]

        # Get OS info
        os_type = g.inspect_get_type(root)
        distro = g.inspect_get_distro(root)
        version = g.inspect_get_major_version(root)

        print(f"Found {distro} {version} ({os_type})")

        # Mount filesystems
        mountpoints = g.inspect_get_mountpoints(root)
        for mount, device in sorted(mountpoints.items(),
                                     key=lambda x: len(x[0])):
            g.mount_ro(device, mount)

        # List packages
        packages = g.inspect_list_applications(root)
        print(f"Installed packages: {len(packages)}")

        # Read a file
        content = g.cat("/etc/hostname")
        print(f"Hostname: {content.strip()}")

        # Extract file
        g.download("/etc/passwd", "./passwd")
```

Perfect for:
- Python-based automation
- Integration with existing tools
- Custom analysis scripts
- Data science workflows

## Coming Soon

**Q1 2024:**
- ✅ CLI tool (done!)
- ✅ Python bindings (done!)
- 🔜 PyPI package
- 🔜 Homebrew formula

**Q2 2024:**
- REST API server
- Web UI
- Container image
- Additional OS support

**Q3 2024:**
- Distributed inspection (cluster mode)
- Cloud provider integration (AWS, Azure, GCP)
- Advanced analytics
- Machine learning for anomaly detection

**Q4 2024:**
- Plugin system
- Custom inspection profiles
- Configuration drift detection
- Enterprise features

## Contributing

GuestKit is open source (LGPL-3.0-or-later) and welcomes contributions!

**Ways to contribute:**
- Report bugs and request features
- Submit pull requests
- Improve documentation
- Write tutorials and guides
- Share your use cases

**GitHub:** https://github.com/ssahani/guestkit

## Conclusion

GuestKit transforms VM disk inspection from a time-consuming, risky manual process into instant, safe, automated analysis.

Whether you're:
- 🔐 **Security engineer** doing incident response
- ☁️ **Cloud architect** planning migrations
- 🛠️ **DevOps engineer** automating infrastructure
- 📊 **Compliance officer** auditing systems
- 🚀 **SRE** troubleshooting production

GuestKit gives you the power to inspect VMs **without the boot wait**.

**Key Takeaways:**

✅ **Speed:** 5 seconds vs. 5 minutes
✅ **Safety:** Read-only, no code execution
✅ **Simplicity:** One command vs. complex procedures
✅ **Automation:** Built for CI/CD and scripting
✅ **Visibility:** Complete OS and config analysis

**Try it today:**

```bash
# Install
cargo install --git https://github.com/ssahani/guestkit

# Inspect your first VM
guestkit inspect your-vm.qcow2

# Go interactive
guestkit interactive your-vm.qcow2

# See all commands
guestkit --help
```

**Join the community:**
- ⭐ Star on GitHub
- 🐛 Report issues
- 💬 Join discussions
- 📖 Read the docs
- 🚀 Share your success stories

The future of VM inspection is here. No booting required.

---

**About the Author**

Built by systems engineers who were tired of waiting for VMs to boot just to check what's inside them. Written in Rust because we believe tools should be fast, safe, and reliable.

**Follow the project:**
- GitHub: https://github.com/ssahani/guestkit
- PyPI: Coming soon
- Documentation: https://github.com/ssahani/guestkit/docs

---

*Have questions or want to share how you're using GuestKit? Open an issue on GitHub or reach out on LinkedIn!*

**Tags:** #Rust #DevOps #CloudComputing #Cybersecurity #VMware #VirtualMachine #Automation #OpenSource #SRE #InfrastructureAsCode
