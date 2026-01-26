// SPDX-License-Identifier: LGPL-3.0-or-later
//! TUI application state management

use anyhow::Result;
use guestctl::guestfs::inspect_enhanced::{
    Database, FirewallInfo, HostEntry, LVMInfo, LogicalVolume, NetworkInterface, PackageInfo,
    RAIDArray, SecurityInfo, SystemService, VolumeGroup, WebServer,
};
use guestctl::Guestfs;
use std::path::Path;

use crate::cli::profiles::{
    ComplianceProfile, InspectionProfile, MigrationProfile, PerformanceProfile, ProfileReport,
    SecurityProfile,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ExportFormat {
    Json,
    Yaml,
    Html,
    Pdf,
}

impl ExportFormat {
    pub fn extension(&self) -> &'static str {
        match self {
            ExportFormat::Json => "json",
            ExportFormat::Yaml => "yaml",
            ExportFormat::Html => "html",
            ExportFormat::Pdf => "pdf",
        }
    }

    pub fn name(&self) -> &'static str {
        match self {
            ExportFormat::Json => "JSON",
            ExportFormat::Yaml => "YAML",
            ExportFormat::Html => "HTML",
            ExportFormat::Pdf => "PDF",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ExportMode {
    Selecting,
    EnteringFilename,
    Exporting,
    Success(String), // filename
    Error(String),   // error message
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum View {
    Dashboard,
    Network,
    Packages,
    Services,
    Security,
    Storage,
    Profiles,
}

impl View {
    pub fn title(&self) -> &str {
        match self {
            View::Dashboard => "Dashboard",
            View::Network => "Network",
            View::Packages => "Packages",
            View::Services => "Services",
            View::Security => "Security",
            View::Storage => "Storage",
            View::Profiles => "Profiles",
        }
    }

    pub fn all() -> Vec<View> {
        vec![
            View::Dashboard,
            View::Network,
            View::Packages,
            View::Services,
            View::Security,
            View::Storage,
            View::Profiles,
        ]
    }
}

pub struct App {
    pub current_view: View,
    pub show_help: bool,
    pub searching: bool,
    pub search_query: String,
    pub scroll_offset: usize,
    pub selected_index: usize,
    pub show_export_menu: bool,
    pub selected_profile_tab: usize,

    // Export state
    pub export_mode: Option<ExportMode>,
    pub export_format: Option<ExportFormat>,
    pub export_filename: String,

    // Inspection data
    pub image_path: String,
    pub os_name: String,
    pub os_version: String,
    pub hostname: String,
    pub kernel_version: String,
    pub architecture: String,
    pub init_system: String,
    pub timezone: String,
    pub locale: String,

    pub network_interfaces: Vec<NetworkInterface>,
    pub dns_servers: Vec<String>,

    pub packages: PackageInfo,
    pub services: Vec<SystemService>,
    pub databases: Vec<Database>,
    pub web_servers: Vec<WebServer>,
    pub firewall: FirewallInfo,
    pub security: SecurityInfo,

    pub hosts: Vec<HostEntry>,
    pub fstab: Vec<(String, String, String)>,
    pub lvm_info: Option<LVMInfo>,
    pub raid_arrays: Vec<RAIDArray>,

    // Profile reports
    pub security_profile: Option<ProfileReport>,
    pub migration_profile: Option<ProfileReport>,
    pub performance_profile: Option<ProfileReport>,
    pub compliance_profile: Option<ProfileReport>,
}

impl App {
    pub fn new(image_path: &Path) -> Result<Self> {
        let mut guestfs = Guestfs::new()?;
        guestfs.add_drive_ro(image_path)?;
        guestfs.launch()?;

        let roots = guestfs.inspect_os()?;
        let root = roots.first().ok_or_else(|| {
            anyhow::anyhow!("No operating systems found in image")
        })?;

        // Gather basic OS info
        let os_name = guestfs.inspect_get_product_name(root)
            .unwrap_or_else(|_| "Unknown".to_string());
        let os_version = guestfs.inspect_get_product_variant(root)
            .unwrap_or_else(|_| "Unknown".to_string());
        let hostname = guestfs.inspect_get_hostname(root)
            .unwrap_or_else(|_| "Unknown".to_string());
        let kernel_version = if let (Ok(major), Ok(minor)) = (
            guestfs.inspect_get_major_version(root),
            guestfs.inspect_get_minor_version(root),
        ) {
            format!("{}.{}", major, minor)
        } else {
            "Unknown".to_string()
        };
        let architecture = guestfs.inspect_get_arch(root)
            .unwrap_or_else(|_| "Unknown".to_string());

        // Gather enhanced inspection data
        let init_system = guestfs.inspect_init_system(root)
            .unwrap_or_else(|_| "unknown".to_string());
        let timezone = guestfs.inspect_timezone(root)
            .unwrap_or_else(|_| "unknown".to_string());
        let locale = guestfs.inspect_locale(root)
            .unwrap_or_else(|_| "unknown".to_string());

        let network_interfaces = guestfs.inspect_network(root)
            .unwrap_or_default();
        let dns_servers = guestfs.inspect_dns(root)
            .unwrap_or_default();

        let packages = guestfs.inspect_packages(root)
            .unwrap_or_else(|_| PackageInfo {
                manager: "unknown".to_string(),
                package_count: 0,
                packages: Vec::new(),
            });

        let services = guestfs.inspect_systemd_services(root)
            .unwrap_or_default();
        let databases = guestfs.inspect_databases(root)
            .unwrap_or_default();
        let web_servers = guestfs.inspect_web_servers(root)
            .unwrap_or_default();
        let firewall = guestfs.inspect_firewall(root)
            .unwrap_or_else(|_| FirewallInfo {
                firewall_type: "none".to_string(),
                enabled: false,
                rules_count: 0,
                zones: Vec::new(),
            });
        let security = guestfs.inspect_security(root)
            .unwrap_or_else(|_| SecurityInfo {
                selinux: "unknown".to_string(),
                apparmor: false,
                fail2ban: false,
                aide: false,
                auditd: false,
                ssh_keys: Vec::new(),
            });

        let hosts = guestfs.inspect_hosts(root)
            .unwrap_or_default();
        let fstab = guestfs.inspect_fstab(root)
            .unwrap_or_default();

        // Storage information
        let lvm_info = guestfs.inspect_lvm(root).ok();
        let raid_arrays = guestfs.inspect_raid(root).unwrap_or_default();

        // Execute profiles
        let security_profile = SecurityProfile.inspect(&mut guestfs, root).ok();
        let migration_profile = MigrationProfile.inspect(&mut guestfs, root).ok();
        let performance_profile = PerformanceProfile.inspect(&mut guestfs, root).ok();
        let compliance_profile = ComplianceProfile.inspect(&mut guestfs, root).ok();

        guestfs.shutdown()?;

        Ok(Self {
            current_view: View::Dashboard,
            show_help: false,
            searching: false,
            search_query: String::new(),
            scroll_offset: 0,
            selected_index: 0,
            show_export_menu: false,
            selected_profile_tab: 0,

            export_mode: None,
            export_format: None,
            export_filename: String::new(),

            image_path: image_path.display().to_string(),
            os_name,
            os_version,
            hostname,
            kernel_version,
            architecture,
            init_system,
            timezone,
            locale,

            network_interfaces,
            dns_servers,
            packages,
            services,
            databases,
            web_servers,
            firewall,
            security,
            hosts,
            fstab,
            lvm_info,
            raid_arrays,

            security_profile,
            migration_profile,
            performance_profile,
            compliance_profile,
        })
    }

    pub fn next_view(&mut self) {
        let views = View::all();
        let current_idx = views.iter().position(|v| v == &self.current_view).unwrap_or(0);
        self.current_view = views[(current_idx + 1) % views.len()];
        self.scroll_offset = 0;
        self.selected_index = 0;
    }

    pub fn previous_view(&mut self) {
        let views = View::all();
        let current_idx = views.iter().position(|v| v == &self.current_view).unwrap_or(0);
        self.current_view = views[(current_idx + views.len() - 1) % views.len()];
        self.scroll_offset = 0;
        self.selected_index = 0;
    }

    pub fn toggle_help(&mut self) {
        self.show_help = !self.show_help;
    }

    pub fn start_search(&mut self) {
        self.searching = true;
        self.search_query.clear();
    }

    pub fn cancel_search(&mut self) {
        self.searching = false;
        self.search_query.clear();
    }

    pub fn is_searching(&self) -> bool {
        self.searching
    }

    pub fn search_input(&mut self, c: char) {
        self.search_query.push(c);
    }

    pub fn search_backspace(&mut self) {
        self.search_query.pop();
    }

    pub fn scroll_up(&mut self) {
        if self.scroll_offset > 0 {
            self.scroll_offset -= 1;
        }
        if self.selected_index > 0 {
            self.selected_index -= 1;
        }
    }

    pub fn scroll_down(&mut self) {
        self.scroll_offset += 1;
        self.selected_index += 1;
    }

    pub fn page_up(&mut self) {
        self.scroll_offset = self.scroll_offset.saturating_sub(10);
        self.selected_index = self.selected_index.saturating_sub(10);
    }

    pub fn page_down(&mut self) {
        self.scroll_offset += 10;
        self.selected_index += 10;
    }

    pub fn scroll_top(&mut self) {
        self.scroll_offset = 0;
        self.selected_index = 0;
    }

    pub fn scroll_bottom(&mut self) {
        // This will be refined per view
        self.scroll_offset = usize::MAX;
    }

    pub fn select_item(&mut self) {
        // Handle item selection based on current view
    }

    pub fn on_tick(&mut self) {
        // Handle periodic updates if needed
    }

    pub fn toggle_export_menu(&mut self) {
        self.show_export_menu = !self.show_export_menu;
        if self.show_export_menu {
            // Reset export state when opening menu
            self.export_mode = Some(ExportMode::Selecting);
            self.export_format = None;
            self.export_filename.clear();
        } else {
            // Clear export state when closing menu
            self.export_mode = None;
        }
    }

    pub fn select_export_format(&mut self, format: ExportFormat) {
        self.export_format = Some(format);
        self.export_mode = Some(ExportMode::EnteringFilename);
        // Generate default filename
        let view_name = match self.current_view {
            View::Dashboard => "dashboard",
            View::Network => "network",
            View::Packages => "packages",
            View::Services => "services",
            View::Security => "security",
            View::Storage => "storage",
            View::Profiles => "profiles",
        };
        self.export_filename = format!(
            "guestkit-{}.{}",
            view_name,
            format.extension()
        );
    }

    pub fn export_input(&mut self, c: char) {
        if matches!(self.export_mode, Some(ExportMode::EnteringFilename)) {
            self.export_filename.push(c);
        }
    }

    pub fn export_backspace(&mut self) {
        if matches!(self.export_mode, Some(ExportMode::EnteringFilename)) {
            self.export_filename.pop();
        }
    }

    pub fn cancel_export(&mut self) {
        if matches!(self.export_mode, Some(ExportMode::EnteringFilename)) {
            // Go back to format selection
            self.export_mode = Some(ExportMode::Selecting);
            self.export_format = None;
            self.export_filename.clear();
        } else {
            // Close export menu
            self.toggle_export_menu();
        }
    }

    pub fn is_exporting(&self) -> bool {
        self.export_mode.is_some()
    }

    pub fn execute_export(&mut self) -> Result<()> {
        if let Some(format) = self.export_format {
            self.export_mode = Some(ExportMode::Exporting);

            // Perform the actual export
            match self.do_export(format, &self.export_filename.clone()) {
                Ok(()) => {
                    self.export_mode = Some(ExportMode::Success(self.export_filename.clone()));
                }
                Err(e) => {
                    self.export_mode = Some(ExportMode::Error(e.to_string()));
                }
            }
        }
        Ok(())
    }

    fn do_export(&self, format: ExportFormat, filename: &str) -> Result<()> {
        use std::fs;
        use std::path::PathBuf;

        let output_path = PathBuf::from(filename);

        // Export based on format
        match format {
            ExportFormat::Json => {
                let data = self.collect_export_data();
                let json = serde_json::to_string_pretty(&data)?;
                fs::write(&output_path, json)?;
            }
            ExportFormat::Yaml => {
                let data = self.collect_export_data();
                let yaml = serde_yaml::to_string(&data)?;
                fs::write(&output_path, yaml)?;
            }
            ExportFormat::Html | ExportFormat::Pdf => {
                // These require InspectionReport format - show message that these are TODO
                return Err(anyhow::anyhow!("HTML/PDF export from TUI coming soon. Use CLI: guestctl inspect <image> --export {}", format.extension()));
            }
        }

        Ok(())
    }

    fn collect_export_data(&self) -> serde_json::Value {
        use serde_json::json;

        match self.current_view {
            View::Dashboard => json!({
                "view": "dashboard",
                "system": {
                    "os_name": self.os_name,
                    "os_version": self.os_version,
                    "hostname": self.hostname,
                    "kernel_version": self.kernel_version,
                    "architecture": self.architecture,
                    "init_system": self.init_system,
                    "timezone": self.timezone,
                    "locale": self.locale,
                },
                "stats": {
                    "packages": self.packages.package_count,
                    "services": self.services.len(),
                    "network_interfaces": self.network_interfaces.len(),
                    "databases": self.databases.len(),
                    "web_servers": self.web_servers.len(),
                },
                "profiles": {
                    "security": self.security_profile.as_ref().and_then(|p| p.overall_risk),
                    "migration": self.migration_profile.as_ref().and_then(|p| p.overall_risk),
                    "performance": self.performance_profile.as_ref().and_then(|p| p.overall_risk),
                    "compliance": self.compliance_profile.as_ref().and_then(|p| p.overall_risk),
                }
            }),
            View::Network => json!({
                "view": "network",
                "interfaces": self.network_interfaces,
                "dns_servers": self.dns_servers,
            }),
            View::Packages => json!({
                "view": "packages",
                "manager": self.packages.manager,
                "count": self.packages.package_count,
                "packages": self.packages.packages,
            }),
            View::Services => json!({
                "view": "services",
                "count": self.services.len(),
                "services": self.services,
            }),
            View::Security => json!({
                "view": "security",
                "selinux": self.security.selinux,
                "apparmor": self.security.apparmor,
                "fail2ban": self.security.fail2ban,
                "aide": self.security.aide,
                "auditd": self.security.auditd,
                "ssh_keys": self.security.ssh_keys,
                "firewall": self.firewall,
            }),
            View::Storage => json!({
                "view": "storage",
                "fstab": self.fstab,
            }),
            View::Profiles => {
                let current_profile = match self.selected_profile_tab {
                    0 => self.security_profile.as_ref().map(|p| ("security", p)),
                    1 => self.migration_profile.as_ref().map(|p| ("migration", p)),
                    2 => self.performance_profile.as_ref().map(|p| ("performance", p)),
                    3 => self.compliance_profile.as_ref().map(|p| ("compliance", p)),
                    _ => None,
                };

                if let Some((name, profile)) = current_profile {
                    json!({
                        "view": "profiles",
                        "profile": name,
                        "report": profile,
                    })
                } else {
                    json!({
                        "view": "profiles",
                        "error": "No profile data available"
                    })
                }
            }
        }
    }

    pub fn next_profile_tab(&mut self) {
        self.selected_profile_tab = (self.selected_profile_tab + 1) % 4;
    }

    pub fn previous_profile_tab(&mut self) {
        self.selected_profile_tab = (self.selected_profile_tab + 3) % 4;
    }

    pub fn get_current_profile_report(&self) -> Option<&ProfileReport> {
        match self.selected_profile_tab {
            0 => self.security_profile.as_ref(),
            1 => self.migration_profile.as_ref(),
            2 => self.performance_profile.as_ref(),
            3 => self.compliance_profile.as_ref(),
            _ => None,
        }
    }
}
