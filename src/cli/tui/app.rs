// SPDX-License-Identifier: LGPL-3.0-or-later
//! TUI application state management

use anyhow::Result;
use chrono::{DateTime, Local};
use guestkit::guestfs::inspect_enhanced::{
    Database, FirewallInfo, HostEntry, LVMInfo, NetworkInterface, Package, PackageInfo,
    RAIDArray, SecurityInfo, SystemService, UserAccount, WebServer,
};
use guestkit::Guestfs;
use std::collections::{HashMap, HashSet};
use std::path::{Path, PathBuf};

use super::config::TuiConfig;
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
    Analytics,
    Timeline,
    Recommendations,
    Topology,
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
    Logs,
    Profiles,
    Files,
}

impl View {
    pub fn title(&self) -> &str {
        match self {
            View::Dashboard => "Dashboard",
            View::Analytics => "Analytics",
            View::Timeline => "Timeline",
            View::Recommendations => "Recommendations",
            View::Topology => "Topology",
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
            View::Logs => "Logs",
            View::Profiles => "Profiles",
            View::Files => "Files",
        }
    }

    pub fn all() -> Vec<View> {
        vec![
            View::Dashboard,
            View::Analytics,
            View::Timeline,
            View::Recommendations,
            View::Topology,
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
            View::Logs,
            View::Profiles,
            View::Files,
        ]
    }
}

/// Sort order for lists
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SortMode {
    Default,
    NameAsc,
    NameDesc,
    VersionAsc,   // For packages
    VersionDesc,
    SizeAsc,      // For storage
    SizeDesc,
    StateAsc,     // For services
    StateDesc,
    UidAsc,       // For users
    UidDesc,
    EnabledFirst, // For services
    DateAsc,      // For users, logs
    DateDesc,
}

impl SortMode {
    /// Get next sort mode for a specific view
    pub fn next(&self, view: &View) -> Self {
        match view {
            View::Packages => match self {
                SortMode::Default => SortMode::NameAsc,
                SortMode::NameAsc => SortMode::NameDesc,
                SortMode::NameDesc => SortMode::VersionAsc,
                SortMode::VersionAsc => SortMode::VersionDesc,
                SortMode::VersionDesc => SortMode::Default,
                _ => SortMode::Default,
            },
            View::Services => match self {
                SortMode::Default => SortMode::NameAsc,
                SortMode::NameAsc => SortMode::NameDesc,
                SortMode::NameDesc => SortMode::StateAsc,
                SortMode::StateAsc => SortMode::StateDesc,
                SortMode::StateDesc => SortMode::EnabledFirst,
                SortMode::EnabledFirst => SortMode::Default,
                _ => SortMode::Default,
            },
            View::Users => match self {
                SortMode::Default => SortMode::NameAsc,
                SortMode::NameAsc => SortMode::NameDesc,
                SortMode::NameDesc => SortMode::UidAsc,
                SortMode::UidAsc => SortMode::UidDesc,
                SortMode::UidDesc => SortMode::Default,
                _ => SortMode::Default,
            },
            View::Storage => match self {
                SortMode::Default => SortMode::NameAsc,
                SortMode::NameAsc => SortMode::NameDesc,
                SortMode::NameDesc => SortMode::SizeAsc,
                SortMode::SizeAsc => SortMode::SizeDesc,
                SortMode::SizeDesc => SortMode::Default,
                _ => SortMode::Default,
            },
            // Other views use simple name sorting
            _ => match self {
                SortMode::Default => SortMode::NameAsc,
                SortMode::NameAsc => SortMode::NameDesc,
                SortMode::NameDesc => SortMode::Default,
                _ => SortMode::Default,
            },
        }
    }

    pub fn label(&self) -> &str {
        match self {
            SortMode::Default => "Default",
            SortMode::NameAsc => "Name ↑",
            SortMode::NameDesc => "Name ↓",
            SortMode::VersionAsc => "Version ↑",
            SortMode::VersionDesc => "Version ↓",
            SortMode::SizeAsc => "Size ↑",
            SortMode::SizeDesc => "Size ↓",
            SortMode::StateAsc => "State ↑",
            SortMode::StateDesc => "State ↓",
            SortMode::UidAsc => "UID ↑",
            SortMode::UidDesc => "UID ↓",
            SortMode::EnabledFirst => "Enabled 1st",
            SortMode::DateAsc => "Date ↑",
            SortMode::DateDesc => "Date ↓",
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
    pub search_results: Vec<usize>, // Filtered item indices
    pub live_filter_enabled: bool,

    // Multi-select state
    pub multi_select_mode: bool,
    pub selected_items: HashSet<usize>, // Set of selected indices
    pub select_all: bool,

    // Quick filters
    pub active_filter: Option<String>,
    pub available_filters: Vec<String>,
    pub scroll_offset: usize,
    pub selected_index: usize,
    pub show_export_menu: bool,
    pub selected_profile_tab: usize,
    pub show_detail: bool,
    pub sort_mode: SortMode,
    pub show_stats_bar: bool,
    pub table_mode: bool, // Toggle between list and table view
    pub comparison_mode: bool, // Toggle comparison/diff view
    pub snapshot_packages: Option<Vec<Package>>, // Snapshot for comparison
    pub snapshot_services: Option<Vec<SystemService>>,
    pub bookmarks: Vec<String>,
    pub search_history: Vec<String>,
    pub notification: Option<(String, u8)>, // (message, ticks_remaining)
    pub last_updated: DateTime<Local>,
    pub refreshing: bool,

    // Mouse state
    pub mouse_position: Option<(u16, u16)>, // (column, row)
    pub last_click_time: Option<std::time::Instant>,
    pub last_click_position: Option<(u16, u16)>,
    pub mouse_down_position: Option<(u16, u16)>, // For drag detection
    pub is_dragging: bool,
    pub drag_start_scroll: usize, // Scroll offset when drag started
    pub hover_index: Option<usize>, // Index of item being hovered
    pub show_context_menu: bool,
    pub context_menu_position: Option<(u16, u16)>,
    pub context_menu_item: Option<usize>,

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
    #[allow(dead_code)]
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

    // Configuration
    #[allow(dead_code)]
    pub config: TuiConfig,

    // File browser state
    pub file_browser: Option<crate::cli::tui::views::files::FileBrowserState>,

    // Guestfs handle for file operations (kept alive for Files view)
    pub guestfs: Option<Guestfs>,
    pub guestfs_root: Option<String>,
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

        // Keep guestfs handle alive for file browser operations
        // Don't shutdown - we'll need it for the Files view

        // Load configuration
        let config = TuiConfig::load();

        // Determine initial view from config
        let current_view = match config.behavior.default_view.as_str() {
            "network" => View::Network,
            "packages" => View::Packages,
            "services" => View::Services,
            "databases" => View::Databases,
            "webservers" => View::WebServers,
            "security" => View::Security,
            "issues" => View::Issues,
            "storage" => View::Storage,
            "users" => View::Users,
            "kernel" => View::Kernel,
            "profiles" => View::Profiles,
            _ => View::Dashboard, // default to Dashboard
        };

        Ok(Self {
            current_view,
            show_help: false,
            searching: false,
            search_query: String::new(),
            search_case_sensitive: config.behavior.search_case_sensitive,
            search_regex_mode: config.behavior.search_regex_mode,
            search_results: Vec::new(),
            live_filter_enabled: true,

            multi_select_mode: false,
            selected_items: HashSet::new(),
            select_all: false,

            active_filter: None,
            available_filters: vec![
                "critical".to_string(),
                "enabled".to_string(),
                "running".to_string(),
                "failed".to_string(),
                "installed".to_string(),
                "dev".to_string(),
            ],
            scroll_offset: 0,
            selected_index: 0,
            show_export_menu: false,
            selected_profile_tab: 0,
            show_detail: false,
            sort_mode: SortMode::Default,
            show_stats_bar: config.ui.show_stats_bar,
            table_mode: false, // Start in list view by default
            comparison_mode: false,
            snapshot_packages: None,
            snapshot_services: None,
            bookmarks: Vec::new(),
            search_history: Vec::new(),
            notification: None,
            last_updated: Local::now(),
            refreshing: false,

            mouse_position: None,
            last_click_time: None,
            last_click_position: None,
            mouse_down_position: None,
            is_dragging: false,
            drag_start_scroll: 0,
            hover_index: None,
            show_context_menu: false,
            context_menu_position: None,
            context_menu_item: None,

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

            config,
            file_browser: None,
            guestfs: Some(guestfs),
            guestfs_root: Some(root.to_string()),
        })
    }

    /// Cleanup guestfs handle on app exit
    pub fn cleanup(&mut self) -> Result<()> {
        if let Some(mut guestfs) = self.guestfs.take() {
            guestfs.shutdown()?;
        }
        Ok(())
    }

    /// Initialize file browser with root directory
    pub fn init_file_browser(&mut self) {
        if self.file_browser.is_none() {
            let mut browser = crate::cli::tui::views::files::FileBrowserState::default();
            // Load initial directory
            if let Some(ref guestfs) = self.guestfs {
                let _ = browser.load_directory(guestfs);
            }
            self.file_browser = Some(browser);
        }
    }

    /// Navigate into selected directory in file browser
    pub fn file_browser_enter(&mut self) {
        if let Some(ref mut browser) = self.file_browser {
            if let Some(_new_path) = browser.enter_directory() {
                // Reload directory after navigation
                if let Some(ref guestfs) = self.guestfs {
                    let _ = browser.load_directory(guestfs);
                }
            }
        }
    }

    /// Navigate to parent directory in file browser
    pub fn file_browser_go_up(&mut self) {
        if let Some(ref mut browser) = self.file_browser {
            browser.go_up();
            // Reload directory after navigation
            if let Some(ref guestfs) = self.guestfs {
                let _ = browser.load_directory(guestfs);
            }
        }
    }

    /// Toggle hidden files in file browser
    pub fn file_browser_toggle_hidden(&mut self) {
        if let Some(ref mut browser) = self.file_browser {
            browser.toggle_hidden();
            // Reload directory to apply filter
            if let Some(ref guestfs) = self.guestfs {
                let _ = browser.load_directory(guestfs);
            }
        }
    }

    /// Move selection up in file browser
    pub fn file_browser_up(&mut self) {
        if let Some(ref mut browser) = self.file_browser {
            browser.move_up();
        }
    }

    /// Move selection down in file browser
    pub fn file_browser_down(&mut self) {
        if let Some(ref mut browser) = self.file_browser {
            let visible_items = 20; // Approximate visible items
            browser.move_down(visible_items);
        }
    }

    pub fn next_view(&mut self) {
        let views = View::all();
        let current_idx = views.iter().position(|v| v == &self.current_view).unwrap_or(0);
        self.current_view = views[(current_idx + 1) % views.len()];
        self.scroll_offset = 0;
        self.selected_index = 0;
        self.show_notification(format!("→ {}", self.current_view.title()));

        // Initialize file browser if entering Files view
        if self.current_view == View::Files {
            self.init_file_browser();
        }
    }

    pub fn previous_view(&mut self) {
        let views = View::all();
        let current_idx = views.iter().position(|v| v == &self.current_view).unwrap_or(0);
        self.current_view = views[(current_idx + views.len() - 1) % views.len()];
        self.scroll_offset = 0;
        self.selected_index = 0;
        self.show_notification(format!("← {}", self.current_view.title()));

        // Initialize file browser if entering Files view
        if self.current_view == View::Files {
            self.init_file_browser();
        }
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
        if self.live_filter_enabled {
            self.update_search_results();
        }
    }

    pub fn search_backspace(&mut self) {
        self.search_query.pop();
        if self.live_filter_enabled {
            self.update_search_results();
        }
    }

    /// Toggle live filtering
    pub fn toggle_live_filter(&mut self) {
        self.live_filter_enabled = !self.live_filter_enabled;
        let status = if self.live_filter_enabled { "enabled" } else { "disabled" };
        self.show_notification(format!("Live filter {}", status));
    }

    /// Update search results based on current query
    pub fn update_search_results(&mut self) {
        if self.search_query.is_empty() {
            self.search_results.clear();
            return;
        }

        let query = if self.search_case_sensitive {
            self.search_query.clone()
        } else {
            self.search_query.to_lowercase()
        };

        self.search_results.clear();

        // Filter based on current view
        match self.current_view {
            View::Packages => {
                // Search in package names (we don't have full package list, so estimate)
                for i in 0..self.packages.package_count {
                    // Placeholder: in real implementation, search actual package names
                    self.search_results.push(i);
                }
            }
            View::Services => {
                for (idx, service) in self.services.iter().enumerate() {
                    let name = if self.search_case_sensitive {
                        service.name.clone()
                    } else {
                        service.name.to_lowercase()
                    };

                    if self.search_regex_mode {
                        if let Ok(re) = regex::Regex::new(&query) {
                            if re.is_match(&name) {
                                self.search_results.push(idx);
                            }
                        }
                    } else if name.contains(&query) {
                        self.search_results.push(idx);
                    }
                }
            }
            View::Network => {
                for (idx, iface) in self.network_interfaces.iter().enumerate() {
                    let name = if self.search_case_sensitive {
                        iface.name.clone()
                    } else {
                        iface.name.to_lowercase()
                    };

                    if name.contains(&query) {
                        self.search_results.push(idx);
                    }
                }
            }
            View::Users => {
                for (idx, user) in self.users.iter().enumerate() {
                    let name = if self.search_case_sensitive {
                        user.username.clone()
                    } else {
                        user.username.to_lowercase()
                    };

                    if name.contains(&query) {
                        self.search_results.push(idx);
                    }
                }
            }
            View::Databases => {
                for (idx, db) in self.databases.iter().enumerate() {
                    let name = if self.search_case_sensitive {
                        db.name.clone()
                    } else {
                        db.name.to_lowercase()
                    };

                    if name.contains(&query) {
                        self.search_results.push(idx);
                    }
                }
            }
            View::WebServers => {
                for (idx, ws) in self.web_servers.iter().enumerate() {
                    let name = if self.search_case_sensitive {
                        ws.name.clone()
                    } else {
                        ws.name.to_lowercase()
                    };

                    if name.contains(&query) {
                        self.search_results.push(idx);
                    }
                }
            }
            View::Kernel => {
                for (idx, module) in self.kernel_modules.iter().enumerate() {
                    let name = if self.search_case_sensitive {
                        module.clone()
                    } else {
                        module.to_lowercase()
                    };

                    if name.contains(&query) {
                        self.search_results.push(idx);
                    }
                }
            }
            _ => {
                // Other views don't support filtering yet
            }
        }

        if !self.search_results.is_empty() {
            self.show_notification(format!("{} matches found", self.search_results.len()));
        } else {
            self.show_notification("No matches found".to_string());
        }
    }

    /// Get filtered items or all items if no filter active
    pub fn get_filtered_count(&self) -> usize {
        if self.searching && self.live_filter_enabled && !self.search_results.is_empty() {
            self.search_results.len()
        } else {
            match self.current_view {
                View::Packages => self.packages.package_count,
                View::Services => self.services.len(),
                View::Network => self.network_interfaces.len(),
                View::Users => self.users.len(),
                View::Databases => self.databases.len(),
                View::WebServers => self.web_servers.len(),
                View::Kernel => self.kernel_modules.len(),
                View::Storage => self.fstab.len(),
                _ => 0,
            }
        }
    }

    pub fn scroll_up(&mut self) {
        // Special handling for Files view
        if self.current_view == View::Files {
            self.file_browser_up();
        } else {
            if self.scroll_offset > 0 {
                self.scroll_offset -= 1;
            }
            if self.selected_index > 0 {
                self.selected_index -= 1;
            }
        }
    }

    pub fn scroll_down(&mut self) {
        // Special handling for Files view
        if self.current_view == View::Files {
            self.file_browser_down();
        } else {
            self.scroll_offset += 1;
            self.selected_index += 1;
        }
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
            View::Packages => {
                // Get filtered/selected packages
                let packages_to_export = self.get_filtered_export_packages();
                json!({
                    "view": "packages",
                    "manager": self.packages.manager,
                    "count": packages_to_export.len(),
                    "total_count": self.packages.package_count,
                    "filtered": packages_to_export.len() != self.packages.packages.len(),
                    "packages": packages_to_export,
                })
            },
            View::Services => {
                let services_to_export = self.get_filtered_export_services();
                json!({
                    "view": "services",
                    "count": services_to_export.len(),
                    "total_count": self.services.len(),
                    "filtered": services_to_export.len() != self.services.len(),
                    "services": services_to_export,
                })
            },
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
            View::Storage => {
                let fstab_to_export = self.get_filtered_export_storage();
                json!({
                    "view": "storage",
                    "fstab": fstab_to_export,
                    "fstab_count": fstab_to_export.len(),
                    "total_fstab_count": self.fstab.len(),
                    "filtered": fstab_to_export.len() != self.fstab.len(),
                    "lvm": self.lvm_info,
                    "raid": self.raid_arrays,
                })
            },
            View::Users => {
                let users_to_export = self.get_filtered_export_users();
                json!({
                    "view": "users",
                    "count": users_to_export.len(),
                    "total_count": self.users.len(),
                    "filtered": users_to_export.len() != self.users.len(),
                    "users": users_to_export,
                })
            },
            View::Kernel => json!({
                "view": "kernel",
                "modules": {
                    "count": self.kernel_modules.len(),
                    "list": self.kernel_modules,
                },
                "parameters": self.kernel_params,
            }),
            View::Analytics => json!({
                "view": "analytics",
                "security_score": {
                    "critical": self.get_risk_summary().0,
                    "high": self.get_risk_summary().1,
                    "medium": self.get_risk_summary().2,
                },
                "package_stats": {
                    "total": self.packages.package_count,
                },
                "service_stats": {
                    "total": self.services.len(),
                    "enabled": self.services.iter().filter(|s| s.enabled).count(),
                },
            }),
            View::Timeline => json!({
                "view": "timeline",
                "system": {
                    "os": self.os_name,
                    "kernel": self.kernel_version,
                },
            }),
            View::Recommendations => json!({
                "view": "recommendations",
                "summary": "System recommendations and optimization suggestions",
            }),
            View::Topology => json!({
                "view": "topology",
                "summary": "System architecture and network topology visualization",
            }),
            View::Logs => json!({
                "view": "logs",
                "summary": "System logs view",
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
        self.sort_mode = self.sort_mode.next(&self.current_view);
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

            // Initialize file browser if jumping to Files view
            if self.current_view == View::Files {
                self.init_file_browser();
            }
        }
    }

    pub fn toggle_stats_bar(&mut self) {
        self.show_stats_bar = !self.show_stats_bar;
        let state = if self.show_stats_bar { "shown" } else { "hidden" };
        self.show_notification(format!("Stats bar {}", state));
    }

    pub fn toggle_table_mode(&mut self) {
        self.table_mode = !self.table_mode;
        let mode = if self.table_mode { "Table" } else { "List" };
        self.show_notification(format!("View mode: {}", mode));
    }

    pub fn toggle_comparison_mode(&mut self) {
        self.comparison_mode = !self.comparison_mode;
        if self.comparison_mode {
            // Take snapshot when entering comparison mode
            self.take_snapshot();
            self.show_notification("Comparison mode enabled - snapshot taken".to_string());
        } else {
            self.show_notification("Comparison mode disabled".to_string());
        }
    }

    pub fn take_snapshot(&mut self) {
        // Take snapshots of current state for comparison
        self.snapshot_packages = Some(self.packages.packages.clone());
        self.snapshot_services = Some(self.services.clone());
        self.show_notification("✓ Snapshot captured".to_string());
    }

    pub fn get_package_diff_stats(&self) -> (usize, usize, usize) {
        // Returns (added, removed, modified)
        if let Some(ref snapshot) = self.snapshot_packages {
            let current_names: std::collections::HashSet<&str> =
                self.packages.packages.iter().map(|p| p.name.as_str()).collect();
            let snapshot_names: std::collections::HashSet<&str> =
                snapshot.iter().map(|p| p.name.as_str()).collect();

            let added = current_names.difference(&snapshot_names).count();
            let removed = snapshot_names.difference(&current_names).count();

            // Check for version changes (modified)
            let mut modified = 0;
            for pkg in &self.packages.packages {
                if let Some(old_pkg) = snapshot.iter().find(|p| p.name == pkg.name) {
                    if old_pkg.version != pkg.version {
                        modified += 1;
                    }
                }
            }

            (added, removed, modified)
        } else {
            (0, 0, 0)
        }
    }

    pub fn get_service_diff_stats(&self) -> (usize, usize, usize) {
        // Returns (started, stopped, changed)
        if let Some(ref snapshot) = self.snapshot_services {
            let mut started = 0;
            let mut stopped = 0;
            let mut changed = 0;

            for svc in &self.services {
                if let Some(old_svc) = snapshot.iter().find(|s| s.name == svc.name) {
                    if old_svc.state != svc.state {
                        if svc.state == "running" && old_svc.state != "running" {
                            started += 1;
                        } else if svc.state != "running" && old_svc.state == "running" {
                            stopped += 1;
                        } else {
                            changed += 1;
                        }
                    }
                } else {
                    // New service
                    if svc.state == "running" {
                        started += 1;
                    }
                }
            }

            // Check for removed services
            for old_svc in snapshot {
                if !self.services.iter().any(|s| s.name == old_svc.name) {
                    if old_svc.state == "running" {
                        stopped += 1;
                    }
                }
            }

            (started, stopped, changed)
        } else {
            (0, 0, 0)
        }
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
    #[allow(dead_code)]
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

    /// Update mouse position for hover effects
    pub fn update_mouse_position(&mut self, col: u16, row: u16) {
        self.mouse_position = Some((col, row));

        // Update hover index based on position
        let content_start_row = if self.show_stats_bar { 8 } else { 7 };

        if !self.show_context_menu && row >= content_start_row {
            let hover_item = (row - content_start_row) as usize + self.scroll_offset;

            // Check if this is a valid item index for current view
            let max_items = match self.current_view {
                View::Packages => self.packages.package_count,
                View::Services => self.services.len(),
                View::Network => self.network_interfaces.len(),
                View::Users => self.users.len(),
                View::Databases => self.databases.len(),
                View::WebServers => self.web_servers.len(),
                View::Storage => self.fstab.len(),
                View::Kernel => self.kernel_modules.len(),
                _ => 0,
            };

            if hover_item < max_items {
                self.hover_index = Some(hover_item);
            } else {
                self.hover_index = None;
            }
        } else {
            self.hover_index = None;
        }
    }

    /// Start mouse drag
    pub fn start_drag(&mut self, col: u16, row: u16) {
        self.mouse_down_position = Some((col, row));
        self.drag_start_scroll = self.scroll_offset;
        self.is_dragging = false; // Will become true on movement
    }

    /// Handle mouse drag movement
    pub fn handle_drag(&mut self, _col: u16, row: u16) {
        if let Some((_, start_row)) = self.mouse_down_position {
            // Check if we've moved enough to consider it a drag
            if (row as i32 - start_row as i32).abs() > 1 {
                self.is_dragging = true;

                // Calculate scroll based on drag distance
                let drag_distance = row as i32 - start_row as i32;
                let new_scroll = (self.drag_start_scroll as i32 - drag_distance).max(0) as usize;

                self.scroll_offset = new_scroll;
            }
        }
    }

    /// End mouse drag
    pub fn end_drag(&mut self) {
        self.mouse_down_position = None;
        self.is_dragging = false;
    }

    /// Show context menu at position
    pub fn show_context_menu_at(&mut self, col: u16, row: u16, item_index: Option<usize>) {
        self.show_context_menu = true;
        self.context_menu_position = Some((col, row));
        self.context_menu_item = item_index;
        self.show_notification("Right-click menu".to_string());
    }

    /// Hide context menu
    pub fn hide_context_menu(&mut self) {
        self.show_context_menu = false;
        self.context_menu_position = None;
        self.context_menu_item = None;
    }

    /// Handle context menu selection
    pub fn handle_context_menu_click(&mut self, row: u16) -> bool {
        if let Some((_, menu_row)) = self.context_menu_position {
            // Context menu items are relative to menu position
            let item_offset = row.saturating_sub(menu_row + 1);

            match item_offset {
                0 => {
                    // Copy to clipboard
                    self.show_notification("Copy (not implemented)".to_string());
                    self.hide_context_menu();
                    return true;
                }
                1 => {
                    // Toggle details
                    self.toggle_detail();
                    self.hide_context_menu();
                    return true;
                }
                2 => {
                    // Export selected
                    self.toggle_export_menu();
                    self.hide_context_menu();
                    return true;
                }
                3 => {
                    // Bookmark
                    if let Some(item_idx) = self.context_menu_item {
                        let bookmark = format!("{} item #{}", self.current_view.title(), item_idx);
                        self.add_bookmark(bookmark);
                        self.hide_context_menu();
                        return true;
                    }
                }
                _ => {}
            }
        }
        false
    }

    /// Handle mouse click at position (left button)
    pub fn handle_mouse_click(&mut self, col: u16, row: u16, terminal_width: u16) -> bool {
        // Hide context menu if clicking outside it
        if self.show_context_menu {
            if !self.is_click_in_context_menu(col, row) {
                self.hide_context_menu();
            } else {
                return self.handle_context_menu_click(row);
            }
        }
        let now = std::time::Instant::now();

        // Check for double-click (within 500ms and same position)
        let is_double_click = if let (Some(last_time), Some(last_pos)) =
            (self.last_click_time, self.last_click_position) {
            now.duration_since(last_time).as_millis() < 500 && last_pos == (col, row)
        } else {
            false
        };

        self.last_click_time = Some(now);
        self.last_click_position = Some((col, row));

        // Handle clicks in different regions

        // Header area (row 0-3): Title and info
        if row <= 3 {
            return false; // No action in header
        }

        // Tab bar area (row 4-6): View tabs
        if row >= 4 && row <= 6 {
            return self.handle_tab_click(col, terminal_width);
        }

        // Help/Export buttons area (check for specific text positions)
        // These are typically in the footer or specific locations
        if self.show_export_menu && row >= 10 && row <= 14 {
            return self.handle_export_menu_click(row);
        }

        // Jump menu
        if self.show_jump_menu && row >= 8 {
            return self.handle_jump_menu_click(row);
        }

        // Content area: List items
        if row > 6 {
            if is_double_click {
                self.toggle_detail();
                return true;
            } else {
                return self.handle_content_click(row);
            }
        }

        false
    }

    /// Handle click on view tabs with precise detection
    fn handle_tab_click(&mut self, col: u16, terminal_width: u16) -> bool {
        let views = View::all();
        let tab_width = terminal_width / views.len() as u16;

        let clicked_index = (col / tab_width) as usize;

        if clicked_index < views.len() {
            self.current_view = views[clicked_index];
            self.scroll_offset = 0;
            self.selected_index = 0;
            self.show_notification(format!("→ {}", views[clicked_index].title()));
            return true;
        }
        false
    }

    /// Handle click in content area (list items)
    fn handle_content_click(&mut self, row: u16) -> bool {
        // Content starts after header (row 0-3) and tabs (row 4-6)
        // Account for stats bar if shown
        let content_start_row = if self.show_stats_bar { 8 } else { 7 };

        if row >= content_start_row {
            let clicked_item = (row - content_start_row) as usize + self.scroll_offset;

            // Set selected index based on current view's content
            match self.current_view {
                View::Packages => {
                    if clicked_item < self.packages.package_count {
                        self.selected_index = clicked_item;
                        return true;
                    }
                }
                View::Services => {
                    if clicked_item < self.services.len() {
                        self.selected_index = clicked_item;
                        return true;
                    }
                }
                View::Network => {
                    if clicked_item < self.network_interfaces.len() {
                        self.selected_index = clicked_item;
                        return true;
                    }
                }
                View::Users => {
                    if clicked_item < self.users.len() {
                        self.selected_index = clicked_item;
                        return true;
                    }
                }
                View::Databases => {
                    if clicked_item < self.databases.len() {
                        self.selected_index = clicked_item;
                        return true;
                    }
                }
                View::WebServers => {
                    if clicked_item < self.web_servers.len() {
                        self.selected_index = clicked_item;
                        return true;
                    }
                }
                View::Storage => {
                    if clicked_item < self.fstab.len() {
                        self.selected_index = clicked_item;
                        return true;
                    }
                }
                View::Kernel => {
                    if clicked_item < self.kernel_modules.len() {
                        self.selected_index = clicked_item;
                        return true;
                    }
                }
                _ => {}
            }
        }
        false
    }

    /// Handle click in export menu
    fn handle_export_menu_click(&mut self, row: u16) -> bool {
        use ExportFormat;

        // Export menu shows format options, typically around row 11-14
        if row >= 11 && row <= 14 {
            let option_index = (row - 11) as usize;
            let format = match option_index {
                0 => Some(ExportFormat::Json),
                1 => Some(ExportFormat::Yaml),
                2 => Some(ExportFormat::Html),
                3 => Some(ExportFormat::Pdf),
                _ => None,
            };

            if let Some(fmt) = format {
                self.select_export_format(fmt);
                return true;
            }
        }
        false
    }

    /// Handle click in jump menu
    fn handle_jump_menu_click(&mut self, row: u16) -> bool {
        // Jump menu items start around row 9
        let item_start_row = 9;

        if row >= item_start_row {
            let clicked_item = (row - item_start_row) as usize;
            let filtered_views = self.get_filtered_views();

            if clicked_item < filtered_views.len() {
                self.jump_selected_index = clicked_item;
                self.jump_menu_select();
                return true;
            }
        }
        false
    }

    /// Check if mouse is hovering over a clickable element
    #[allow(dead_code)]
    pub fn is_hovering_clickable(&self, _col: u16, row: u16) -> bool {
        // Tabs area
        if row >= 4 && row <= 6 {
            return true;
        }

        // Content items
        let content_start_row = if self.show_stats_bar { 8 } else { 7 };
        if row >= content_start_row {
            return true;
        }

        // Export menu
        if self.show_export_menu && row >= 11 && row <= 14 {
            return true;
        }

        // Jump menu
        if self.show_jump_menu && row >= 9 {
            return true;
        }

        // Context menu
        if self.show_context_menu {
            return self.is_click_in_context_menu(_col, row);
        }

        false
    }

    /// Check if click is inside context menu
    fn is_click_in_context_menu(&self, col: u16, row: u16) -> bool {
        if let Some((menu_col, menu_row)) = self.context_menu_position {
            // Context menu is approximately 20 chars wide and 5 rows tall
            let menu_width = 20;
            let menu_height = 5;

            col >= menu_col
                && col < menu_col + menu_width
                && row >= menu_row
                && row < menu_row + menu_height
        } else {
            false
        }
    }

    /// Handle right-click at position
    pub fn handle_right_click(&mut self, col: u16, row: u16) -> bool {
        // Close existing context menu first
        if self.show_context_menu {
            self.hide_context_menu();
            return true;
        }

        // Determine what was right-clicked
        let content_start_row = if self.show_stats_bar { 8 } else { 7 };

        if row >= content_start_row {
            let clicked_item = (row - content_start_row) as usize + self.scroll_offset;

            // Check if it's a valid item
            let max_items = match self.current_view {
                View::Packages => self.packages.package_count,
                View::Services => self.services.len(),
                View::Network => self.network_interfaces.len(),
                View::Users => self.users.len(),
                View::Databases => self.databases.len(),
                View::WebServers => self.web_servers.len(),
                View::Storage => self.fstab.len(),
                View::Kernel => self.kernel_modules.len(),
                _ => 0,
            };

            if clicked_item < max_items {
                // Show context menu for this item
                self.show_context_menu_at(col, row, Some(clicked_item));
                return true;
            }
        }

        // Right-click on other areas shows general context menu
        self.show_context_menu_at(col, row, None);
        true
    }

    /// Handle middle-click at position
    pub fn handle_middle_click(&mut self, _col: u16, row: u16) -> bool {
        // Middle-click for quick actions
        let content_start_row = if self.show_stats_bar { 8 } else { 7 };

        if row >= content_start_row {
            let clicked_item = (row - content_start_row) as usize + self.scroll_offset;

            // Set selected and toggle detail in one action
            match self.current_view {
                View::Packages => {
                    if clicked_item < self.packages.package_count {
                        self.selected_index = clicked_item;
                        self.toggle_detail();
                        return true;
                    }
                }
                View::Services => {
                    if clicked_item < self.services.len() {
                        self.selected_index = clicked_item;
                        self.toggle_detail();
                        return true;
                    }
                }
                View::Network => {
                    if clicked_item < self.network_interfaces.len() {
                        self.selected_index = clicked_item;
                        self.toggle_detail();
                        return true;
                    }
                }
                _ => {}
            }
        }

        false
    }

    /// Toggle multi-select mode
    pub fn toggle_multi_select(&mut self) {
        self.multi_select_mode = !self.multi_select_mode;
        if !self.multi_select_mode {
            self.selected_items.clear();
            self.select_all = false;
        }
        let status = if self.multi_select_mode { "ON" } else { "OFF" };
        self.show_notification(format!("Multi-select: {}", status));
    }

    /// Toggle selection of current item
    pub fn toggle_item_selection(&mut self) {
        if !self.multi_select_mode {
            self.multi_select_mode = true;
            self.show_notification("Multi-select: ON".to_string());
        }

        let idx = self.selected_index;
        if self.selected_items.contains(&idx) {
            self.selected_items.remove(&idx);
        } else {
            self.selected_items.insert(idx);
        }

        self.show_notification(format!("{} items selected", self.selected_items.len()));
    }

    /// Select all items in current view
    pub fn select_all_items(&mut self) {
        if !self.multi_select_mode {
            self.multi_select_mode = true;
        }

        let max_items = self.get_filtered_count();

        if self.select_all {
            // Deselect all
            self.selected_items.clear();
            self.select_all = false;
            self.show_notification("Deselected all".to_string());
        } else {
            // Select all
            self.selected_items.clear();
            for i in 0..max_items {
                self.selected_items.insert(i);
            }
            self.select_all = true;
            self.show_notification(format!("Selected all {} items", max_items));
        }
    }

    /// Clear all selections
    pub fn clear_selection(&mut self) {
        self.selected_items.clear();
        self.select_all = false;
        self.multi_select_mode = false;
        self.show_notification("Selection cleared".to_string());
    }

    /// Check if item is selected
    pub fn is_item_selected(&self, index: usize) -> bool {
        self.selected_items.contains(&index)
    }

    /// Get count of selected items
    pub fn get_selected_count(&self) -> usize {
        self.selected_items.len()
    }

    /// Perform bulk action on selected items
    pub fn bulk_action(&mut self, action: &str) {
        if self.selected_items.is_empty() {
            self.show_notification("No items selected".to_string());
            return;
        }

        match action {
            "export" => {
                self.toggle_export_menu();
                self.show_notification(format!("Exporting {} items", self.selected_items.len()));
            }
            "bookmark" => {
                // Collect bookmarks first to avoid borrow conflict
                let bookmarks: Vec<String> = self.selected_items.iter()
                    .map(|idx| format!("{} item #{}", self.current_view.title(), idx))
                    .collect();
                let count = bookmarks.len();
                for bookmark in bookmarks {
                    self.add_bookmark(bookmark);
                }
                self.show_notification(format!("Bookmarked {} items", count));
                self.clear_selection();
            }
            "delete" => {
                // Placeholder for delete action
                self.show_notification(format!("Delete {} items (not implemented)", self.selected_items.len()));
            }
            _ => {
                self.show_notification(format!("Unknown action: {}", action));
            }
        }
    }

    /// Apply quick filter
    pub fn apply_filter(&mut self, filter: &str) {
        if self.active_filter.as_deref() == Some(filter) {
            // Toggle off if same filter
            self.active_filter = None;
            self.show_notification(format!("Filter '{}' removed", filter));
        } else {
            self.active_filter = Some(filter.to_string());
            self.show_notification(format!("Filter: {}", filter));
            self.update_filtered_items();
        }
    }

    /// Update items based on active filter
    fn update_filtered_items(&mut self) {
        if self.active_filter.is_none() {
            return;
        }

        let filter = self.active_filter.as_ref().unwrap();

        match filter.as_str() {
            "critical" => {
                // Filter critical security issues
                self.current_view = View::Issues;
                self.scroll_offset = 0;
            }
            "enabled" => {
                // Filter enabled services
                if self.current_view == View::Services {
                    // In real implementation, would filter the list
                    self.show_notification(format!("{} enabled services",
                        self.services.iter().filter(|s| s.enabled).count()));
                }
            }
            "running" => {
                // Filter running services
                if self.current_view == View::Services {
                    self.show_notification(format!("{} running services",
                        self.services.iter().filter(|s| s.state == "running").count()));
                }
            }
            "failed" => {
                // Filter failed services
                if self.current_view == View::Services {
                    self.show_notification(format!("{} failed services",
                        self.services.iter().filter(|s| s.state == "failed").count()));
                }
            }
            "dev" => {
                // Filter development packages
                if self.current_view == View::Packages {
                    let dev_count = self.packages.packages.iter()
                        .filter(|p| p.name.contains("devel") || p.name.contains("-dev"))
                        .count();
                    self.show_notification(format!("{} dev packages", dev_count));
                }
            }
            _ => {}
        }
    }

    /// Cycle through available filters
    pub fn cycle_filter(&mut self) {
        if self.available_filters.is_empty() {
            return;
        }

        // Clone filter name to avoid borrow conflict
        let next_filter = if let Some(current) = &self.active_filter {
            if let Some(idx) = self.available_filters.iter().position(|f| f == current) {
                let next_idx = (idx + 1) % self.available_filters.len();
                self.available_filters[next_idx].clone()
            } else {
                self.available_filters[0].clone()
            }
        } else {
            self.available_filters[0].clone()
        };

        self.apply_filter(&next_filter);
    }

    /// Get active filter label for display
    pub fn get_filter_label(&self) -> Option<String> {
        self.active_filter.as_ref().map(|f| {
            match f.as_str() {
                "critical" => "🔴 Critical",
                "enabled" => "✅ Enabled",
                "running" => "▶️  Running",
                "failed" => "❌ Failed",
                "installed" => "📦 Installed",
                "dev" => "🔧 Dev Packages",
                _ => f.as_str(),
            }.to_string()
        })
    }

    /// Get sorted package indices based on current sort mode
    pub fn get_sorted_package_indices(&self) -> Vec<usize> {
        let mut indices: Vec<usize> = (0..self.packages.packages.len()).collect();

        match self.sort_mode {
            SortMode::NameAsc => {
                indices.sort_by(|&a, &b| {
                    self.packages.packages[a].name.to_lowercase()
                        .cmp(&self.packages.packages[b].name.to_lowercase())
                });
            }
            SortMode::NameDesc => {
                indices.sort_by(|&a, &b| {
                    self.packages.packages[b].name.to_lowercase()
                        .cmp(&self.packages.packages[a].name.to_lowercase())
                });
            }
            SortMode::VersionAsc => {
                indices.sort_by(|&a, &b| {
                    self.packages.packages[a].version.to_lowercase()
                        .cmp(&self.packages.packages[b].version.to_lowercase())
                });
            }
            SortMode::VersionDesc => {
                indices.sort_by(|&a, &b| {
                    self.packages.packages[b].version.to_lowercase()
                        .cmp(&self.packages.packages[a].version.to_lowercase())
                });
            }
            _ => {} // Default order
        }

        indices
    }

    /// Get sorted service indices based on current sort mode
    pub fn get_sorted_service_indices(&self) -> Vec<usize> {
        let mut indices: Vec<usize> = (0..self.services.len()).collect();

        match self.sort_mode {
            SortMode::NameAsc => {
                indices.sort_by(|&a, &b| {
                    self.services[a].name.to_lowercase()
                        .cmp(&self.services[b].name.to_lowercase())
                });
            }
            SortMode::NameDesc => {
                indices.sort_by(|&a, &b| {
                    self.services[b].name.to_lowercase()
                        .cmp(&self.services[a].name.to_lowercase())
                });
            }
            SortMode::StateAsc => {
                indices.sort_by(|&a, &b| {
                    self.services[a].state.cmp(&self.services[b].state)
                });
            }
            SortMode::StateDesc => {
                indices.sort_by(|&a, &b| {
                    self.services[b].state.cmp(&self.services[a].state)
                });
            }
            SortMode::EnabledFirst => {
                indices.sort_by(|&a, &b| {
                    // Enabled first (true > false in reverse)
                    self.services[b].enabled.cmp(&self.services[a].enabled)
                        .then(self.services[a].name.to_lowercase()
                            .cmp(&self.services[b].name.to_lowercase()))
                });
            }
            _ => {} // Default order
        }

        indices
    }

    /// Get sorted user indices based on current sort mode
    pub fn get_sorted_user_indices(&self) -> Vec<usize> {
        let mut indices: Vec<usize> = (0..self.users.len()).collect();

        match self.sort_mode {
            SortMode::NameAsc => {
                indices.sort_by(|&a, &b| {
                    self.users[a].username.to_lowercase()
                        .cmp(&self.users[b].username.to_lowercase())
                });
            }
            SortMode::NameDesc => {
                indices.sort_by(|&a, &b| {
                    self.users[b].username.to_lowercase()
                        .cmp(&self.users[a].username.to_lowercase())
                });
            }
            SortMode::UidAsc => {
                indices.sort_by(|&a, &b| {
                    self.users[a].uid.cmp(&self.users[b].uid)
                });
            }
            SortMode::UidDesc => {
                indices.sort_by(|&a, &b| {
                    self.users[b].uid.cmp(&self.users[a].uid)
                });
            }
            _ => {} // Default order
        }

        indices
    }

    /// Get sorted storage (fstab) indices based on current sort mode
    pub fn get_sorted_storage_indices(&self) -> Vec<usize> {
        let mut indices: Vec<usize> = (0..self.fstab.len()).collect();

        match self.sort_mode {
            SortMode::NameAsc => {
                indices.sort_by(|&a, &b| {
                    self.fstab[a].0.to_lowercase()
                        .cmp(&self.fstab[b].0.to_lowercase())
                });
            }
            SortMode::NameDesc => {
                indices.sort_by(|&a, &b| {
                    self.fstab[b].0.to_lowercase()
                        .cmp(&self.fstab[a].0.to_lowercase())
                });
            }
            SortMode::SizeAsc => {
                // For fstab, sort by mountpoint instead of size
                indices.sort_by(|&a, &b| {
                    self.fstab[a].1.to_lowercase()
                        .cmp(&self.fstab[b].1.to_lowercase())
                });
            }
            SortMode::SizeDesc => {
                // For fstab, sort by mountpoint reverse
                indices.sort_by(|&a, &b| {
                    self.fstab[b].1.to_lowercase()
                        .cmp(&self.fstab[a].1.to_lowercase())
                });
            }
            _ => {} // Default order
        }

        indices
    }

    /// Get filtered/selected packages for export
    fn get_filtered_export_packages(&self) -> Vec<&Package> {
        // If multi-select mode with items selected, export only selected
        if self.multi_select_mode && !self.selected_items.is_empty() {
            return self.selected_items
                .iter()
                .filter_map(|&idx| self.packages.packages.get(idx))
                .collect();
        }

        // Get sorted and filtered indices (respects search and filters)
        let sorted_indices = self.get_sorted_package_indices();

        // Apply search filter if active
        let filtered_indices: Vec<usize> = if self.is_searching() && !self.search_query.is_empty() {
            sorted_indices
                .into_iter()
                .filter(|&idx| {
                    let pkg = &self.packages.packages[idx];
                    pkg.name.to_lowercase().contains(&self.search_query.to_lowercase())
                        || pkg.version.contains(&self.search_query)
                })
                .collect()
        } else {
            sorted_indices
        };

        // Return packages in filtered order
        filtered_indices
            .iter()
            .filter_map(|&idx| self.packages.packages.get(idx))
            .collect()
    }

    /// Get filtered/selected services for export
    fn get_filtered_export_services(&self) -> Vec<&SystemService> {
        // If multi-select mode with items selected, export only selected
        if self.multi_select_mode && !self.selected_items.is_empty() {
            return self.selected_items
                .iter()
                .filter_map(|&idx| self.services.get(idx))
                .collect();
        }

        // Get sorted and filtered indices
        let sorted_indices = self.get_sorted_service_indices();

        // Apply search filter if active
        let filtered_indices: Vec<usize> = if self.is_searching() && !self.search_query.is_empty() {
            sorted_indices
                .into_iter()
                .filter(|&idx| {
                    let svc = &self.services[idx];
                    svc.name.to_lowercase().contains(&self.search_query.to_lowercase())
                        || svc.state.to_lowercase().contains(&self.search_query.to_lowercase())
                })
                .collect()
        } else {
            sorted_indices
        };

        // Return services in filtered order
        filtered_indices
            .iter()
            .filter_map(|&idx| self.services.get(idx))
            .collect()
    }

    /// Get filtered/selected users for export
    fn get_filtered_export_users(&self) -> Vec<&UserAccount> {
        // If multi-select mode with items selected, export only selected
        if self.multi_select_mode && !self.selected_items.is_empty() {
            return self.selected_items
                .iter()
                .filter_map(|&idx| self.users.get(idx))
                .collect();
        }

        // Get sorted and filtered indices
        let sorted_indices = self.get_sorted_user_indices();

        // Apply search filter if active
        let filtered_indices: Vec<usize> = if self.is_searching() && !self.search_query.is_empty() {
            sorted_indices
                .into_iter()
                .filter(|&idx| {
                    let user = &self.users[idx];
                    user.username.to_lowercase().contains(&self.search_query.to_lowercase())
                        || user.uid.contains(&self.search_query)
                        || user.shell.to_lowercase().contains(&self.search_query.to_lowercase())
                        || user.home.to_lowercase().contains(&self.search_query.to_lowercase())
                })
                .collect()
        } else {
            sorted_indices
        };

        // Return users in filtered order
        filtered_indices
            .iter()
            .filter_map(|&idx| self.users.get(idx))
            .collect()
    }

    /// Get filtered/selected storage entries for export
    fn get_filtered_export_storage(&self) -> Vec<&(String, String, String)> {
        // If multi-select mode with items selected, export only selected
        if self.multi_select_mode && !self.selected_items.is_empty() {
            return self.selected_items
                .iter()
                .filter_map(|&idx| self.fstab.get(idx))
                .collect();
        }

        // Get sorted and filtered indices
        let sorted_indices = self.get_sorted_storage_indices();

        // Apply search filter if active
        let filtered_indices: Vec<usize> = if self.is_searching() && !self.search_query.is_empty() {
            sorted_indices
                .into_iter()
                .filter(|&idx| {
                    let (device, mountpoint, fstype) = &self.fstab[idx];
                    device.to_lowercase().contains(&self.search_query.to_lowercase())
                        || mountpoint.to_lowercase().contains(&self.search_query.to_lowercase())
                        || fstype.to_lowercase().contains(&self.search_query.to_lowercase())
                })
                .collect()
        } else {
            sorted_indices
        };

        // Return fstab entries in filtered order
        filtered_indices
            .iter()
            .filter_map(|&idx| self.fstab.get(idx))
            .collect()
    }
}
