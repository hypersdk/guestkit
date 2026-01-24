# VM Comparison Guide

GuestKit provides tools to compare VMs and detect configuration drift.

## Table of Contents

- [Overview](#overview)
- [Diff Command](#diff-command)
- [Compare Command](#compare-command)
- [Use Cases](#use-cases)
- [Examples](#examples)

## Overview

GuestKit offers two comparison modes:

- **diff** - Compare two VMs to show differences
- **compare** - Compare multiple VMs against a baseline

Both commands help identify:
- Configuration drift
- Package differences
- Service changes
- User account changes
- Network configuration changes

## Diff Command

Compare two disk images and show detailed differences.

### Usage

```bash
# Basic diff
guestkit diff vm-before.qcow2 vm-after.qcow2

# JSON output
guestkit diff vm1.qcow2 vm2.qcow2 --output json

# YAML output
guestkit diff vm1.qcow2 vm2.qcow2 --output yaml
```

### What It Compares

#### Operating System
- Hostname changes
- OS version upgrades
- Architecture (should not change)
- Distribution changes (rare but possible)

#### Packages
- **Added**: New packages installed
- **Removed**: Packages uninstalled
- **Updated**: Version changes
- Kernel version changes

#### Services
- **Enabled**: Newly enabled services
- **Disabled**: Services that were stopped/disabled
- **Changed**: Service state changes

#### Users
- **Added**: New user accounts
- **Removed**: Deleted accounts
- **Modified**: UID or home directory changes

#### Network Configuration
- IP address changes
- MAC address changes
- DNS server changes
- DHCP configuration changes

#### System Configuration
- Timezone changes
- Locale changes
- SELinux mode changes
- Cloud-init status changes

### Example Output (Text)

```
=== OS Differences ===
  hostname: fedora-dev → fedora-prod

=== Package Differences ===
  Added (5):
    + nginx-1.24.0
    + certbot-2.7.4
    + python3-certbot-nginx-2.7.4
    + mod_ssl-2.4.58
    + openssl-3.1.4

  Removed (2):
    - apache2-2.4.57
    - apache2-utils-2.4.57

  Updated (12):
    ~ python3: 3.11.5 → 3.11.6
    ~ kernel: 6.5.6 → 6.6.8
    ~ systemd: 254.5 → 254.7
    ~ openssl: 3.1.1 → 3.1.4
    ~ curl: 8.2.1 → 8.4.0
    ~ git: 2.41.0 → 2.43.0
    ~ vim: 9.0.1672 → 9.0.2103
    ~ bash: 5.2.15 → 5.2.21
    ~ coreutils: 9.3 → 9.4
    ~ glibc: 2.38 → 2.38.1
    ~ krb5: 1.21.1 → 1.21.2
    ~ sudo: 1.9.14 → 1.9.15

=== Service Differences ===
  Enabled:
    + nginx.service
    + certbot.timer

  Disabled:
    - apache2.service

=== User Differences ===
  Added: webadmin (uid: 1002)

=== Network Differences ===
  eth0 IP: 192.168.1.100 → 192.168.1.150
```

### Example Output (JSON)

```json
{
  "os_changes": [
    {
      "field": "hostname",
      "before": "fedora-dev",
      "after": "fedora-prod"
    }
  ],
  "package_changes": {
    "added": ["nginx-1.24.0", "certbot-2.7.4"],
    "removed": ["apache2-2.4.57"],
    "updated": [
      {
        "name": "python3",
        "before": "3.11.5",
        "after": "3.11.6"
      }
    ]
  },
  "service_changes": {
    "enabled": ["nginx.service"],
    "disabled": ["apache2.service"]
  },
  "user_changes": {
    "added": [{"username": "webadmin", "uid": "1002"}],
    "removed": []
  },
  "network_changes": [
    {
      "field": "eth0 IP",
      "before": "192.168.1.100",
      "after": "192.168.1.150"
    }
  ]
}
```

## Compare Command

Compare multiple VMs against a baseline to identify deviations.

### Usage

```bash
# Compare VMs against baseline
guestkit compare baseline.qcow2 vm1.qcow2 vm2.qcow2 vm3.qcow2

# Compare all VMs in directory
guestkit compare golden-image.qcow2 prod-*.qcow2
```

### What It Compares

- Hostname
- OS version
- Package count
- Service configuration
- User accounts

### Example Output

```
=== Comparison Report ===

                    Baseline    VM1        VM2        VM3
-------------------------------------------------------------
Hostname            golden      web1       web2       db1
OS Version          39.0        39.0       40.0       39.0
Package Count       1247        1250       1248       1189
```

### Identifying Issues

The compare command helps spot:
- **Version Drift**: VM2 running Fedora 40 while others on 39
- **Package Drift**: VM3 has fewer packages (1189 vs ~1247)
- **Configuration Inconsistency**: Different package counts indicate different configurations

## Use Cases

### 1. Configuration Drift Detection

Monitor VMs over time to detect unauthorized changes:

```bash
#!/bin/bash
# Weekly drift detection

BASELINE="baseline-$(date +%Y-%m).qcow2"
CURRENT="prod-server.qcow2"

# Create baseline if it doesn't exist
if [ ! -f "$BASELINE" ]; then
    echo "Creating baseline..."
    cp "$CURRENT" "$BASELINE"
    exit 0
fi

# Compare current against baseline
DRIFT=$(guestkit diff "$BASELINE" "$CURRENT" --output json | \
    jq -r '[
        (.package_changes.added | length),
        (.package_changes.removed | length),
        (.service_changes.enabled | length),
        (.service_changes.disabled | length)
    ] | add')

if [ "$DRIFT" -gt 0 ]; then
    echo "ALERT: Configuration drift detected!"
    guestkit diff "$BASELINE" "$CURRENT" > drift-report-$(date +%Y-%m-%d).txt
    echo "Drift report saved"
else
    echo "No drift detected"
fi
```

### 2. Change Validation

Verify that VM changes match intended changes:

```bash
#!/bin/bash
# Validate deployment changes

BEFORE="vm-before-deployment.qcow2"
AFTER="vm-after-deployment.qcow2"
EXPECTED_PACKAGES=("nginx" "certbot")

# Get diff
guestkit diff "$BEFORE" "$AFTER" --output json > deployment-diff.json

# Validate expected packages were added
for pkg in "${EXPECTED_PACKAGES[@]}"; do
    if jq -e ".package_changes.added | any(startswith(\"$pkg\"))" deployment-diff.json > /dev/null; then
        echo "✓ $pkg installed"
    else
        echo "✗ $pkg NOT installed (expected)"
        exit 1
    fi
done

echo "Deployment validation passed"
```

### 3. Golden Image Compliance

Ensure production VMs match golden image:

```bash
#!/bin/bash
# Compliance check against golden image

GOLDEN="golden-image-2026-01.qcow2"
COMPLIANT=0
NON_COMPLIANT=0

for vm in prod-*.qcow2; do
    NAME=$(basename "$vm" .qcow2)

    # Compare against golden
    guestkit diff "$GOLDEN" "$vm" --output json > "compliance-$NAME.json"

    # Check for critical differences
    CRITICAL=$(jq -r '[
        (.os_changes | length),
        (.service_changes.disabled | length)
    ] | add' "compliance-$NAME.json")

    if [ "$CRITICAL" -eq 0 ]; then
        echo "✓ $NAME: COMPLIANT"
        ((COMPLIANT++))
    else
        echo "✗ $NAME: NON-COMPLIANT"
        ((NON_COMPLIANT++))
    fi
done

echo ""
echo "Compliance Summary:"
echo "  Compliant: $COMPLIANT"
echo "  Non-Compliant: $NON_COMPLIANT"
```

### 4. Migration Validation

Verify VM migration was successful:

```bash
#!/bin/bash
# Post-migration validation

SOURCE="source-vm.qcow2"
TARGET="migrated-vm.qcow2"

# Compare source and target
guestkit diff "$SOURCE" "$TARGET" --output json > migration-diff.json

# Expected differences (hostname, IP)
EXPECTED_DIFFS=$(jq -r '.os_changes | length' migration-diff.json)

# Unexpected differences (packages, users)
PACKAGE_DIFF=$(jq -r '[.package_changes.added | length, .package_changes.removed | length] | add' migration-diff.json)
USER_DIFF=$(jq -r '[.user_changes.added | length, .user_changes.removed | length] | add' migration-diff.json)

if [ "$PACKAGE_DIFF" -gt 0 ] || [ "$USER_DIFF" -gt 0 ]; then
    echo "✗ Migration validation FAILED"
    echo "  Package differences: $PACKAGE_DIFF"
    echo "  User differences: $USER_DIFF"
    exit 1
else
    echo "✓ Migration validation PASSED"
    echo "  Expected differences: $EXPECTED_DIFFS (hostname, IP, etc.)"
fi
```

### 5. Fleet-wide Consistency Check

Ensure all VMs in a cluster are consistent:

```bash
#!/bin/bash
# Fleet consistency check

BASELINE="web-01.qcow2"
INCONSISTENT=()

for vm in web-*.qcow2; do
    [ "$vm" = "$BASELINE" ] && continue

    NAME=$(basename "$vm" .qcow2)

    # Compare against baseline
    DIFF=$(guestkit diff "$BASELINE" "$vm" --output json | \
        jq -r '[.package_changes.added | length, .package_changes.removed | length] | add')

    if [ "$DIFF" -gt 0 ]; then
        echo "⚠ $NAME: $DIFF package differences"
        INCONSISTENT+=("$NAME")
    else
        echo "✓ $NAME: Consistent"
    fi
done

if [ ${#INCONSISTENT[@]} -gt 0 ]; then
    echo ""
    echo "WARNING: Inconsistent VMs detected:"
    printf '  - %s\n' "${INCONSISTENT[@]}"
    exit 1
fi

echo ""
echo "✓ Fleet is consistent"
```

## Examples

### Generate Diff Report

```bash
#!/bin/bash
# Generate comprehensive diff report

VM1="$1"
VM2="$2"

if [ -z "$VM1" ] || [ -z "$VM2" ]; then
    echo "Usage: $0 <vm1.qcow2> <vm2.qcow2>"
    exit 1
fi

NAME1=$(basename "$VM1" .qcow2)
NAME2=$(basename "$VM2" .qcow2)
REPORT_DIR="diff-$NAME1-vs-$NAME2"

mkdir -p "$REPORT_DIR"

# Text report
guestkit diff "$VM1" "$VM2" > "$REPORT_DIR/diff-report.txt"

# JSON for automation
guestkit diff "$VM1" "$VM2" --output json > "$REPORT_DIR/diff-data.json"

# YAML for readability
guestkit diff "$VM1" "$VM2" --output yaml > "$REPORT_DIR/diff-data.yaml"

# Extract package changes only
jq '.package_changes' "$REPORT_DIR/diff-data.json" > "$REPORT_DIR/package-changes.json"

# Extract service changes only
jq '.service_changes' "$REPORT_DIR/diff-data.json" > "$REPORT_DIR/service-changes.json"

echo "Diff report generated in $REPORT_DIR/"
ls -lh "$REPORT_DIR"
```

### Track Changes Over Time

```bash
#!/bin/bash
# Track VM changes over time

VM="prod-server.qcow2"
HISTORY_DIR="change-history"

mkdir -p "$HISTORY_DIR"

# Take snapshot of current state
DATE=$(date +%Y-%m-%d)
guestkit inspect "$VM" --output json > "$HISTORY_DIR/snapshot-$DATE.json"

# Compare with previous day
YESTERDAY=$(date -d "yesterday" +%Y-%m-%d)
if [ -f "$HISTORY_DIR/snapshot-$YESTERDAY.json" ]; then
    # Note: This compares snapshots, not live VMs
    # For live comparison, use:
    # guestkit diff yesterday-vm.qcow2 today-vm.qcow2

    diff "$HISTORY_DIR/snapshot-$YESTERDAY.json" "$HISTORY_DIR/snapshot-$DATE.json" > "$HISTORY_DIR/changes-$DATE.diff"

    if [ -s "$HISTORY_DIR/changes-$DATE.diff" ]; then
        echo "Changes detected on $DATE"
        echo "See $HISTORY_DIR/changes-$DATE.diff"
    else
        echo "No changes detected"
    fi
fi
```

### Bulk Comparison Report

```bash
#!/bin/bash
# Generate comparison matrix for multiple VMs

BASELINE="$1"
shift
VMS=("$@")

if [ -z "$BASELINE" ] || [ ${#VMS[@]} -eq 0 ]; then
    echo "Usage: $0 <baseline.qcow2> <vm1.qcow2> <vm2.qcow2> ..."
    exit 1
fi

REPORT_FILE="comparison-matrix-$(date +%Y-%m-%d).txt"

{
    echo "=== VM Comparison Matrix ==="
    echo "Baseline: $(basename "$BASELINE" .qcow2)"
    echo "Date: $(date)"
    echo ""

    # Run guestkit compare command
    guestkit compare "$BASELINE" "${VMS[@]}"

    echo ""
    echo "=== Individual Diffs ==="
    echo ""

    for vm in "${VMS[@]}"; do
        NAME=$(basename "$vm" .qcow2)
        echo "━━━ $NAME vs Baseline ━━━"
        guestkit diff "$BASELINE" "$vm" | head -30
        echo ""
    done
} | tee "$REPORT_FILE"

echo "Report saved to $REPORT_FILE"
```

## Best Practices

### Diff Command

1. **Meaningful Names**: Use descriptive VM names to make diff output clearer
2. **Version Control**: Store diff JSON outputs in Git for historical tracking
3. **Automation**: Integrate diff checks into CI/CD pipelines
4. **Thresholds**: Define acceptable change thresholds (e.g., max 5 package updates)
5. **Documentation**: Document expected differences for planned changes

### Compare Command

1. **Golden Image**: Maintain a golden image baseline for comparison
2. **Regular Checks**: Run weekly compliance checks against baseline
3. **Alerting**: Set up alerts for VMs that deviate from baseline
4. **Remediation**: Auto-remediate or flag VMs with excessive drift
5. **Update Baseline**: Periodically update baseline after verified changes

### General

1. **Caching**: Use `--cache` with diff/compare for faster repeated checks
2. **Filtering**: Use `jq` to filter JSON output for specific changes of interest
3. **Reporting**: Combine text and JSON outputs - text for humans, JSON for machines
4. **Trending**: Track drift metrics over time to identify patterns
5. **Validation**: Always validate critical changes manually before auto-remediation

## Limitations

### Current Limitations

1. **Package Versions**: Kernel versions are compared, but full package version tracking requires integration with package data
2. **Deep Inspection**: Comparison uses inspection data; file-level diffs require separate tools
3. **Live Comparison**: Both VMs must be accessible as disk images (not running VMs)
4. **Performance**: Large VMs may take time to inspect before comparison

### Workarounds

1. **Cache First Inspection**: Use `--cache` to speed up repeated comparisons
2. **Batch Processing**: Use `inspect-batch` to pre-cache multiple VMs
3. **Incremental Snapshots**: Take regular snapshots for temporal comparison

## See Also

- [Profiles Guide](PROFILES_GUIDE.md) - Specialized inspection profiles
- [Export Guide](EXPORT_GUIDE.md) - Report generation and export
- [CLI Guide](CLI_GUIDE.md) - Complete CLI reference
- [Caching Guide](CLI_GUIDE.md#caching) - Performance optimization with caching
