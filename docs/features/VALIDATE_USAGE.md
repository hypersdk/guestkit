# Validate Command - Policy-Based Validation

## Overview
The `validate` command checks disk images against security policies and compliance benchmarks, providing comprehensive validation reports with remediation guidance.

## Quick Start

### Validate Against Industry Benchmark
```bash
# CIS Ubuntu 20.04 Benchmark
guestkit validate ubuntu-vm.qcow2 --benchmark cis-ubuntu

# CIS RHEL 8 Benchmark
guestkit validate rhel-vm.qcow2 --benchmark cis-rhel

# NIST Cybersecurity Framework
guestkit validate server.qcow2 --benchmark nist

# PCI DSS Requirements
guestkit validate payment-server.qcow2 --benchmark pci

# HIPAA Security Rule
guestkit validate healthcare-vm.qcow2 --benchmark hipaa
```

### Validate Against Custom Policy
```bash
# From YAML file
guestkit validate vm.qcow2 --policy custom-policy.yaml

# Generate example policy
guestkit validate vm.qcow2 --example-policy -o my-policy.yaml
```

### Output Formats
```bash
# Text format (default)
guestkit validate vm.qcow2 --benchmark cis-ubuntu

# JSON format
guestkit validate vm.qcow2 --benchmark cis-ubuntu --format json -o report.json

# Strict mode (exit 1 on failures)
guestkit validate vm.qcow2 --benchmark cis-ubuntu --strict
```

## Supported Benchmarks

| Benchmark | Code | Description |
|-----------|------|-------------|
| CIS Ubuntu 20.04 | `cis-ubuntu` | Center for Internet Security Ubuntu Benchmark |
| CIS RHEL 8 | `cis-rhel` | Center for Internet Security RHEL Benchmark |
| NIST CSF | `nist` | NIST Cybersecurity Framework |
| PCI DSS | `pci` | Payment Card Industry Data Security Standard |
| HIPAA | `hipaa` | Health Insurance Portability and Accountability Act |

## Policy File Format

Policies are defined in YAML format:

```yaml
name: "My Security Policy"
version: "1.0.0"
description: "Custom security policy for production VMs"

rules:
  - id: "PKG-001"
    name: "SSH Server Installed"
    description: "Ensure OpenSSH is installed"
    severity: "high"
    rule_type:
      type: "package_installed"
      package: "openssh-server"
    remediation: "apt-get install openssh-server"

  - id: "PKG-002"
    name: "Telnet Forbidden"
    description: "Telnet must not be installed"
    severity: "critical"
    rule_type:
      type: "package_forbidden"
      package: "telnet"
    remediation: "apt-get remove telnet"

  - id: "FILE-001"
    name: "SSH Config Exists"
    description: "/etc/ssh/sshd_config must exist"
    severity: "high"
    rule_type:
      type: "file_exists"
      path: "/etc/ssh/sshd_config"

  - id: "PERM-001"
    name: "SSH Config Permissions"
    description: "SSH config must have 600 permissions"
    severity: "critical"
    rule_type:
      type: "file_permissions"
      path: "/etc/ssh/sshd_config"
      mode: "600"
    remediation: "chmod 600 /etc/ssh/sshd_config"

  - id: "CONTENT-001"
    name: "Root Login Disabled"
    description: "PermitRootLogin must be set to no"
    severity: "critical"
    rule_type:
      type: "file_contains"
      path: "/etc/ssh/sshd_config"
      pattern: "PermitRootLogin no"
    remediation: "Set 'PermitRootLogin no' in sshd_config"

  - id: "SVC-001"
    name: "Firewall Enabled"
    description: "Firewall service must be enabled"
    severity: "high"
    rule_type:
      type: "service_enabled"
      service: "firewalld"
    remediation: "systemctl enable firewalld"

  - id: "USER-001"
    name: "Admin User Exists"
    description: "Admin user must exist"
    severity: "medium"
    rule_type:
      type: "user_exists"
      username: "admin"
    remediation: "useradd admin"
```

## Rule Types

### 1. Package Rules

**package_installed** - Package must be installed
```yaml
rule_type:
  type: "package_installed"
  package: "openssh-server"
```

**package_forbidden** - Package must not be installed
```yaml
rule_type:
  type: "package_forbidden"
  package: "telnet"
```

### 2. File Rules

**file_exists** - File must exist
```yaml
rule_type:
  type: "file_exists"
  path: "/etc/passwd"
```

**file_not_exists** - File must not exist
```yaml
rule_type:
  type: "file_not_exists"
  path: "/root/.rhosts"
```

**file_contains** - File must contain pattern
```yaml
rule_type:
  type: "file_contains"
  path: "/etc/ssh/sshd_config"
  pattern: "PermitRootLogin no"
```

**file_permissions** - File must have exact permissions
```yaml
rule_type:
  type: "file_permissions"
  path: "/etc/shadow"
  mode: "400"  # Octal format without leading 0
```

### 3. Service Rules

**service_enabled** - Systemd service must be enabled
```yaml
rule_type:
  type: "service_enabled"
  service: "sshd"
```

**service_disabled** - Systemd service must be disabled
```yaml
rule_type:
  type: "service_disabled"
  service: "telnet"
```

### 4. User Rules

**user_exists** - User must exist in /etc/passwd
```yaml
rule_type:
  type: "user_exists"
  username: "admin"
```

**user_not_exists** - User must not exist
```yaml
rule_type:
  type: "user_not_exists"
  username: "guest"
```

## Severity Levels

- `critical` - Security-critical issues
- `high` - High-severity issues
- `medium` - Medium-severity issues
- `low` - Low-severity issues

## Validation Report

### Text Format

```
🔍 Policy Validation Report
==========================

Image: production-vm.qcow2
Policy: CIS Ubuntu 20.04 Benchmark
Time: 2024-02-02T21:00:00Z

📊 Summary
----------
Total Rules: 15
✅ Passed: 12
❌ Failed: 2
⚠️  Warnings: 1
⏭️  Skipped: 0

📈 Compliance Score: 85.7%

❌ Failed Checks
---------------
  ❌ [critical] SSH Root Login Disabled
    💡 Set 'PermitRootLogin no' in /etc/ssh/sshd_config

  ❌ [high] SSH Config Permissions
    💡 chmod 600 /etc/ssh/sshd_config

⚠️  Good compliance, but improvements needed
```

### JSON Format

```json
{
  "image_path": "production-vm.qcow2",
  "policy_name": "CIS Ubuntu 20.04 Benchmark",
  "timestamp": "2024-02-02T21:00:00Z",
  "results": [
    {
      "rule_id": "CIS-5.2.4",
      "rule_name": "Ensure SSH root login is disabled",
      "status": "Fail",
      "message": "Ensure SSH root login is disabled - Check failed",
      "severity": "critical",
      "remediation": "Set 'PermitRootLogin no' in /etc/ssh/sshd_config"
    }
  ],
  "summary": {
    "total_rules": 15,
    "passed": 12,
    "failed": 2,
    "warnings": 1,
    "skipped": 0,
    "errors": 0,
    "compliance_score": 85.7
  }
}
```

## Compliance Scoring

The compliance score is calculated as:
```
score = (passed / (total - skipped)) * 100
```

**Score Interpretation:**
- `>= 90%` - ✅ Excellent compliance
- `75-89%` - ⚠️ Good compliance, improvements needed
- `50-74%` - ❌ Poor compliance, significant issues
- `< 50%` - 🔥 Critical compliance failure

## CI/CD Integration

### GitHub Actions
```yaml
- name: Validate VM Security
  run: |
    guestkit validate ${{ env.VM_IMAGE }} \
      --benchmark cis-ubuntu \
      --format json \
      --output validation-report.json \
      --strict

- name: Upload Report
  if: always()
  uses: actions/upload-artifact@v3
  with:
    name: validation-report
    path: validation-report.json
```

### GitLab CI
```yaml
validate:
  stage: security
  script:
    - guestkit validate vm.qcow2 --benchmark cis-rhel --strict
  artifacts:
    when: always
    reports:
      junit: validation-report.json
```

### Jenkins
```groovy
stage('Validate Security') {
    steps {
        sh '''
            guestkit validate ${VM_IMAGE} \
                --benchmark pci \
                --format json \
                --output validation.json \
                --strict
        '''
    }
    post {
        always {
            archiveArtifacts 'validation.json'
        }
    }
}
```

## Examples

### Production Validation Pipeline
```bash
#!/bin/bash
# validate-production.sh

VM_IMAGE="production-vm.qcow2"
POLICY="production-policy.yaml"

echo "🔍 Validating production VM..."

# Run validation
guestkit validate "$VM_IMAGE" \
    --policy "$POLICY" \
    --format json \
    --output validation-report.json

# Check results
SCORE=$(jq '.summary.compliance_score' validation-report.json)
FAILURES=$(jq '.summary.failed' validation-report.json)

echo "📊 Compliance Score: ${SCORE}%"

if [ "$FAILURES" -gt 0 ]; then
    echo "❌ Validation failed with $FAILURES issues"
    jq -r '.results[] | select(.status == "Fail") | "  - [\(.severity)] \(.rule_name)"' validation-report.json
    exit 1
fi

echo "✅ Validation passed!"
```

### Multi-Benchmark Validation
```bash
#!/bin/bash
# validate-all-benchmarks.sh

IMAGE="$1"

for benchmark in cis-ubuntu nist pci; do
    echo "Validating against $benchmark..."
    guestkit validate "$IMAGE" \
        --benchmark "$benchmark" \
        --format json \
        --output "${benchmark}-report.json"
done

echo "All validations complete!"
```

## Best Practices

1. **Start with Industry Benchmarks** - Use CIS or NIST as baseline
2. **Customize for Your Needs** - Create custom policies for specific requirements
3. **Version Control Policies** - Store policies in git
4. **Automate Validation** - Integrate into CI/CD pipelines
5. **Use Strict Mode** - Fail builds on policy violations
6. **Review Reports** - Regularly review and act on findings
7. **Update Policies** - Keep policies current with security best practices

## Limitations

Current version:
- Systemd-based service checking only
- Limited to file-based validation
- No runtime behavior validation
- Custom rules not yet fully implemented

## Future Enhancements

- [ ] Runtime behavior validation
- [ ] Network configuration validation
- [ ] Kernel parameter validation
- [ ] SELinux/AppArmor policy validation
- [ ] Custom rule DSL
- [ ] Policy templates library
- [ ] Automated remediation
- [ ] Historical compliance tracking
- [ ] Policy violation trending

---

*Last updated: 2024-02-02*
