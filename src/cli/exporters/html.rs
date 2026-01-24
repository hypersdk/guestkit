// SPDX-License-Identifier: LGPL-3.0-or-later
//! HTML report generation

use crate::cli::formatters::InspectionReport;
use anyhow::Result;
use askama::Template;

/// Package information for template
#[derive(Debug, Clone)]
pub struct PackageInfo {
    pub name: String,
    pub version: String,
}

/// Service information for template
#[derive(Debug, Clone)]
pub struct ServiceInfo {
    pub name: String,
    pub state: String,
}

/// User information for template
#[derive(Debug, Clone)]
pub struct UserInfo {
    pub username: String,
    pub uid: String,
    pub home: String,
}

/// Network interface information for template
#[derive(Debug, Clone)]
pub struct NetworkInfo {
    pub name: String,
    pub ip: String,
    pub mac: String,
}

/// HTML inspection report template
#[derive(Template)]
#[template(path = "inspection_report.html")]
pub struct InspectionReportTemplate {
    pub vm_name: String,
    pub timestamp: String,
    pub os_type: String,
    pub distro: String,
    pub version: String,
    pub arch: String,
    pub os_section: bool,
    pub hostname: String,
    pub product_name: String,
    pub package_format: String,
    pub package_manager: String,
    pub packages: Vec<PackageInfo>,
    pub packages_count: usize,
    pub services: Vec<ServiceInfo>,
    pub services_count: usize,
    pub users: Vec<UserInfo>,
    pub users_count: usize,
    pub network: Vec<NetworkInfo>,
}

/// Generate HTML report from inspection data
pub fn generate_html_report(report: &InspectionReport) -> Result<String> {
    let vm_name = report
        .os
        .hostname
        .clone()
        .unwrap_or_else(|| "Unknown".to_string());
    let timestamp = chrono::Local::now().format("%Y-%m-%d %H:%M:%S").to_string();

    let os_type = report
        .os
        .os_type
        .clone()
        .unwrap_or_else(|| "Unknown".to_string());
    let distro = report
        .os
        .distribution
        .clone()
        .unwrap_or_else(|| "Unknown".to_string());

    let version = if let Some(ref v) = report.os.version {
        format!("{}.{}", v.major, v.minor)
    } else {
        "Unknown".to_string()
    };

    let arch = report
        .os
        .architecture
        .clone()
        .unwrap_or_else(|| "Unknown".to_string());
    let hostname = report
        .os
        .hostname
        .clone()
        .unwrap_or_else(|| "Unknown".to_string());
    let product_name = report
        .os
        .product_name
        .clone()
        .unwrap_or_else(|| "Unknown".to_string());
    let package_format = report
        .os
        .package_format
        .clone()
        .unwrap_or_else(|| "Unknown".to_string());
    let package_manager = report
        .os
        .package_manager
        .clone()
        .unwrap_or_else(|| "Unknown".to_string());

    // Convert packages
    let packages: Vec<PackageInfo> = if let Some(ref pkg_section) = report.packages {
        // Take kernels for HTML display
        pkg_section
            .kernels
            .iter()
            .take(100)
            .map(|k| PackageInfo {
                name: k.clone(),
                version: format!("{} package", pkg_section.format),
            })
            .collect()
    } else {
        Vec::new()
    };

    // Convert services
    let services: Vec<ServiceInfo> = if let Some(ref svc_section) = report.services {
        svc_section
            .enabled_services
            .iter()
            .map(|s| ServiceInfo {
                name: s.name.clone(),
                state: s.state.clone(),
            })
            .collect()
    } else {
        Vec::new()
    };

    // Convert users
    let users: Vec<UserInfo> = if let Some(ref user_section) = report.users {
        user_section
            .regular_users
            .iter()
            .map(|u| UserInfo {
                username: u.username.clone(),
                uid: u.uid.clone(),
                home: u.home.clone(),
            })
            .collect()
    } else {
        Vec::new()
    };

    // Convert network interfaces
    let network: Vec<NetworkInfo> = if let Some(ref net_section) = report.network {
        if let Some(ref interfaces) = net_section.interfaces {
            interfaces
                .iter()
                .map(|i| NetworkInfo {
                    name: i.name.clone(),
                    ip: i.ip_address.join(", "),
                    mac: i.mac_address.clone(),
                })
                .collect()
        } else {
            Vec::new()
        }
    } else {
        Vec::new()
    };

    let packages_count = packages.len();
    let services_count = services.len();
    let users_count = users.len();

    let template = InspectionReportTemplate {
        vm_name,
        timestamp,
        os_type,
        distro,
        version,
        arch,
        os_section: true,
        hostname,
        product_name,
        package_format,
        package_manager,
        packages,
        packages_count,
        services,
        services_count,
        users,
        users_count,
        network,
    };

    Ok(template.render()?)
}
