// SPDX-License-Identifier: LGPL-3.0-or-later
//! Enhanced inspection operations for comprehensive guest analysis

use crate::core::Result;
use crate::guestfs::Guestfs;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Network interface information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetworkInterface {
    pub name: String,
    pub ip_address: Vec<String>,
    pub mac_address: String,
    pub dhcp: bool,
}

/// User account information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserAccount {
    pub username: String,
    pub uid: String,
    pub gid: String,
    pub home: String,
    pub shell: String,
}

/// System service information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SystemService {
    pub name: String,
    pub enabled: bool,
    pub state: String,
}

/// LVM information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LVMInfo {
    pub physical_volumes: Vec<String>,
    pub volume_groups: Vec<String>,
    pub logical_volumes: Vec<String>,
}

/// Boot configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BootConfig {
    pub bootloader: String,
    pub default_entry: String,
    pub timeout: String,
    pub kernel_cmdline: String,
}

/// Certificate information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Certificate {
    pub path: String,
    pub subject: String,
    pub issuer: String,
    pub expiry: String,
}

impl Guestfs {
    /// Inspect network configuration
    pub fn inspect_network(&mut self, root: &str) -> Result<Vec<NetworkInterface>> {
        let mut interfaces = Vec::new();

        if self.mount(root, "/").is_err() {
            return Ok(interfaces);
        }

        // Try different network configuration locations

        // 1. Check /etc/network/interfaces (Debian/Ubuntu)
        if let Ok(content) = self.cat("/etc/network/interfaces") {
            interfaces.extend(self.parse_debian_interfaces(&content));
        }

        // 2. Check /etc/sysconfig/network-scripts/ (RHEL/CentOS/Fedora)
        if let Ok(files) = self.ls("/etc/sysconfig/network-scripts") {
            for file in files.iter().filter(|f| f.starts_with("ifcfg-")) {
                let path = format!("/etc/sysconfig/network-scripts/{}", file);
                if let Ok(content) = self.cat(&path) {
                    if let Some(iface) = self.parse_rhel_interface(&content, file) {
                        interfaces.push(iface);
                    }
                }
            }
        }

        // 3. Check netplan (newer Ubuntu)
        if self.is_dir("/etc/netplan").unwrap_or(false) {
            if let Ok(files) = self.ls("/etc/netplan") {
                for file in files
                    .iter()
                    .filter(|f| f.ends_with(".yaml") || f.ends_with(".yml"))
                {
                    let path = format!("/etc/netplan/{}", file);
                    if let Ok(_content) = self.cat(&path) {
                        // Basic netplan detection
                        // Full YAML parsing would require additional dependencies
                    }
                }
            }
        }

        self.umount("/").ok();
        Ok(interfaces)
    }

    fn parse_debian_interfaces(&self, content: &str) -> Vec<NetworkInterface> {
        let mut interfaces = Vec::new();
        let mut current_iface: Option<NetworkInterface> = None;

        for line in content.lines() {
            let line = line.trim();
            if line.starts_with("iface ") {
                if let Some(iface) = current_iface.take() {
                    interfaces.push(iface);
                }

                let parts: Vec<&str> = line.split_whitespace().collect();
                if parts.len() >= 4 {
                    current_iface = Some(NetworkInterface {
                        name: parts[1].to_string(),
                        ip_address: Vec::new(),
                        mac_address: String::new(),
                        dhcp: parts[3] == "dhcp",
                    });
                }
            } else if line.starts_with("address ") && current_iface.is_some() {
                if let Some(ref mut iface) = current_iface {
                    let addr = line.split_whitespace().nth(1).unwrap_or("").to_string();
                    iface.ip_address.push(addr);
                }
            }
        }

        if let Some(iface) = current_iface {
            interfaces.push(iface);
        }

        interfaces
    }

    fn parse_rhel_interface(&self, content: &str, filename: &str) -> Option<NetworkInterface> {
        let mut iface = NetworkInterface {
            name: filename
                .strip_prefix("ifcfg-")
                .unwrap_or(filename)
                .to_string(),
            ip_address: Vec::new(),
            mac_address: String::new(),
            dhcp: false,
        };

        for line in content.lines() {
            let line = line.trim();
            if let Some((key, value)) = line.split_once('=') {
                let value = value.trim_matches('"');
                match key {
                    "BOOTPROTO" => iface.dhcp = value == "dhcp",
                    "IPADDR" => iface.ip_address.push(value.to_string()),
                    "HWADDR" | "MACADDR" => iface.mac_address = value.to_string(),
                    _ => {}
                }
            }
        }

        Some(iface)
    }

    /// Get DNS configuration
    pub fn inspect_dns(&mut self, root: &str) -> Result<Vec<String>> {
        let mut dns_servers = Vec::new();

        if self.mount(root, "/").is_err() {
            return Ok(dns_servers);
        }

        if let Ok(content) = self.cat("/etc/resolv.conf") {
            for line in content.lines() {
                let line = line.trim();
                if line.starts_with("nameserver ") {
                    if let Some(server) = line.split_whitespace().nth(1) {
                        dns_servers.push(server.to_string());
                    }
                }
            }
        }

        self.umount("/").ok();
        Ok(dns_servers)
    }

    /// List user accounts
    pub fn inspect_users(&mut self, root: &str) -> Result<Vec<UserAccount>> {
        let mut users = Vec::new();

        if self.mount(root, "/").is_err() {
            return Ok(users);
        }

        if let Ok(content) = self.cat("/etc/passwd") {
            for line in content.lines() {
                let parts: Vec<&str> = line.split(':').collect();
                if parts.len() >= 7 {
                    users.push(UserAccount {
                        username: parts[0].to_string(),
                        uid: parts[2].to_string(),
                        gid: parts[3].to_string(),
                        home: parts[5].to_string(),
                        shell: parts[6].to_string(),
                    });
                }
            }
        }

        self.umount("/").ok();
        Ok(users)
    }

    /// Get SSH configuration
    pub fn inspect_ssh_config(&mut self, root: &str) -> Result<HashMap<String, String>> {
        let mut config = HashMap::new();

        if self.mount(root, "/").is_err() {
            return Ok(config);
        }

        if let Ok(content) = self.cat("/etc/ssh/sshd_config") {
            for line in content.lines() {
                let line = line.trim();
                if line.is_empty() || line.starts_with('#') {
                    continue;
                }
                if let Some((key, value)) = line.split_once(char::is_whitespace) {
                    config.insert(key.to_string(), value.trim().to_string());
                }
            }
        }

        self.umount("/").ok();
        Ok(config)
    }

    /// Check SELinux status
    pub fn inspect_selinux(&mut self, root: &str) -> Result<String> {
        if self.mount(root, "/").is_err() {
            return Ok("unknown".to_string());
        }

        let status = if let Ok(content) = self.cat("/etc/selinux/config") {
            let mut selinux_mode = "unknown".to_string();
            for line in content.lines() {
                if line.starts_with("SELINUX=") {
                    selinux_mode = line.split('=').nth(1).unwrap_or("unknown").to_string();
                    break;
                }
            }
            selinux_mode
        } else {
            "disabled".to_string()
        };

        self.umount("/").ok();
        Ok(status)
    }

    /// List systemd services
    pub fn inspect_systemd_services(&mut self, root: &str) -> Result<Vec<SystemService>> {
        let mut services = Vec::new();

        if self.mount(root, "/").is_err() {
            return Ok(services);
        }

        // Check for enabled services in /etc/systemd/system
        if let Ok(links) = self.ls("/etc/systemd/system/multi-user.target.wants") {
            for link in links {
                services.push(SystemService {
                    name: link.strip_suffix(".service").unwrap_or(&link).to_string(),
                    enabled: true,
                    state: "enabled".to_string(),
                });
            }
        }

        self.umount("/").ok();
        Ok(services)
    }

    /// Get timezone information
    pub fn inspect_timezone(&mut self, root: &str) -> Result<String> {
        if self.mount(root, "/").is_err() {
            return Ok("unknown".to_string());
        }

        let timezone = if let Ok(content) = self.cat("/etc/timezone") {
            content.trim().to_string()
        } else if self.is_symlink("/etc/localtime").unwrap_or(false) {
            if let Ok(target) = self.readlink("/etc/localtime") {
                // Extract timezone from path like /usr/share/zoneinfo/America/New_York
                target
                    .strip_prefix("/usr/share/zoneinfo/")
                    .unwrap_or(&target)
                    .to_string()
            } else {
                "unknown".to_string()
            }
        } else {
            "unknown".to_string()
        };

        self.umount("/").ok();
        Ok(timezone)
    }

    /// Get locale information
    pub fn inspect_locale(&mut self, root: &str) -> Result<String> {
        if self.mount(root, "/").is_err() {
            return Ok("unknown".to_string());
        }

        let locale = if let Ok(content) = self.cat("/etc/locale.conf") {
            let mut lang = "unknown".to_string();
            for line in content.lines() {
                if line.starts_with("LANG=") {
                    lang = line.split('=').nth(1).unwrap_or("unknown").to_string();
                    break;
                }
            }
            lang
        } else if let Ok(content) = self.cat("/etc/default/locale") {
            let mut lang = "unknown".to_string();
            for line in content.lines() {
                if line.starts_with("LANG=") {
                    lang = line
                        .split('=')
                        .nth(1)
                        .unwrap_or("unknown")
                        .trim_matches('"')
                        .to_string();
                    break;
                }
            }
            lang
        } else {
            "unknown".to_string()
        };

        self.umount("/").ok();
        Ok(locale)
    }

    /// Detect LVM configuration
    pub fn inspect_lvm(&mut self, _root: &str) -> Result<LVMInfo> {
        let mut lvm_info = LVMInfo {
            physical_volumes: Vec::new(),
            volume_groups: Vec::new(),
            logical_volumes: Vec::new(),
        };

        // Check if LVM is present
        if let Ok(pvs) = self.pvs() {
            lvm_info.physical_volumes = pvs;
        }

        if let Ok(vgs) = self.vgs() {
            lvm_info.volume_groups = vgs;
        }

        if let Ok(lvs) = self.lvs() {
            lvm_info.logical_volumes = lvs;
        }

        Ok(lvm_info)
    }

    /// Detect cloud-init
    pub fn inspect_cloud_init(&mut self, root: &str) -> Result<bool> {
        if self.mount(root, "/").is_err() {
            return Ok(false);
        }

        let has_cloud_init = self.exists("/etc/cloud/cloud.cfg").unwrap_or(false)
            || self.exists("/usr/bin/cloud-init").unwrap_or(false);

        self.umount("/").ok();
        Ok(has_cloud_init)
    }

    /// Get language runtime versions
    pub fn inspect_runtimes(&mut self, root: &str) -> Result<HashMap<String, String>> {
        let mut runtimes = HashMap::new();

        if self.mount(root, "/").is_err() {
            return Ok(runtimes);
        }

        // Python
        for py in &["python3", "python", "python2"] {
            let path = format!("/usr/bin/{}", py);
            if self.exists(&path).unwrap_or(false) {
                // Try to get version from symlink or binary
                runtimes.insert(py.to_string(), "installed".to_string());
            }
        }

        // Node.js
        if self.exists("/usr/bin/node").unwrap_or(false) {
            runtimes.insert("nodejs".to_string(), "installed".to_string());
        }

        // Ruby
        if self.exists("/usr/bin/ruby").unwrap_or(false) {
            runtimes.insert("ruby".to_string(), "installed".to_string());
        }

        // Java
        if self.exists("/usr/bin/java").unwrap_or(false) {
            runtimes.insert("java".to_string(), "installed".to_string());
        }

        // Go
        if self.exists("/usr/bin/go").unwrap_or(false) {
            runtimes.insert("go".to_string(), "installed".to_string());
        }

        // Perl
        if self.exists("/usr/bin/perl").unwrap_or(false) {
            runtimes.insert("perl".to_string(), "installed".to_string());
        }

        self.umount("/").ok();
        Ok(runtimes)
    }

    /// Detect container runtimes
    pub fn inspect_container_runtimes(&mut self, root: &str) -> Result<Vec<String>> {
        let mut runtimes = Vec::new();

        if self.mount(root, "/").is_err() {
            return Ok(runtimes);
        }

        if self.exists("/usr/bin/docker").unwrap_or(false) {
            runtimes.push("docker".to_string());
        }

        if self.exists("/usr/bin/podman").unwrap_or(false) {
            runtimes.push("podman".to_string());
        }

        if self.exists("/usr/bin/containerd").unwrap_or(false) {
            runtimes.push("containerd".to_string());
        }

        if self.exists("/usr/bin/crio").unwrap_or(false)
            || self.exists("/usr/bin/cri-o").unwrap_or(false)
        {
            runtimes.push("cri-o".to_string());
        }

        self.umount("/").ok();
        Ok(runtimes)
    }

    /// List cron jobs
    pub fn inspect_cron(&mut self, root: &str) -> Result<Vec<String>> {
        let mut cron_jobs = Vec::new();

        if self.mount(root, "/").is_err() {
            return Ok(cron_jobs);
        }

        // System crontab
        if let Ok(content) = self.cat("/etc/crontab") {
            for line in content.lines() {
                let line = line.trim();
                if !line.is_empty() && !line.starts_with('#') {
                    cron_jobs.push(line.to_string());
                }
            }
        }

        // Cron directories
        for dir in &[
            "/etc/cron.d",
            "/etc/cron.daily",
            "/etc/cron.hourly",
            "/etc/cron.weekly",
            "/etc/cron.monthly",
        ] {
            if let Ok(files) = self.ls(dir) {
                for file in files {
                    cron_jobs.push(format!(
                        "{}/{}",
                        dir.strip_prefix("/etc/").unwrap_or(dir),
                        file
                    ));
                }
            }
        }

        self.umount("/").ok();
        Ok(cron_jobs)
    }

    /// List systemd timers
    pub fn inspect_systemd_timers(&mut self, root: &str) -> Result<Vec<String>> {
        let mut timers = Vec::new();

        if self.mount(root, "/").is_err() {
            return Ok(timers);
        }

        if let Ok(links) = self.ls("/etc/systemd/system/timers.target.wants") {
            timers = links
                .into_iter()
                .filter(|f| f.ends_with(".timer"))
                .collect();
        }

        self.umount("/").ok();
        Ok(timers)
    }

    /// List SSL certificates
    pub fn inspect_certificates(&mut self, root: &str) -> Result<Vec<String>> {
        let mut certs = Vec::new();

        if self.mount(root, "/").is_err() {
            return Ok(certs);
        }

        // Common certificate locations
        for dir in &["/etc/ssl/certs", "/etc/pki/tls/certs", "/etc/pki/ca-trust"] {
            if let Ok(files) = self.ls(dir) {
                for file in files
                    .iter()
                    .filter(|f| f.ends_with(".crt") || f.ends_with(".pem"))
                {
                    certs.push(format!("{}/{}", dir, file));
                }
            }
        }

        self.umount("/").ok();
        Ok(certs)
    }

    /// Get kernel parameters
    pub fn inspect_kernel_params(&mut self, root: &str) -> Result<HashMap<String, String>> {
        let mut params = HashMap::new();

        if self.mount(root, "/").is_err() {
            return Ok(params);
        }

        if let Ok(content) = self.cat("/etc/sysctl.conf") {
            for line in content.lines() {
                let line = line.trim();
                if line.is_empty() || line.starts_with('#') {
                    continue;
                }
                if let Some((key, value)) = line.split_once('=') {
                    params.insert(key.trim().to_string(), value.trim().to_string());
                }
            }
        }

        self.umount("/").ok();
        Ok(params)
    }

    /// Detect virtualization guest tools
    pub fn inspect_vm_tools(&mut self, root: &str) -> Result<Vec<String>> {
        let mut tools = Vec::new();

        if self.mount(root, "/").is_err() {
            return Ok(tools);
        }

        // VMware Tools
        if self.exists("/usr/bin/vmware-toolbox-cmd").unwrap_or(false)
            || self.exists("/etc/vmware-tools").unwrap_or(false)
        {
            tools.push("vmware-tools".to_string());
        }

        // QEMU Guest Agent
        if self.exists("/usr/bin/qemu-ga").unwrap_or(false) {
            tools.push("qemu-guest-agent".to_string());
        }

        // VirtualBox Guest Additions
        if self.exists("/usr/sbin/VBoxService").unwrap_or(false)
            || self.exists("/opt/VBoxGuestAdditions").unwrap_or(false)
        {
            tools.push("virtualbox-guest-additions".to_string());
        }

        // Hyper-V tools
        if self.exists("/usr/sbin/hv_kvp_daemon").unwrap_or(false) {
            tools.push("hyper-v-tools".to_string());
        }

        self.umount("/").ok();
        Ok(tools)
    }

    /// Get boot configuration
    pub fn inspect_boot_config(&mut self, root: &str) -> Result<BootConfig> {
        let mut config = BootConfig {
            bootloader: "unknown".to_string(),
            default_entry: "unknown".to_string(),
            timeout: "unknown".to_string(),
            kernel_cmdline: String::new(),
        };

        if self.mount(root, "/").is_err() {
            return Ok(config);
        }

        // Check GRUB2
        for grub_cfg in &["/boot/grub2/grub.cfg", "/boot/grub/grub.cfg"] {
            if self.exists(grub_cfg).unwrap_or(false) {
                config.bootloader = "GRUB2".to_string();

                if let Ok(content) = self.cat(grub_cfg) {
                    // Parse basic GRUB config
                    for line in content.lines() {
                        if line.contains("set timeout=") {
                            config.timeout =
                                line.split('=').nth(1).unwrap_or("unknown").to_string();
                        } else if line.contains("set default=") {
                            config.default_entry =
                                line.split('=').nth(1).unwrap_or("unknown").to_string();
                        }
                    }
                }
                break;
            }
        }

        self.umount("/").ok();
        Ok(config)
    }

    /// Get swap information
    pub fn inspect_swap(&mut self, root: &str) -> Result<Vec<String>> {
        let mut swap_devices = Vec::new();

        if self.mount(root, "/").is_err() {
            return Ok(swap_devices);
        }

        if let Ok(content) = self.cat("/etc/fstab") {
            for line in content.lines() {
                let line = line.trim();
                if line.is_empty() || line.starts_with('#') {
                    continue;
                }
                if line.contains("swap") {
                    let parts: Vec<&str> = line.split_whitespace().collect();
                    if !parts.is_empty() {
                        swap_devices.push(parts[0].to_string());
                    }
                }
            }
        }

        self.umount("/").ok();
        Ok(swap_devices)
    }

    /// Get fstab mounts
    pub fn inspect_fstab(&mut self, root: &str) -> Result<Vec<(String, String, String)>> {
        let mut mounts = Vec::new();

        if self.mount(root, "/").is_err() {
            return Ok(mounts);
        }

        if let Ok(content) = self.cat("/etc/fstab") {
            for line in content.lines() {
                let line = line.trim();
                if line.is_empty() || line.starts_with('#') {
                    continue;
                }
                let parts: Vec<&str> = line.split_whitespace().collect();
                if parts.len() >= 3 {
                    mounts.push((
                        parts[0].to_string(), // device
                        parts[1].to_string(), // mountpoint
                        parts[2].to_string(), // fstype
                    ));
                }
            }
        }

        self.umount("/").ok();
        Ok(mounts)
    }

    // ==================== Windows-Specific Inspection ====================

    /// Inspect Windows software from registry
    pub fn inspect_windows_software(&mut self, root: &str) -> Result<Vec<WindowsApplication>> {
        let mut applications = Vec::new();

        if self.mount(root, "/").is_err() {
            return Ok(applications);
        }

        // Get SOFTWARE registry hive path
        let systemroot = self
            .inspect_get_windows_systemroot(root)
            .unwrap_or_else(|_| "/Windows".to_string());
        let software_path = format!("{}/System32/config/SOFTWARE", systemroot);

        // Resolve to host path for direct file access
        let host_path = self.resolve_guest_path(&software_path)?;

        // Parse using nt_hive2
        if let Ok(apps) = super::windows_registry::parse_installed_software(host_path.as_path()) {
            for app in apps {
                applications.push(WindowsApplication {
                    name: app.name,
                    version: app.version,
                    publisher: app.publisher,
                    install_date: "Unknown".to_string(),
                });
            }
        }

        self.umount("/").ok();
        Ok(applications)
    }

    /// Inspect Windows services from registry
    pub fn inspect_windows_services(&mut self, root: &str) -> Result<Vec<WindowsService>> {
        let mut services = Vec::new();

        if self.mount(root, "/").is_err() {
            return Ok(services);
        }

        let systemroot = self
            .inspect_get_windows_systemroot(root)
            .unwrap_or_else(|_| "/Windows".to_string());

        // Parse from SYSTEM registry hive using nt_hive2
        let system_path = format!("{}/System32/config/SYSTEM", systemroot);
        let host_path = self.resolve_guest_path(&system_path)?;

        if let Ok(svcs) = super::windows_registry::parse_windows_services(host_path.as_path()) {
            for svc in svcs {
                services.push(WindowsService {
                    name: svc.name,
                    display_name: svc.display_name,
                    start_type: svc.start_type,
                    status: "Unknown".to_string(), // Status requires runtime info
                });
            }
        }

        self.umount("/").ok();
        Ok(services)
    }

    /// Inspect Windows network configuration
    pub fn inspect_windows_network(&mut self, root: &str) -> Result<Vec<WindowsNetworkAdapter>> {
        let mut adapters = Vec::new();

        if self.mount(root, "/").is_err() {
            return Ok(adapters);
        }

        let systemroot = self
            .inspect_get_windows_systemroot(root)
            .unwrap_or_else(|_| "/Windows".to_string());

        // Parse SYSTEM registry for network configuration using nt_hive2
        let system_path = format!("{}/System32/config/SYSTEM", systemroot);
        let host_path = self.resolve_guest_path(&system_path)?;

        if let Ok(net_adapters) =
            super::windows_registry::parse_network_adapters(host_path.as_path())
        {
            for adapter in net_adapters {
                adapters.push(WindowsNetworkAdapter {
                    name: adapter.name,
                    description: adapter.description,
                    dhcp_enabled: adapter.dhcp_enabled,
                    ip_address: adapter.ip_address,
                    mac_address: adapter.mac_address,
                    dns_servers: adapter.dns_servers,
                });
            }
        }

        self.umount("/").ok();
        Ok(adapters)
    }

    /// Inspect Windows updates and patches
    pub fn inspect_windows_updates(&mut self, root: &str) -> Result<Vec<WindowsUpdate>> {
        let mut updates = Vec::new();

        if self.mount(root, "/").is_err() {
            return Ok(updates);
        }

        let systemroot = self
            .inspect_get_windows_systemroot(root)
            .unwrap_or_else(|_| "/Windows".to_string());

        // Parse installed updates from registry
        let software_path = format!("{}/System32/config/SOFTWARE", systemroot);
        let software_host = self.resolve_guest_path(&software_path)?;
        if let Ok(reg_updates) =
            super::windows_registry::parse_installed_updates(software_host.as_path())
        {
            for upd in reg_updates {
                updates.push(WindowsUpdate {
                    kb: upd.kb_number,
                    title: upd.title,
                    installed_date: upd.installed_date,
                    update_type: upd.update_type,
                });
            }
        }

        // Parse CBS.log for component updates
        let cbs_log_path = format!("{}/Logs/CBS/CBS.log", systemroot);
        if let Ok(content) = self.cat(&cbs_log_path) {
            let cbs_updates = super::windows_registry::parse_cbs_log(&content);
            for upd in cbs_updates {
                updates.push(WindowsUpdate {
                    kb: upd.kb_number,
                    title: upd.title,
                    installed_date: upd.installed_date,
                    update_type: upd.update_type,
                });
            }
        }

        // Check Windows Update DataStore
        let update_db_path = format!(
            "{}/SoftwareDistribution/DataStore/DataStore.edb",
            systemroot
        );
        let db_host = self.resolve_guest_path(&update_db_path).ok();
        if let Some(db_path) = db_host {
            if let Ok(db_updates) =
                super::windows_registry::parse_update_datastore(db_path.as_path())
            {
                for upd in db_updates {
                    updates.push(WindowsUpdate {
                        kb: upd.kb_number,
                        title: upd.title,
                        installed_date: upd.installed_date,
                        update_type: upd.update_type,
                    });
                }
            }
        }

        // Detect hotfixes from filesystem
        let systemroot_host = self.resolve_guest_path(&systemroot)?;
        if let Ok(hotfixes) =
            super::windows_registry::detect_hotfixes_from_filesystem(systemroot_host.as_path())
        {
            for upd in hotfixes {
                updates.push(WindowsUpdate {
                    kb: upd.kb_number,
                    title: upd.title,
                    installed_date: upd.installed_date,
                    update_type: upd.update_type,
                });
            }
        }

        self.umount("/").ok();
        Ok(updates)
    }

    /// Inspect Windows event logs
    pub fn inspect_windows_events(
        &mut self,
        root: &str,
        log_name: &str,
        limit: usize,
    ) -> Result<Vec<WindowsEventLogEntry>> {
        let mut events = Vec::new();

        if self.mount(root, "/").is_err() {
            return Ok(events);
        }

        let systemroot = self
            .inspect_get_windows_systemroot(root)
            .unwrap_or_else(|_| "/Windows".to_string());
        let event_log_path = format!("{}/System32/winevt/Logs/{}.evtx", systemroot, log_name);

        // Resolve to host path for direct file access
        if let Ok(evtx_host) = self.resolve_guest_path(&event_log_path) {
            // Parse EVTX file using evtx crate
            if let Ok(parsed_events) =
                super::windows_registry::parse_evtx_file(evtx_host.as_path(), limit)
            {
                for evt in parsed_events {
                    events.push(WindowsEventLogEntry {
                        event_id: evt.event_id as i32,
                        level: evt.level,
                        source: evt.source,
                        message: evt.message,
                        time_created: evt.time_created,
                    });
                }
            }
        }

        self.umount("/").ok();
        Ok(events)
    }

    // ==================== Windows Parsing Helpers ====================

    #[allow(dead_code)]
    fn parse_windows_software(&mut self, content: &str) -> Vec<WindowsApplication> {
        let mut applications = Vec::new();

        for line in content.lines() {
            if line.contains("\"DisplayName\"=") {
                let name = line
                    .split('=')
                    .nth(1)
                    .unwrap_or("")
                    .trim_matches('"')
                    .to_string();
                if !name.is_empty() {
                    applications.push(WindowsApplication {
                        name,
                        version: "Unknown".to_string(),
                        publisher: "Unknown".to_string(),
                        install_date: "Unknown".to_string(),
                    });
                }
            }
        }

        applications
    }

    #[allow(dead_code)]
    fn parse_windows_service(&mut self, content: &str, filename: &str) -> Option<WindowsService> {
        let service_name = filename.trim_end_matches(".ini");
        let mut display_name = service_name.to_string();
        let mut start_type = "Unknown".to_string();
        let mut status = "Unknown".to_string();

        for line in content.lines() {
            if line.starts_with("DisplayName=") {
                display_name = line.split('=').nth(1).unwrap_or("").to_string();
            } else if line.starts_with("Start=") {
                let start_val = line.split('=').nth(1).unwrap_or("");
                start_type = match start_val {
                    "auto" => "Automatic".to_string(),
                    "demand" => "Manual".to_string(),
                    "disabled" => "Disabled".to_string(),
                    _ => "Unknown".to_string(),
                };
                status = if start_val == "auto" {
                    "Running"
                } else {
                    "Stopped"
                }
                .to_string();
            }
        }

        Some(WindowsService {
            name: service_name.to_string(),
            display_name,
            start_type,
            status,
        })
    }

    #[allow(dead_code)]
    fn parse_windows_services_from_registry(&mut self, content: &str) -> Vec<WindowsService> {
        let services = Vec::new();

        // Parse registry format for services
        let mut _current_service: Option<String> = None;

        for line in content.lines() {
            if line.contains("\\Services\\") {
                if let Some(service_name) = line.split("\\Services\\").nth(1) {
                    let service_name = service_name.trim_matches(']').trim_matches('[');
                    _current_service = Some(service_name.to_string());
                }
            }
        }

        services
    }

    #[allow(dead_code)]
    fn parse_windows_network_adapters(&mut self, content: &str) -> Vec<WindowsNetworkAdapter> {
        let mut adapters = Vec::new();

        // Parse registry for network adapter info
        for line in content.lines() {
            if line.contains("DhcpNameServer") {
                if let Some(dns) = line.split('=').nth(1) {
                    adapters.push(WindowsNetworkAdapter {
                        name: "Ethernet".to_string(),
                        description: "Network Adapter".to_string(),
                        mac_address: "00:00:00:00:00:00".to_string(),
                        ip_address: Vec::new(),
                        dns_servers: dns
                            .trim_matches('"')
                            .split_whitespace()
                            .map(|s| s.to_string())
                            .collect(),
                        dhcp_enabled: true,
                    });
                }
            }
        }

        adapters
    }

    #[allow(dead_code)]
    fn parse_windows_cbs_log(&mut self, _content: &str) -> Vec<WindowsUpdate> {
        

        // Basic CBS log parsing would go here
        // For now, return empty

        Vec::new()
    }
}

/// Windows application information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WindowsApplication {
    pub name: String,
    pub version: String,
    pub publisher: String,
    pub install_date: String,
}

/// Windows service information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WindowsService {
    pub name: String,
    pub display_name: String,
    pub start_type: String,
    pub status: String,
}

/// Windows network adapter information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WindowsNetworkAdapter {
    pub name: String,
    pub description: String,
    pub mac_address: String,
    pub ip_address: Vec<String>,
    pub dns_servers: Vec<String>,
    pub dhcp_enabled: bool,
}

/// Windows update information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WindowsUpdate {
    pub kb: String,
    pub title: String,
    pub installed_date: String,
    pub update_type: String,
}

/// Windows event log entry
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WindowsEventLogEntry {
    pub event_id: i32,
    pub level: String,
    pub source: String,
    pub message: String,
    pub time_created: String,
}
