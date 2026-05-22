// SPDX-License-Identifier: LGPL-3.0-or-later
//! Staged inspection loading, lazy views, and refresh for the TUI.

use super::app::{App, CompareSummary, IssueRiskFilter, LayoutMode, View};
use super::cache;
use super::loading::{LoadingStage, LoadingState};
use super::palette::PaletteAction;
use crate::cli::profiles::{
    ComplianceProfile, HardeningProfile, InspectionProfile, MigrationProfile, PerformanceProfile,
    SecurityProfile,
};
use crate::guestfs::inspect_enhanced::{
    FirewallInfo, PackageInfo, SecurityInfo,
};
use crate::Guestfs;
use anyhow::Result;
use std::collections::HashSet;
use std::path::Path;

impl View {
    pub fn name(self) -> &'static str {
        match self {
            View::Dashboard => "dashboard",
            View::Analytics => "analytics",
            View::Timeline => "timeline",
            View::Recommendations => "recommendations",
            View::Topology => "topology",
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
            View::Logs => "logs",
            View::Profiles => "profiles",
            View::Files => "files",
        }
    }

    pub fn from_name(s: &str) -> Option<Self> {
        match s.to_lowercase().as_str() {
            "dashboard" => Some(View::Dashboard),
            "analytics" => Some(View::Analytics),
            "timeline" => Some(View::Timeline),
            "recommendations" => Some(View::Recommendations),
            "topology" => Some(View::Topology),
            "network" => Some(View::Network),
            "packages" => Some(View::Packages),
            "services" => Some(View::Services),
            "databases" => Some(View::Databases),
            "webservers" | "web_servers" => Some(View::WebServers),
            "security" => Some(View::Security),
            "issues" => Some(View::Issues),
            "storage" => Some(View::Storage),
            "users" => Some(View::Users),
            "kernel" => Some(View::Kernel),
            "logs" => Some(View::Logs),
            "profiles" => Some(View::Profiles),
            "files" => Some(View::Files),
            _ => None,
        }
    }

    pub fn group(self) -> &'static str {
        match self {
            View::Dashboard | View::Analytics | View::Timeline | View::Recommendations
            | View::Topology => "Overview",
            View::Network | View::Packages | View::Services | View::Databases
            | View::WebServers | View::Users | View::Kernel | View::Logs | View::Storage
            | View::Files => "System",
            View::Security | View::Issues | View::Profiles => "Security",
        }
    }

    /// Heavy views loaded on first visit when lazy loading is active.
    pub fn is_lazy(self) -> bool {
        matches!(
            self,
            View::Packages
                | View::Kernel
                | View::Databases
                | View::WebServers
                | View::Logs
                | View::Analytics
                | View::Timeline
                | View::Recommendations
                | View::Topology
        )
    }
}

impl App {
    /// Fast path: mount image and read OS metadata only.
    pub fn bootstrap(image_path: &Path, compare_path: Option<&Path>) -> Result<Self> {
        let config = super::config::TuiConfig::load();
        let mut guestfs = Guestfs::new()?;
        guestfs.add_drive_ro(image_path)?;
        guestfs.launch()?;

        let roots = guestfs.inspect_os()?;
        let root = roots
            .first()
            .ok_or_else(|| anyhow::anyhow!("No operating systems found in image"))?
            .clone();

        guestfs.mount_ro(&root, "/")?;

        let os_name = guestfs
            .inspect_get_product_name(&root)
            .unwrap_or_else(|_| "Unknown".to_string());
        let os_version = guestfs
            .inspect_get_product_variant(&root)
            .unwrap_or_else(|_| "Unknown".to_string());
        let hostname = guestfs
            .inspect_get_hostname(&root)
            .unwrap_or_else(|_| "Unknown".to_string());
        let kernel_version = if let (Ok(major), Ok(minor)) = (
            guestfs.inspect_get_major_version(&root),
            guestfs.inspect_get_minor_version(&root),
        ) {
            format!("{}.{}", major, minor)
        } else {
            "Unknown".to_string()
        };
        let architecture = guestfs
            .inspect_get_arch(&root)
            .unwrap_or_else(|_| "Unknown".to_string());
        let init_system = guestfs
            .inspect_init_system(&root)
            .unwrap_or_else(|_| "unknown".to_string());
        let timezone = guestfs
            .inspect_timezone(&root)
            .unwrap_or_else(|_| "unknown".to_string());
        let locale = guestfs
            .inspect_locale(&root)
            .unwrap_or_else(|_| "unknown".to_string());

        let layout_mode = match config.views.default_layout.as_str() {
            "list" => LayoutMode::ListOnly,
            "detail" => LayoutMode::DetailFull,
            _ => LayoutMode::SplitDetail,
        };

        let current_view = View::from_name(&config.behavior.default_view).unwrap_or(View::Dashboard);

        let mut loaded_views = HashSet::new();
        loaded_views.insert(View::Dashboard);

        let mut app = Self::empty_shell(
            image_path,
            compare_path,
            config,
            guestfs,
            root,
            os_name,
            os_version,
            hostname,
            kernel_version,
            architecture,
            init_system,
            timezone,
            locale,
            current_view,
            layout_mode,
            loaded_views,
        );

        app.loading = Some(LoadingState::new());
        Ok(app)
    }

    /// Advance one loading stage; returns true when fully loaded.
    pub fn advance_loading(&mut self) -> Result<bool> {
        if self.loading.is_none() {
            return Ok(true);
        }

        let stage = self.loading.as_ref().map(|l| l.stage).unwrap_or(LoadingStage::Done);
        if stage == LoadingStage::Done {
            self.loading = None;
            return Ok(true);
        }

        let root = self.inspect_root.clone();

        match stage {
            LoadingStage::Bootstrap => {
                self.set_loading_stage(LoadingStage::NetworkSecurity);
            }
            LoadingStage::NetworkSecurity => {
                if let Some(ref mut gfs) = self.guestfs {
                    self.network_interfaces = gfs.inspect_network(&root).unwrap_or_default();
                    self.dns_servers = gfs.inspect_dns(&root).unwrap_or_default();
                    self.firewall = gfs.inspect_firewall(&root).unwrap_or_else(|_| FirewallInfo {
                        firewall_type: "none".to_string(),
                        enabled: false,
                        rules_count: 0,
                        zones: Vec::new(),
                    });
                    self.security = gfs.inspect_security(&root).unwrap_or_else(|_| SecurityInfo {
                        selinux: "unknown".to_string(),
                        apparmor: false,
                        fail2ban: false,
                        aide: false,
                        auditd: false,
                        ssh_keys: Vec::new(),
                    });
                    self.users = gfs.inspect_users(&root).unwrap_or_default();
                }
                self.loaded_views.insert(View::Network);
                self.loaded_views.insert(View::Users);
                self.loaded_views.insert(View::Security);
                self.set_loading_stage(LoadingStage::PackagesServices);
            }
            LoadingStage::PackagesServices => {
                if let Some(ref mut gfs) = self.guestfs {
                    self.packages = gfs.inspect_packages(&root).unwrap_or_else(|_| PackageInfo {
                        manager: "unknown".to_string(),
                        package_count: 0,
                        packages: Vec::new(),
                    });
                    self.services = gfs.inspect_systemd_services(&root).unwrap_or_default();
                    self.databases = gfs.inspect_databases(&root).unwrap_or_default();
                    self.web_servers = gfs.inspect_web_servers(&root).unwrap_or_default();
                }
                self.loaded_views.insert(View::Packages);
                self.loaded_views.insert(View::Services);
                self.loaded_views.insert(View::Databases);
                self.loaded_views.insert(View::WebServers);
                self.set_loading_stage(LoadingStage::Profiles);
            }
            LoadingStage::Profiles => {
                if let Some(ref mut gfs) = self.guestfs {
                    self.security_profile = SecurityProfile.inspect(gfs, &root).ok();
                    self.migration_profile = MigrationProfile.inspect(gfs, &root).ok();
                    self.performance_profile = PerformanceProfile.inspect(gfs, &root).ok();
                    self.compliance_profile = ComplianceProfile.inspect(gfs, &root).ok();
                    self.hardening_profile = HardeningProfile.inspect(gfs, &root).ok();
                }
                self.loaded_views.insert(View::Profiles);
                self.loaded_views.insert(View::Issues);
                self.set_loading_stage(LoadingStage::StorageKernel);
            }
            LoadingStage::StorageKernel => {
                if let Some(ref mut gfs) = self.guestfs {
                    self._hosts = gfs.inspect_hosts(&root).unwrap_or_default();
                    self.fstab = gfs.inspect_fstab(&root).unwrap_or_default();
                    self.lvm_info = gfs.inspect_lvm(&root).ok();
                    self.raid_arrays = gfs.inspect_raid(&root).unwrap_or_default();
                    self.kernel_modules = gfs.inspect_kernel_modules(&root).unwrap_or_default();
                    self.kernel_params = gfs.inspect_kernel_params(&root).unwrap_or_default();
                }
                self.loaded_views.insert(View::Storage);
                self.loaded_views.insert(View::Kernel);
                self.set_loading_stage(LoadingStage::CompareImage);
            }
            LoadingStage::CompareImage => {
                if let Some(compare) = self.compare_image_path.clone() {
                    self.load_compare_image(&compare)?;
                }
                self.loading = None;
                self.last_updated = chrono::Local::now();
                let _ = cache::write_cached_flag(&self._image_path_buf);
            }
            LoadingStage::Done => {}
        }

        Ok(self.loading.is_none())
    }

    fn set_loading_stage(&mut self, stage: LoadingStage) {
        if let Some(loading) = &mut self.loading {
            loading.stage = stage;
            loading.message = stage.label().to_string();
        }
    }

    fn load_compare_image(&mut self, path: &Path) -> Result<()> {
        let mut gfs = Guestfs::new()?;
        gfs.add_drive_ro(path)?;
        gfs.launch()?;
        let roots = gfs.inspect_os()?;
        let root = roots.first().ok_or_else(|| anyhow::anyhow!("No OS in compare image"))?;
        gfs.mount_ro(root, "/")?;

        let os_name = gfs
            .inspect_get_product_name(root)
            .unwrap_or_else(|_| "Unknown".to_string());
        let hostname = gfs
            .inspect_get_hostname(root)
            .unwrap_or_else(|_| "Unknown".to_string());
        let packages = gfs.inspect_packages(root).ok();
        let pkg_count = packages.as_ref().map(|p| p.package_count).unwrap_or(0);
        let (critical, high, medium) = {
            let sec = SecurityProfile.inspect(&mut gfs, root).ok();
            let mut c = 0usize;
            let mut h = 0usize;
            let mut m = 0usize;
            if let Some(report) = sec {
                for section in &report.sections {
                    for f in &section.findings {
                        match f.risk_level {
                            Some(crate::cli::profiles::RiskLevel::Critical) => c += 1,
                            Some(crate::cli::profiles::RiskLevel::High) => h += 1,
                            Some(crate::cli::profiles::RiskLevel::Medium) => m += 1,
                            _ => {}
                        }
                    }
                }
            }
            (c, h, m)
        };
        let _ = gfs.shutdown();

        self.compare_summary = Some(CompareSummary {
            path: path.display().to_string(),
            os_name,
            hostname,
            package_count: pkg_count,
            critical,
            high,
            medium,
        });
        Ok(())
    }

    pub fn ensure_view_loaded(&mut self, view: View) -> Result<()> {
        if !view.is_lazy() || self.loaded_views.contains(&view) {
            return Ok(());
        }
        let root = self.inspect_root.clone();
        if let Some(ref mut gfs) = self.guestfs {
            match view {
                View::Packages => {
                    self.packages = gfs.inspect_packages(&root).unwrap_or_else(|_| PackageInfo {
                        manager: "unknown".to_string(),
                        package_count: 0,
                        packages: Vec::new(),
                    });
                }
                View::Kernel => {
                    self.kernel_modules = gfs.inspect_kernel_modules(&root).unwrap_or_default();
                    self.kernel_params = gfs.inspect_kernel_params(&root).unwrap_or_default();
                }
                View::Databases => {
                    self.databases = gfs.inspect_databases(&root).unwrap_or_default();
                }
                View::WebServers => {
                    self.web_servers = gfs.inspect_web_servers(&root).unwrap_or_default();
                }
                View::Analytics | View::Timeline | View::Recommendations | View::Topology | View::Logs => {}
                _ => {}
            }
        }
        self.loaded_views.insert(view);
        Ok(())
    }

    pub fn reload_current_view(&mut self, full: bool) -> Result<()> {
        self.refreshing = true;
        let root = self.inspect_root.clone();
        if full {
            self.loaded_views.clear();
            self.loaded_views.insert(View::Dashboard);
            if let Some(ref mut gfs) = self.guestfs {
                self.packages = gfs.inspect_packages(&root).unwrap_or_else(|_| PackageInfo {
                    manager: "unknown".to_string(),
                    package_count: 0,
                    packages: Vec::new(),
                });
                self.services = gfs.inspect_systemd_services(&root).unwrap_or_default();
                self.security_profile = SecurityProfile.inspect(gfs, &root).ok();
                self.loaded_views.insert(View::Packages);
                self.loaded_views.insert(View::Services);
                self.loaded_views.insert(View::Profiles);
                self.loaded_views.insert(View::Issues);
            }
        } else {
            let view = self.current_view;
            self.loaded_views.remove(&view);
            self.ensure_view_loaded(view)?;
            if let Some(ref mut gfs) = self.guestfs {
                match view {
                    View::Packages => {
                        self.packages = gfs.inspect_packages(&root).unwrap_or_else(|_| PackageInfo {
                            manager: "unknown".to_string(),
                            package_count: 0,
                            packages: Vec::new(),
                        });
                    }
                    View::Services => {
                        self.services = gfs.inspect_systemd_services(&root).unwrap_or_default();
                    }
                    View::Network => {
                        self.network_interfaces = gfs.inspect_network(&root).unwrap_or_default();
                    }
                    View::Issues | View::Profiles => {
                        self.security_profile = SecurityProfile.inspect(gfs, &root).ok();
                        self.compliance_profile = ComplianceProfile.inspect(gfs, &root).ok();
                        self.hardening_profile = HardeningProfile.inspect(gfs, &root).ok();
                    }
                    View::Files => {
                        self.init_file_browser();
                    }
                    _ => {}
                }
            }
            self.loaded_views.insert(view);
        }
        self.refreshing = false;
        self.last_updated = chrono::Local::now();
        Ok(())
    }

    pub fn run_palette_action(&mut self, action: PaletteAction) {
        match action {
            PaletteAction::Goto(v) => {
                self.set_view(v);
            }
            PaletteAction::ExportJson => {
                self.current_view = View::Dashboard;
                self.toggle_export_menu();
                self.select_export_format(super::app::ExportFormat::Json);
            }
            PaletteAction::ExportHtml => {
                self.current_view = View::Issues;
                self.toggle_export_menu();
                self.select_export_format(super::app::ExportFormat::Html);
            }
            PaletteAction::Refresh => {
                let _ = self.reload_current_view(false);
                self.show_notification("View refreshed".to_string());
            }
            PaletteAction::RefreshFull => {
                let _ = self.reload_current_view(true);
                self.show_notification("Full re-inspect complete".to_string());
            }
            PaletteAction::CompareToggle => self.toggle_comparison_mode(),
            PaletteAction::PinView => self.pin_current_view(),
            PaletteAction::Help => self.toggle_help(),
            PaletteAction::Unknown => {
                self.show_notification("Unknown command — try 'goto issues'".to_string());
            }
        }
    }

    pub fn set_view(&mut self, view: View) {
        let _ = self.ensure_view_loaded(view);
        self.current_view = view;
        self.scroll_offset = 0;
        self.selected_index = 0;
        if view == View::Files {
            self.init_file_browser();
        }
    }

    pub fn pin_current_view(&mut self) {
        let name = self.current_view.name().to_string();
        if !self.pinned_views.contains(&name) {
            self.pinned_views.push(name.clone());
            self.show_notification(format!("Pinned {}", self.current_view.title()));
        }
    }

    pub fn cycle_layout_mode(&mut self) {
        self.layout_mode = match self.layout_mode {
            LayoutMode::ListOnly => LayoutMode::SplitDetail,
            LayoutMode::SplitDetail => LayoutMode::DetailFull,
            LayoutMode::DetailFull => LayoutMode::ListOnly,
        };
        let label = match self.layout_mode {
            LayoutMode::ListOnly => "List",
            LayoutMode::SplitDetail => "Split",
            LayoutMode::DetailFull => "Detail",
        };
        self.show_notification(format!("Layout: {}", label));
    }

    pub fn cycle_issue_filter(&mut self) {
        self.issue_filter = match self.issue_filter {
            IssueRiskFilter::All => IssueRiskFilter::Critical,
            IssueRiskFilter::Critical => IssueRiskFilter::High,
            IssueRiskFilter::High => IssueRiskFilter::Medium,
            IssueRiskFilter::Medium => IssueRiskFilter::All,
        };
        self.scroll_offset = 0;
        let label = match self.issue_filter {
            IssueRiskFilter::All => "All",
            IssueRiskFilter::Critical => "Critical",
            IssueRiskFilter::High => "High",
            IssueRiskFilter::Medium => "Medium",
        };
        self.show_notification(format!("Filter: {}", label));
    }

    pub fn tab_titles_ordered(&self) -> Vec<(View, String)> {
        let all = View::all();
        let mut pinned: Vec<View> = Vec::new();
        let mut rest: Vec<View> = Vec::new();
        for v in all {
            if self.pinned_views.iter().any(|p| p == v.name()) {
                pinned.push(*v);
            } else {
                rest.push(*v);
            }
        }
        let ordered: Vec<View> = pinned.into_iter().chain(rest).collect();
        ordered
            .into_iter()
            .map(|v| {
                let title = self.tab_title_for(v);
                (v, title)
            })
            .collect()
    }

    fn tab_title_for(&self, v: View) -> String {
        let icon = if self.config.ui.icon_mode == "ascii" {
            ""
        } else {
            match v {
                View::Dashboard => "📊 ",
                View::Issues => "⚠️ ",
                View::Files => "📂 ",
                _ => "",
            }
        };
        let count = match v {
            View::Network => Some(self.network_interfaces.len()),
            View::Packages => Some(self.packages.package_count),
            View::Services => Some(self.services.len()),
            View::Issues => {
                let (c, h, m) = self.get_risk_summary();
                let t = c + h + m;
                if t > 0 { Some(t) } else { None }
            }
            _ => None,
        };
        if let Some(n) = count {
            format!("{}{} ({})", icon, v.title(), n)
        } else {
            format!("{}{}", icon, v.title())
        }
    }

    pub fn export_migration_bundle(&mut self) -> Result<String> {
        let out = format!(
            r#"{{
  "artifact_version": "1",
  "source": "guestkit-tui",
  "image": {:?},
  "os_hint": {:?},
  "hostname": {:?},
  "architecture": {:?},
  "pipeline": {{
    "enable_pipeline": true,
    "enable_rdp": null
  }}
}}"#,
            self.image_path, self.os_name, self.hostname, self.architecture
        );
        let path = format!("guestkit-migration-{}.json", self.hostname.replace('.', "-"));
        std::fs::write(&path, &out)?;
        self.show_notification(format!("Wrote {}", path));
        Ok(path)
    }
}
