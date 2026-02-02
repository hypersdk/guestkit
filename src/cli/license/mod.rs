// SPDX-License-Identifier: LGPL-3.0-or-later
//! License compliance checking module

pub mod scanner;
pub mod database;
pub mod analyzer;
pub mod reporter;

use analyzer::LicenseAnalyzer;
use anyhow::Result;
use guestkit::Guestfs;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::Path;

/// License information for a package
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PackageLicense {
    pub package_name: String,
    pub version: String,
    pub license: String,
    pub license_type: LicenseType,
    pub risk_level: RiskLevel,
    pub compatible_with: Vec<String>,
    pub incompatible_with: Vec<String>,
}

/// License type classification
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum LicenseType {
    Permissive,
    Copyleft,
    StrongCopyleft,
    Proprietary,
    PublicDomain,
    Unknown,
}

/// Risk level for license compliance
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum RiskLevel {
    Low,
    Medium,
    High,
    Critical,
}

impl RiskLevel {
    pub fn emoji(&self) -> &str {
        match self {
            Self::Low => "🟢",
            Self::Medium => "🟡",
            Self::High => "🟠",
            Self::Critical => "🔴",
        }
    }
}

/// License compliance report
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LicenseReport {
    pub image_path: String,
    pub scanned_at: String,
    pub total_packages: usize,
    pub packages: Vec<PackageLicense>,
    pub license_summary: HashMap<String, usize>,
    pub risk_summary: HashMap<RiskLevel, usize>,
    pub violations: Vec<LicenseViolation>,
    pub statistics: LicenseStatistics,
}

/// License violation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LicenseViolation {
    pub package_name: String,
    pub violation_type: ViolationType,
    pub description: String,
    pub risk_level: RiskLevel,
    pub remediation: String,
}

/// Type of license violation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ViolationType {
    ProhibitedLicense,
    IncompatibleLicenses,
    MissingLicense,
    CommercialRestriction,
}

/// License statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LicenseStatistics {
    pub permissive_licenses: usize,
    pub copyleft_licenses: usize,
    pub strong_copyleft_licenses: usize,
    pub proprietary_licenses: usize,
    pub unknown_licenses: usize,
    pub compliance_score: f64,
}

/// Scan disk image for license compliance
pub fn scan_licenses<P: AsRef<Path>>(
    image_path: P,
    prohibited: &[String],
    verbose: bool,
) -> Result<LicenseReport> {
    let image_path_str = image_path.as_ref().display().to_string();

    if verbose {
        println!("📋 Scanning licenses in: {}", image_path_str);
    }

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

    // Scan packages
    let packages = scanner::scan_package_licenses(&mut g, root, verbose)?;

    // Analyze licenses
    let analyzer = LicenseAnalyzer::new();
    let violations = analyzer.find_violations(&packages, prohibited);

    // Calculate statistics
    let statistics = calculate_statistics(&packages);
    let license_summary = calculate_license_summary(&packages);
    let risk_summary = calculate_risk_summary(&packages);

    // Shutdown guestfs
    g.shutdown()?;

    Ok(LicenseReport {
        image_path: image_path_str,
        scanned_at: chrono::Utc::now().to_rfc3339(),
        total_packages: packages.len(),
        packages,
        license_summary,
        risk_summary,
        violations,
        statistics,
    })
}

fn calculate_statistics(packages: &[PackageLicense]) -> LicenseStatistics {
    let permissive = packages.iter().filter(|p| p.license_type == LicenseType::Permissive).count();
    let copyleft = packages.iter().filter(|p| p.license_type == LicenseType::Copyleft).count();
    let strong_copyleft = packages.iter().filter(|p| p.license_type == LicenseType::StrongCopyleft).count();
    let proprietary = packages.iter().filter(|p| p.license_type == LicenseType::Proprietary).count();
    let unknown = packages.iter().filter(|p| p.license_type == LicenseType::Unknown).count();

    // Compliance score based on license clarity
    let known_licenses = packages.len() - unknown;
    let compliance_score = if packages.is_empty() {
        0.0
    } else {
        (known_licenses as f64 / packages.len() as f64) * 100.0
    };

    LicenseStatistics {
        permissive_licenses: permissive,
        copyleft_licenses: copyleft,
        strong_copyleft_licenses: strong_copyleft,
        proprietary_licenses: proprietary,
        unknown_licenses: unknown,
        compliance_score,
    }
}

fn calculate_license_summary(packages: &[PackageLicense]) -> HashMap<String, usize> {
    let mut summary = HashMap::new();
    for pkg in packages {
        *summary.entry(pkg.license.clone()).or_insert(0) += 1;
    }
    summary
}

fn calculate_risk_summary(packages: &[PackageLicense]) -> HashMap<RiskLevel, usize> {
    let mut summary = HashMap::new();
    for pkg in packages {
        *summary.entry(pkg.risk_level.clone()).or_insert(0) += 1;
    }
    summary
}

/// Generate attribution notices
pub fn generate_attribution(report: &LicenseReport) -> String {
    let mut output = String::new();

    output.push_str("THIRD-PARTY SOFTWARE NOTICES AND INFORMATION\n");
    output.push_str("===========================================\n\n");
    output.push_str(&format!("This software incorporates components from the projects listed below.\n"));
    output.push_str(&format!("Generated: {}\n\n", report.scanned_at));

    // Group by license
    let mut by_license: HashMap<String, Vec<&PackageLicense>> = HashMap::new();
    for pkg in &report.packages {
        by_license.entry(pkg.license.clone()).or_default().push(pkg);
    }

    for (license, packages) in by_license.iter() {
        output.push_str(&format!("\n{} ({} packages)\n", license, packages.len()));
        output.push_str(&format!("{}\n", "=".repeat(license.len() + 20)));

        for pkg in packages {
            output.push_str(&format!("  - {} {}\n", pkg.package_name, pkg.version));
        }
    }

    output
}
