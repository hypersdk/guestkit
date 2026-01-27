# 🔧 Offline Patch & Fix Preview Mode

**Status:** Phase 1 Complete (Foundation)
**Version:** 0.3.1+
**Last Updated:** 2026-01-27

## Overview

The Offline Patch & Fix Preview Mode enables safe, reviewable VM fixes with complete separation of concerns. Instead of directly applying changes, GuestKit generates detailed fix plans that can be previewed, reviewed, exported as scripts, and applied with safety checks.

## Workflow

```
Inspect → Diagnose → Generate Plan → Review → Approve → Execute
```

This workflow matches enterprise change management requirements and provides:
- **Safety**: See exactly what will change before applying
- **Auditability**: Plans are version-controllable artifacts
- **Scriptability**: Export plans as bash/ansible for review
- **Reversibility**: Backup and rollback capabilities
- **Collaboration**: Security team generates, ops team applies

## Architecture

### Core Components

#### 1. **Plan Types** (`types.rs`)

Complete data structures for representing fix plans:

```rust
pub struct FixPlan {
    pub version: String,
    pub vm: String,
    pub generated: DateTime<Utc>,
    pub profile: String,
    pub overall_risk: String,
    pub estimated_duration: String,
    pub metadata: PlanMetadata,
    pub operations: Vec<Operation>,
    pub post_apply: Vec<PostApplyAction>,
}
```

**Operation Types:**
- `FileEdit` - Line-by-line file modifications
- `PackageInstall` - Package installation
- `ServiceOperation` - Service management (enable/start/restart)
- `SELinuxMode` - SELinux mode changes
- `RegistryEdit` - Windows registry modifications
- `CommandExec` - Arbitrary command execution
- `FileCopy` - File copy operations
- `DirectoryCreate` - Directory creation
- `FilePermissions` - Permission/ownership changes

**Priority Levels:**
- Critical 🔴
- High 🟠
- Medium 🟡
- Low 🟢
- Info ℹ️

#### 2. **Plan Generator** (`generator.rs`)

Converts security profile findings into executable fix plans:

```rust
let generator = PlanGenerator::new("vm.qcow2".to_string());
let plan = generator.from_security_profile(&security_report)?;
```

**Features:**
- Heuristic-based remediation parsing
- Automatic dependency detection
- Duration estimation
- Post-apply action generation
- Risk level mapping

#### 3. **Plan Preview** (`preview.rs`)

Human-readable plan display with colors and formatting:

```rust
PlanPreview::display(&plan);        // Formatted output
PlanPreview::display_diff(&plan);   // Unified diff view
PlanPreview::print_summary(&plan);  // Summary statistics
```

**Output Modes:**
- **Formatted Text**: Color-coded, grouped by priority
- **Unified Diff**: Git-style diffs for file changes
- **Summary**: Quick statistics overview

#### 4. **Plan Applicator** (`apply.rs`) - Phase 2

Executes fix plans with safety checks:

```rust
let applicator = PlanApplicator::new("vm.qcow2".to_string(), false);
let result = applicator.apply(&plan)?;
```

**Features (Planned):**
- Dry-run validation
- Circular dependency detection
- Backup before apply
- Rollback capability
- Progress tracking

#### 5. **Plan Exporter** (`export.rs`)

Export plans to various formats:

```rust
// Export as bash script
let script = PlanExporter::to_bash(&plan)?;

// Export as Ansible playbook
let playbook = PlanExporter::to_ansible(&plan)?;

// Export as JSON/YAML
let json = PlanExporter::to_json(&plan)?;
let yaml = PlanExporter::to_yaml(&plan)?;
```

**Export Formats:**
- **Bash**: Executable shell scripts with error handling
- **Ansible**: Playbooks for configuration management
- **JSON**: Machine-readable for automation
- **YAML**: Human-readable configuration

## Usage Examples

### CLI Usage (Planned - Phase 2)

```bash
# Generate fix plan from security profile
guestctl profile security vm.qcow2 --plan security-fixes.yaml

# Preview the plan
guestctl plan preview security-fixes.yaml

# Show as unified diff
guestctl plan diff security-fixes.yaml

# Export as executable script
guestctl plan export security-fixes.yaml --format bash > fixes.sh
guestctl plan export security-fixes.yaml --format ansible > fixes.yml

# Validate plan (dry-run simulation)
guestctl plan validate security-fixes.yaml

# Apply with confirmation prompts
guestctl plan apply security-fixes.yaml --interactive

# Apply automatically (for automation)
guestctl plan apply security-fixes.yaml --yes

# Apply with backup
guestctl plan apply security-fixes.yaml --backup /backup/vm-state

# Rollback if needed
guestctl plan rollback security-fixes.yaml
```

### Programmatic Usage

```rust
use guestkit::cli::plan::*;

// Generate plan
let generator = PlanGenerator::new("vm.qcow2".to_string());
let plan = generator.from_security_profile(&security_report)?;

// Preview
PlanPreview::display(&plan);

// Export to bash
let script = PlanExporter::to_bash(&plan)?;
std::fs::write("fixes.sh", script)?;

// Validate
let applicator = PlanApplicator::new("vm.qcow2".to_string(), false);
let validation = applicator.validate(&plan)?;

if validation.valid {
    // Apply (dry-run)
    let applicator_dry = PlanApplicator::new("vm.qcow2".to_string(), true);
    let result = applicator_dry.apply(&plan)?;
}
```

## Plan Format

### YAML Example

```yaml
version: "1.0"
vm: production-web-01.qcow2
generated: "2026-01-27T14:30:00Z"
profile: security
overall_risk: high
estimated_duration: "5-10 minutes"

metadata:
  author: guestkit-profiles
  review_required: true
  reversible: true
  description: Security hardening plan
  tags:
    - security
    - automated

operations:
  - id: sec-001
    type: file_edit
    priority: critical
    description: Disable root SSH login
    risk: low
    reversible: true
    file: /etc/ssh/sshd_config
    backup: true
    changes:
      - line: 28
        before: "PermitRootLogin yes"
        after: "PermitRootLogin no"
        context: |
          # Authentication:
          LoginGraceTime 2m
          PermitRootLogin yes  # CHANGE THIS
          StrictModes yes
    validation:
      command: "sshd -t"
      expected_exit: 0

  - id: sec-002
    type: package_install
    priority: high
    description: Install fail2ban
    risk: low
    reversible: true
    packages:
      - fail2ban
    estimated_size: "~15MB"

  - id: sec-003
    type: service_operation
    priority: high
    description: Enable and start firewalld
    risk: low
    reversible: true
    service: firewalld
    state: enabled
    start: true
    depends_on:
      - sec-004
    validation:
      command: "firewall-cmd --state"
      expected_output: "running"

post_apply:
  - type: service_restart
    services:
      - sshd
      - firewalld
  - type: validation
    command: "firewall-cmd --state"
    expected_output: "running"
  - type: reboot_required
    reason: "SELinux mode change requires reboot"
```

### Bash Script Export

```bash
#!/bin/bash
# Generated by GuestKit v0.3.1
# Profile: security
# VM: production-web-01.qcow2
# Generated: 2026-01-27T14:30:00Z

set -e

echo "Applying security fixes..."

# Create backup
BACKUP_DIR="/backup/$(date +%Y%m%d-%H%M%S)"
mkdir -p "$BACKUP_DIR"

# sec-001: Disable root SSH login
cp "/etc/ssh/sshd_config" "$BACKUP_DIR/"
sed -i 's/PermitRootLogin yes/PermitRootLogin no/g' "/etc/ssh/sshd_config"
sshd -t || { echo "Validation failed for sec-001"; exit 1; }

# sec-004: Install firewalld
dnf install -y firewalld

# sec-003: Enable and start firewalld
systemctl enable firewalld
systemctl start firewalld
firewall-cmd --state || { echo "Validation failed for sec-003"; exit 1; }

# Post-apply actions
systemctl restart sshd
systemctl restart firewalld
firewall-cmd --state

echo "✓ All fixes applied successfully"
```

### Ansible Playbook Export

```yaml
---
- name: GuestKit security Fixes
  hosts: vm
  become: yes
  tasks:
    - name: Disable root SSH login
      lineinfile:
        path: /etc/ssh/sshd_config
        regexp: '^PermitRootLogin yes$'
        line: 'PermitRootLogin no'
        backup: yes
      notify: restart sshd

    - name: Install firewalld
      package:
        name:
          - firewalld
        state: present

    - name: Enable and start firewalld
      service:
        name: firewalld
        enabled: yes
        state: started
```

## Preview Output

```
📋 Fix Plan Preview
════════════════════════════════════════════════════════════

VM: production-web-01.qcow2
Profile: security (HIGH risk)
Operations: 6
Estimated Duration: 5-10 minutes

━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

🔴 CRITICAL Priority (2 operations)

[sec-001] Disable root SSH login
  File: /etc/ssh/sshd_config
  Line 28: PermitRootLogin yes → PermitRootLogin no
  Risk: Low | Reversible: Yes

[sec-005] Set SELinux to enforcing mode
  File: /etc/selinux/config
  permissive → enforcing
  ⚠️  Requires reboot to take full effect

🟠 HIGH Priority (3 operations)

[sec-002] Install fail2ban
  Packages: fail2ban (~15MB)

[sec-003] Enable firewalld service
  Service: firewalld (enable + start)
  Depends on: [sec-004]

[sec-004] Install firewalld
  Packages: firewalld

━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

Dependencies:
  sec-004 → sec-003

Post-Apply Actions:
  • Restart services: sshd, firewalld
  • Validate: firewall-cmd --state
  ⚠️ Reboot required: SELinux mode change requires reboot

Backup: Will create automatic backup
Rollback: Available for all operations
```

## Safety Features

### 1. **Validation Before Apply**
- Check VM exists
- Detect circular dependencies
- Verify all dependencies exist
- Warn about non-reversible operations

### 2. **Dry-Run Mode**
- Simulate application without changes
- Report what would be done
- Validate plan structure

### 3. **Backup Creation**
- Automatic backup before applying
- Timestamped backup directory
- Includes all modified files

### 4. **Rollback Capability** (Planned)
- Restore from backup
- Undo individual operations
- Transaction-like behavior

### 5. **Dependency Management**
- Automatic dependency detection
- Topological sort for execution order
- Circular dependency prevention

## Current Limitations (Phase 1)

- ✅ Plan generation from profiles
- ✅ Preview and diff display
- ✅ Export to bash/ansible/json/yaml
- ✅ Validation framework
- ⏳ **Actual plan application** (Phase 2)
- ⏳ **TUI integration** (Phase 2)
- ⏳ **Rollback execution** (Phase 2)
- ⏳ **Progress tracking** (Phase 2)
- ⏳ **CLI commands** (Phase 2)

## Roadmap

### Phase 2: Application & Safety
- Implement actual plan application
- Backup creation and management
- Rollback execution
- Progress tracking
- Error handling and recovery

### Phase 3: TUI Integration
- Interactive plan viewer
- Checkbox operation selection
- Real-time preview
- Apply from TUI

### Phase 4: Advanced Features
- Plan merging and composition
- Incremental application
- Remote application (via SSH)
- Fleet-wide plan deployment
- Plan versioning and history

## Use Cases

### 1. **Security Hardening**
```bash
# Generate security fixes
guestctl profile security prod-web.qcow2 --plan security.yaml

# Review and approve
guestctl plan preview security.yaml

# Export for change control
guestctl plan export security.yaml --format bash > security-fixes.sh

# Apply in maintenance window
guestctl plan apply security.yaml --backup /backups/
```

### 2. **Fleet Management**
```bash
# Generate plan from one VM
guestctl profile security template.qcow2 --plan fleet-security.yaml

# Export to Ansible
guestctl plan export fleet-security.yaml --format ansible > fleet.yml

# Apply to entire fleet
ansible-playbook -i inventory fleet.yml
```

### 3. **Compliance Automation**
```bash
# Generate compliance fixes
guestctl profile compliance vm.qcow2 --plan compliance.yaml

# Store in version control
git add compliance.yaml
git commit -m "Add compliance fixes for Q1 2026"

# Review in PR
git push origin compliance-fixes

# Apply after approval
guestctl plan apply compliance.yaml
```

### 4. **Migration Preparation**
```bash
# Generate migration fixes
guestctl profile migration hyperv-vm.vhdx --plan migration.yaml

# Preview changes
guestctl plan preview migration.yaml

# Export as runbook
guestctl plan export migration.yaml --format bash > migration-runbook.sh

# Execute during migration
bash migration-runbook.sh
```

## Best Practices

1. **Always Preview First**
   - Review plans before applying
   - Check dependencies and order
   - Validate risk levels

2. **Version Control Plans**
   - Store plans in git
   - Review in pull requests
   - Track changes over time

3. **Test in Staging**
   - Apply to test VMs first
   - Validate results
   - Then promote to production

4. **Use Backups**
   - Always enable backup before apply
   - Keep backups for rollback
   - Test restore procedures

5. **Document Changes**
   - Add descriptions to plans
   - Tag appropriately
   - Include rationale in metadata

## Integration with Existing Features

- **Security Profiles**: Generate plans from security findings
- **Compliance Profiles**: Automated compliance remediation
- **Migration Profiles**: Pre-migration fixes
- **TUI Dashboard**: Visual plan management (Phase 3)
- **Batch Processing**: Apply plans to fleets
- **Export Formats**: HTML/PDF reports with plans

## Contributing

To extend the plan system:

1. **Add New Operation Types**: Edit `types.rs`
2. **Improve Remediation Parsing**: Edit `generator.rs`
3. **Add Export Formats**: Edit `export.rs`
4. **Enhance Preview**: Edit `preview.rs`

## References

- [Security Profiles](security-profiles.md)
- [Profile System](../architecture/security-profiles.md)
- [Export Formats](export-formats.md)
- [TUI Dashboard](tui-dashboard.md)

---

**Last Updated:** 2026-01-27
**Status:** Phase 1 Complete, Phase 2 In Progress
