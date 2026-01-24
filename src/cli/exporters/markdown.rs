// SPDX-License-Identifier: LGPL-3.0-or-later
//! Markdown report generation

use crate::cli::formatters::InspectionReport;
use anyhow::Result;

/// Generate Markdown report from inspection data
pub fn generate_markdown_report(report: &InspectionReport) -> Result<String> {
    let mut md = String::new();

    // Header
    let vm_name = report.os.hostname.clone().unwrap_or_else(|| "Unknown".to_string());
    let timestamp = chrono::Local::now().format("%Y-%m-%d %H:%M:%S").to_string();

    md.push_str(&format!("# VM Inspection Report: {}\n\n", vm_name));
    md.push_str(&format!("**Generated:** {}\n\n", timestamp));
    md.push_str("---\n\n");

    // Operating System
    md.push_str("## Operating System\n\n");

    if let Some(ref os_type) = report.os.os_type {
        md.push_str(&format!("- **Type:** {}\n", os_type));
    }

    if let Some(ref distro) = report.os.distribution {
        md.push_str(&format!("- **Distribution:** {}\n", distro));
    }

    if let Some(ref version) = report.os.version {
        md.push_str(&format!("- **Version:** {}.{}\n", version.major, version.minor));
    }

    if let Some(ref arch) = report.os.architecture {
        md.push_str(&format!("- **Architecture:** {}\n", arch));
    }

    if let Some(ref hostname) = report.os.hostname {
        md.push_str(&format!("- **Hostname:** {}\n", hostname));
    }

    if let Some(ref product) = report.os.product_name {
        md.push_str(&format!("- **Product:** {}\n", product));
    }

    if let Some(ref pkg_format) = report.os.package_format {
        md.push_str(&format!("- **Package Format:** {}\n", pkg_format));
    }

    if let Some(ref pkg_mgmt) = report.os.package_manager {
        md.push_str(&format!("- **Package Management:** {}\n", pkg_mgmt));
    }

    md.push_str("\n");

    // Packages
    if let Some(ref packages) = report.packages {
        md.push_str(&format!("## Packages\n\n"));
        md.push_str(&format!("**Total Packages:** {}\n\n", packages.count));

        if !packages.kernels.is_empty() {
            md.push_str("### Installed Kernels\n\n");
            for kernel in &packages.kernels {
                md.push_str(&format!("- {}\n", kernel));
            }
            md.push_str("\n");
        }
    }

    // Services
    if let Some(ref services) = report.services {
        md.push_str("## Services\n\n");
        md.push_str(&format!("**Enabled Services:** {}\n\n", services.enabled_services.len()));

        if !services.enabled_services.is_empty() {
            md.push_str("| Service | State |\n");
            md.push_str("|---------|-------|\n");

            for service in services.enabled_services.iter().take(50) {
                md.push_str(&format!("| {} | {} |\n", service.name, service.state));
            }

            if services.enabled_services.len() > 50 {
                md.push_str(&format!("\n*...and {} more services*\n", services.enabled_services.len() - 50));
            }

            md.push_str("\n");
        }
    }

    // Users
    if let Some(ref users) = report.users {
        md.push_str("## User Accounts\n\n");
        md.push_str(&format!("**Regular Users:** {}\n", users.regular_users.len()));
        md.push_str(&format!("**System Accounts:** {}\n\n", users.system_users_count));

        if !users.regular_users.is_empty() {
            md.push_str("### Regular Users\n\n");
            md.push_str("| Username | UID | Home Directory |\n");
            md.push_str("|----------|-----|----------------|\n");

            for user in &users.regular_users {
                md.push_str(&format!("| {} | {} | {} |\n", user.username, user.uid, user.home));
            }

            md.push_str("\n");
        }
    }

    // Network
    if let Some(ref network) = report.network {
        md.push_str("## Network Configuration\n\n");

        if let Some(ref interfaces) = network.interfaces {
            md.push_str(&format!("**Network Interfaces:** {}\n\n", interfaces.len()));

            md.push_str("| Interface | IP Address | MAC Address | DHCP |\n");
            md.push_str("|-----------|------------|-------------|------|\n");

            for iface in interfaces {
                let dhcp = if iface.dhcp { "Yes" } else { "No" };
                md.push_str(&format!("| {} | {} | {} | {} |\n",
                    iface.name,
                    iface.ip_address.join(", "),
                    iface.mac_address,
                    dhcp
                ));
            }

            md.push_str("\n");
        }

        if let Some(ref dns) = network.dns_servers {
            if !dns.is_empty() {
                md.push_str("### DNS Servers\n\n");
                for server in dns {
                    md.push_str(&format!("- {}\n", server));
                }
                md.push_str("\n");
            }
        }

    }

    // Filesystems
    if let Some(ref storage) = report.storage {
        if let Some(ref fstab_mounts) = storage.fstab_mounts {
            md.push_str("## Filesystems\n\n");
            md.push_str(&format!("**Filesystems:** {}\n\n", fstab_mounts.len()));

            md.push_str("| Device | Mount Point | Filesystem Type |\n");
            md.push_str("|--------|-------------|------------------|\n");

            for fs in fstab_mounts {
                md.push_str(&format!("| {} | {} | {} |\n",
                    fs.device,
                    fs.mountpoint,
                    fs.fstype
                ));
            }

            md.push_str("\n");
        }
    }

    // Storage
    if let Some(ref storage) = report.storage {
        md.push_str("## Storage\n\n");

        if let Some(ref lvm) = storage.lvm {
            if !lvm.physical_volumes.is_empty() {
                md.push_str("### LVM Physical Volumes\n\n");
                for pv in &lvm.physical_volumes {
                    md.push_str(&format!("- {}\n", pv));
                }
                md.push_str("\n");
            }

            if !lvm.volume_groups.is_empty() {
                md.push_str("### LVM Volume Groups\n\n");
                for vg in &lvm.volume_groups {
                    md.push_str(&format!("- {}\n", vg));
                }
                md.push_str("\n");
            }

            if !lvm.logical_volumes.is_empty() {
                md.push_str("### LVM Logical Volumes\n\n");
                for lv in &lvm.logical_volumes {
                    md.push_str(&format!("- {}\n", lv));
                }
                md.push_str("\n");
            }
        }
    }

    // System Configuration
    if let Some(ref config) = report.system_config {
        md.push_str("## System Configuration\n\n");

        if let Some(ref timezone) = config.timezone {
            md.push_str(&format!("- **Timezone:** {}\n", timezone));
        }

        if let Some(ref selinux) = config.selinux {
            md.push_str(&format!("- **SELinux:** {}\n", selinux));
        }


        md.push_str("\n");
    }

    // Footer
    md.push_str("---\n\n");
    md.push_str("*Generated by GuestKit - Pure Rust VM Inspection Toolkit*\n");

    Ok(md)
}
