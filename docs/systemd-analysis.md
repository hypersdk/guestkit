# Systemd Analysis Guide

This guide covers guestctl's comprehensive systemd analysis capabilities for deep VM inspection without running the VM.

## Overview

Guestctl provides three powerful commands for analyzing systemd-based Linux VMs:

1. **`systemd-journal`** - Analyze systemd journal logs
2. **`systemd-services`** - Inspect services and dependencies
3. **`systemd-boot`** - Analyze boot performance

All commands work offline by mounting VM disk images read-only and extracting systemd data for analysis.

---

## systemd-journal: Journal Log Analysis

Analyze systemd journal logs to troubleshoot issues, audit security events, and understand system behavior.

### Basic Usage

```bash
# View all journal entries
guestctl systemd-journal vm.qcow2

# Show only errors (priority 0-3)
guestctl systemd-journal vm.qcow2 --errors

# Show only warnings (priority 4)
guestctl systemd-journal vm.qcow2 --warnings

# Display statistics summary
guestctl systemd-journal vm.qcow2 --stats
```

### Filtering Options

```bash
# Filter by priority level (0=emerg, 3=err, 4=warning, 6=info)
guestctl systemd-journal vm.qcow2 --priority 3

# Filter by systemd unit
guestctl systemd-journal vm.qcow2 --unit sshd.service

# Limit number of entries
guestctl systemd-journal vm.qcow2 --limit 100

# Combine filters
guestctl systemd-journal vm.qcow2 --unit nginx.service --priority 4 --limit 50
```

### Priority Levels

| Level | Name    | Description |
|-------|---------|-------------|
| 0     | EMERG   | Emergency: system is unusable |
| 1     | ALERT   | Alert: action must be taken immediately |
| 2     | CRIT    | Critical conditions |
| 3     | ERR     | Error conditions |
| 4     | WARNING | Warning conditions |
| 5     | NOTICE  | Normal but significant |
| 6     | INFO    | Informational messages |
| 7     | DEBUG   | Debug-level messages |

### Output Format

Journal entries are displayed with color coding:

```
2024-01-26 14:30:45 [ERR] sshd.service: Failed password for root
2024-01-26 14:31:02 [WARNING] systemd: Unit network.service entered failed state
2024-01-26 14:31:15 [INFO] systemd: Started Session 42 of user john
```

- **Red**: Emergency, Alert, Critical errors (0-2)
- **Bright Red**: Errors (3)
- **Yellow**: Warnings (4)
- **Cyan**: Notices (5)
- **White**: Info and Debug (6-7)

### Statistics Output

The `--stats` flag provides a comprehensive overview:

```bash
guestctl systemd-journal vm.qcow2 --stats
```

Output includes:
- Total entry count
- Error count (priority 0-3)
- Warning count (priority 4)
- Breakdown by priority level
- Top 10 units by entry count

### Use Cases

**Security Audit:**
```bash
# Find authentication failures
guestctl systemd-journal vm.qcow2 --unit sshd.service --errors

# Check for sudo usage
guestctl systemd-journal vm.qcow2 --unit sudo.service
```

**Troubleshooting:**
```bash
# Find all errors in the last boot
guestctl systemd-journal vm.qcow2 --errors

# Check specific service issues
guestctl systemd-journal vm.qcow2 --unit postgresql.service --warnings
```

**Forensics:**
```bash
# Get journal statistics for investigation
guestctl systemd-journal vm.qcow2 --stats

# List all critical events
guestctl systemd-journal vm.qcow2 --priority 2
```

---

## systemd-services: Service Analysis

Inspect systemd services, analyze dependencies, and identify issues.

### Basic Usage

```bash
# List all services
guestctl systemd-services vm.qcow2

# Show only failed services
guestctl systemd-services vm.qcow2 --failed

# Show dependency tree for a service
guestctl systemd-services vm.qcow2 --service sshd.service
```

### Service Listing

Default output shows all services with their state and description:

```bash
guestctl systemd-services vm.qcow2
```

Output format:
```
Service                                            State           Description
----------------------------------------------------------------------------------------------------
sshd.service                                       active          OpenSSH server daemon
nginx.service                                      active          A high performance web server
postgresql.service                                 inactive        PostgreSQL database server
failed-service.service                             failed          Example failed service
```

**Color coding:**
- **Green**: Active services
- **Red**: Failed services
- **Dimmed**: Inactive services

### JSON Output

For programmatic processing:

```bash
guestctl systemd-services vm.qcow2 --output json > services.json
```

### Dependency Analysis

Show the full dependency tree for a service:

```bash
guestctl systemd-services vm.qcow2 --service nginx.service
```

Output shows:
- Service name
- Total dependency count
- Hierarchical dependency tree
- Dependency types (Requires, Wants, After, Before)

### Dependency Diagrams

Generate Mermaid diagrams for visualization:

```bash
guestctl systemd-services vm.qcow2 --service sshd.service --diagram > sshd-deps.md
```

The diagram shows:
- Service relationships
- Dependency direction
- Can be rendered in GitHub, GitLab, or Mermaid Live Editor

### Failed Services

Quickly identify problematic services:

```bash
guestctl systemd-services vm.qcow2 --failed
```

Output includes:
- Service name (highlighted in red)
- Service description
- Empty output if no failures

### Use Cases

**Health Check:**
```bash
# Quick scan for failed services
guestctl systemd-services vm.qcow2 --failed

# List all services for manual review
guestctl systemd-services vm.qcow2
```

**Dependency Analysis:**
```bash
# Understand service startup order
guestctl systemd-services vm.qcow2 --service postgresql.service

# Visualize complex dependencies
guestctl systemd-services vm.qcow2 --service network.target --diagram
```

**Migration Planning:**
```bash
# Export service list for documentation
guestctl systemd-services vm.qcow2 --output json > current-services.json

# Compare services between VMs
diff <(guestctl systemd-services vm1.qcow2 --output json) \
     <(guestctl systemd-services vm2.qcow2 --output json)
```

---

## systemd-boot: Boot Performance Analysis

Analyze boot performance, identify slow services, and get optimization recommendations.

### Basic Usage

```bash
# Show boot timing and slowest services
guestctl systemd-boot vm.qcow2

# Show top 20 slowest services
guestctl systemd-boot vm.qcow2 --top 20

# Get optimization recommendations
guestctl systemd-boot vm.qcow2 --recommendations

# Display summary statistics
guestctl systemd-boot vm.qcow2 --summary

# Generate boot timeline diagram
guestctl systemd-boot vm.qcow2 --timeline > boot-timeline.md
```

### Boot Timing Breakdown

Default output shows overall boot performance:

```
Boot Performance Analysis
─────────────────────────

Total Boot Time: 15.23s
  - Kernel:     3.45s
  - Initrd:     2.10s
  - Userspace:  9.68s

Top 10 Slowest Services:

Service                                            Time
────────────────────────────────────────────────────────
postgresql.service                                 5.23s
mariadb.service                                    3.89s
NetworkManager-wait-online.service                 2.45s
systemd-networkd.service                           1.67s
```

**Color coding:**
- **Red**: >3 seconds (needs optimization)
- **Yellow**: 1-3 seconds (could be improved)
- **Green**: <1 second (good performance)

### Optimization Recommendations

Get actionable advice for improving boot time:

```bash
guestctl systemd-boot vm.qcow2 --recommendations
```

Example output:
```
Boot Optimization Recommendations
──────────────────────────────────

⚠ Service 'postgresql.service' takes 5.23s to activate. Consider optimization.
⚠ Service 'NetworkManager-wait-online.service' takes 2.45s to activate. Consider optimization.
✓ Boot performance looks good!
```

Recommendations include:
- Services taking >3s to activate
- High kernel boot time (>5s)
- Slow initrd (>3s)
- Overall slow boot (>30s)

### Summary Statistics

Detailed boot metrics:

```bash
guestctl systemd-boot vm.qcow2 --summary
```

Output includes:
- Total boot time
- Kernel, initrd, userspace breakdown
- Top 5 slowest services with timing
- Formatted for easy reading

### Boot Timeline Diagram

Visualize the boot process:

```bash
guestctl systemd-boot vm.qcow2 --timeline > timeline.md
```

Generates a Mermaid Gantt chart showing:
- Kernel initialization phase
- Initrd phase
- Userspace service startup
- Service timing and parallelization
- Critical path services

**Color coding in diagram:**
- **Critical**: Services >3s (red)
- **Active**: Services 1-3s (yellow)
- **Done**: Services <1s (green)

### Use Cases

**Performance Troubleshooting:**
```bash
# Identify boot bottlenecks
guestctl systemd-boot vm.qcow2 --top 20

# Get specific recommendations
guestctl systemd-boot vm.qcow2 --recommendations
```

**Optimization Validation:**
```bash
# Before optimization
guestctl systemd-boot vm-before.qcow2 --summary > before.txt

# After optimization
guestctl systemd-boot vm-after.qcow2 --summary > after.txt

# Compare results
diff before.txt after.txt
```

**Documentation:**
```bash
# Generate boot timeline for documentation
guestctl systemd-boot vm.qcow2 --timeline > docs/boot-analysis.md

# Export metrics for reporting
guestctl systemd-boot vm.qcow2 --summary > reports/boot-metrics.txt
```

**Capacity Planning:**
```bash
# Analyze multiple VMs
for vm in vm*.qcow2; do
    echo "=== $vm ===" >> boot-report.txt
    guestctl systemd-boot "$vm" --summary >> boot-report.txt
    echo "" >> boot-report.txt
done
```

---

## Advanced Workflows

### Complete System Audit

Comprehensive VM analysis combining all commands:

```bash
#!/bin/bash
VM="production-server.qcow2"
REPORT_DIR="audit-$(date +%Y%m%d)"
mkdir -p "$REPORT_DIR"

echo "Analyzing $VM..."

# Journal analysis
echo "1. Extracting journal errors..."
guestctl systemd-journal "$VM" --errors > "$REPORT_DIR/journal-errors.txt"
guestctl systemd-journal "$VM" --stats > "$REPORT_DIR/journal-stats.txt"

# Service analysis
echo "2. Checking services..."
guestctl systemd-services "$VM" --failed > "$REPORT_DIR/failed-services.txt"
guestctl systemd-services "$VM" --output json > "$REPORT_DIR/services.json"

# Boot performance
echo "3. Analyzing boot performance..."
guestctl systemd-boot "$VM" --summary > "$REPORT_DIR/boot-summary.txt"
guestctl systemd-boot "$VM" --recommendations > "$REPORT_DIR/boot-recommendations.txt"

echo "Audit complete! Results in $REPORT_DIR/"
```

### Security Hardening Check

Identify potential security issues:

```bash
#!/bin/bash
VM="$1"

echo "=== Security Audit Report ==="
echo ""

# Check for failed authentication
echo "Failed SSH Logins:"
guestctl systemd-journal "$VM" --unit sshd.service --errors | grep -i "failed\|invalid" || echo "None found"
echo ""

# Check for failed services
echo "Failed Services:"
guestctl systemd-services "$VM" --failed || echo "None found"
echo ""

# Check for critical journal entries
echo "Critical Events:"
guestctl systemd-journal "$VM" --priority 2 --limit 20 || echo "None found"
```

### Performance Baseline Creation

Establish performance metrics for comparison:

```bash
#!/bin/bash
VM="$1"
BASELINE_DIR="baseline-$(date +%Y%m%d)"
mkdir -p "$BASELINE_DIR"

# Capture boot metrics
guestctl systemd-boot "$VM" --summary > "$BASELINE_DIR/boot-metrics.txt"
guestctl systemd-boot "$VM" --top 30 > "$BASELINE_DIR/slow-services.txt"

# Capture service status
guestctl systemd-services "$VM" --output json > "$BASELINE_DIR/services.json"

# Capture journal stats
guestctl systemd-journal "$VM" --stats > "$BASELINE_DIR/journal-stats.txt"

echo "Baseline saved to $BASELINE_DIR/"
```

---

## Troubleshooting

### "No operating systems found"

The disk image doesn't contain a recognizable OS or partitions:

```bash
# Verify image format
guestctl detect vm.qcow2

# Check image integrity
qemu-img check vm.qcow2

# Try verbose mode
guestctl -v systemd-journal vm.qcow2
```

### "No journal entries found"

The VM doesn't use systemd or journal isn't available:

```bash
# Check if systemd is used
guestctl inspect vm.qcow2 | grep -i init

# Verify journal directory exists
guestctl cat vm.qcow2 /var/log/journal
```

### Empty Service List

Systemd directories aren't accessible or don't exist:

```bash
# Check systemd directories
guestctl cat vm.qcow2 /etc/systemd/system
guestctl cat vm.qcow2 /lib/systemd/system
```

### "No service timing data available"

Boot timing data isn't available in the VM:

```bash
# This is expected if the VM hasn't captured boot data
# The command will show estimated timing instead
# Use --summary to see estimated values
guestctl systemd-boot vm.qcow2 --summary
```

---

## Best Practices

### Read-Only Analysis

All commands mount disks read-only for safety:
- No modifications to VM disk images
- Safe for production VMs
- Can analyze running VM snapshots

### Temporary Files

Analysis creates temporary directories automatically:
- Cleaned up on command completion
- Ensure sufficient /tmp space (typically <100MB)
- Use `df -h /tmp` to check available space

### Large Journal Files

For VMs with extensive journals:
```bash
# Use filtering to reduce data
guestctl systemd-journal vm.qcow2 --priority 4 --limit 1000

# Focus on specific time periods
# (Future enhancement - timestamp filtering)

# Get stats first to understand volume
guestctl systemd-journal vm.qcow2 --stats
```

### Performance Considerations

- Analysis time depends on VM size and journal volume
- Typical analysis: 5-30 seconds
- Large journals (>100MB): 30-60 seconds
- Use `--verbose` to see progress

### Combining with Other Tools

```bash
# Export for external analysis
guestctl systemd-services vm.qcow2 --output json | jq '.[] | select(.state=="failed")'

# Generate reports
guestctl systemd-boot vm.qcow2 --timeline | pandoc -o boot-timeline.pdf

# Compare VMs
diff <(guestctl systemd-services vm1.qcow2 --failed) \
     <(guestctl systemd-services vm2.qcow2 --failed)
```

---

## Limitations

### Current Limitations

1. **Text-based journal parsing**: Binary journal files (.journal) are not yet fully supported. Currently requires exported text-based journal files.

2. **Estimated boot timing**: Without systemd-analyze output in the VM, boot times are estimated (15s total: 3s kernel, 2s initrd, 10s userspace).

3. **Service state detection**: Service states (active/inactive/failed) are determined from unit files, not runtime status.

4. **Offline analysis only**: Cannot analyze live/running VMs, only disk images.

### Future Enhancements

- Binary journal file parsing
- Timestamp-based filtering for journal entries
- Real-time journal streaming for live VMs
- Service unit validation and linting
- CIS benchmark compliance checking
- Integration with systemd-nspawn for containerized inspection
- Critical chain path analysis for boot optimization

---

## Examples Repository

Find complete examples and scripts at:
`examples/systemd-analysis/`

Including:
- Complete audit scripts
- Security hardening checks
- Performance baseline tools
- Comparison utilities
- Report generators

---

## See Also

- [Main README](../README.md) - Getting started guide
- [CLI Reference](cli-reference.md) - Complete command reference
- [Inspection Guide](inspection-guide.md) - General VM inspection
- [Troubleshooting](troubleshooting.md) - Common issues

---

## Support

For issues, feature requests, or questions:
- GitHub Issues: https://github.com/ssahani/guestctl/issues
- Documentation: https://github.com/ssahani/guestctl/tree/main/docs

**Version:** 0.3.1+
**Last Updated:** January 26, 2026
