// SPDX-License-Identifier: LGPL-3.0-or-later
//! CVE vulnerability lookup

use super::VulnerabilityInfo;
use anyhow::Result;
use std::collections::HashMap;
use once_cell::sync::Lazy;

/// Known CVEs for demonstration (in production, this would query a CVE database)
static KNOWN_CVES: Lazy<HashMap<&'static str, Vec<(&'static str, &'static str, f64)>>> = Lazy::new(|| {
    let mut m: HashMap<&'static str, Vec<(&'static str, &'static str, f64)>> = HashMap::new();

    // Example CVEs (package_name -> [(cve_id, severity, score)])
    m.insert("openssl", vec![
        ("CVE-2024-0727", "high", 7.5),
        ("CVE-2023-6129", "medium", 5.3),
    ]);

    m.insert("nginx", vec![
        ("CVE-2023-44487", "high", 7.5),
    ]);

    m.insert("curl", vec![
        ("CVE-2023-46218", "medium", 6.5),
    ]);

    m.insert("python3", vec![
        ("CVE-2023-40217", "medium", 5.3),
    ]);

    m
});

/// Lookup CVEs for a package
pub fn lookup_cves(package_name: &str, package_version: &str) -> Result<Vec<VulnerabilityInfo>> {
    let mut vulnerabilities = Vec::new();

    // Check if we have known CVEs for this package
    if let Some(cves) = KNOWN_CVES.get(package_name) {
        for (cve_id, severity, score) in cves {
            vulnerabilities.push(VulnerabilityInfo {
                cve: cve_id.to_string(),
                severity: severity.to_string(),
                score: Some(*score),
                description: format!(
                    "Vulnerability in {} {}",
                    package_name, package_version
                ),
                fixed_version: None,
            });
        }
    }

    Ok(vulnerabilities)
}

/// Filter vulnerabilities by severity
#[allow(dead_code)]
pub fn filter_by_severity(
    vulnerabilities: &[VulnerabilityInfo],
    min_severity: &str,
) -> Vec<VulnerabilityInfo> {
    let min_rank = severity_rank(min_severity);

    vulnerabilities
        .iter()
        .filter(|v| severity_rank(&v.severity) >= min_rank)
        .cloned()
        .collect()
}

#[allow(dead_code)]
fn severity_rank(severity: &str) -> u8 {
    match severity.to_lowercase().as_str() {
        "critical" => 4,
        "high" => 3,
        "medium" => 2,
        "low" => 1,
        _ => 0,
    }
}
