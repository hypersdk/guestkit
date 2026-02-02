# Inventory Feature - SBOM Generation

## Overview
The `inventory` command generates a Software Bill of Materials (SBOM) for disk images, providing comprehensive visibility into installed software, dependencies, licenses, and vulnerabilities.

## Command Syntax

```bash
guestkit inventory [OPTIONS] <IMAGE>
```

## Options

```
OPTIONS:
    -f, --format <FORMAT>           Output format [default: spdx] [possible: spdx, cyclonedx, json, csv]
    -o, --output <FILE>             Output file (stdout if not specified)
        --include-licenses          Include license information for each package
        --include-files             Include file manifests
        --include-cves              Include known CVE mappings
        --severity <LEVEL>          Filter CVEs by severity [critical, high, medium, low]
        --exclude-os                Exclude OS base packages
        --exclude <PATTERNS>        Exclude packages matching patterns
        --with-dependencies         Include dependency trees
        --with-provenance           Include package provenance data
        --format-version <VER>      SBOM format version [default: latest]
        --namespace <URI>           SPDX namespace URI
        --creator <NAME>            Creator information
```

## Examples

### Basic SBOM Generation

```bash
# Generate SPDX SBOM
guestkit inventory production-vm.qcow2 --output sbom.spdx.json

# CycloneDX format
guestkit inventory app-server.qcow2 --format cyclonedx --output bom.json

# Simple JSON format
guestkit inventory dev-vm.qcow2 --format json --output inventory.json
```

### Advanced Usage

```bash
# SBOM with licenses and CVEs
guestkit inventory webapp.qcow2 \
    --format spdx \
    --include-licenses \
    --include-cves \
    --severity critical,high \
    --output webapp-sbom.json

# Application-only SBOM (exclude OS)
guestkit inventory app.qcow2 \
    --exclude-os \
    --with-dependencies \
    --output app-only-sbom.json

# Detailed SBOM with file manifests
guestkit inventory production.qcow2 \
    --format cyclonedx \
    --include-files \
    --with-provenance \
    --output detailed-bom.xml
```

### Comparison and Analysis

```bash
# Compare two inventories
guestkit inventory diff prod.qcow2 staging.qcow2 \
    --show-new \
    --show-removed \
    --show-version-changes

# Find vulnerable packages
guestkit inventory webapp.qcow2 \
    --include-cves \
    --severity critical \
    --format csv \
    --output vulnerabilities.csv

# License compliance check
guestkit inventory app.qcow2 \
    --include-licenses \
    --exclude GPL-3.0,AGPL \
    --output license-report.json
```

## Output Formats

### SPDX 2.3 (JSON)

```json
{
  "spdxVersion": "SPDX-2.3",
  "dataLicense": "CC0-1.0",
  "SPDXID": "SPDXRef-DOCUMENT",
  "name": "production-vm.qcow2",
  "documentNamespace": "https://example.com/sbom/production-vm-20240202",
  "creationInfo": {
    "created": "2024-02-02T12:00:00Z",
    "creators": ["Tool: guestkit-0.3.2"],
    "licenseListVersion": "3.21"
  },
  "packages": [
    {
      "SPDXID": "SPDXRef-Package-nginx",
      "name": "nginx",
      "versionInfo": "1.24.0-2ubuntu1",
      "supplier": "Organization: Ubuntu",
      "downloadLocation": "NOASSERTION",
      "filesAnalyzed": false,
      "licenseConcluded": "BSD-2-Clause",
      "licenseDeclared": "BSD-2-Clause",
      "copyrightText": "NOASSERTION",
      "externalRefs": [
        {
          "referenceCategory": "SECURITY",
          "referenceType": "cpe23Type",
          "referenceLocator": "cpe:2.3:a:nginx:nginx:1.24.0:*:*:*:*:*:*:*"
        }
      ]
    }
  ],
  "relationships": [
    {
      "spdxElementId": "SPDXRef-DOCUMENT",
      "relationshipType": "DESCRIBES",
      "relatedSpdxElement": "SPDXRef-Package-nginx"
    }
  ]
}
```

### CycloneDX 1.5 (JSON)

```json
{
  "bomFormat": "CycloneDX",
  "specVersion": "1.5",
  "serialNumber": "urn:uuid:3e671687-395b-41f5-a30f-a58921a69b79",
  "version": 1,
  "metadata": {
    "timestamp": "2024-02-02T12:00:00Z",
    "tools": [
      {
        "vendor": "guestkit",
        "name": "guestkit",
        "version": "0.3.2"
      }
    ],
    "component": {
      "type": "application",
      "name": "production-vm",
      "version": "1.0.0"
    }
  },
  "components": [
    {
      "type": "library",
      "bom-ref": "pkg:deb/ubuntu/nginx@1.24.0-2ubuntu1",
      "name": "nginx",
      "version": "1.24.0-2ubuntu1",
      "purl": "pkg:deb/ubuntu/nginx@1.24.0-2ubuntu1",
      "licenses": [
        {
          "license": {
            "id": "BSD-2-Clause"
          }
        }
      ],
      "hashes": [
        {
          "alg": "SHA-256",
          "content": "abc123..."
        }
      ]
    }
  ],
  "dependencies": [
    {
      "ref": "pkg:deb/ubuntu/nginx@1.24.0-2ubuntu1",
      "dependsOn": [
        "pkg:deb/ubuntu/libc6@2.35-0ubuntu3"
      ]
    }
  ],
  "vulnerabilities": [
    {
      "id": "CVE-2024-1234",
      "source": {
        "name": "NVD",
        "url": "https://nvd.nist.gov/vuln/detail/CVE-2024-1234"
      },
      "ratings": [
        {
          "severity": "high",
          "score": 7.5,
          "method": "CVSSv3"
        }
      ],
      "affects": [
        {
          "ref": "pkg:deb/ubuntu/nginx@1.24.0-2ubuntu1"
        }
      ]
    }
  ]
}
```

### Simple JSON Format

```json
{
  "image": "production-vm.qcow2",
  "scanned_at": "2024-02-02T12:00:00Z",
  "os": {
    "name": "Ubuntu",
    "version": "22.04",
    "architecture": "x86_64"
  },
  "packages": [
    {
      "name": "nginx",
      "version": "1.24.0-2ubuntu1",
      "type": "deb",
      "license": "BSD-2-Clause",
      "size": 1234567,
      "installed_date": "2024-01-15T10:30:00Z",
      "files": [
        "/usr/sbin/nginx",
        "/etc/nginx/nginx.conf"
      ],
      "dependencies": [
        "libc6 (>= 2.34)",
        "libssl3 (>= 3.0.0)"
      ],
      "vulnerabilities": [
        {
          "cve": "CVE-2024-1234",
          "severity": "high",
          "score": 7.5,
          "description": "Buffer overflow in nginx..."
        }
      ]
    }
  ],
  "statistics": {
    "total_packages": 487,
    "total_size": "2.3 GB",
    "vulnerabilities": {
      "critical": 2,
      "high": 15,
      "medium": 43,
      "low": 89
    },
    "licenses": {
      "GPL-2.0": 123,
      "MIT": 89,
      "Apache-2.0": 56,
      "BSD-3-Clause": 34
    }
  }
}
```

### CSV Format

```csv
Package,Version,License,CVEs,Severity,Size,Install Date
nginx,1.24.0-2ubuntu1,BSD-2-Clause,CVE-2024-1234,high,1234567,2024-01-15
openssl,3.0.2-0ubuntu1,Apache-2.0,CVE-2024-5678,critical,2345678,2024-01-10
python3,3.10.6-1,PSF,none,none,567890,2024-01-05
```

## Diff Command

Compare inventories between two images:

```bash
guestkit inventory diff prod.qcow2 staging.qcow2 [OPTIONS]

OPTIONS:
    --show-new              Show packages only in second image
    --show-removed          Show packages only in first image
    --show-version-changes  Show packages with version differences
    --show-all             Show all differences
    --format <FORMAT>      Output format [default: text]
```

### Diff Output Example

```
Inventory Comparison: prod.qcow2 vs staging.qcow2
==================================================

📦 NEW PACKAGES (in staging.qcow2):
  + redis-server 7.0.11-1ubuntu1
  + nodejs 18.16.0-1nodesource1

❌ REMOVED PACKAGES (from staging.qcow2):
  - memcached 1.6.14-1
  - apache2 2.4.52-1ubuntu4

🔄 VERSION CHANGES:
  nginx: 1.24.0-2ubuntu1 → 1.25.1-1ubuntu1 (upgrade)
  openssl: 3.0.2-0ubuntu1 → 3.0.8-1ubuntu1 (security update)

⚠️  NEW VULNERABILITIES:
  redis-server: CVE-2024-9999 (high)

✅ FIXED VULNERABILITIES:
  openssl: CVE-2024-5678 (critical)

Summary:
  New packages: 2
  Removed packages: 2
  Version changes: 2
  Security improvements: 1 critical fixed
  Security concerns: 1 high introduced
```

## Use Cases

### 1. Supply Chain Security

```bash
# Generate comprehensive SBOM for supply chain transparency
guestkit inventory production.qcow2 \
    --format spdx \
    --include-licenses \
    --include-cves \
    --with-provenance \
    --output production-sbom.spdx.json

# Sign SBOM
cosign sign-blob --key key.pem production-sbom.spdx.json \
    --output-signature production-sbom.spdx.json.sig
```

### 2. License Compliance

```bash
# Check for GPL violations
guestkit inventory commercial-app.qcow2 \
    --include-licenses \
    --exclude-os \
    --format json | \
jq '.packages[] | select(.license | contains("GPL"))'

# Generate attribution notices
guestkit inventory app.qcow2 \
    --include-licenses \
    --format text \
    --output THIRD_PARTY_NOTICES.txt
```

### 3. Vulnerability Management

```bash
# Critical vulnerabilities report
guestkit inventory webapp.qcow2 \
    --include-cves \
    --severity critical,high \
    --format csv \
    --output vulnerabilities.csv

# Compare vulnerability status
guestkit inventory diff \
    webapp-before-patch.qcow2 \
    webapp-after-patch.qcow2 \
    --show-cve-changes
```

### 4. Dependency Analysis

```bash
# Full dependency tree
guestkit inventory app.qcow2 \
    --with-dependencies \
    --format json | \
jq '.packages[] | select(.name == "nginx") | .dependencies'

# Find all packages depending on OpenSSL
guestkit inventory webapp.qcow2 \
    --with-dependencies \
    --format json | \
jq '.packages[] | select(.dependencies[] | contains("libssl"))'
```

### 5. CI/CD Integration

```bash
#!/bin/bash
# Generate and verify SBOM in CI pipeline

# Build image
packer build vm.json

# Generate SBOM
guestkit inventory output-vm.qcow2 \
    --format cyclonedx \
    --include-cves \
    --output sbom.json

# Check for critical vulnerabilities
CRITICAL=$(jq '.vulnerabilities[] | select(.ratings[].severity == "critical") | .id' sbom.json)

if [ -n "$CRITICAL" ]; then
    echo "❌ Critical vulnerabilities found: $CRITICAL"
    exit 1
fi

# Upload SBOM to artifact store
aws s3 cp sbom.json s3://sbom-bucket/$(date +%Y%m%d)/sbom.json
```

## Integration Examples

### Dependency-Track

```bash
# Generate CycloneDX SBOM for Dependency-Track
guestkit inventory vm.qcow2 \
    --format cyclonedx \
    --include-cves \
    --output bom.json

# Upload to Dependency-Track
curl -X PUT "https://dtrack.example.com/api/v1/bom" \
    -H "X-Api-Key: $API_KEY" \
    -H "Content-Type: application/json" \
    -d @bom.json
```

### Grype Integration

```bash
# Generate SBOM and scan with Grype
guestkit inventory vm.qcow2 --format spdx -o sbom.spdx.json
grype sbom:sbom.spdx.json
```

### OSV Scanner

```bash
# Scan SBOM with OSV
guestkit inventory vm.qcow2 --format cyclonedx -o bom.json
osv-scanner --sbom=bom.json
```

## Performance

- Typical scan time: 30-60 seconds for standard VM (500 packages)
- Large systems (5000+ packages): 2-5 minutes
- Parallel processing for batch operations
- Caching for repeated scans

## Caching

```bash
# First scan (slow)
guestkit inventory large-vm.qcow2 --output sbom.json

# Subsequent scans (fast - uses cache)
guestkit inventory large-vm.qcow2 --output sbom.json

# Force refresh
guestkit inventory large-vm.qcow2 --no-cache --output sbom.json
```

## Error Handling

```bash
# Validation
guestkit inventory vm.qcow2 --validate-sbom

# Strict mode (fail on warnings)
guestkit inventory vm.qcow2 --strict

# Verbose output
guestkit inventory vm.qcow2 --verbose
```

## Comparison with Other Tools

| Feature | GuestKit | Syft | Trivy | Tern |
|---------|----------|------|-------|------|
| Disk Image Support | ✅ | ❌ | ⚠️ | ❌ |
| SPDX Format | ✅ | ✅ | ✅ | ✅ |
| CycloneDX Format | ✅ | ✅ | ✅ | ❌ |
| License Detection | ✅ | ✅ | ✅ | ✅ |
| CVE Mapping | ✅ | ❌ | ✅ | ❌ |
| Dependency Trees | ✅ | ⚠️ | ⚠️ | ✅ |
| File Manifests | ✅ | ✅ | ❌ | ✅ |
| Offline Support | ✅ | ✅ | ⚠️ | ✅ |

## Future Enhancements

- [ ] Container image SBOM generation
- [ ] Incremental SBOM updates
- [ ] SBOM signing and verification
- [ ] SBOM attestation (in-toto)
- [ ] Machine learning for license detection
- [ ] Real-time CVE monitoring
- [ ] SBOM diff visualization (web UI)
- [ ] Custom SBOM formats
- [ ] SBOM merge for multi-image systems

---

*Feature Status: Proposed*
*Priority: High*
*Estimated Effort: 2-3 weeks*
