# Migrate Feature - Smart Migration Planning

## Overview
The `migrate` command helps plan and execute migrations between different operating systems, platforms, and cloud providers with intelligent compatibility analysis and automated remediation suggestions.

## Command Structure

```bash
guestkit migrate <SUBCOMMAND> [OPTIONS] <IMAGE>
```

### Subcommands

- `plan` - Generate migration plan
- `analyze` - Analyze compatibility
- `convert` - Execute migration
- `validate` - Validate migration
- `rollback` - Rollback migration

## Syntax

```bash
# Generate migration plan
guestkit migrate plan <IMAGE> --target <TARGET> [OPTIONS]

# Analyze compatibility
guestkit migrate analyze <IMAGE> --target <TARGET> [OPTIONS]

# Execute migration
guestkit migrate convert <IMAGE> --target <TARGET> --output <OUTPUT> [OPTIONS]

# Validate migrated image
guestkit migrate validate <IMAGE> --baseline <BASELINE> [OPTIONS]

# Rollback migration
guestkit migrate rollback <IMAGE> --snapshot <SNAPSHOT> [OPTIONS]
```

## Migration Types

### 1. OS Upgrade Migration

Migrate between OS versions (e.g., CentOS 7 → Rocky Linux 9)

```bash
# Plan OS upgrade
guestkit migrate plan centos7-server.qcow2 \
    --target rocky9 \
    --output migration-plan.html

# Analyze compatibility
guestkit migrate analyze centos7-server.qcow2 \
    --target rocky9 \
    --check-packages \
    --check-configs \
    --check-services

# Execute migration
guestkit migrate convert centos7-server.qcow2 \
    --target rocky9 \
    --output rocky9-server.qcow2 \
    --preserve-data \
    --backup-snapshot
```

**Supported Migration Paths:**
- CentOS 7 → Rocky Linux 8/9, AlmaLinux 8/9
- Ubuntu 20.04 → Ubuntu 22.04/24.04
- Debian 10 → Debian 11/12
- RHEL 7/8 → RHEL 9
- openSUSE Leap → Tumbleweed

### 2. Cloud Platform Migration

Migrate VMs between cloud providers

```bash
# VMware to AWS
guestkit migrate plan vmware-vm.vmdk \
    --from vmware \
    --to aws \
    --instance-type t3.large \
    --region us-east-1 \
    --output aws-migration-plan.json

# Azure to GCP
guestkit migrate convert azure-vm.vhd \
    --from azure \
    --to gcp \
    --machine-type n2-standard-4 \
    --project my-project \
    --output gcp-image.tar.gz

# On-prem to Cloud
guestkit migrate plan on-prem.qcow2 \
    --to aws \
    --optimize-cost \
    --rightsizing \
    --output cloud-migration.pdf
```

**Supported Platforms:**
- VMware ESXi ↔ KVM/QEMU
- Hyper-V ↔ KVM/QEMU
- On-premises ↔ AWS/Azure/GCP
- AWS ↔ Azure ↔ GCP

### 3. Containerization Migration

Migrate VM to container

```bash
# VM to Container
guestkit migrate plan app-vm.qcow2 \
    --target container \
    --extract-app /opt/myapp \
    --output containerization-plan.md

# Execute containerization
guestkit migrate convert app-vm.qcow2 \
    --target container \
    --app-path /opt/myapp \
    --output myapp:latest \
    --generate-dockerfile
```

## Migration Plan Output

### HTML Report

```bash
guestkit migrate plan centos7.qcow2 --target rocky9 --output report.html
```

**Report Sections:**
1. Executive Summary
2. Compatibility Analysis
3. Package Migration Map
4. Configuration Changes
5. Risk Assessment
6. Step-by-Step Plan
7. Rollback Procedures
8. Testing Checklist

### JSON Format

```json
{
  "migration_plan": {
    "source": {
      "image": "centos7-server.qcow2",
      "os": "CentOS Linux 7.9",
      "kernel": "3.10.0-1160",
      "packages": 487,
      "services": 42
    },
    "target": {
      "os": "Rocky Linux 9.3",
      "kernel": "5.14.0",
      "architecture": "x86_64"
    },
    "compatibility": {
      "overall_score": 85,
      "critical_issues": 3,
      "warnings": 12,
      "compatible_packages": 412,
      "incompatible_packages": 15,
      "deprecated_packages": 18
    },
    "package_mapping": [
      {
        "source": "python2",
        "source_version": "2.7.5",
        "target": "python3.9",
        "target_version": "3.9.16",
        "action": "upgrade",
        "complexity": "high",
        "manual_intervention": true,
        "notes": "Python 2 is EOL. Applications need porting to Python 3."
      },
      {
        "source": "systemd",
        "source_version": "219",
        "target": "systemd",
        "target_version": "252",
        "action": "upgrade",
        "complexity": "medium",
        "manual_intervention": false,
        "notes": "Unit files may need syntax updates."
      }
    ],
    "configuration_changes": [
      {
        "file": "/etc/sysconfig/network-scripts/ifcfg-eth0",
        "type": "deprecated",
        "action": "migrate_to_nmcli",
        "complexity": "low",
        "automated": true
      },
      {
        "file": "/etc/yum.repos.d/CentOS-Base.repo",
        "type": "replace",
        "action": "update_to_rocky_repos",
        "complexity": "low",
        "automated": true
      }
    ],
    "risks": [
      {
        "severity": "high",
        "category": "compatibility",
        "description": "15 packages have no direct equivalent in Rocky Linux 9",
        "mitigation": "Identify alternative packages or build from source",
        "estimated_effort": "2-4 hours"
      },
      {
        "severity": "medium",
        "category": "application",
        "description": "Python 2 scripts need migration to Python 3",
        "mitigation": "Use 2to3 tool and manual testing",
        "estimated_effort": "4-8 hours"
      }
    ],
    "migration_steps": [
      {
        "step": 1,
        "phase": "preparation",
        "task": "Create full backup snapshot",
        "command": "guestkit snapshot create centos7.qcow2",
        "estimated_time": "5 minutes",
        "critical": true
      },
      {
        "step": 2,
        "phase": "preparation",
        "task": "Document current configuration",
        "command": "guestkit inspect centos7.qcow2 --export config.json",
        "estimated_time": "2 minutes",
        "critical": true
      },
      {
        "step": 3,
        "phase": "migration",
        "task": "Update package repositories",
        "commands": [
          "sed -i 's/centos/rocky/g' /etc/yum.repos.d/*.repo",
          "yum clean all"
        ],
        "estimated_time": "3 minutes",
        "critical": true
      }
    ],
    "rollback_plan": {
      "method": "snapshot",
      "snapshot_name": "pre-migration-20240202",
      "steps": [
        "Stop running VM",
        "Restore from snapshot",
        "Verify system integrity",
        "Resume operations"
      ],
      "estimated_rollback_time": "10 minutes"
    },
    "testing_checklist": [
      "Boot and verify kernel version",
      "Check all services start correctly",
      "Verify network connectivity",
      "Test application functionality",
      "Validate data integrity",
      "Performance benchmarking"
    ],
    "estimated_migration_time": {
      "automated": "45 minutes",
      "manual": "6-10 hours",
      "testing": "4 hours",
      "total": "1-2 days"
    }
  }
}
```

## Cloud Migration Examples

### AWS Migration

```bash
# Generate AWS migration plan
guestkit migrate plan on-prem.qcow2 \
    --to aws \
    --region us-east-1 \
    --instance-type auto \
    --rightsizing \
    --cost-estimate \
    --output aws-plan.html

# Execute migration
guestkit migrate convert on-prem.qcow2 \
    --to aws \
    --output ami-12345 \
    --region us-east-1 \
    --ami-name "migrated-server" \
    --tags "Environment=prod,App=web"
```

**AWS-Specific Analysis:**
- Instance type recommendations
- EBS volume optimization
- VPC/security group mapping
- IAM role requirements
- Cost projection
- Reserved instance opportunities

### Multi-Cloud Strategy

```bash
# Compare cloud providers
guestkit migrate analyze vm.qcow2 \
    --compare aws,azure,gcp \
    --optimize cost \
    --output comparison.xlsx

# Generate multi-cloud migration
guestkit migrate plan vm.qcow2 \
    --targets aws,azure \
    --output multi-cloud-plan.pdf
```

## Compatibility Checks

### Package Compatibility

```bash
# Check package compatibility
guestkit migrate analyze centos7.qcow2 \
    --target rocky9 \
    --check-packages \
    --output package-compatibility.csv
```

**Output:**
```csv
Package,Current Version,Target Version,Status,Alternative,Action Required
httpd,2.4.6,2.4.57,compatible,none,automatic upgrade
python2,2.7.5,N/A,deprecated,python3.9,manual migration
mysql,5.7,8.0,major upgrade,mariadb-10.5,review compatibility
```

### Configuration Compatibility

```bash
# Check configuration files
guestkit migrate analyze vm.qcow2 \
    --target ubuntu-24.04 \
    --check-configs \
    --output config-changes.json
```

**Detects:**
- Deprecated configuration syntax
- Changed default values
- Removed/renamed options
- New required parameters

### Service Compatibility

```bash
# Check systemd services
guestkit migrate analyze rhel7.qcow2 \
    --target rhel9 \
    --check-services \
    --output service-migration.txt
```

## Advanced Features

### Automated Remediation

```bash
# Auto-fix compatible issues
guestkit migrate convert vm.qcow2 \
    --target rocky9 \
    --auto-remediate \
    --output migrated.qcow2
```

**Auto-remediates:**
- Repository URL updates
- Package name changes
- Configuration file syntax
- Service unit files

### Dry Run Mode

```bash
# Simulate migration
guestkit migrate convert vm.qcow2 \
    --target ubuntu-24.04 \
    --dry-run \
    --verbose \
    --output simulation-report.txt
```

### Staged Migration

```bash
# Multi-stage migration
guestkit migrate convert centos7.qcow2 \
    --target rocky9 \
    --staged \
    --stages centos7,centos8,rocky9 \
    --pause-between-stages \
    --output-dir staged-migration/
```

## Risk Assessment

```bash
# Detailed risk analysis
guestkit migrate analyze production-db.qcow2 \
    --target postgres-15 \
    --risk-assessment detailed \
    --output risk-report.pdf
```

**Risk Categories:**
- **Critical**: Data loss potential, service outages
- **High**: Breaking changes, major compatibility issues
- **Medium**: Deprecated features, minor incompatibilities
- **Low**: Warnings, cosmetic changes

**Risk Score:** 0-100 (Higher = Riskier)
- 0-25: Low risk (automated migration safe)
- 26-50: Medium risk (review recommended)
- 51-75: High risk (testing required)
- 76-100: Critical risk (extensive planning needed)

## Validation

```bash
# Validate migrated system
guestkit migrate validate migrated.qcow2 \
    --baseline original.qcow2 \
    --check-all \
    --output validation-report.html
```

**Validation Checks:**
- All services running
- Configuration parity
- Data integrity
- Performance benchmarks
- Security posture

## Rollback

```bash
# Rollback migration
guestkit migrate rollback migrated.qcow2 \
    --snapshot pre-migration \
    --verify \
    --output rollback-log.txt
```

## Integration Examples

### CI/CD Pipeline

```bash
#!/bin/bash
# Automated migration pipeline

set -e

IMAGE="centos7-app.qcow2"
TARGET="rocky9"

# Generate plan
echo "📋 Generating migration plan..."
guestkit migrate plan $IMAGE --target $TARGET -o plan.json

# Check risk score
RISK_SCORE=$(jq '.compatibility.overall_score' plan.json)
if [ $RISK_SCORE -lt 70 ]; then
    echo "❌ Risk score too high: $RISK_SCORE"
    exit 1
fi

# Create backup
echo "💾 Creating backup..."
guestkit snapshot create $IMAGE -o pre-migration

# Execute migration
echo "🚀 Starting migration..."
guestkit migrate convert $IMAGE \
    --target $TARGET \
    --output migrated.qcow2 \
    --auto-remediate

# Validate
echo "✅ Validating..."
guestkit migrate validate migrated.qcow2 \
    --baseline $IMAGE \
    --output validation.json

# Check validation
SUCCESS=$(jq '.validation.success' validation.json)
if [ "$SUCCESS" != "true" ]; then
    echo "❌ Validation failed, rolling back..."
    guestkit migrate rollback migrated.qcow2 --snapshot pre-migration
    exit 1
fi

echo "✅ Migration completed successfully!"
```

### Terraform Integration

```hcl
# terraform/main.tf

resource "null_resource" "vm_migration" {
  provisioner "local-exec" {
    command = <<-EOT
      guestkit migrate convert ${var.source_vm} \
        --to aws \
        --region ${var.aws_region} \
        --output ${var.ami_name} \
        --tags "${var.tags}"
    EOT
  }
}

output "migration_ami" {
  value = null_resource.vm_migration.id
}
```

## Performance

- **Analysis Time:** 2-5 minutes (typical VM)
- **Migration Time:** 30-90 minutes (depending on size and complexity)
- **Validation Time:** 5-10 minutes

## Best Practices

1. **Always create backups** before migration
2. **Test in staging** environment first
3. **Review migration plan** thoroughly
4. **Validate** after migration
5. **Keep rollback plan** ready
6. **Monitor** migrated system closely

## Limitations

- Cannot migrate running VMs (must be powered off)
- Custom kernel modules may need recompilation
- Third-party software compatibility not guaranteed
- Some proprietary drivers may not migrate

## Future Enhancements

- [ ] Live migration support
- [ ] Application-level migration
- [ ] Database migration helpers
- [ ] Multi-VM migration orchestration
- [ ] Cloud cost calculator
- [ ] ML-based compatibility prediction
- [ ] Automated testing suite generation

---

*Feature Status: Proposed*
*Priority: High*
*Estimated Effort: 4-6 weeks*
*Dependencies: inventory, validate, simulate features*
