// SPDX-License-Identifier: LGPL-3.0-or-later
//! SBOM generation utilities

use super::Inventory;

/// Generate summary statistics for SBOM
pub fn generate_summary(inventory: &Inventory) -> String {
    let mut summary = String::new();

    summary.push_str(&format!("📦 Software Bill of Materials (SBOM)\n"));
    summary.push_str(&format!("=====================================\n\n"));
    summary.push_str(&format!("Image: {}\n", inventory.image_path));
    summary.push_str(&format!("OS: {} {}\n", inventory.os_name, inventory.os_version));
    summary.push_str(&format!("Architecture: {}\n", inventory.architecture));
    summary.push_str(&format!("Scanned: {}\n\n", inventory.scanned_at));

    summary.push_str(&format!("📊 Statistics\n"));
    summary.push_str(&format!("-------------\n"));
    summary.push_str(&format!("Total Packages: {}\n", inventory.statistics.total_packages));
    summary.push_str(&format!("Total Size: {}\n\n", format_size(inventory.statistics.total_size)));

    if !inventory.statistics.vulnerabilities.is_empty() {
        summary.push_str(&format!("⚠️  Vulnerabilities\n"));
        summary.push_str(&format!("------------------\n"));
        for (severity, count) in &inventory.statistics.vulnerabilities {
            let emoji = match severity.as_str() {
                "critical" => "🔴",
                "high" => "🟠",
                "medium" => "🟡",
                "low" => "🟢",
                _ => "⚪",
            };
            summary.push_str(&format!("{} {}: {}\n", emoji, severity, count));
        }
        summary.push('\n');
    }

    if !inventory.statistics.licenses.is_empty() {
        summary.push_str(&format!("⚖️  Licenses (Top 10)\n"));
        summary.push_str(&format!("--------------------\n"));
        let mut licenses: Vec<_> = inventory.statistics.licenses.iter().collect();
        licenses.sort_by(|a, b| b.1.cmp(a.1));
        for (license, count) in licenses.iter().take(10) {
            summary.push_str(&format!("{}: {}\n", license, count));
        }
    }

    summary
}

fn format_size(bytes: i64) -> String {
    const KB: i64 = 1024;
    const MB: i64 = KB * 1024;
    const GB: i64 = MB * 1024;

    if bytes >= GB {
        format!("{:.2} GB", bytes as f64 / GB as f64)
    } else if bytes >= MB {
        format!("{:.2} MB", bytes as f64 / MB as f64)
    } else if bytes >= KB {
        format!("{:.2} KB", bytes as f64 / KB as f64)
    } else {
        format!("{} bytes", bytes)
    }
}
