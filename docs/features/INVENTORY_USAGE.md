# Inventory Command - Quick Start Guide

## Overview
The `inventory` command generates a Software Bill of Materials (SBOM) from disk images, providing comprehensive package information, licenses, and vulnerabilities.

## Installation
```bash
cargo build --release
./target/release/guestkit inventory --help
```

## Basic Usage

### Generate SPDX SBOM
```bash
guestkit inventory vm.qcow2 --format spdx -o sbom.spdx.json
```

### Generate CycloneDX SBOM
```bash
guestkit inventory vm.qcow2 --format cyclonedx -o bom.cdx.json
```

### Show Summary
```bash
guestkit inventory vm.qcow2 --summary
```

Output:
```
📦 Software Bill of Materials (SBOM)
=====================================

Image: vm.qcow2
OS: Ubuntu 22.04
Architecture: x86_64
Scanned: 2024-02-02T21:00:00Z

📊 Statistics
-------------
Total Packages: 487
Total Size: 2.3 GB

⚠️  Vulnerabilities
------------------
🔴 critical: 2
🟠 high: 15
🟡 medium: 43
🟢 low: 89

⚖️  Licenses (Top 10)
--------------------
GPL-3.0-or-later: 123
MIT: 89
Apache-2.0: 56
BSD-3-Clause: 34
```

### Generate with Licenses and CVEs
```bash
guestkit inventory vm.qcow2 \
  --include-licenses \
  --include-cves \
  --format spdx \
  -o full-sbom.json
```

### Export as CSV
```bash
guestkit inventory vm.qcow2 --format csv -o packages.csv
```

## Output Formats

| Format | Extension | Description |
|--------|-----------|-------------|
| `spdx` | .spdx.json | SPDX 2.3 standard format |
| `cyclonedx` | .cdx.json | CycloneDX 1.5 BOM format |
| `json` | .json | Simple JSON format |
| `csv` | .csv | CSV spreadsheet format |

## Options

### Include Additional Data
- `--include-licenses` - Add license information for each package
- `--include-cves` - Include CVE vulnerability mappings
- `--include-files` - Add file manifests

### Filter Vulnerabilities
```bash
guestkit inventory vm.qcow2 \
  --include-cves \
  --severity critical,high \
  --format csv -o critical-vulns.csv
```

### Verbose Output
```bash
guestkit inventory vm.qcow2 --verbose --summary
```

## Integration Examples

### With Grype
```bash
guestkit inventory vm.qcow2 --format spdx -o sbom.json
grype sbom:sbom.json
```

### With OSV Scanner
```bash
guestkit inventory vm.qcow2 --format cyclonedx -o bom.json
osv-scanner --sbom=bom.json
```

### With Dependency-Track
```bash
guestkit inventory vm.qcow2 --format cyclonedx -o bom.json

curl -X PUT "https://dtrack.example.com/api/v1/bom" \
  -H "X-Api-Key: $API_KEY" \
  -H "Content-Type: application/json" \
  -d @bom.json
```

## CI/CD Integration

### GitHub Actions
```yaml
- name: Generate SBOM
  run: |
    guestkit inventory vm.qcow2 \
      --format spdx \
      --include-licenses \
      --include-cves \
      -o sbom.json

- name: Upload SBOM
  uses: actions/upload-artifact@v3
  with:
    name: sbom
    path: sbom.json
```

### GitLab CI
```yaml
sbom:
  stage: security
  script:
    - guestkit inventory $IMAGE_FILE --format cyclonedx -o bom.json
  artifacts:
    paths:
      - bom.json
```

## Supported Systems

### Linux Distributions
- ✅ Debian/Ubuntu (DEB packages)
- ✅ RHEL/CentOS/Rocky/AlmaLinux (RPM packages)
- ✅ Fedora (RPM packages)
- ✅ openSUSE (RPM packages)

### Package Managers
- ✅ APT (Debian/Ubuntu)
- ✅ YUM/DNF (RHEL-based)
- ✅ Zypper (openSUSE)

## Known Licenses

The inventory command includes built-in license detection for 25+ common packages:

- nginx → BSD-2-Clause
- apache2/httpd → Apache-2.0
- openssl → Apache-2.0
- python3 → PSF-2.0
- bash → GPL-3.0-or-later
- curl → MIT
- git → GPL-2.0-only
- redis → BSD-3-Clause
- postgresql → PostgreSQL
- mysql/mariadb → GPL-2.0-only

And many more...

## CVE Database

The current implementation includes example CVEs for demonstration. In production, this would integrate with:
- NVD (National Vulnerability Database)
- OSV (Open Source Vulnerabilities)
- GitHub Advisory Database
- Snyk Vulnerability DB

## Limitations

Current version:
- File manifest support is basic
- CVE data is examples only (production would use real CVE DB)
- License detection limited to well-known packages
- No dependency tree visualization yet

## Troubleshooting

### "No operating systems found"
```bash
# Verify image format
guestkit detect vm.qcow2

# Try with verbose mode
guestkit inventory vm.qcow2 --verbose
```

### "Unsupported package format"
The image uses a package format not yet supported. Currently supported:
- DEB (Debian/Ubuntu)
- RPM (RHEL/CentOS/Fedora)

## Future Enhancements

- [ ] Real CVE database integration
- [ ] Dependency tree visualization
- [ ] File manifest generation
- [ ] Alpine APK support
- [ ] Arch Linux pacman support
- [ ] Container image SBOM
- [ ] SBOM signing and verification
- [ ] SBOM diff between images
- [ ] Web UI for SBOM visualization

## Performance

Typical scan times:
- Small VM (100 packages): 10-15 seconds
- Medium VM (500 packages): 30-45 seconds
- Large VM (2000+ packages): 1-2 minutes

## Contributing

See [CONTRIBUTING.md](../../CONTRIBUTING.md) for guidelines on:
- Adding new license mappings
- Integrating CVE databases
- Supporting new package formats
- Improving SBOM formats

---

## Security Posture

GuestKit itself has undergone a comprehensive security audit with 54 issues fixed across input validation, error propagation, and safe resource handling. This complements the security scanning capabilities the inventory command provides for inspected disk images.

*Last updated: 2024-02-02*
