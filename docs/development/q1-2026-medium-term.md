# Q1 2026 Medium-Term Implementation Plan

Detailed implementation plan for three key Q1 priorities.

## Overview

**Timeline:** February - March 2026
**Focus Areas:**
1. Performance Optimization (20%+ improvement)
2. Export Enhancements (HTML, PDF, Markdown)
3. Testing Improvements (Quality assurance)

**Success Metrics:**
- Inspection speed: 20%+ faster
- Export formats: 3 new formats
- Test coverage: 85%+
- Zero critical bugs

---

## 1. Performance Optimization (20%+ Improvement)

**Goal:** Achieve 20%+ performance improvement across all operations.

### Current Performance Baseline

| Operation | Current Time | Target Time | Improvement |
|-----------|--------------|-------------|-------------|
| Appliance launch | ~2.5s | ~2.0s | 20% |
| OS inspection | ~500ms | ~400ms | 20% |
| Package listing | ~3.5s | ~2.8s | 20% |
| File operations | ~15ms | ~12ms | 20% |
| Cache lookup | ~500ms | <100ms | 80% |
| QCOW2 → RAW inspection | ~30s | ~24s | 20% |

### Strategy 1: Binary Cache (bincode)

**Impact:** 60-80% faster cache operations

**Implementation:**

```rust
// src/core/cache.rs

use bincode::{serialize, deserialize};
use serde::{Serialize, Deserialize};
use std::fs;
use std::path::PathBuf;

#[derive(Serialize, Deserialize)]
pub struct CachedInspection {
    pub timestamp: u64,
    pub disk_hash: String,
    pub os_info: OsInfo,
    pub filesystems: Vec<Filesystem>,
    pub users: Vec<User>,
    pub packages: Vec<Package>,
}

pub struct BinaryCache {
    cache_dir: PathBuf,
}

impl BinaryCache {
    pub fn new() -> Result<Self> {
        let cache_dir = dirs::cache_dir()
            .ok_or_else(|| anyhow!("Could not find cache directory"))?
            .join("guestctl")
            .join("binary");

        fs::create_dir_all(&cache_dir)?;

        Ok(Self { cache_dir })
    }

    pub fn save(&self, key: &str, data: &CachedInspection) -> Result<()> {
        let path = self.cache_dir.join(format!("{}.bin", key));
        let encoded = serialize(data)?;
        fs::write(path, encoded)?;
        Ok(())
    }

    pub fn load(&self, key: &str) -> Result<CachedInspection> {
        let path = self.cache_dir.join(format!("{}.bin", key));
        let bytes = fs::read(path)?;
        let data = deserialize(&bytes)?;
        Ok(data)
    }

    pub fn exists(&self, key: &str) -> bool {
        self.cache_dir.join(format!("{}.bin", key)).exists()
    }
}
```

**Benefits:**
- 5-10x faster than JSON serialization
- Smaller cache files (50-70% reduction)
- Type-safe deserialization

**Timeline:** Week 1-2 of February

### Strategy 2: Parallel Processing (rayon)

**Impact:** 3-4x faster for batch operations

**Implementation:**

```rust
// src/cli/commands.rs

use rayon::prelude::*;

pub fn inspect_batch(
    images: Vec<PathBuf>,
    parallel: usize,
    cache: bool,
) -> Result<Vec<InspectionResult>> {
    // Configure thread pool
    let pool = rayon::ThreadPoolBuilder::new()
        .num_threads(parallel)
        .build()?;

    pool.install(|| {
        images
            .par_iter()
            .map(|image| {
                inspect_single(image, cache)
            })
            .collect::<Result<Vec<_>>>()
    })
}

pub fn inspect_single(image: &PathBuf, cache: bool) -> Result<InspectionResult> {
    let mut g = Guestfs::new()?;
    g.add_drive_ro(image)?;
    g.launch()?;

    let result = perform_inspection(&mut g)?;

    if cache {
        save_to_cache(image, &result)?;
    }

    g.shutdown()?;
    Ok(result)
}
```

**Benefits:**
- 4 workers = ~4x speedup for batch operations
- Automatic work distribution
- CPU utilization optimization

**Timeline:** Week 2 of February

### Strategy 3: Memory Optimization

**Impact:** 30-40% memory reduction

**Implementation:**

```rust
// Use Arc for shared data
use std::sync::Arc;

#[derive(Clone)]
pub struct OptimizedGuestfs {
    inner: Arc<GuestfsInner>,
}

// Implement Copy-on-Write for large data structures
use std::borrow::Cow;

pub struct InspectionResult<'a> {
    os_info: Cow<'a, OsInfo>,
    filesystems: Cow<'a, [Filesystem]>,
    packages: Cow<'a, [Package]>,
}

// Use string interning for repeated values
use string_cache::DefaultAtom as Atom;

#[derive(Clone)]
pub struct Package {
    name: Atom,        // Interned string
    version: Atom,     // Interned string
    arch: Atom,        // Interned string
}
```

**Benefits:**
- Reduced memory allocations
- Faster cloning operations
- Lower memory footprint for large datasets

**Timeline:** Week 3 of February

### Strategy 4: Profiling & Flamegraphs

**Setup:**

```bash
# Install profiling tools
cargo install flamegraph
cargo install cargo-profdata

# Generate flamegraph
sudo cargo flamegraph --bin guestctl -- inspect test.qcow2

# Generate callgrind profile
valgrind --tool=callgrind --callgrind-out-file=callgrind.out \
  target/release/guestctl inspect test.qcow2

# Analyze with kcachegrind
kcachegrind callgrind.out
```

**Profiling Points:**

```rust
// src/core/profiling.rs

use std::time::Instant;

pub struct ProfileScope {
    name: &'static str,
    start: Instant,
}

impl ProfileScope {
    pub fn new(name: &'static str) -> Self {
        Self {
            name,
            start: Instant::now(),
        }
    }
}

impl Drop for ProfileScope {
    fn drop(&mut self) {
        let elapsed = self.start.elapsed();
        if elapsed.as_millis() > 10 {
            eprintln!("[PROFILE] {} took {:?}", self.name, elapsed);
        }
    }
}

// Usage
pub fn inspect_os(&mut self) -> Result<Vec<String>> {
    let _scope = ProfileScope::new("inspect_os");
    // ... implementation
}
```

**Timeline:** Week 1 of February (ongoing)

### Strategy 5: Loop Device Optimization

**Impact:** Eliminate NBD overhead for RAW/IMG/ISO

**Implementation:**

```rust
// src/disk/loop_device.rs

use std::process::Command;

pub struct OptimizedLoopDevice {
    device: String,
    read_ahead_kb: usize,
}

impl OptimizedLoopDevice {
    pub fn attach(image_path: &str) -> Result<Self> {
        // Find free loop device
        let output = Command::new("losetup")
            .arg("-f")
            .output()?;
        let device = String::from_utf8(output.stdout)?.trim().to_string();

        // Attach with optimizations
        Command::new("losetup")
            .arg("--read-only")
            .arg("--direct-io=on")        // Direct I/O
            .arg("--partscan")            // Auto partition scan
            .arg(&device)
            .arg(image_path)
            .status()?;

        // Set read-ahead (improves sequential reads)
        let read_ahead_kb = 2048; // 2MB
        Command::new("blockdev")
            .arg("--setra")
            .arg(read_ahead_kb.to_string())
            .arg(&device)
            .status()?;

        Ok(Self { device, read_ahead_kb })
    }
}
```

**Benefits:**
- Direct I/O bypasses page cache
- Larger read-ahead improves sequential performance
- Automatic partition detection

**Timeline:** Week 2-3 of February

### Performance Testing Suite

```rust
// benches/performance.rs

use criterion::{criterion_group, criterion_main, Criterion, BenchmarkId};
use guestctl::Guestfs;

fn benchmark_inspection(c: &mut Criterion) {
    let mut group = c.benchmark_group("inspection");

    // Test different image formats
    for format in &["raw", "qcow2", "vmdk"] {
        group.bench_with_input(
            BenchmarkId::new("inspect", format),
            format,
            |b, &format| {
                b.iter(|| {
                    let mut g = Guestfs::new().unwrap();
                    g.add_drive_ro(&format!("test.{}", format)).unwrap();
                    g.launch().unwrap();
                    g.inspect_os().unwrap();
                    g.shutdown().unwrap();
                });
            },
        );
    }

    group.finish();
}

fn benchmark_cache(c: &mut Criterion) {
    let mut group = c.benchmark_group("cache");

    group.bench_function("json_cache", |b| {
        b.iter(|| {
            // JSON cache operations
        });
    });

    group.bench_function("binary_cache", |b| {
        b.iter(|| {
            // Binary cache operations
        });
    });

    group.finish();
}

criterion_group!(benches, benchmark_inspection, benchmark_cache);
criterion_main!(benches);
```

**Run benchmarks:**

```bash
# Run all benchmarks
cargo bench

# Run specific benchmark
cargo bench -- inspection

# Compare with baseline
cargo bench --bench performance -- --save-baseline before
# ... make changes ...
cargo bench --bench performance -- --baseline before
```

**Timeline:** Week 1 of February (setup), ongoing monitoring

### Performance Dashboard

**Metrics to Track:**

```yaml
# performance-metrics.yml

benchmarks:
  - name: appliance_launch
    baseline: 2500ms
    target: 2000ms
    current: TBD

  - name: os_inspection
    baseline: 500ms
    target: 400ms
    current: TBD

  - name: cache_lookup
    baseline: 500ms
    target: 100ms
    current: TBD

  - name: package_listing
    baseline: 3500ms
    target: 2800ms
    current: TBD

memory:
  - name: peak_memory
    baseline: 512MB
    target: 350MB
    current: TBD

  - name: cache_size
    baseline: 50MB
    target: 15MB
    current: TBD
```

**Automated Tracking:**

```bash
#!/bin/bash
# scripts/track-performance.sh

echo "Running performance benchmarks..."
cargo bench --bench performance 2>&1 | tee bench-results.txt

# Extract metrics
LAUNCH_TIME=$(grep "appliance_launch" bench-results.txt | awk '{print $2}')
INSPECT_TIME=$(grep "os_inspection" bench-results.txt | awk '{print $2}')

# Update dashboard
echo "Launch: $LAUNCH_TIME" >> performance-log.txt
echo "Inspect: $INSPECT_TIME" >> performance-log.txt

# Check if targets met
if (( $(echo "$LAUNCH_TIME < 2000" | bc -l) )); then
    echo "✅ Launch time target met!"
else
    echo "❌ Launch time target not met (${LAUNCH_TIME}ms > 2000ms)"
fi
```

### Implementation Timeline

**Week 1 (Feb 3-9):**
- [ ] Set up profiling infrastructure
- [ ] Implement binary cache (bincode)
- [ ] Baseline performance measurements

**Week 2 (Feb 10-16):**
- [ ] Implement parallel processing (rayon)
- [ ] Optimize loop device handling
- [ ] Profile and identify hotspots

**Week 3 (Feb 17-23):**
- [ ] Memory optimization (Arc, Cow)
- [ ] Fix identified bottlenecks
- [ ] Performance testing

**Week 4 (Feb 24-Mar 2):**
- [ ] Final optimizations
- [ ] Documentation
- [ ] Performance validation (20%+ achieved)

---

## 2. Export Enhancements

**Goal:** Add 3 new export formats with rich formatting.

### HTML Export with Charts

**Features:**
- Interactive charts (Chart.js)
- Responsive design
- Printable reports
- Embedded CSS/JS (self-contained)

**Implementation:**

```rust
// src/cli/exporters/html_enhanced.rs

use askama::Template;
use chrono::Utc;

#[derive(Template)]
#[template(path = "report.html")]
pub struct HtmlReport<'a> {
    pub title: &'a str,
    pub generated_at: String,
    pub os_info: &'a OsInfo,
    pub filesystems: &'a [Filesystem],
    pub packages: &'a [Package],
    pub users: &'a [User],
    pub network: &'a NetworkInfo,
    pub chart_data: ChartData,
}

pub struct ChartData {
    pub disk_usage: Vec<(String, f64)>,
    pub package_distribution: Vec<(String, usize)>,
    pub user_distribution: Vec<(String, usize)>,
}

impl HtmlReport<'_> {
    pub fn generate(&self, output_path: &str) -> Result<()> {
        let html = self.render()?;
        std::fs::write(output_path, html)?;
        Ok(())
    }
}
```

**Template:**

```html
<!-- src/cli/templates/report.html -->
<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>{{ title }} - GuestCtl Report</title>
    <script src="https://cdn.jsdelivr.net/npm/chart.js@4.4.0/dist/chart.umd.min.js"></script>
    <style>
        * { margin: 0; padding: 0; box-sizing: border-box; }
        body {
            font-family: 'Segoe UI', Tahoma, Geneva, Verdana, sans-serif;
            line-height: 1.6;
            padding: 20px;
            max-width: 1200px;
            margin: 0 auto;
            background: #f5f5f5;
        }
        .header {
            background: linear-gradient(135deg, #667eea 0%, #764ba2 100%);
            color: white;
            padding: 40px;
            border-radius: 10px;
            margin-bottom: 30px;
            box-shadow: 0 4px 6px rgba(0,0,0,0.1);
        }
        .header h1 {
            font-size: 2.5em;
            margin-bottom: 10px;
        }
        .header .subtitle {
            opacity: 0.9;
            font-size: 1.1em;
        }
        .section {
            background: white;
            padding: 30px;
            margin-bottom: 20px;
            border-radius: 10px;
            box-shadow: 0 2px 4px rgba(0,0,0,0.1);
        }
        .section h2 {
            color: #667eea;
            margin-bottom: 20px;
            padding-bottom: 10px;
            border-bottom: 2px solid #667eea;
        }
        .metric {
            display: inline-block;
            background: #f0f0f0;
            padding: 15px 25px;
            margin: 10px 10px 10px 0;
            border-radius: 8px;
            border-left: 4px solid #667eea;
        }
        .metric-label {
            font-size: 0.9em;
            color: #666;
            margin-bottom: 5px;
        }
        .metric-value {
            font-size: 1.5em;
            font-weight: bold;
            color: #333;
        }
        .chart-container {
            position: relative;
            height: 400px;
            margin: 30px 0;
        }
        table {
            width: 100%;
            border-collapse: collapse;
            margin-top: 20px;
        }
        th, td {
            padding: 12px;
            text-align: left;
            border-bottom: 1px solid #ddd;
        }
        th {
            background: #667eea;
            color: white;
            font-weight: 600;
        }
        tr:hover {
            background: #f5f5f5;
        }
        .footer {
            text-align: center;
            padding: 20px;
            color: #666;
            font-size: 0.9em;
        }
        @media print {
            body { background: white; }
            .section { box-shadow: none; page-break-inside: avoid; }
        }
    </style>
</head>
<body>
    <div class="header">
        <h1>🖥️ VM Inspection Report</h1>
        <div class="subtitle">{{ title }}</div>
        <div class="subtitle">Generated: {{ generated_at }}</div>
    </div>

    <div class="section">
        <h2>📊 Overview</h2>
        <div class="metric">
            <div class="metric-label">Operating System</div>
            <div class="metric-value">{{ os_info.product_name }}</div>
        </div>
        <div class="metric">
            <div class="metric-label">Architecture</div>
            <div class="metric-value">{{ os_info.arch }}</div>
        </div>
        <div class="metric">
            <div class="metric-label">Hostname</div>
            <div class="metric-value">{{ os_info.hostname }}</div>
        </div>
        <div class="metric">
            <div class="metric-label">Packages</div>
            <div class="metric-value">{{ packages.len() }}</div>
        </div>
    </div>

    <div class="section">
        <h2>💾 Disk Usage</h2>
        <div class="chart-container">
            <canvas id="diskChart"></canvas>
        </div>
    </div>

    <div class="section">
        <h2>📦 Package Distribution</h2>
        <div class="chart-container">
            <canvas id="packageChart"></canvas>
        </div>
    </div>

    <div class="section">
        <h2>📁 Filesystems</h2>
        <table>
            <thead>
                <tr>
                    <th>Device</th>
                    <th>Type</th>
                    <th>Size</th>
                    <th>UUID</th>
                </tr>
            </thead>
            <tbody>
                {% for fs in filesystems %}
                <tr>
                    <td>{{ fs.device }}</td>
                    <td>{{ fs.fs_type }}</td>
                    <td>{{ fs.size_gb }} GB</td>
                    <td><code>{{ fs.uuid }}</code></td>
                </tr>
                {% endfor %}
            </tbody>
        </table>
    </div>

    <div class="section">
        <h2>👥 User Accounts</h2>
        <table>
            <thead>
                <tr>
                    <th>Username</th>
                    <th>UID</th>
                    <th>Home Directory</th>
                    <th>Shell</th>
                </tr>
            </thead>
            <tbody>
                {% for user in users %}
                <tr>
                    <td>{{ user.username }}</td>
                    <td>{{ user.uid }}</td>
                    <td>{{ user.home_dir }}</td>
                    <td>{{ user.shell }}</td>
                </tr>
                {% endfor %}
            </tbody>
        </table>
    </div>

    <div class="footer">
        Generated by <strong>GuestCtl v0.3.1</strong> |
        <a href="https://github.com/ssahani/guestkit">GitHub</a>
    </div>

    <script>
        // Disk usage chart
        const diskCtx = document.getElementById('diskChart').getContext('2d');
        new Chart(diskCtx, {
            type: 'doughnut',
            data: {
                labels: {{ chart_data.disk_usage_labels|json }},
                datasets: [{
                    data: {{ chart_data.disk_usage_values|json }},
                    backgroundColor: [
                        '#667eea', '#764ba2', '#f093fb', '#4facfe',
                        '#43e97b', '#fa709a', '#fee140', '#30cfd0'
                    ]
                }]
            },
            options: {
                responsive: true,
                maintainAspectRatio: false,
                plugins: {
                    legend: { position: 'right' },
                    title: {
                        display: true,
                        text: 'Disk Space Distribution'
                    }
                }
            }
        });

        // Package distribution chart
        const pkgCtx = document.getElementById('packageChart').getContext('2d');
        new Chart(pkgCtx, {
            type: 'bar',
            data: {
                labels: {{ chart_data.package_labels|json }},
                datasets: [{
                    label: 'Number of Packages',
                    data: {{ chart_data.package_values|json }},
                    backgroundColor: '#667eea'
                }]
            },
            options: {
                responsive: true,
                maintainAspectRatio: false,
                plugins: {
                    legend: { display: false },
                    title: {
                        display: true,
                        text: 'Top Package Categories'
                    }
                }
            }
        });
    </script>
</body>
</html>
```

**Usage:**

```bash
# Generate HTML report with charts
sudo guestctl inspect vm.qcow2 --export html --output report.html

# Open in browser
xdg-open report.html
```

**Timeline:** Week 1-2 of March

### PDF Export

**Implementation using wkhtmltopdf:**

```rust
// src/cli/exporters/pdf.rs

use std::process::Command;

pub struct PdfExporter;

impl PdfExporter {
    pub fn export(html_path: &str, pdf_path: &str) -> Result<()> {
        // First generate HTML
        let html_report = HtmlReport::generate()?;
        let temp_html = "/tmp/guestctl-report.html";
        std::fs::write(temp_html, html_report)?;

        // Convert HTML to PDF
        Command::new("wkhtmltopdf")
            .arg("--enable-local-file-access")
            .arg("--enable-javascript")
            .arg("--javascript-delay")
            .arg("2000") // Wait for charts to render
            .arg("--print-media-type")
            .arg(temp_html)
            .arg(pdf_path)
            .status()?;

        // Cleanup
        std::fs::remove_file(temp_html)?;

        Ok(())
    }
}
```

**Usage:**

```bash
# Generate PDF report
sudo guestctl inspect vm.qcow2 --export pdf --output report.pdf
```

**Timeline:** Week 2 of March

### Markdown Export with Diagrams

**Implementation with Mermaid:**

```rust
// src/cli/exporters/markdown.rs

use askama::Template;

#[derive(Template)]
#[template(path = "report.md", escape = "none")]
pub struct MarkdownReport<'a> {
    pub title: &'a str,
    pub os_info: &'a OsInfo,
    pub filesystems: &'a [Filesystem],
    pub packages: &'a [Package],
    pub network: &'a NetworkInfo,
}
```

**Template:**

```markdown
<!-- src/cli/templates/report.md -->
# {{ title }} - VM Inspection Report

**Generated:** {{ generated_at }}
**Tool:** GuestCtl v0.3.1

---

## 📊 System Overview

```mermaid
graph TD
    A[VM: {{ os_info.hostname }}] --> B[OS: {{ os_info.product_name }}]
    A --> C[Architecture: {{ os_info.arch }}]
    A --> D[Packages: {{ packages.len() }}]
    B --> E[Init: {{ os_info.init_system }}]
```

## 💾 Operating System

| Property | Value |
|----------|-------|
| **Type** | {{ os_info.os_type }} |
| **Distribution** | {{ os_info.distro }} |
| **Product** | {{ os_info.product_name }} |
| **Version** | {{ os_info.version }} |
| **Architecture** | {{ os_info.arch }} |
| **Hostname** | {{ os_info.hostname }} |
| **Init System** | {{ os_info.init_system }} |
| **Package Format** | {{ os_info.package_format }} |

## 📁 Filesystems

```mermaid
pie title Disk Space Usage
{% for fs in filesystems %}
    "{{ fs.device }}" : {{ fs.size_gb }}
{% endfor %}
```

### Filesystem Details

| Device | Type | Size | Mount Point | UUID |
|--------|------|------|-------------|------|
{% for fs in filesystems -%}
| `{{ fs.device }}` | {{ fs.fs_type }} | {{ fs.size_gb }} GB | {{ fs.mount_point }} | `{{ fs.uuid }}` |
{% endfor %}

## 🌐 Network Configuration

{% if network.interfaces %}
### Interfaces

| Interface | IP Address | MAC Address | Status |
|-----------|------------|-------------|--------|
{% for iface in network.interfaces -%}
| {{ iface.name }} | {{ iface.ip }} | {{ iface.mac }} | {{ iface.status }} |
{% endfor %}
{% endif %}

{% if network.dns_servers %}
### DNS Servers

{% for dns in network.dns_servers -%}
- `{{ dns }}`
{% endfor %}
{% endif %}

## 👥 User Accounts

### Regular Users ({{ users.regular_count }})

| Username | UID | Home Directory | Shell |
|----------|-----|----------------|-------|
{% for user in users.regular -%}
| {{ user.username }} | {{ user.uid }} | {{ user.home_dir }} | {{ user.shell }} |
{% endfor %}

### System Users

Total: **{{ users.system_count }}** system accounts

## 📦 Installed Packages

**Total Packages:** {{ packages.len() }}

### Top 20 Packages

| Package | Version | Architecture |
|---------|---------|--------------|
{% for pkg in packages | slice(end=20) -%}
| {{ pkg.name }} | {{ pkg.version }} | {{ pkg.arch }} |
{% endfor %}

<details>
<summary>View all {{ packages.len() }} packages</summary>

| Package | Version | Architecture |
|---------|---------|--------------|
{% for pkg in packages -%}
| {{ pkg.name }} | {{ pkg.version }} | {{ pkg.arch }} |
{% endfor %}

</details>

## 🔐 Security Configuration

{% if security %}
| Setting | Value |
|---------|-------|
| **SELinux** | {{ security.selinux }} |
| **AppArmor** | {{ security.apparmor }} |
| **Firewall** | {{ security.firewall }} |
| **SSH Root Login** | {{ security.ssh_root_login }} |
| **SSH Password Auth** | {{ security.ssh_password_auth }} |
{% endif %}

---

## 📋 Migration Checklist

Use this checklist for VM migration:

- [ ] Backup original VM
- [ ] Document current configuration
- [ ] Convert disk format (if needed)
- [ ] Update device paths (/dev/sda → /dev/vda)
- [ ] Modify fstab/crypttab
- [ ] Update bootloader configuration
- [ ] Install VirtIO drivers (Windows)
- [ ] Test boot in target environment
- [ ] Verify network configuration
- [ ] Confirm services start correctly

---

**Report Generated by [GuestCtl](https://github.com/ssahani/guestkit) v0.3.1**
```

**Usage:**

```bash
# Generate Markdown report
sudo guestctl inspect vm.qcow2 --export markdown --output report.md

# View in GitHub/GitLab (Mermaid diagrams render automatically)
# Or use markdown viewer
glow report.md
```

**Timeline:** Week 3 of March

### Export Command Enhancements

```rust
// src/cli/commands.rs

#[derive(Parser)]
pub struct ExportArgs {
    /// Input disk image
    image: PathBuf,

    /// Export format
    #[arg(short, long, value_enum)]
    format: ExportFormat,

    /// Output file
    #[arg(short, long)]
    output: PathBuf,

    /// Include charts (HTML/PDF only)
    #[arg(long, default_value = "true")]
    charts: bool,

    /// Include Mermaid diagrams (Markdown only)
    #[arg(long, default_value = "true")]
    diagrams: bool,

    /// Template file (custom template)
    #[arg(long)]
    template: Option<PathBuf>,
}

#[derive(ValueEnum, Clone)]
pub enum ExportFormat {
    Html,
    Pdf,
    Markdown,
    Json,
    Yaml,
    Csv,
}

pub fn export_report(args: ExportArgs) -> Result<()> {
    // Inspect VM
    let inspection = inspect_vm(&args.image)?;

    match args.format {
        ExportFormat::Html => {
            let html = HtmlReport::new(&inspection, args.charts);
            html.generate(&args.output)?;
        }
        ExportFormat::Pdf => {
            // Generate HTML first, then PDF
            let html = HtmlReport::new(&inspection, args.charts);
            let temp_html = "/tmp/guestctl-temp.html";
            html.generate(temp_html)?;
            PdfExporter::export(temp_html, &args.output)?;
        }
        ExportFormat::Markdown => {
            let md = MarkdownReport::new(&inspection, args.diagrams);
            md.generate(&args.output)?;
        }
        ExportFormat::Json => {
            let json = serde_json::to_string_pretty(&inspection)?;
            std::fs::write(&args.output, json)?;
        }
        // ... other formats
    }

    Ok(())
}
```

### Testing Export Formats

```rust
// tests/test_exports.rs

#[test]
fn test_html_export() {
    let args = ExportArgs {
        image: PathBuf::from("test.qcow2"),
        format: ExportFormat::Html,
        output: PathBuf::from("/tmp/test-report.html"),
        charts: true,
        diagrams: false,
        template: None,
    };

    export_report(args).unwrap();

    // Verify HTML file exists and is valid
    assert!(PathBuf::from("/tmp/test-report.html").exists());

    // Parse HTML and verify structure
    let html = std::fs::read_to_string("/tmp/test-report.html").unwrap();
    assert!(html.contains("<canvas id=\"diskChart\">"));
    assert!(html.contains("Chart.js"));
}

#[test]
fn test_markdown_export() {
    // ... similar test for Markdown
}
```

**Timeline:** Week 4 of March (testing)

---

## 3. Testing Improvements

**Goal:** Achieve 85%+ test coverage with comprehensive quality assurance.

### Current Test Coverage

```bash
# Install tarpaulin for coverage
cargo install cargo-tarpaulin

# Run coverage
cargo tarpaulin --out Html --output-dir coverage

# Current coverage: ~40%
# Target coverage: 85%+
```

### Testing Strategy

**Test Pyramid:**
```
        /\
       /  \      10% - E2E Tests (Integration)
      /____\
     /      \    30% - Integration Tests
    /________\
   /          \  60% - Unit Tests
  /____________\
```

### Unit Tests (60% of tests)

**Target:** 1,000+ unit tests

```rust
// src/guestfs/inspect.rs

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_os_type_parsing() {
        assert_eq!(OsType::from_str("linux"), OsType::Linux);
        assert_eq!(OsType::from_str("windows"), OsType::Windows);
        assert_eq!(OsType::from_str("unknown"), OsType::Unknown);
    }

    #[test]
    fn test_distro_detection() {
        let ubuntu = Distro::from_str("ubuntu");
        assert_eq!(ubuntu, Distro::Ubuntu);
        assert_eq!(ubuntu.package_manager(), Some("deb"));
    }

    #[test]
    fn test_version_parsing() {
        let version = Version::new(22, 4);
        assert_eq!(version.major, 22);
        assert_eq!(version.minor, 4);
        assert_eq!(version.to_string(), "22.4");
    }

    #[test]
    fn test_inspect_result_serialization() {
        let result = InspectionResult {
            os_type: OsType::Linux,
            distro: Distro::Ubuntu,
            version: Version::new(22, 4),
            hostname: "test-vm".to_string(),
        };

        let json = serde_json::to_string(&result).unwrap();
        let deserialized: InspectionResult = serde_json::from_str(&json).unwrap();

        assert_eq!(result, deserialized);
    }

    #[test]
    #[should_panic(expected = "Invalid OS type")]
    fn test_invalid_os_type() {
        OsType::from_str("invalid").expect("Invalid OS type");
    }
}
```

**Timeline:** Ongoing throughout February-March

### Integration Tests (30% of tests)

**Target:** 300+ integration tests

```rust
// tests/integration_basic.rs

use guestctl::Guestfs;
use tempfile::TempDir;

#[test]
fn test_full_inspection_workflow() -> Result<()> {
    let tmp = TempDir::new()?;
    let disk_path = tmp.path().join("test.raw");

    // Create test disk
    create_test_disk(&disk_path, "ubuntu", "22.04")?;

    // Inspect disk
    let mut g = Guestfs::new()?;
    g.add_drive_ro(&disk_path)?;
    g.launch()?;

    let roots = g.inspect_os()?;
    assert_eq!(roots.len(), 1);

    let root = &roots[0];
    assert_eq!(g.inspect_get_type(root)?, "linux");
    assert_eq!(g.inspect_get_distro(root)?, "ubuntu");

    g.shutdown()?;
    Ok(())
}

#[test]
fn test_cache_functionality() -> Result<()> {
    // Test caching saves time
    let disk = "test.qcow2";

    // First run (no cache)
    let start = Instant::now();
    inspect_with_cache(disk, true)?;
    let first_duration = start.elapsed();

    // Second run (with cache)
    let start = Instant::now();
    inspect_with_cache(disk, true)?;
    let second_duration = start.elapsed();

    // Cache should be 10x+ faster
    assert!(second_duration < first_duration / 10);
    Ok(())
}
```

**Timeline:** Week 1-3 of March

### End-to-End Tests (10% of tests)

**Target:** 100+ E2E tests

```rust
// tests/e2e/cli_tests.rs

use assert_cmd::Command;
use predicates::prelude::*;

#[test]
fn test_inspect_command() {
    let mut cmd = Command::cargo_bin("guestctl").unwrap();

    cmd.arg("inspect")
        .arg("test.qcow2")
        .assert()
        .success()
        .stdout(predicate::str::contains("Operating Systems"))
        .stdout(predicate::str::contains("Ubuntu"));
}

#[test]
fn test_export_html() {
    let mut cmd = Command::cargo_bin("guestctl").unwrap();

    cmd.arg("inspect")
        .arg("test.qcow2")
        .arg("--export")
        .arg("html")
        .arg("--output")
        .arg("/tmp/report.html")
        .assert()
        .success();

    // Verify HTML file exists
    assert!(std::path::Path::new("/tmp/report.html").exists());
}

#[test]
fn test_json_output() {
    let mut cmd = Command::cargo_bin("guestctl").unwrap();

    cmd.arg("inspect")
        .arg("test.qcow2")
        .arg("--output")
        .arg("json")
        .assert()
        .success()
        .stdout(predicate::str::is_json());
}
```

**Timeline:** Week 3-4 of March

### Property-Based Testing

**Using proptest:**

```rust
// tests/property_tests.rs

use proptest::prelude::*;

proptest! {
    #[test]
    fn test_version_parsing_never_panics(major in 0u32..100, minor in 0u32..100) {
        let version = Version::new(major, minor);
        let _ = version.to_string(); // Should never panic
    }

    #[test]
    fn test_device_path_validation(path in "[a-z/]+") {
        let is_valid = is_valid_device_path(&path);
        // Always returns a boolean, never panics
        assert!(is_valid || !is_valid);
    }

    #[test]
    fn test_cache_key_generation(s in "\\PC*") {
        let key = generate_cache_key(&s);
        // Key should be valid hex string
        assert!(key.chars().all(|c| c.is_ascii_hexdigit()));
    }
}
```

**Timeline:** Week 2 of March

### Fuzzing

**Using cargo-fuzz:**

```rust
// fuzz/fuzz_targets/partition_parser.rs

#![no_main]
use libfuzzer_sys::fuzz_target;
use guestctl::disk::PartitionTable;

fuzz_target!(|data: &[u8]| {
    // Should never panic on any input
    let _ = PartitionTable::parse(data);
});
```

```bash
# Run fuzzing
cargo install cargo-fuzz
cargo fuzz run partition_parser

# Run for 1 hour
cargo fuzz run partition_parser -- -max_total_time=3600
```

**Timeline:** Week 3 of March

### Coverage Reporting

```bash
# Generate coverage report
cargo tarpaulin \
    --out Html \
    --output-dir coverage \
    --exclude-files 'tests/*' 'benches/*' \
    --ignore-panics \
    --timeout 300

# Upload to codecov
bash <(curl -s https://codecov.io/bash)
```

**Add to CI:**

```yaml
# .github/workflows/coverage.yml

name: Coverage

on: [push, pull_request]

jobs:
  coverage:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v3

      - name: Install Rust
        uses: actions-rs/toolchain@v1
        with:
          toolchain: stable

      - name: Install tarpaulin
        run: cargo install cargo-tarpaulin

      - name: Generate coverage
        run: cargo tarpaulin --out Xml

      - name: Upload to codecov
        uses: codecov/codecov-action@v3
```

**Timeline:** Week 4 of March

### Test Documentation

```rust
/// Tests that OS detection works correctly for Ubuntu
///
/// # Test Setup
/// Creates a mock Ubuntu 22.04 disk image
///
/// # Assertions
/// - OS type is detected as "linux"
/// - Distribution is "ubuntu"
/// - Version is 22.4
///
/// # Example
/// ```
/// let result = detect_ubuntu()?;
/// assert_eq!(result.distro, "ubuntu");
/// ```
#[test]
fn test_ubuntu_detection() {
    // ... test implementation
}
```

**Timeline:** Ongoing

### Implementation Timeline

**Week 1 (Mar 3-9):**
- [ ] Add 200+ unit tests
- [ ] Property-based testing setup
- [ ] Coverage baseline (current)

**Week 2 (Mar 10-16):**
- [ ] Add 100+ integration tests
- [ ] Fuzzing setup
- [ ] Coverage: 60%+

**Week 3 (Mar 17-23):**
- [ ] Add 50+ E2E tests
- [ ] CI/CD coverage integration
- [ ] Coverage: 75%+

**Week 4 (Mar 24-31):**
- [ ] Additional tests to reach 85%+
- [ ] Test documentation
- [ ] Final validation

---

## Success Metrics & Validation

### Performance Metrics

| Metric | Baseline | Target | Validation |
|--------|----------|--------|------------|
| Appliance launch | 2.5s | <2.0s | cargo bench |
| OS inspection | 500ms | <400ms | cargo bench |
| Cache lookup | 500ms | <100ms | cargo bench |
| Package listing | 3.5s | <2.8s | cargo bench |
| Memory usage | 512MB | <350MB | valgrind |

### Export Metrics

| Feature | Target | Validation |
|---------|--------|------------|
| HTML export | ✅ Working | Manual test |
| PDF export | ✅ Working | Manual test |
| Markdown export | ✅ Working | Manual test |
| Chart rendering | ✅ Interactive | Browser test |
| Mermaid diagrams | ✅ Rendering | GitHub preview |

### Testing Metrics

| Metric | Target | Validation |
|--------|--------|------------|
| Test coverage | 85%+ | tarpaulin |
| Unit tests | 1,000+ | cargo test --lib |
| Integration tests | 300+ | cargo test --test '*' |
| E2E tests | 100+ | cargo test --test e2e |
| CI passing | ✅ Green | GitHub Actions |

---

## Deliverables

### Week 4 (Feb)
- ✅ Binary cache implementation
- ✅ Parallel processing
- ✅ 10-15% performance improvement

### Week 8 (Mar)
- ✅ HTML export with charts
- ✅ PDF export
- ✅ Markdown export with diagrams

### Week 12 (End of Q1)
- ✅ 20%+ performance improvement validated
- ✅ 85%+ test coverage achieved
- ✅ All export formats working
- ✅ Comprehensive test suite
- ✅ Documentation complete

---

## Risk Mitigation

**Performance Risks:**
- Profiling identifies no major bottlenecks → Incremental optimizations
- Binary cache corruption → Validation and fallback to JSON

**Export Risks:**
- wkhtmltopdf not available → Fallback to HTML only
- Chart.js CDN issues → Embed library locally

**Testing Risks:**
- Hard to reach 85% → Focus on critical paths first
- Flaky tests → Proper test isolation and mocking

---

## Next Steps

1. **Review and approve** this implementation plan
2. **Assign tasks** to development milestones
3. **Set up tracking** in GitHub Projects
4. **Begin Week 1** implementation (Feb 3)

**Ready to start? Let me know if you want to proceed with any specific area first!**
