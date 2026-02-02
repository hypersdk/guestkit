// SPDX-License-Identifier: LGPL-3.0-or-later
//! License detection and mapping

use std::collections::HashMap;
use once_cell::sync::Lazy;

/// Common license mappings for well-known packages
static LICENSE_MAP: Lazy<HashMap<&'static str, &'static str>> = Lazy::new(|| {
    let mut m = HashMap::new();

    // Common packages and their licenses
    m.insert("nginx", "BSD-2-Clause");
    m.insert("apache2", "Apache-2.0");
    m.insert("httpd", "Apache-2.0");
    m.insert("openssl", "Apache-2.0");
    m.insert("libssl", "Apache-2.0");
    m.insert("python3", "PSF-2.0");
    m.insert("python2", "PSF-2.0");
    m.insert("perl", "Artistic-2.0");
    m.insert("bash", "GPL-3.0-or-later");
    m.insert("coreutils", "GPL-3.0-or-later");
    m.insert("gcc", "GPL-3.0-or-later");
    m.insert("glibc", "LGPL-2.1-or-later");
    m.insert("libc6", "LGPL-2.1-or-later");
    m.insert("zlib", "Zlib");
    m.insert("curl", "MIT");
    m.insert("git", "GPL-2.0-only");
    m.insert("nodejs", "MIT");
    m.insert("npm", "Artistic-2.0");
    m.insert("redis", "BSD-3-Clause");
    m.insert("postgresql", "PostgreSQL");
    m.insert("mysql", "GPL-2.0-only");
    m.insert("mariadb", "GPL-2.0-only");
    m.insert("vim", "Vim");
    m.insert("emacs", "GPL-3.0-or-later");
    m.insert("systemd", "LGPL-2.1-or-later");
    m.insert("openssh", "BSD-2-Clause");
    m.insert("sqlite3", "Public-Domain");

    m
});

/// Detect license for a package
pub fn detect_license(package_name: &str, _package_type: &str) -> Option<String> {
    // Try exact match first
    if let Some(license) = LICENSE_MAP.get(package_name) {
        return Some(license.to_string());
    }

    // Try prefix match for libraries
    for (key, license) in LICENSE_MAP.iter() {
        if package_name.starts_with(key) {
            return Some(license.to_string());
        }
    }

    None
}

/// Check if a license is GPL-family
#[allow(dead_code)]
pub fn is_gpl_license(license: &str) -> bool {
    license.starts_with("GPL") || license.starts_with("AGPL") || license.starts_with("LGPL")
}

/// Check if a license is permissive
#[allow(dead_code)]
pub fn is_permissive_license(license: &str) -> bool {
    matches!(
        license,
        "MIT" | "BSD-2-Clause" | "BSD-3-Clause" | "Apache-2.0" | "ISC" | "Zlib"
    )
}

/// Get license category
#[allow(dead_code)]
pub fn license_category(license: &str) -> &'static str {
    if is_permissive_license(license) {
        "Permissive"
    } else if is_gpl_license(license) {
        "Copyleft"
    } else if license == "Public-Domain" {
        "Public Domain"
    } else {
        "Other"
    }
}
