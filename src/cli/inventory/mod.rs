// SPDX-License-Identifier: LGPL-3.0-or-later
//! Software Bill of Materials (SBOM) generation module

pub mod sbom;
pub mod formats;
pub mod cve;
pub mod licenses;

use anyhow::{Context, Result};
use chrono::Utc;
use guestkit::Guestfs;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::Path;


/// SBOM output format
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SbomFormat {
    Spdx,
    CycloneDx,
    Json,
    Csv,
}

impl SbomFormat {
    pub fn from_str(s: &str) -> Result<Self> {
        match s.to_lowercase().as_str() {
            "spdx" => Ok(Self::Spdx),
            "cyclonedx" => Ok(Self::CycloneDx),
            "json" => Ok(Self::Json),
            "csv" => Ok(Self::Csv),
            _ => anyhow::bail!("Unknown format: {}", s),
        }
    }
}

/// Package information for SBOM
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PackageInfo {
    pub name: String,
    pub version: String,
    pub package_type: String,
    pub license: Option<String>,
    pub size: Option<i64>,
    pub installed_date: Option<String>,
    pub files: Vec<String>,
    pub dependencies: Vec<String>,
    pub vulnerabilities: Vec<VulnerabilityInfo>,
    pub checksum: Option<String>,
}

/// Vulnerability information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VulnerabilityInfo {
    pub cve: String,
    pub severity: String,
    pub score: Option<f64>,
    pub description: String,
    pub fixed_version: Option<String>,
}

/// Complete inventory data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Inventory {
    pub image_path: String,
    pub scanned_at: String,
    pub os_name: String,
    pub os_version: String,
    pub architecture: String,
    pub packages: Vec<PackageInfo>,
    pub statistics: InventoryStatistics,
}

/// Inventory statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InventoryStatistics {
    pub total_packages: usize,
    pub total_size: i64,
    pub vulnerabilities: HashMap<String, usize>,
    pub licenses: HashMap<String, usize>,
}

/// Generate inventory from disk image
pub fn generate_inventory<P: AsRef<Path>>(
    image_path: P,
    include_licenses: bool,
    include_cves: bool,
    include_files: bool,
) -> Result<Inventory> {
    let image_path_str = image_path.as_ref().display().to_string();

    // Initialize guestfs
    let mut g = Guestfs::new()?;
    g.add_drive_opts(&image_path, true, None)?;
    g.launch()?;

    // Inspect OS
    let roots = g.inspect_os()?;
    if roots.is_empty() {
        anyhow::bail!("No operating systems found in disk image");
    }

    let root = &roots[0];

    // Mount filesystems
    let mountpoints = g.inspect_get_mountpoints(root)?;
    for (mp, dev) in mountpoints {
        let _ = g.mount(&dev, &mp);
    }

    // Get OS information
    let os_name = g.inspect_get_product_name(root)
        .unwrap_or_else(|_| "Unknown".to_string());
    let os_version = g.inspect_get_product_variant(root)
        .unwrap_or_else(|_| "Unknown".to_string());
    let architecture = g.inspect_get_arch(root)
        .unwrap_or_else(|_| "Unknown".to_string());

    // Scan packages
    let packages = scan_packages(&mut g, root, include_licenses, include_cves, include_files)?;

    // Calculate statistics
    let statistics = calculate_statistics(&packages);

    let inventory = Inventory {
        image_path: image_path_str,
        scanned_at: Utc::now().to_rfc3339(),
        os_name,
        os_version,
        architecture,
        packages,
        statistics,
    };

    // Shutdown guestfs
    g.shutdown()?;

    Ok(inventory)
}

/// Scan packages from the guest OS
fn scan_packages(
    g: &mut Guestfs,
    root: &str,
    include_licenses: bool,
    include_cves: bool,
    include_files: bool,
) -> Result<Vec<PackageInfo>> {
    let package_format = g.inspect_get_package_format(root)?;

    match package_format.as_str() {
        "deb" => scan_deb_packages(g, root, include_licenses, include_cves, include_files),
        "rpm" => scan_rpm_packages(g, root, include_licenses, include_cves, include_files),
        _ => anyhow::bail!("Unsupported package format: {}", package_format),
    }
}

/// Scan Debian/Ubuntu packages
fn scan_deb_packages(
    g: &mut Guestfs,
    root: &str,
    include_licenses: bool,
    include_cves: bool,
    _include_files: bool,
) -> Result<Vec<PackageInfo>> {
    let applications = g.inspect_list_applications2(root)?;
    let mut packages = Vec::new();

    for (name, version, _release) in applications {
        let mut pkg = PackageInfo {
            name: name.clone(),
            version: version.clone(),
            package_type: "deb".to_string(),
            license: None,
            size: None,
            installed_date: None,
            files: Vec::new(),
            dependencies: Vec::new(),
            vulnerabilities: Vec::new(),
            checksum: None,
        };

        // Add license information if requested
        if include_licenses {
            pkg.license = licenses::detect_license(&name, "deb");
        }

        // Add CVE information if requested
        if include_cves {
            pkg.vulnerabilities = cve::lookup_cves(&name, &version)?;
        }

        packages.push(pkg);
    }

    Ok(packages)
}

/// Scan RPM-based packages
fn scan_rpm_packages(
    g: &mut Guestfs,
    root: &str,
    include_licenses: bool,
    include_cves: bool,
    _include_files: bool,
) -> Result<Vec<PackageInfo>> {
    let applications = g.inspect_list_applications2(root)?;
    let mut packages = Vec::new();

    for (name, version, _release) in applications {
        let mut pkg = PackageInfo {
            name: name.clone(),
            version: version.clone(),
            package_type: "rpm".to_string(),
            license: None,
            size: None,
            installed_date: None,
            files: Vec::new(),
            dependencies: Vec::new(),
            vulnerabilities: Vec::new(),
            checksum: None,
        };

        // Add license information if requested
        if include_licenses {
            pkg.license = licenses::detect_license(&name, "rpm");
        }

        // Add CVE information if requested
        if include_cves {
            pkg.vulnerabilities = cve::lookup_cves(&name, &version)?;
        }

        packages.push(pkg);
    }

    Ok(packages)
}

/// Calculate inventory statistics
fn calculate_statistics(packages: &[PackageInfo]) -> InventoryStatistics {
    let mut total_size = 0i64;
    let mut vulnerabilities: HashMap<String, usize> = HashMap::new();
    let mut licenses: HashMap<String, usize> = HashMap::new();

    for pkg in packages {
        if let Some(size) = pkg.size {
            total_size += size;
        }

        for vuln in &pkg.vulnerabilities {
            *vulnerabilities.entry(vuln.severity.clone()).or_insert(0) += 1;
        }

        if let Some(license) = &pkg.license {
            *licenses.entry(license.clone()).or_insert(0) += 1;
        }
    }

    InventoryStatistics {
        total_packages: packages.len(),
        total_size,
        vulnerabilities,
        licenses,
    }
}

/// Export inventory to specified format
pub fn export_inventory(
    inventory: &Inventory,
    format: SbomFormat,
    output: Option<&str>,
) -> Result<()> {
    let content = match format {
        SbomFormat::Spdx => {
            let doc = formats::to_spdx(inventory)?;
            serde_json::to_string_pretty(&doc)?
        }
        SbomFormat::CycloneDx => {
            let bom = formats::to_cyclonedx(inventory)?;
            serde_json::to_string_pretty(&bom)?
        }
        SbomFormat::Json => {
            serde_json::to_string_pretty(inventory)?
        }
        SbomFormat::Csv => {
            formats::to_csv(inventory)?
        }
    };

    if let Some(path) = output {
        std::fs::write(path, content)
            .context(format!("Failed to write to {}", path))?;
        println!("✅ SBOM written to: {}", path);
    } else {
        println!("{}", content);
    }

    Ok(())
}
