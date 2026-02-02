# License Compliance Command

Scan disk images for license compliance, detect violations, and generate attribution notices.

## Features

- **License Scanning**: Extract licenses from all installed packages
- **Risk Assessment**: Classify packages by risk level (Low, Medium, High, Critical)
- **Violation Detection**: Find prohibited licenses, missing licenses, and compatibility issues
- **Attribution Generation**: Create third-party attribution notices
- **Multiple Formats**: Output as text, JSON, or CSV
- **License Database**: Comprehensive database of known licenses with compatibility info

## Usage

### Basic License Scan

```bash
guestctl license fedora.qcow2
```

### Scan with Prohibited Licenses

```bash
guestctl license ubuntu.qcow2 --prohibit AGPL-3.0 --prohibit GPL-3.0
```

### Show Detailed Package Information

```bash
guestctl license debian.img --details
```

### Generate Attribution Notices

```bash
guestctl license centos.qcow2 --attribution
```

### Export to JSON

```bash
guestctl license rhel.img --format json --output licenses.json
```

### Export to CSV

```bash
guestctl license ubuntu.qcow2 --format csv --output licenses.csv
```

### Strict Mode (Fail on Violations)

```bash
guestctl license fedora.img --strict
```

## Output Formats

### Text Format (Default)

```
📋 License Compliance Report
============================

Image: ubuntu-22.04.qcow2
Scanned: 2024-01-15T10:30:00Z
Total Packages: 1,247

📊 License Statistics
--------------------
✅ Permissive: 856
⚖️  Copyleft: 234
🔒 Strong Copyleft: 89
💼 Proprietary: 12
❓ Unknown: 56

⚠️  Risk Summary
---------------
🟢 Low: 892
🟡 Medium: 278
🟠 High: 65
🔴 Critical: 12

📈 Compliance Score: 95.5%

🚨 License Violations
--------------------
🔴 systemd - Package uses prohibited license: LGPL-2.1
   💡 Remove package systemd or obtain license exception

📜 License Distribution (Top 10)
--------------------------------
GPL-2.0: 234
MIT: 198
Apache-2.0: 176
BSD-3-Clause: 145
LGPL-2.1: 98
GPL-3.0: 67
ISC: 45
BSD-2-Clause: 34
MPL-2.0: 23
PSF-2.0: 19

✅ No license violations found!
```

### JSON Format

```json
{
  "image_path": "ubuntu-22.04.qcow2",
  "scanned_at": "2024-01-15T10:30:00Z",
  "total_packages": 1247,
  "packages": [
    {
      "package_name": "bash",
      "version": "5.1.16",
      "license": "GPL-3.0-or-later",
      "license_type": "StrongCopyleft",
      "risk_level": "High",
      "compatible_with": ["LGPL-3.0", "Apache-2.0"],
      "incompatible_with": ["Proprietary"]
    }
  ],
  "license_summary": {
    "GPL-2.0": 234,
    "MIT": 198,
    "Apache-2.0": 176
  },
  "risk_summary": {
    "Low": 892,
    "Medium": 278,
    "High": 65,
    "Critical": 12
  },
  "violations": [],
  "statistics": {
    "permissive_licenses": 856,
    "copyleft_licenses": 234,
    "strong_copyleft_licenses": 89,
    "proprietary_licenses": 12,
    "unknown_licenses": 56,
    "compliance_score": 95.5
  }
}
```

### CSV Format

```csv
Package,Version,License,Type,Risk Level
bash,5.1.16,GPL-3.0-or-later,StrongCopyleft,High
coreutils,8.32,GPL-3.0-or-later,StrongCopyleft,High
curl,7.81.0,MIT,Permissive,Low
```

### Attribution Format

```
THIRD-PARTY SOFTWARE NOTICES AND INFORMATION
===========================================

This software incorporates components from the projects listed below.
Generated: 2024-01-15T10:30:00Z

MIT (198 packages)
==========================================
  - curl 7.81.0
  - jq 1.6
  - libxml2 2.9.14
  ...

Apache-2.0 (176 packages)
==========================================
  - python3 3.10.6
  - docker 20.10.12
  - kubernetes 1.25.0
  ...
```

## License Types

### Permissive
- **MIT**: Maximum freedom, minimal restrictions
- **Apache-2.0**: Patent grant, attribution required
- **BSD-2/3-Clause**: Simple permissive licenses
- **ISC**: Functionally equivalent to MIT
- **Zlib**: Common for compression libraries

### Weak Copyleft
- **LGPL-2.1/3.0**: Library GPL, allows proprietary linking
- **MPL-2.0**: File-level copyleft

### Strong Copyleft
- **GPL-2.0/3.0**: Requires derivative works to be GPL
- **AGPL-3.0**: Network copyleft (highest risk)

### Public Domain
- **Unlicense**: No restrictions
- **Public-Domain**: Explicitly public domain

## Risk Levels

| Level | Emoji | Description | Examples |
|-------|-------|-------------|----------|
| **Low** | 🟢 | Permissive licenses, minimal restrictions | MIT, BSD, Apache-2.0 |
| **Medium** | 🟡 | Weak copyleft, requires attention | LGPL, MPL |
| **High** | 🟠 | Strong copyleft, significant obligations | GPL-2.0, GPL-3.0 |
| **Critical** | 🔴 | Maximum restrictions, network copyleft | AGPL-3.0 |

## Violation Types

### Prohibited License
Package uses a license explicitly prohibited by policy.

**Example**: Using AGPL when prohibited
```
Remediation: Remove package or obtain license exception
```

### Missing License
Package has unknown or undeclared license.

**Example**: Package without license metadata
```
Remediation: Investigate and document package license
```

### Commercial Restriction
License has restrictions on commercial use.

**Example**: AGPL network copyleft requirements
```
Remediation: Review AGPL requirements or replace package
```

### Incompatible Licenses
Multiple licenses that cannot be combined.

**Example**: GPL and proprietary code
```
Remediation: Separate components or relicense
```

## Compliance Workflows

### Software Audit

1. **Scan the image**:
   ```bash
   guestctl license app.qcow2 --format json --output audit.json
   ```

2. **Review violations**:
   ```bash
   guestctl license app.qcow2 --details | grep "🔴\|🟠"
   ```

3. **Generate attribution**:
   ```bash
   guestctl license app.qcow2 --attribution --output NOTICES.txt
   ```

### Policy Enforcement

```bash
# Block AGPL and GPL-3.0
guestctl license prod.img \
  --prohibit AGPL-3.0 \
  --prohibit GPL-3.0 \
  --strict

# Exit code 1 if violations found
if [ $? -eq 0 ]; then
  echo "✅ License compliance passed"
else
  echo "❌ License violations detected"
  exit 1
fi
```

### CI/CD Integration

```yaml
# .github/workflows/license-check.yml
name: License Compliance

on: [push, pull_request]

jobs:
  license-check:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v2

      - name: Scan licenses
        run: |
          guestctl license image.qcow2 \
            --prohibit AGPL-3.0 \
            --format json \
            --output licenses.json \
            --strict

      - name: Upload results
        uses: actions/upload-artifact@v2
        with:
          name: license-report
          path: licenses.json
```

## Best Practices

### 1. Define Clear Policies
Establish which licenses are acceptable for your use case:
- **Open Source Projects**: Usually permissive + copyleft OK
- **Commercial Products**: Typically permissive only
- **Enterprise**: May prohibit strong copyleft (GPL, AGPL)

### 2. Regular Scanning
Scan images at multiple stages:
- **Development**: Catch issues early
- **CI/CD**: Enforce policy automatically
- **Release**: Final compliance check
- **Periodic Audits**: Detect new packages

### 3. Document Decisions
When exceptions are needed:
```bash
# Document why exception was granted
echo "libgpl-exception: Legal approved 2024-01-15" >> LICENSE_EXCEPTIONS.txt
```

### 4. Generate Attribution
Include attribution in your distributions:
```bash
guestctl license app.qcow2 --attribution --output THIRD_PARTY_NOTICES.txt
```

### 5. Track Changes
Monitor license changes over time:
```bash
# Compare against baseline
diff baseline-licenses.json current-licenses.json
```

## Common Scenarios

### Scenario 1: Unknown Licenses

**Problem**: Many packages show "Unknown" license.

**Solution**:
1. Check if packages have non-standard license declarations
2. Update license detection rules in `inventory/licenses.rs`
3. Manually document licenses in your SBOM

### Scenario 2: AGPL Violation

**Problem**: Critical AGPL violation detected.

**Options**:
1. **Replace**: Find alternative package with different license
2. **Isolate**: Run AGPL code in separate service/process
3. **Comply**: Accept AGPL terms and make code available
4. **Commercial**: Purchase commercial license if available

### Scenario 3: GPL Compatibility

**Problem**: Mixing GPL-2.0 and GPL-3.0 code.

**Solution**:
- GPL-2.0-only is incompatible with GPL-3.0
- GPL-2.0-or-later is compatible with GPL-3.0
- Consider upgrading to GPL-3.0 or using LGPL

### Scenario 4: Attribution Requirements

**Problem**: Many licenses require attribution.

**Solution**:
```bash
# Generate comprehensive attribution
guestctl license app.qcow2 --attribution --output NOTICES.txt

# Include in your distribution
cp NOTICES.txt /path/to/release/
```

## Integration Examples

### With Inventory Command

Combine license and SBOM data:

```bash
# Generate SBOM
guestctl inventory app.qcow2 --format spdx --output sbom.json

# Check licenses
guestctl license app.qcow2 --format json --output licenses.json

# Combine for comprehensive audit
jq -s '.[0] * .[1]' sbom.json licenses.json > full-audit.json
```

### With Validation Command

License checks in policy:

```yaml
# policy.yaml
rules:
  - name: no-agpl
    type: command_output
    command: "guestctl license {image} --prohibit AGPL-3.0 --strict"
    expected_exit_code: 0
    severity: critical
```

## Troubleshooting

### Issue: "No operating system found"

```bash
# Verify image format
qemu-img info image.qcow2

# Try different image
guestctl license image.img
```

### Issue: "License detection failing"

```bash
# Run with verbose output
RUST_LOG=debug guestctl license image.qcow2

# Check if packages are installed
guestctl inspect image.qcow2 | jq '.applications | length'
```

### Issue: "Too many unknown licenses"

The license detection relies on package metadata. Some distributions may have incomplete license information. Consider:
- Manually documenting licenses
- Using distribution-specific tools to gather license data
- Contributing improved license detection rules

## Exit Codes

| Code | Meaning |
|------|---------|
| 0 | Success, no violations (or violations found but not strict mode) |
| 1 | Violations found and strict mode enabled |
| 2 | Error during scanning or processing |

## See Also

- [Inventory Command](INVENTORY.md) - Generate Software Bill of Materials
- [Validate Command](VALIDATE.md) - Policy-based validation
- [SPDX License List](https://spdx.org/licenses/) - Standard license identifiers
- [Choose a License](https://choosealicense.com/) - License selection guide
