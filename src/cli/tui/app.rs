// SPDX-License-Identifier: LGPL-3.0-or-later
//! TUI application state management

use anyhow::Result;
use chrono::{DateTime, Local};
use guestctl::guestfs::inspect_enhanced::{
    Database, FirewallInfo, HostEntry, LVMInfo, NetworkInterface, PackageInfo,
    RAIDArray, SecurityInfo, SystemService, UserAccount, WebServer,
};
use guestctl::Guestfs;
use std::collections::HashMap;
use std::path::{Path, PathBuf};

use crate::cli::profiles::{
    ComplianceProfile, HardeningProfile, InspectionProfile, MigrationProfile, PerformanceProfile,
    ProfileReport, SecurityProfile,
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
    Databases,
    WebServers,
    Security,
    Issues,
    Storage,
    Users,
    Kernel,
    Profiles,
}

impl View {
    pub fn title(&self) -> &str {
        match self {
            View::Dashboard => "Dashboard",
            View::Network => "Network",
            View::Packages => "Packages",
            View::Services => "Services",
            View::Databases => "Databases",
            View::WebServers => "WebServers",
            View::Security => "Security",
            View::Issues => "Issues",
            View::Storage => "Storage",
            View::Users => "Users",
            View::Kernel => "Kernel",
            View::Profiles => "Profiles",
        }
    }

    pub fn all() -> Vec<View> {
        vec![
            View::Dashboard,
            View::Network,
            View::Packages,
            View::Services,
            View::Databases,
            View::WebServers,
            View::Security,
            View::Issues,
            View::Storage,
            View::Users,
            View::Kernel,
            View::Profiles,
        ]
    }
}

/// Sort order for lists
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SortMode {
    NameAsc,
    NameDesc,
    Default,
}

impl SortMode {
    pub fn next(&self) -> Self {
        match self {
            SortMode::Default => SortMode::NameAsc,
            SortMode::NameAsc => SortMode::NameDesc,
            SortMode::NameDesc => SortMode::Default,
        }
    }

    pub fn label(&self) -> &str {
        match self {
            SortMode::Default => "Default",
            SortMode::NameAsc => "Name ↑",
            SortMode::NameDesc => "Name ↓",
        }
    }
}

pub struct App {
    pub current_view: View,
    pub show_help: bool,
    pub searching: bool,
    pub search_query: String,
    pub search_case_sensitive: bool,
    pub search_regex_mode: bool,
    pub scroll_offset: usize,
    pub selected_index: usize,
    pub show_export_menu: bool,
    pub selected_profile_tab: usize,
    pub show_detail: bool,
    pub sort_mode: SortMode,
    pub show_stats_bar: bool,
    pub bookmarks: Vec<String>,
    pub search_history: Vec<String>,
    pub notification: Option<(String, u8)>, // (message, ticks_remaining)
    pub last_updated: DateTime<Local>,
    pub refreshing: bool,

    // Jump menu state
    pub show_jump_menu: bool,
    pub jump_query: String,
    pub jump_selected_index: usize,

    // Export state
    pub export_mode: Option<ExportMode>,
    pub export_format: Option<ExportFormat>,
    pub export_filename: String,

    // Inspection data
    pub image_path: String,
    pub image_path_buf: PathBuf,
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
    pub users: Vec<UserAccount>,

    pub _hosts: Vec<HostEntry>,
    pub fstab: Vec<(String, String, String)>,
    pub lvm_info: Option<LVMInfo>,
    pub raid_arrays: Vec<RAIDArray>,

    // Kernel configuration
    pub kernel_modules: Vec<String>,
    pub kernel_params: HashMap<String, String>,

    // Profile reports
    pub security_profile: Option<ProfileReport>,
    pub migration_profile: Option<ProfileReport>,
    pub performance_profile: Option<ProfileReport>,
    pub compliance_profile: Option<ProfileReport>,
    pub hardening_profile: Option<ProfileReport>,
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

        // Mount the root filesystem once before gathering all inspection data
        guestfs.mount_ro(root, "/")?;

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

        // User accounts
        let users = guestfs.inspect_users(root)
            .unwrap_or_default();

        // Storage information
        let lvm_info = guestfs.inspect_lvm(root).ok();
        let raid_arrays = guestfs.inspect_raid(root).unwrap_or_default();

        // Kernel configuration
        let kernel_modules = guestfs.inspect_kernel_modules(root)
            .unwrap_or_default();
        let kernel_params = guestfs.inspect_kernel_params(root)
            .unwrap_or_default();

        // Execute profiles
        let security_profile = SecurityProfile.inspect(&mut guestfs, root).ok();
        let migration_profile = MigrationProfile.inspect(&mut guestfs, root).ok();
        let performance_profile = PerformanceProfile.inspect(&mut guestfs, root).ok();
        let compliance_profile = ComplianceProfile.inspect(&mut guestfs, root).ok();
        let hardening_profile = HardeningProfile.inspect(&mut guestfs, root).ok();

        guestfs.shutdown()?;

        Ok(Self {
            current_view: View::Dashboard,
            show_help: false,
            searching: false,
            search_query: String::new(),
            search_case_sensitive: false,
            search_regex_mode: false,
            scroll_offset: 0,
            selected_index: 0,
            show_export_menu: false,
            selected_profile_tab: 0,
            show_detail: false,
            sort_mode: SortMode::Default,
            show_stats_bar: true,
            bookmarks: Vec::new(),
            search_history: Vec::new(),
            notification: None,
            last_updated: Local::now(),
            refreshing: false,

            show_jump_menu: false,
            jump_query: String::new(),
            jump_selected_index: 0,

            export_mode: None,
            export_format: None,
            export_filename: String::new(),

            image_path: image_path.display().to_string(),
            image_path_buf: image_path.to_path_buf(),
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
            users,
            _hosts: hosts,
            fstab,
            lvm_info,
            raid_arrays,

            kernel_modules,
            kernel_params,

            security_profile,
            migration_profile,
            performance_profile,
            compliance_profile,
            hardening_profile,
        })
    }

    pub fn next_view(&mut self) {
        let views = View::all();
        let current_idx = views.iter().position(|v| v == &self.current_view).unwrap_or(0);
        self.current_view = views[(current_idx + 1) % views.len()];
        self.scroll_offset = 0;
        self.selected_index = 0;
        self.show_notification(format!("→ {}", self.current_view.title()));
    }

    pub fn previous_view(&mut self) {
        let views = View::all();
        let current_idx = views.iter().position(|v| v == &self.current_view).unwrap_or(0);
        self.current_view = views[(current_idx + views.len() - 1) % views.len()];
        self.scroll_offset = 0;
        self.selected_index = 0;
        self.show_notification(format!("← {}", self.current_view.title()));
    }

    pub fn toggle_help(&mut self) {
        self.show_help = !self.show_help;
    }

    pub fn start_search(&mut self) {
        self.searching = true;
        self.search_query.clear();
    }

    pub fn cancel_search(&mut self) {
        // Save to history before clearing
        if !self.search_query.is_empty() {
            self.add_to_search_history(self.search_query.clone());
            self.show_notification("Search saved to history".to_string());
        }
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
        // Decrement notification timer
        if let Some((_, ref mut ticks)) = self.notification {
            if *ticks > 0 {
                *ticks -= 1;
            } else {
                self.notification = None;
            }
        }
    }

    pub fn show_notification(&mut self, message: String) {
        // Show notification for 8 ticks (2 seconds at 250ms tick rate)
        self.notification = Some((message, 8));
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
            View::Databases => "databases",
            View::WebServers => "webservers",
            View::Security => "security",
            View::Issues => "issues",
            View::Storage => "storage",
            View::Users => "users",
            View::Kernel => "kernel",
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
                    self.show_notification(format!("✓ Exported to {}", self.export_filename));
                }
                Err(e) => {
                    self.export_mode = Some(ExportMode::Error(e.to_string()));
                    self.show_notification(format!("✗ Export failed: {}", e));
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
                    "users": self.users.len(),
                },
                "profiles": {
                    "security": self.security_profile.as_ref().and_then(|p| p.overall_risk),
                    "migration": self.migration_profile.as_ref().and_then(|p| p.overall_risk),
                    "performance": self.performance_profile.as_ref().and_then(|p| p.overall_risk),
                    "compliance": self.compliance_profile.as_ref().and_then(|p| p.overall_risk),
                    "hardening": self.hardening_profile.as_ref().and_then(|p| p.overall_risk),
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
            View::Databases => json!({
                "view": "databases",
                "count": self.databases.len(),
                "databases": self.databases,
            }),
            View::WebServers => json!({
                "view": "webservers",
                "count": self.web_servers.len(),
                "web_servers": self.web_servers,
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
            View::Issues => {
                let (critical, high, medium) = self.get_risk_summary();
                let mut all_sections = Vec::new();

                if let Some(sec) = &self.security_profile {
                    all_sections.extend(sec.sections.clone());
                }
                if let Some(hard) = &self.hardening_profile {
                    all_sections.extend(hard.sections.clone());
                }
                if let Some(comp) = &self.compliance_profile {
                    all_sections.extend(comp.sections.clone());
                }

                json!({
                    "view": "issues",
                    "summary": {
                        "critical": critical,
                        "high": high,
                        "medium": medium,
                        "total": critical + high + medium,
                    },
                    "sections": all_sections,
                })
            }
            View::Storage => json!({
                "view": "storage",
                "fstab": self.fstab,
                "lvm": self.lvm_info,
                "raid": self.raid_arrays,
            }),
            View::Users => json!({
                "view": "users",
                "count": self.users.len(),
                "users": self.users,
            }),
            View::Kernel => json!({
                "view": "kernel",
                "modules": {
                    "count": self.kernel_modules.len(),
                    "list": self.kernel_modules,
                },
                "parameters": self.kernel_params,
            }),
            View::Profiles => {
                let current_profile = match self.selected_profile_tab {
                    0 => self.security_profile.as_ref().map(|p| ("security", p)),
                    1 => self.migration_profile.as_ref().map(|p| ("migration", p)),
                    2 => self.performance_profile.as_ref().map(|p| ("performance", p)),
                    3 => self.compliance_profile.as_ref().map(|p| ("compliance", p)),
                    4 => self.hardening_profile.as_ref().map(|p| ("hardening", p)),
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
        self.selected_profile_tab = (self.selected_profile_tab + 1) % 5;
        let profile_names = ["Security", "Migration", "Performance", "Compliance", "Hardening"];
        self.show_notification(format!("→ {} Profile", profile_names[self.selected_profile_tab]));
    }

    pub fn previous_profile_tab(&mut self) {
        self.selected_profile_tab = (self.selected_profile_tab + 4) % 5;
        let profile_names = ["Security", "Migration", "Performance", "Compliance", "Hardening"];
        self.show_notification(format!("← {} Profile", profile_names[self.selected_profile_tab]));
    }

    pub fn get_current_profile_report(&self) -> Option<&ProfileReport> {
        match self.selected_profile_tab {
            0 => self.security_profile.as_ref(),
            1 => self.migration_profile.as_ref(),
            2 => self.performance_profile.as_ref(),
            3 => self.compliance_profile.as_ref(),
            4 => self.hardening_profile.as_ref(),
            _ => None,
        }
    }

    pub fn toggle_detail(&mut self) {
        self.show_detail = !self.show_detail;
    }

    pub fn cycle_sort_mode(&mut self) {
        self.sort_mode = self.sort_mode.next();
        // Reset scroll when sorting changes
        self.scroll_offset = 0;
        self.selected_index = 0;
        self.show_notification(format!("Sort: {}", self.sort_mode.label()));
    }

    pub fn jump_to_view(&mut self, index: usize) {
        let views = View::all();
        if index < views.len() {
            self.current_view = views[index];
            self.scroll_offset = 0;
            self.selected_index = 0;
            self.show_notification(format!("⚡ {} ({})", self.current_view.title(), index + 1));
        }
    }

    pub fn toggle_stats_bar(&mut self) {
        self.show_stats_bar = !self.show_stats_bar;
        let state = if self.show_stats_bar { "shown" } else { "hidden" };
        self.show_notification(format!("Stats bar {}", state));
    }

    pub fn add_bookmark(&mut self, item: String) {
        if !self.bookmarks.contains(&item) {
            self.bookmarks.push(item.clone());
            // Keep only last 20 bookmarks
            if self.bookmarks.len() > 20 {
                self.bookmarks.remove(0);
            }
            self.show_notification(format!("✓ Bookmarked: {}", item));
        } else {
            self.show_notification("⚠ Already bookmarked".to_string());
        }
    }

    #[allow(dead_code)]
    pub fn clear_bookmarks(&mut self) {
        self.bookmarks.clear();
    }

    pub fn add_to_search_history(&mut self, query: String) {
        if !query.is_empty() && !self.search_history.contains(&query) {
            self.search_history.push(query);
            // Keep only last 10 searches
            if self.search_history.len() > 10 {
                self.search_history.remove(0);
            }
        }
    }

    pub fn get_risk_summary(&self) -> (usize, usize, usize) {
        let mut critical = 0;
        let mut high = 0;
        let mut medium = 0;

        let profiles = vec![
            &self.security_profile,
            &self.migration_profile,
            &self.performance_profile,
            &self.compliance_profile,
            &self.hardening_profile,
        ];

        for profile in profiles {
            if let Some(p) = profile {
                if let Some(risk) = p.overall_risk {
                    use crate::cli::profiles::RiskLevel;
                    match risk {
                        RiskLevel::Critical => critical += 1,
                        RiskLevel::High => high += 1,
                        RiskLevel::Medium => medium += 1,
                        _ => {}
                    }
                }
            }
        }

        (critical, high, medium)
    }

    /// Calculate overall system health score (0-100)
    pub fn calculate_health_score(&self) -> u8 {
        let mut score: u8 = 100;

        // Deduct points for critical/high/medium risks
        let (critical, high, medium) = self.get_risk_summary();
        score = score.saturating_sub((critical * 20) as u8);
        score = score.saturating_sub((high * 10) as u8);
        score = score.saturating_sub((medium * 5) as u8);

        // Deduct points for missing security features
        if &self.security.selinux == "disabled" {
            score = score.saturating_sub(10);
        }
        if !self.firewall.enabled {
            score = score.saturating_sub(15);
        }
        if !self.security.auditd {
            score = score.saturating_sub(5);
        }
        if !self.security.fail2ban {
            score = score.saturating_sub(5);
        }
        if !self.security.aide {
            score = score.saturating_sub(5);
        }

        // Bonus points for good practices
        if self.security.apparmor || &self.security.selinux != "disabled" {
            score = (score + 5).min(100);
        }

        score
    }

    /// Get health status message and color based on score
    pub fn get_health_status(&self) -> (&str, &str) {
        let score = self.calculate_health_score();
        match score {
            90..=100 => ("Excellent", "green"),
            75..=89 => ("Good", "yellow"),
            60..=74 => ("Fair", "orange"),
            40..=59 => ("Poor", "red"),
            _ => ("Critical", "red"),
        }
    }

    /// Get formatted timestamp of last update
    pub fn get_last_updated_formatted(&self) -> String {
        self.last_updated.format("%H:%M:%S").to_string()
    }

    /// Get time since last update in human-readable format
    pub fn get_time_since_update(&self) -> String {
        let duration = Local::now().signed_duration_since(self.last_updated);

        if duration.num_seconds() < 60 {
            format!("{}s ago", duration.num_seconds())
        } else if duration.num_minutes() < 60 {
            format!("{}m ago", duration.num_minutes())
        } else if duration.num_hours() < 24 {
            format!("{}h ago", duration.num_hours())
        } else {
            format!("{}d ago", duration.num_days())
        }
    }

    /// Initiate refresh (note: actual refresh would need background thread)
    pub fn start_refresh(&mut self) {
        self.refreshing = true;
        self.show_notification("Refreshing data...".to_string());
    }

    /// Mark refresh as complete
    pub fn complete_refresh(&mut self) {
        self.refreshing = false;
        self.last_updated = Local::now();
        self.show_notification("✓ Data refreshed".to_string());
    }

    /// Toggle case-sensitive search
    pub fn toggle_case_sensitive(&mut self) {
        self.search_case_sensitive = !self.search_case_sensitive;
        let status = if self.search_case_sensitive { "ON" } else { "OFF" };
        self.show_notification(format!("Case-sensitive: {}", status));
    }

    /// Toggle regex search mode
    pub fn toggle_regex_mode(&mut self) {
        self.search_regex_mode = !self.search_regex_mode;
        let status = if self.search_regex_mode { "ON" } else { "OFF" };
        self.show_notification(format!("Regex mode: {}", status));
    }

    /// Get search mode indicator string
    pub fn get_search_mode_indicator(&self) -> String {
        let mut indicators = Vec::new();
        if self.search_case_sensitive {
            indicators.push("Aa");
        }
        if self.search_regex_mode {
            indicators.push(".*");
        }
        if indicators.is_empty() {
            String::new()
        } else {
            format!("[{}] ", indicators.join(" "))
        }
    }

    /// Toggle jump menu visibility
    pub fn toggle_jump_menu(&mut self) {
        self.show_jump_menu = !self.show_jump_menu;
        if self.show_jump_menu {
            self.jump_query.clear();
            self.jump_selected_index = 0;
        }
    }

    /// Handle jump menu input
    pub fn jump_menu_input(&mut self, c: char) {
        self.jump_query.push(c);
        self.jump_selected_index = 0; // Reset selection when query changes
    }

    /// Handle jump menu backspace
    pub fn jump_menu_backspace(&mut self) {
        self.jump_query.pop();
        self.jump_selected_index = 0; // Reset selection when query changes
    }

    /// Get filtered views based on jump query
    pub fn get_filtered_views(&self) -> Vec<(usize, View, String)> {
        let views = View::all();

        if self.jump_query.is_empty() {
            // Show all views with their index
            return views.iter().enumerate()
                .map(|(idx, v)| (idx, *v, v.title().to_string()))
                .collect();
        }

        // Fuzzy matching: check if query chars appear in order in the view title
        let query_lower = self.jump_query.to_lowercase();
        views.iter().enumerate()
            .filter_map(|(idx, v)| {
                let title = v.title();
                let title_lower = title.to_lowercase();

                // Simple fuzzy match: all query chars must appear in order
                let mut query_chars = query_lower.chars();
                let mut current_query_char = query_chars.next();

                for title_char in title_lower.chars() {
                    if let Some(qc) = current_query_char {
                        if qc == title_char {
                            current_query_char = query_chars.next();
                        }
                    }
                }

                // If we consumed all query chars, it's a match
                if current_query_char.is_none() {
                    // Highlight matching characters
                    let mut highlighted = String::new();
                    let mut query_chars = query_lower.chars().peekable();

                    for tc in title.chars() {
                        if let Some(&qc) = query_chars.peek() {
                            if tc.to_lowercase().to_string() == qc.to_string() {
                                highlighted.push_str(&format!("[{}]", tc));
                                query_chars.next();
                            } else {
                                highlighted.push(tc);
                            }
                        } else {
                            highlighted.push(tc);
                        }
                    }

                    Some((idx, *v, highlighted))
                } else {
                    None
                }
            })
            .collect()
    }

    /// Navigate jump menu selection
    pub fn jump_menu_next(&mut self) {
        let filtered_count = self.get_filtered_views().len();
        if filtered_count > 0 {
            self.jump_selected_index = (self.jump_selected_index + 1) % filtered_count;
        }
    }

    pub fn jump_menu_previous(&mut self) {
        let filtered_count = self.get_filtered_views().len();
        if filtered_count > 0 {
            if self.jump_selected_index == 0 {
                self.jump_selected_index = filtered_count - 1;
            } else {
                self.jump_selected_index -= 1;
            }
        }
    }

    /// Execute jump to selected view
    pub fn jump_menu_select(&mut self) {
        let filtered_views = self.get_filtered_views();
        if let Some((_, view, _)) = filtered_views.get(self.jump_selected_index) {
            self.current_view = *view;
            self.scroll_offset = 0;
            self.selected_index = 0;
            self.show_jump_menu = false;
            self.show_notification(format!("→ {}", view.title()));
        }
    }
}
