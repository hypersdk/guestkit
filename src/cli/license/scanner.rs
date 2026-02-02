// SPDX-License-Identifier: LGPL-3.0-or-later
//! License scanner for packages

use super::{PackageLicense, database::LICENSE_DB};
use crate::cli::inventory::licenses;
use anyhow::Result;
use guestkit::Guestfs;

/// Scan package licenses from disk image
pub fn scan_package_licenses(
    g: &mut Guestfs,
    root: &str,
    verbose: bool,
) -> Result<Vec<PackageLicense>> {
    let applications = g.inspect_list_applications2(root)?;
    let mut packages = Vec::new();

    for (name, version, _release) in applications {
        if verbose && packages.len() % 50 == 0 {
            println!("  Scanned {} packages...", packages.len());
        }

        // Get license from detection
        let license_str = licenses::detect_license(&name, "")
            .unwrap_or_else(|| "Unknown".to_string());

        // Get license info from database
        let license_type = LICENSE_DB.get_type(&license_str);
        let risk_level = LICENSE_DB.get_risk_level(&license_str);

        let (compatible_with, incompatible_with) = if let Some(info) = LICENSE_DB.get(&license_str) {
            (info.compatible_with.clone(), info.incompatible_with.clone())
        } else {
            (vec![], vec![])
        };

        packages.push(PackageLicense {
            package_name: name,
            version,
            license: license_str,
            license_type,
            risk_level,
            compatible_with,
            incompatible_with,
        });
    }

    if verbose {
        println!("  Total packages scanned: {}", packages.len());
    }

    Ok(packages)
}
