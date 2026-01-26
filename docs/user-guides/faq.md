# Frequently Asked Questions (FAQ)

Common questions and answers about guestctl.

## General Questions

### What is guestctl?

guestctl is a pure Rust toolkit for VM disk inspection and manipulation without booting the VM. It provides:
- Beautiful emoji-enhanced CLI output
- Complete OS detection (Linux & Windows)
- VM migration support (fstab/crypttab rewriter)
- Windows registry parsing
- Python bindings for automation
- 578 disk image manipulation functions

### How does guestctl compare to libguestfs?

| Feature | guestctl | libguestfs |
|---------|----------|------------|
| **Language** | Pure Rust | C + bindings |
| **Dependencies** | Minimal (qemu-img optional) | Many C libraries |
| **Installation** | Single binary | Complex dependencies |
| **Performance** | Fast (loop devices) | Good |
| **API Coverage** | 97.4% (578 functions) | 100% (native) |
| **Windows Support** | Registry parsing built-in | Requires Windows tools |
| **Visual Output** | Beautiful emojis + colors | Plain text |
| **Migration** | Built-in fstab/crypttab rewriter | Manual scripting |

**Bottom Line:** guestctl is easier to install, faster for common formats (RAW/IMG/ISO), and has better UX. libguestfs has been around longer and has complete API coverage.

### Why does guestctl require sudo?

guestctl requires elevated privileges for:
- Mounting loop devices (`losetup`)
- Loading NBD kernel module (`modprobe nbd`)
- Accessing `/dev/nbd*` devices
- Mounting filesystems

**Exception:** Read-only operations with cached results don't need sudo:
```bash
# First run needs sudo
sudo guestctl inspect vm.qcow2 --cache

# Subsequent runs from cache (no sudo!)
guestctl inspect vm.qcow2 --cache
```

### Is guestctl production-ready?

**Yes!** guestctl v0.3.1+ is production-ready with:
- 97.4% API implementation coverage (578 functions)
- Comprehensive test suite with CI/CD
- Used in [hyper2kvm](https://github.com/ssahani/hyper2kvm) for production VM migrations
- Pure Rust for memory safety
- Extensive documentation and examples

### Can I use guestctl on running VMs?

**Read-only: Yes (with caution)**
```bash
# Read-only inspection of running VM (risky but possible)
guestctl inspect running-vm.qcow2
```

**Read-write: NO!**
Never modify a disk image while the VM is running. This will cause:
- Data corruption
- Filesystem damage
- Potential data loss

**Best Practice:** Shut down VM before using guestctl for modifications.

## Installation & Setup

### How do I install guestctl?

**From crates.io:**
```bash
cargo install guestkit
```

**From source:**
```bash
git clone https://github.com/ssahani/guestkit
cd guestkit
cargo build --release
sudo cp target/release/guestctl /usr/local/bin/
```

**Python bindings:**
```bash
pip install guestctl
```

### What are the system requirements?

**Operating System:**
- Linux (primary support)
- macOS (limited support, no KVM)
- Windows WSL2 (experimental)

**Dependencies:**
- **Required:** Rust 1.70+ (for building)
- **Optional:** qemu-img (for format conversion)
- **Runtime:** Linux kernel with loop device support (built-in)

**Hardware:**
- KVM support recommended for performance
- Minimum 2GB RAM
- 1GB free disk space

### Why does installation fail on Ubuntu 20.04?

**Issue:** Rust version too old

**Solution:**
```bash
# Update Rust to latest
rustup update

# Or install newer Rust
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
source $HOME/.cargo/env

# Then install guestctl
cargo install guestkit
```

## Usage Questions

### How do I inspect a QCOW2 file?

```bash
# Basic inspection
sudo guestctl inspect vm.qcow2

# With JSON output
sudo guestctl inspect vm.qcow2 --output json

# With caching for speed
sudo guestctl inspect vm.qcow2 --cache
```

### Can I inspect Windows VMs?

**Yes!** guestctl v0.3.1+ has full Windows support:

```bash
sudo guestctl inspect windows.qcow2
```

Output includes:
- Windows version (Windows 10, 11, Server)
- Windows edition (Home, Pro, Enterprise)
- Build number
- Hostname
- Installed applications
- Users

See [Windows Support Guide](windows-support.md) for details.

### How do I extract files from a VM?

```bash
# Extract single file
sudo guestctl extract vm.qcow2 /etc/hostname ./hostname.txt

# Or use interactive mode
sudo guestctl interactive vm.qcow2
> mount /
> download /etc/hostname ./hostname.txt
> exit

# Or use cat to view
sudo guestctl cat vm.qcow2 /etc/hostname
```

### How do I list files in a VM?

```bash
# List directory
sudo guestctl list vm.qcow2 /etc

# Or interactive mode
sudo guestctl interactive vm.qcow2
> mount /
> ls /etc
> exit
```

### Why is guestctl slow on QCOW2 files?

**Reason:** QCOW2 requires NBD (Network Block Device) which is slower than loop devices.

**Solutions:**
1. **Use caching:**
   ```bash
   sudo guestctl inspect vm.qcow2 --cache  # First run slow, subsequent fast
   ```

2. **Convert to RAW:**
   ```bash
   qemu-img convert -O raw vm.qcow2 vm.raw
   sudo guestctl inspect vm.raw  # Much faster (loop device)
   ```

3. **Use parallel processing:**
   ```bash
   sudo guestctl inspect-batch *.qcow2 --parallel 4
   ```

### How do I compare two VMs?

```bash
# Compare two VMs
sudo guestctl diff vm1.qcow2 vm2.qcow2

# Compare multiple VMs against baseline
sudo guestctl compare baseline.qcow2 vm1.qcow2 vm2.qcow2 vm3.qcow2
```

## Migration Questions

### How do I migrate from Hyper-V to KVM?

Complete workflow:

```bash
# 1. Convert VHDX to QCOW2
qemu-img convert -f vhdx -O qcow2 vm.vhdx vm.qcow2

# 2. Inspect and plan
sudo guestctl inspect vm.qcow2 --profile migration

# 3. Modify configuration (Linux VMs)
sudo guestctl interactive vm.qcow2
> mount /
> command "sed -i 's/\/dev\/sda/\/dev\/vda/g' /etc/fstab"
> exit

# 4. For Windows: Inject VirtIO drivers
virt-win-reg vm.qcow2 --merge virtio-drivers.reg

# 5. Test boot
virt-install --name test --disk vm.qcow2 --import
```

See [VM Migration Guide](vm-migration.md) for complete details.

### Can guestctl rewrite fstab automatically?

**Yes!** v0.3.1+ includes universal fstab/crypttab rewriter:

```rust
use guestctl::guestfs::Guestfs;
use std::collections::HashMap;

let mut g = Guestfs::new()?;
g.add_drive("vm.qcow2")?;
g.launch()?;

let roots = g.inspect_os()?;
let mut device_map = HashMap::new();
device_map.insert("/dev/sda", "/dev/vda");

g.rewrite_fstab(&roots[0], &device_map)?;
g.shutdown()?;
```

### How do I migrate encrypted VMs (LUKS)?

```bash
# Inspect encrypted VM
sudo guestctl inspect encrypted.qcow2

# Open LUKS volume
sudo guestctl interactive encrypted.qcow2
> luks-open /dev/sda2 luks-root <passphrase>
> mount /dev/mapper/luks-root /
> # Make modifications
> umount /
> luks-close luks-root
> exit
```

See [VM Migration Guide - Encrypted Volumes](vm-migration.md#encrypted-volume-migration).

## Windows-Specific Questions

### Does guestctl work with Windows VMs?

**Yes!** Full Windows support in v0.3.1+:
- Windows 7 through Windows 11
- Windows Server 2008 R2 through 2022
- Registry parsing for version detection
- User account listing
- Installed application detection
- VirtIO driver injection support

### How does Windows version detection work?

guestctl reads Windows registry hives directly from disk:

```rust
// Internal implementation (simplified)
1. Mount Windows system partition
2. Read C:\Windows\System32\config\SOFTWARE registry hive
3. Parse "HKLM\SOFTWARE\Microsoft\Windows NT\CurrentVersion"
4. Extract ProductName, CurrentBuild, EditionID
5. Map to human-readable version (Windows 11 Pro, etc.)
```

### Can I modify Windows registry offline?

**Yes:**

```bash
# Interactive modification
sudo guestctl interactive windows.qcow2
> mount C:
> registry-set "HKLM\\SYSTEM\\..." "KeyName" "Value"
> exit

# Or use virt-win-reg
virt-win-reg windows.qcow2 --merge custom.reg
```

See [Windows Support Guide](windows-support.md) for details.

### How do I inject VirtIO drivers for Windows?

**Before migration:**
```bash
# Create registry entries for VirtIO drivers
cat > virtio.reg <<EOF
[HKEY_LOCAL_MACHINE\SYSTEM\CurrentControlSet\Services\vioscsi]
"Start"=dword:00000000
EOF

# Inject into registry
virt-win-reg windows.qcow2 --merge virtio.reg
```

**After first boot:**
- Attach virtio-win.iso to VM
- Install drivers from Device Manager

See [Windows Support - VirtIO Drivers](windows-support.md#virtio-driver-injection).

## Performance Questions

### Why is first run slow, subsequent runs fast?

**Caching!** guestctl caches inspection results:

```bash
# First run: ~30 seconds (full inspection)
sudo guestctl inspect vm.qcow2 --cache

# Second run: <0.5 seconds (from cache)
sudo guestctl inspect vm.qcow2 --cache

# Cache is invalidated if VM file changes
```

**Cache location:** `~/.cache/guestctl/`

### How can I speed up batch processing?

**Use parallel processing:**

```bash
# Inspect 8 VMs in parallel (4 workers)
sudo guestctl inspect-batch vm*.qcow2 --parallel 4 --cache

# With progress bar
sudo guestctl inspect-batch vm*.qcow2 --parallel 4 --cache --progress
```

**Performance:** 4x speedup with 4 workers, plus 60x from caching on subsequent runs.

### Should I use RAW or QCOW2 format?

| Criteria | Recommendation |
|----------|----------------|
| **Performance** | RAW (loop device, faster) |
| **Disk space** | QCOW2 (compressed, smaller) |
| **Snapshots** | QCOW2 (built-in snapshots) |
| **Simplicity** | RAW (no overhead) |
| **Development** | QCOW2 (snapshots useful) |
| **Production databases** | RAW (best I/O) |

**Best of both worlds:**
```bash
# Develop with QCOW2 (snapshots)
guestctl inspect dev.qcow2

# Deploy with RAW (performance)
qemu-img convert -O raw dev.qcow2 prod.raw
guestctl inspect prod.raw  # Fast!
```

## API & Development Questions

### Can I use guestctl from Python?

**Yes!** Full Python bindings:

```python
from guestctl import Guestfs

with Guestfs() as g:
    g.add_drive_ro("vm.qcow2")
    g.launch()

    roots = g.inspect_os()
    for root in roots:
        print(f"OS: {g.inspect_get_distro(root)}")
        print(f"Hostname: {g.inspect_get_hostname(root)}")
```

**Installation:**
```bash
pip install guestctl
```

See [Python Bindings Guide](python-bindings.md).

### What API functions are available?

**578 functions across 95 modules including:**
- OS inspection (30+ functions)
- File operations (50+ functions)
- Partition management (20+ functions)
- Filesystem operations (40+ functions)
- LVM support (15+ functions)
- Archive operations (10+ functions)
- Windows registry (15+ functions)
- And many more...

See [API Reference](../api/rust-reference.md) for complete list.

### How do I contribute to guestctl?

```bash
# 1. Fork repository
gh repo fork ssahani/guestkit

# 2. Clone your fork
git clone https://github.com/YOUR_USERNAME/guestkit
cd guestkit

# 3. Create feature branch
git checkout -b feature/my-enhancement

# 4. Make changes and test
cargo test
cargo clippy

# 5. Submit pull request
gh pr create --title "Add: My enhancement"
```

See [CONTRIBUTING.md](../../CONTRIBUTING.md) for guidelines.

## Troubleshooting Questions

### "Failed to launch appliance" error?

**Causes:**
1. KVM not available
2. Insufficient permissions
3. Memory constraints

**Solutions:**
```bash
# 1. Check KVM
ls -la /dev/kvm
# If missing: Enable VT-x/AMD-V in BIOS

# 2. Check permissions
sudo usermod -a -G kvm $USER
# Logout and login for group to take effect

# 3. Check memory
free -h
# Need at least 1GB free

# 4. Try without KVM (slower)
export LIBGUESTFS_BACKEND=direct
sudo guestctl inspect vm.qcow2
```

### "No OS detected" error?

**Common causes:**
1. Disk not bootable
2. Encrypted without passphrase
3. Unsupported OS
4. Corrupted filesystem

**Debug steps:**
```bash
# Check disk structure
sudo guestctl filesystems vm.qcow2

# Try interactive mode
sudo guestctl interactive vm.qcow2
> list-devices
> list-partitions
> mount /dev/sda1
> ls /
```

See [Troubleshooting Guide](troubleshooting.md).

### Why can't guestctl access NBD devices?

**Error:** "Could not load NBD module"

**Solution:**
```bash
# Load NBD module manually
sudo modprobe nbd max_part=8

# Make persistent
echo "nbd" | sudo tee /etc/modules-load.d/nbd.conf

# Alternative: Use RAW format (no NBD needed)
qemu-img convert -O raw vm.qcow2 vm.raw
```

### "Permission denied" even with sudo?

**SELinux issue:**

```bash
# Check SELinux status
getenforce

# Temporary fix
sudo setenforce 0
sudo guestctl inspect vm.qcow2

# Permanent fix (add SELinux policy)
# Or disable SELinux in /etc/selinux/config
```

## Licensing Questions

### What license is guestctl under?

**LGPL-3.0-or-later** (GNU Lesser General Public License v3.0 or later)

**What this means:**
- ✅ Free to use commercially
- ✅ Can link dynamically in proprietary software
- ✅ Can modify for internal use
- ⚠️ Must share modifications if distributing
- ⚠️ Must preserve license notices

### Can I use guestctl in commercial products?

**Yes!** LGPL allows commercial use.

**Requirements:**
- Include LGPL license text
- Credit guestctl project
- If you modify guestctl itself, share modifications
- If you just use guestctl as library, no source sharing required

### Is there commercial support?

**Community support:**
- GitHub Issues: https://github.com/ssahani/guestkit/issues
- GitHub Discussions: https://github.com/ssahani/guestkit/discussions

**Commercial support:** Contact ssahani@vmware.com for enterprise support options.

## Feature Requests

### Will guestctl support feature X?

Check the [Enhancement Roadmap](../development/enhancement-roadmap.md) for planned features.

**Request a feature:**
```bash
# Open feature request on GitHub
gh issue create --title "Feature: My feature" --label enhancement
```

### When will async Python API be available?

**Status:** Code is ready, waiting for pyo3-asyncio to support PyO3 0.22+

**Timeline:** Expected in v0.4.0 (Q2 2026)

**Current workaround:** Use threading in Python:
```python
from concurrent.futures import ThreadPoolExecutor
from guestctl import Guestfs

def inspect_vm(path):
    with Guestfs() as g:
        g.add_drive_ro(path)
        g.launch()
        return g.inspect_os()

with ThreadPoolExecutor(max_workers=4) as executor:
    results = executor.map(inspect_vm, vm_paths)
```

### Will guestctl support ARM/aarch64?

**Partial support** in current version:
- ✅ Can run on ARM hosts
- ✅ Can inspect ARM VM images
- ⚠️ Limited KVM support on ARM

**Full ARM support** planned for v0.5.0.

## Getting Help

### Where can I get help?

1. **Documentation:** Start with [Getting Started Guide](getting-started.md)
2. **FAQ:** This document
3. **GitHub Discussions:** https://github.com/ssahani/guestkit/discussions
4. **GitHub Issues:** https://github.com/ssahani/guestkit/issues (for bugs)
5. **Email:** ssahani@vmware.com (for private inquiries)

### How do I report a bug?

```bash
# Use GitHub CLI
gh issue create \
  --title "Bug: Description" \
  --label bug \
  --body "Steps to reproduce:
1. Run guestctl inspect vm.qcow2
2. Error occurs: ...

Expected: ...
Actual: ...

Version: $(guestctl version)
OS: $(uname -a)"
```

**Include:**
- guestctl version
- OS and version
- Disk image format
- Complete error message
- Steps to reproduce

### How can I contribute documentation?

Documentation contributions welcome!

```bash
# Clone repo
git clone https://github.com/ssahani/guestkit
cd guestkit/docs

# Edit documentation
vim docs/user-guides/my-guide.md

# Submit PR
gh pr create --title "Docs: Improve my-guide"
```

## Quick Links

- [Getting Started](getting-started.md) - Quick start guide
- [CLI Guide](cli-guide.md) - Command reference
- [VM Migration Guide](vm-migration.md) - Migration workflows
- [Windows Support](windows-support.md) - Windows-specific features
- [Best Practices](best-practices.md) - Expert recommendations
- [Troubleshooting](troubleshooting.md) - Problem resolution
- [GitHub Repository](https://github.com/ssahani/guestkit)
- [Issue Tracker](https://github.com/ssahani/guestkit/issues)

## Still Have Questions?

If your question isn't answered here:
1. Search [GitHub Discussions](https://github.com/ssahani/guestkit/discussions)
2. Search [GitHub Issues](https://github.com/ssahani/guestkit/issues)
3. Ask in [new Discussion](https://github.com/ssahani/guestkit/discussions/new)
4. For bugs, [create Issue](https://github.com/ssahani/guestkit/issues/new)

We're here to help! 🚀
