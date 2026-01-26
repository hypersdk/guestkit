// SPDX-License-Identifier: LGPL-3.0-or-later
//! HTML report generation with Chart.js visualizations

use crate::cli::formatters::InspectionReport;
use anyhow::Result;
use guestctl::export::{HtmlExporter, HtmlExportOptions};
use guestctl::export::html::{InspectionData, FilesystemInfo, PackageInfo, UserInfo, NetworkInterface};
use tempfile::NamedTempFile;

/// Generate HTML report from inspection data using Chart.js
pub fn generate_html_report(report: &InspectionReport) -> Result<String> {
    // Convert InspectionReport to InspectionData for the HTML exporter
    let data = convert_to_inspection_data(report);

    // Create HTML exporter with custom options
    let exporter = HtmlExporter::with_options(HtmlExportOptions {
        include_charts: true,
        include_styles: true,
        dark_theme: false,
        include_toc: true,
        responsive: true,
    });

    // Generate HTML to a temporary file
    let temp_file = NamedTempFile::new()?;
    exporter.generate(temp_file.path(), &data)?;

    // Read the generated HTML
    let html = std::fs::read_to_string(temp_file.path())?;

    Ok(html)
}

/// Convert InspectionReport to InspectionData
fn convert_to_inspection_data(report: &InspectionReport) -> InspectionData {
    let hostname = report.os.hostname.clone().unwrap_or_else(|| "Unknown".to_string());
    let os_type = report.os.os_type.clone().unwrap_or_else(|| "Unknown".to_string());
    let distribution = report.os.distribution.clone().unwrap_or_else(|| "Unknown".to_string());
    let version = if let Some(ref v) = report.os.version {
        format!("{}.{}", v.major, v.minor)
    } else {
        "Unknown".to_string()
    };
    let architecture = report.os.architecture.clone().unwrap_or_else(|| "Unknown".to_string());
    let product_name = report.os.product_name.clone().unwrap_or_else(|| "Unknown".to_string());
    let package_format = report.os.package_format.clone().unwrap_or_else(|| "Unknown".to_string());
    let package_manager = report.os.package_manager.clone().unwrap_or_else(|| "Unknown".to_string());

    // Convert filesystems from storage/fstab_mounts
    let filesystems = if let Some(ref storage_section) = report.storage {
        if let Some(ref mounts) = storage_section.fstab_mounts {
            mounts.iter().map(|fs| {
                FilesystemInfo {
                    device: fs.device.clone(),
                    mountpoint: fs.mountpoint.clone(),
                    fstype: fs.fstype.clone(),
                    size: 0,  // Size not available in fstab
                    used: 0,  // Used not available in fstab
                    available: 0,  // Available not available in fstab
                }
            }).collect()
        } else {
            Vec::new()
        }
    } else {
        Vec::new()
    };

    // Convert packages
    let packages = if let Some(ref pkg_section) = report.packages {
        pkg_section.kernels.iter().take(100).map(|k| {
            PackageInfo {
                name: k.clone(),
                version: format!("{} package", pkg_section.format),
                arch: architecture.clone(),
            }
        }).collect()
    } else {
        Vec::new()
    };

    // Convert users
    let users = if let Some(ref user_section) = report.users {
        user_section.regular_users.iter().map(|u| {
            UserInfo {
                username: u.username.clone(),
                uid: u.uid.clone(),
                home: u.home.clone(),
                shell: u.shell.clone(),
            }
        }).collect()
    } else {
        Vec::new()
    };

    // Convert network interfaces
    let interfaces = if let Some(ref net_section) = report.network {
        if let Some(ref intfs) = net_section.interfaces {
            intfs.iter().map(|i| {
                NetworkInterface {
                    name: i.name.clone(),
                    mac_address: i.mac_address.clone(),
                    ip_addresses: i.ip_address.join(", "),
                    state: "up".to_string(), // Assume up if listed
                }
            }).collect()
        } else {
            Vec::new()
        }
    } else {
        Vec::new()
    };

    InspectionData {
        hostname,
        os_type,
        distribution,
        version,
        architecture,
        product_name,
        package_format,
        package_manager,
        kernel_version: None, // Not available in current report format
        total_memory: None, // Not available in current report format
        vcpus: None, // Not available in current report format
        filesystems,
        packages,
        users,
        interfaces,
    }
}
