# Best Practices Guide

Expert recommendations for using guestctl effectively and safely.

## General Principles

### 1. Always Use Read-Only Mode for Inspection

```bash
# Good: Read-only inspection (default)
guestctl inspect vm.qcow2

# Good: Explicit read-only
guestctl interactive vm.qcow2 --readonly

# Caution: Read-write mode (only when modifying)
guestctl interactive vm.qcow2 --read-write
```

**Why:** Read-only mode prevents accidental modifications and allows safe inspection of running VMs.

### 2. Backup Before Modification

```bash
# Always backup before making changes
cp vm.qcow2 vm.qcow2.backup

# Or create a snapshot
qemu-img create -f qcow2 -b vm.qcow2 -F qcow2 vm-snapshot.qcow2

# Then modify safely
guestctl interactive vm.qcow2 --read-write
```

**Why:** Mistakes in VM modification can render systems unbootable.

### 3. Use Appropriate Output Formats

```bash
# Human reading: Pretty text (default)
guestctl inspect vm.qcow2

# Automation/scripting: JSON
guestctl inspect vm.qcow2 --output json | jq '.os.hostname'

# Documentation: HTML export
guestctl inspect vm.qcow2 --export html --export-output report.html

# Data analysis: CSV
guestctl packages vm.qcow2 --output csv | sort
```

**Why:** Different formats serve different purposes efficiently.

### 4. Leverage Caching for Repeated Inspections

```bash
# First run: ~30 seconds
guestctl inspect vm.qcow2 --cache

# Subsequent runs: <0.5 seconds (60x faster!)
guestctl inspect vm.qcow2 --cache

# Clear cache when VM changes
guestctl cache-clear
```

**Why:** Caching dramatically speeds up workflow for unchanged VMs.

### 5. Use Inspection Profiles

```bash
# Security audit
guestctl inspect vm.qcow2 --profile security

# Migration planning
guestctl inspect vm.qcow2 --profile migration

# Performance analysis
guestctl inspect vm.qcow2 --profile performance
```

**Why:** Profiles focus on relevant information for your task.

## Disk Format Optimization

### Choose the Right Format

| Format | Use Case | Advantages | Disadvantages |
|--------|----------|------------|---------------|
| **RAW** | Production, databases | Fastest performance, simple | Large file size, no compression |
| **QCOW2** | Development, testing | Compression, snapshots | Slightly slower than RAW |
| **VMDK** | VMware compatibility | Wide support | Vendor-specific |
| **VDI** | VirtualBox | Native VirtualBox format | Limited tool support |

### Conversion Recommendations

```bash
# For best performance: Convert to RAW
qemu-img convert -f qcow2 -O raw vm.qcow2 vm.raw

# For space efficiency: Convert to compressed QCOW2
qemu-img convert -f vmdk -O qcow2 -c vm.vmdk vm.qcow2

# For repeated inspections: Use RAW with loop device
guestctl inspect vm.raw  # Fast: loop device used
```

**Best Practice:** Use RAW for production VMs, QCOW2 for development/testing.

## Migration Best Practices

### Pre-Migration Checklist

```bash
# 1. Complete inventory
guestctl inspect source-vm.qcow2 --profile migration --output json > inventory.json

# 2. Check disk space
du -sh source-vm.qcow2
df -h /target/path

# 3. Verify format compatibility
guestctl detect source-vm.qcow2

# 4. Test migration on non-production copy first
cp source-vm.qcow2 test-vm.qcow2
# ... perform migration on test-vm.qcow2
# ... verify test-vm boots successfully
# ... then migrate production VM
```

### Device Mapping

```bash
# Use UUIDs instead of device paths
# Good: UUID-based fstab
UUID=abc123... /     ext4  defaults  0 1

# Avoid: Device path-based fstab (breaks during migration)
/dev/sda1      /     ext4  defaults  0 1
```

**Why:** UUIDs are stable across hypervisors and migrations.

### Post-Migration Verification

```bash
# After migration, verify:
guestctl inspect migrated-vm.qcow2
# Check:
# - OS detects correctly
# - Network configuration present
# - Users and services listed
# - No errors in output

# Test boot in target hypervisor
virt-install --name test-boot --disk migrated-vm.qcow2 --import
```

## Performance Optimization

### Disk I/O

```bash
# Use loop devices for RAW/IMG/ISO (faster)
guestctl inspect disk.raw  # Fast: loop device

# Convert QCOW2 to RAW for better performance
qemu-img convert -O raw vm.qcow2 vm.raw

# Optimize QCOW2 after modifications
qemu-img convert -O qcow2 -c vm.qcow2 vm-optimized.qcow2
```

### Parallel Processing

```bash
# Inspect multiple VMs in parallel
guestctl inspect-batch *.qcow2 --parallel 4 --cache

# Use parallel with cache for best performance
# First run: 4x speedup (parallel)
# Subsequent runs: 60x speedup (cache)
```

### Resource Management

```bash
# For large VMs, use streaming operations
guestctl cat large-vm.qcow2 /var/log/huge.log | grep ERROR

# Instead of extracting entire file
# Avoid: guestctl extract large-vm.qcow2 /var/log/huge.log ./huge.log
```

## Security Best Practices

### Principle of Least Privilege

```bash
# Use read-only mode (no sudo required for inspection)
guestctl inspect vm.qcow2

# Only use elevated privileges when necessary
sudo guestctl interactive vm.qcow2 --read-write
```

### Secure Password Handling

```bash
# Never hardcode passwords
# Bad: echo "password123" | guestctl ...

# Good: Use environment variables
export LUKS_PASSPHRASE="$(pass show vm-encryption)"
guestctl interactive encrypted-vm.qcow2

# Or prompt user
read -s -p "Enter LUKS passphrase: " LUKS_PASS
```

### Audit Before Execution

```bash
# When using --profile security, review findings
guestctl inspect vm.qcow2 --profile security --output json > audit.json

# Check for:
jq '.security.ssh_permit_root_login' audit.json     # Should be "no"
jq '.security.ssh_password_auth' audit.json         # Should be "no"
jq '.security.firewall_status' audit.json           # Should be "active"
```

### Secure File Extraction

```bash
# Be cautious extracting files (malware risk)
guestctl extract untrusted-vm.qcow2 /suspicious/file.exe ./malware-sample.exe

# Run in isolated environment or scan with antivirus
clamscan ./malware-sample.exe
```

## Windows-Specific Best Practices

### VirtIO Driver Injection

```bash
# Always inject VirtIO drivers BEFORE first boot
virt-win-reg windows.qcow2 --merge virtio-drivers.reg

# Test boot with VirtIO drivers attached
# If boot fails, attach virtio-win.iso for manual installation
```

### Registry Modification Safety

```bash
# Backup registry before modifications
guestctl interactive windows.qcow2
> mount C:
> download "C:\\Windows\\System32\\config\\SYSTEM" ./SYSTEM.backup
> download "C:\\Windows\\System32\\config\\SOFTWARE" ./SOFTWARE.backup

# Make registry changes
> registry-set "HKLM\\..." "..." "..."

# If something breaks, restore from backup
> upload ./SYSTEM.backup "C:\\Windows\\System32\\config\\SYSTEM"
```

### Activation Handling

```bash
# Check activation before migration
guestctl inspect windows.qcow2 | grep -i activation

# Document product key (if visible)
# Re-activate after migration if needed
# Note: Significant hardware changes may require reactivation
```

## Python API Best Practices

### Always Use Context Managers

```python
from guestctl import Guestfs

# Good: Automatic cleanup
with Guestfs() as g:
    g.add_drive_ro("vm.qcow2")
    g.launch()
    roots = g.inspect_os()
    # ... work with VM
# Automatically calls shutdown()

# Avoid: Manual cleanup (easy to forget)
g = Guestfs()
g.add_drive_ro("vm.qcow2")
g.launch()
# ... work with VM
g.shutdown()  # Must remember to call this!
```

### Error Handling

```python
from guestctl import Guestfs
import logging

def safe_inspect(image_path):
    try:
        with Guestfs() as g:
            g.add_drive_ro(image_path)
            g.launch()
            roots = g.inspect_os()

            if not roots:
                logging.warning(f"No OS detected in {image_path}")
                return None

            return g.inspect_get_distro(roots[0])

    except Exception as e:
        logging.error(f"Failed to inspect {image_path}: {e}")
        return None
```

### Type Hints

```python
from guestctl import Guestfs
from typing import List, Optional

def get_users(image_path: str) -> Optional[List[str]]:
    """Get list of regular usernames from VM image."""
    with Guestfs() as g:
        g.add_drive_ro(image_path)
        g.launch()

        roots = g.inspect_os()
        if not roots:
            return None

        users = g.inspect_users(roots[0])
        return [u.username for u in users if 1000 <= int(u.uid) < 65534]
```

## Automation Best Practices

### Scripting for Reliability

```bash
#!/bin/bash
set -euo pipefail  # Exit on error, undefined var, pipe failure

IMAGE="$1"

# Validate input
if [[ ! -f "$IMAGE" ]]; then
    echo "Error: Image file not found: $IMAGE" >&2
    exit 1
fi

# Check file format
FORMAT=$(guestctl detect "$IMAGE" --output json | jq -r '.format')
if [[ "$FORMAT" == "unknown" ]]; then
    echo "Error: Unknown disk format: $IMAGE" >&2
    exit 1
fi

# Perform operation with error handling
if ! guestctl inspect "$IMAGE" --output json > output.json; then
    echo "Error: Inspection failed for $IMAGE" >&2
    exit 1
fi

# Validate JSON output
if ! jq empty output.json 2>/dev/null; then
    echo "Error: Invalid JSON output" >&2
    exit 1
fi

echo "Success: Inspection complete"
```

### Logging

```bash
# Enable verbose logging for troubleshooting
export RUST_LOG=debug
guestctl inspect vm.qcow2 2> debug.log

# Or selective logging
export RUST_LOG=guestctl::guestfs=debug
guestctl inspect vm.qcow2
```

### CI/CD Integration

```yaml
# GitHub Actions example
- name: Inspect VM image
  run: |
    # Install guestctl
    cargo install guestkit

    # Inspect test image
    guestctl inspect test-vm.qcow2 --output json > inspection.json

    # Validate expected configuration
    jq -e '.os.distribution == "ubuntu"' inspection.json
    jq -e '.os.hostname == "test-vm"' inspection.json
```

## Troubleshooting Best Practices

### Systematic Debugging

```bash
# 1. Verify file integrity
file vm.qcow2
qemu-img info vm.qcow2

# 2. Check format detection
guestctl detect vm.qcow2

# 3. Inspect with verbose output
guestctl inspect vm.qcow2 --verbose 2>&1 | tee debug.log

# 4. Try interactive mode
guestctl interactive vm.qcow2
> ls
> mount /
> ls /
```

### Common Issues

**Problem:** "Failed to launch appliance"
```bash
# Solution: Check KVM availability
ls -la /dev/kvm
# If missing, enable virtualization in BIOS

# Or use TCG mode (slower)
export LIBGUESTFS_BACKEND=direct
```

**Problem:** "No operating system detected"
```bash
# Solution: Check if disk is bootable
guestctl filesystems vm.qcow2
guestctl list vm.qcow2 /boot

# Verify boot configuration
guestctl interactive vm.qcow2
> mount /
> cat /etc/fstab
> ls /boot
```

## Documentation Best Practices

### Document Your Workflow

```bash
# Create inspection reports
guestctl inspect vm.qcow2 --export markdown --export-output inventory.md

# Include in documentation
# VMs are self-documenting with guestctl!
```

### Version Control

```bash
# Track VM configuration changes
guestctl inspect vm.qcow2 --output json > vm-config-$(date +%Y%m%d).json

# Commit to git
git add vm-config-*.json
git commit -m "Update VM configuration snapshot"

# Compare configurations
diff <(jq -S . vm-config-old.json) <(jq -S . vm-config-new.json)
```

## Testing Best Practices

### Test Migrations

```bash
# Always test on copies
cp production.qcow2 test-migration.qcow2

# Perform migration
guestctl interactive test-migration.qcow2 --read-write
# ... make changes ...

# Verify boot
virt-install --name test --disk test-migration.qcow2 --import

# Only proceed with production after successful test
```

### Regression Testing

```bash
# Baseline inspection
guestctl inspect vm.qcow2 --output json > baseline.json

# After changes
guestctl inspect vm.qcow2 --output json > modified.json

# Compare (should be minimal differences)
diff <(jq -S . baseline.json) <(jq -S . modified.json)
```

## Maintenance Best Practices

### Regular Audits

```bash
# Monthly security audit
guestctl inspect vm.qcow2 --profile security --output json > audit-$(date +%Y-%m).json

# Check for issues
jq '.security.ssh_permit_root_login' audit-*.json
jq '.security.outdated_packages' audit-*.json
```

### Cache Management

```bash
# View cache statistics
guestctl cache-stats

# Clear stale cache entries (run weekly)
guestctl cache-clear --older-than 7d

# Or clear all cache
guestctl cache-clear
```

### Update guestctl

```bash
# Check version
guestctl version

# Update to latest
cargo install guestkit --force

# Or from source
cd guestkit
git pull
cargo build --release
cargo install --path .
```

## Checklist Summary

### Before VM Modification
- [ ] Create backup/snapshot
- [ ] Test on non-production VM first
- [ ] Document current state
- [ ] Verify you have correct permissions
- [ ] Read operation documentation

### During Modification
- [ ] Use `--verbose` for important operations
- [ ] Save operation logs
- [ ] Verify each step before proceeding
- [ ] Test incrementally, not all at once

### After Modification
- [ ] Run `guestctl inspect` to verify
- [ ] Test boot in target environment
- [ ] Document changes made
- [ ] Update configuration management
- [ ] Create new backup

### For Production Use
- [ ] Use read-only mode by default
- [ ] Enable caching for performance
- [ ] Use JSON output for automation
- [ ] Implement error handling
- [ ] Log all operations
- [ ] Have rollback plan

## Further Reading

- [CLI Guide](cli-guide.md) - Complete command reference
- [VM Migration Guide](vm-migration.md) - Migration workflows
- [Windows Support](windows-support.md) - Windows-specific practices
- [Troubleshooting](troubleshooting.md) - Problem resolution
- [Python Bindings](python-bindings.md) - Python API best practices

## Support

For questions about best practices:
- GitHub Discussions: https://github.com/ssahani/guestkit/discussions
- Tag with: `best-practices`, `help-wanted`
- Share your own best practices to help the community!
