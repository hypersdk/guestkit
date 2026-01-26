// SPDX-License-Identifier: LGPL-3.0-or-later
//! Enhanced inspection operations for comprehensive guest analysis
use crate::core::Result;
use crate::guestfs::Guestfs;
use serde::{Deserialize, Serialize};
use serde_yaml::Value;
use std::collections::HashMap;
use std::str::FromStr;

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
        // Try to mount if not already mounted
        let was_mounted = self.mounted.contains_key("/");
        if !was_mounted {
            if self.mount_ro(root, "/").is_err() {
                return Ok(interfaces);
            }
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
                    if let Ok(content) = self.cat(&path) {
                        interfaces.extend(self.parse_netplan(&content));
                    }
                }
            }
        }
        if !was_mounted {
            self.umount("/").ok();
        }
        Ok(interfaces)
    }

    fn parse_debian_interfaces(&self, content: &str) -> Vec<NetworkInterface> {
        let mut interfaces = Vec::new();
        let mut current_iface: Option<NetworkInterface> = None;
        for line in content.lines() {
            let line = line.trim();
            if line.is_empty() || line.starts_with('#') {
                continue;
            }
            if line.starts_with("iface ") {
                if let Some(iface) = current_iface.take() {
                    interfaces.push(iface);
                }
                let parts: Vec<&str> = line.split_whitespace().collect();
                if parts.len() >= 4 {
                    let dhcp = match parts[3] {
                        "dhcp" => true,
                        "static" | "manual" => false,
                        _ => false,
                    };
                    current_iface = Some(NetworkInterface {
                        name: parts[1].to_string(),
                        ip_address: Vec::new(),
                        mac_address: String::new(),
                        dhcp,
                    });
                }
            } else if let Some(ref mut iface) = current_iface {
                if line.starts_with("address ") {
                    let addr = line.split_whitespace().nth(1).unwrap_or("").to_string();
                    if !addr.is_empty() {
                        iface.ip_address.push(addr);
                    }
                } else if line.starts_with("hwaddress ") || line.starts_with("hwaddr ") {
                    let mac = line.split_whitespace().nth(1).unwrap_or("").to_string();
                    if !mac.is_empty() {
                        iface.mac_address = mac;
                    }
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
        let mut index = 0;
        for line in content.lines() {
            let line = line.trim();
            if line.is_empty() || line.starts_with('#') {
                continue;
            }
            if let Some((key, value)) = line.split_once('=') {
                let key = key.trim();
                let value = value.trim_matches('"').trim();
                match key {
                    "BOOTPROTO" => iface.dhcp = matches!(value.to_lowercase().as_str(), "dhcp" | "bootp"),
                    "IPADDR" => iface.ip_address.push(value.to_string()),
                    k if k.starts_with("IPADDR") => {
                        if let Ok(num) = usize::from_str(&k[6..]) {
                            if num >= index {
                                while iface.ip_address.len() <= num {
                                    iface.ip_address.push(String::new());
                                }
                                iface.ip_address[num] = value.to_string();
                                index = num + 1;
                            }
                        } else {
                            iface.ip_address.push(value.to_string());
                        }
                    }
                    "HWADDR" | "MACADDR" => iface.mac_address = value.to_string(),
                    _ => {}
                }
            }
        }
        Some(iface)
    }

    fn parse_netplan(&self, content: &str) -> Vec<NetworkInterface> {
        let mut interfaces = Vec::new();
        if let Ok(yaml) = serde_yaml::from_str::<Value>(content) {
            if let Some(network) = yaml.get("network") {
                if let Some(ethernets) = network.get("ethernets") {
                    if let Some(mapping) = ethernets.as_mapping() {
                        for (name, iface_val) in mapping {
                            if let Some(name_str) = name.as_str() {
                                let mut intf = NetworkInterface {
                                    name: name_str.to_string(),
                                    ip_address: Vec::new(),
                                    mac_address: String::new(),
                                    dhcp: false,
                                };
                                if let Some(iface) = iface_val.as_mapping() {
                                    if let Some(dhcp4) = iface.get("dhcp4") {
                                        intf.dhcp = dhcp4.as_bool().unwrap_or(false);
                                    } else if let Some(dhcp) = iface.get("dhcp") {
                                        intf.dhcp = dhcp.as_bool().unwrap_or(false);
                                    }
                                    if let Some(addresses) = iface.get("addresses") {
                                        if let Some(addrs) = addresses.as_sequence() {
                                            for addr in addrs {
                                                if let Some(s) = addr.as_str() {
                                                    intf.ip_address.push(s.to_string());
                                                }
                                            }
                                        }
                                    }
                                    if let Some(match_obj) = iface.get("match") {
                                        if let Some(mac) = match_obj.get("macaddress") {
                                            intf.mac_address = mac.as_str().unwrap_or("").to_string();
                                        }
                                    } else if let Some(mac) = iface.get("macaddress") {
                                        intf.mac_address = mac.as_str().unwrap_or("").to_string();
                                    }
                                }
                                interfaces.push(intf);
                            }
                        }
                    }
                }
            }
        }
        interfaces
    }

    /// Get DNS configuration
    pub fn inspect_dns(&mut self, root: &str) -> Result<Vec<String>> {
        let mut dns_servers = Vec::new();
        // Try to mount if not already mounted
        let was_mounted = self.mounted.contains_key("/");
        if !was_mounted {
            if self.mount_ro(root, "/").is_err() {
                return Ok(dns_servers);
            }
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
        if !was_mounted {
            self.umount("/").ok();
        }
        Ok(dns_servers)
    }

    /// List user accounts
    pub fn inspect_users(&mut self, root: &str) -> Result<Vec<UserAccount>> {
        let mut users = Vec::new();
        let was_mounted = self.mounted.contains_key("/");
        if !was_mounted {
            if self.mount_ro(root, "/").is_err() {
                return Ok(users);
            }
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
        if !was_mounted {
            self.umount("/").ok();
        }
        Ok(users)
    }

    /// Get SSH configuration
    pub fn inspect_ssh_config(&mut self, root: &str) -> Result<HashMap<String, String>> {
        let mut config = HashMap::new();
        let was_mounted = self.mounted.contains_key("/");
        if !was_mounted {
            if self.mount_ro(root, "/").is_err() {
                return Ok(config);
            }
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
        if !was_mounted {
            self.umount("/").ok();
        }
        Ok(config)
    }

    /// Check SELinux status
    pub fn inspect_selinux(&mut self, root: &str) -> Result<String> {
        let was_mounted = self.mounted.contains_key("/");
        if !was_mounted {
            if self.mount_ro(root, "/").is_err() {
                return Ok("unknown".to_string());
            }
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
        if !was_mounted {
            self.umount("/").ok();
        }
        Ok(status)
    }

    /// List systemd services
    pub fn inspect_systemd_services(&mut self, root: &str) -> Result<Vec<SystemService>> {
        let mut services = Vec::new();
        let was_mounted = self.mounted.contains_key("/");
        if !was_mounted {
            if self.mount_ro(root, "/").is_err() {
                return Ok(services);
            }
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
        if !was_mounted {
            self.umount("/").ok();
        }
        Ok(services)
    }

    /// Get timezone information
    pub fn inspect_timezone(&mut self, root: &str) -> Result<String> {
        let was_mounted = self.mounted.contains_key("/");
        if !was_mounted {
            if self.mount_ro(root, "/").is_err() {
                return Ok("unknown".to_string());
            }
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
        if !was_mounted {
            self.umount("/").ok();
        }
        Ok(timezone)
    }

    /// Get locale information
    pub fn inspect_locale(&mut self, root: &str) -> Result<String> {
        let was_mounted = self.mounted.contains_key("/");
        if !was_mounted {
            if self.mount_ro(root, "/").is_err() {
                return Ok("unknown".to_string());
            }
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
        if !was_mounted {
            self.umount("/").ok();
        }
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
        let was_mounted = self.mounted.contains_key("/");
        if !was_mounted {
            if self.mount_ro(root, "/").is_err() {
                return Ok(false);
            }
        }
        let has_cloud_init = self.exists("/etc/cloud/cloud.cfg").unwrap_or(false)
            || self.exists("/usr/bin/cloud-init").unwrap_or(false);
        if !was_mounted {
            self.umount("/").ok();
        }
        Ok(has_cloud_init)
    }

    /// Get language runtime versions
    pub fn inspect_runtimes(&mut self, root: &str) -> Result<HashMap<String, String>> {
        let mut runtimes = HashMap::new();
        let was_mounted = self.mounted.contains_key("/");
        if !was_mounted {
            if self.mount_ro(root, "/").is_err() {
                return Ok(runtimes);
            }
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
        if !was_mounted {
            self.umount("/").ok();
        }
        Ok(runtimes)
    }

    /// Detect container runtimes
    pub fn inspect_container_runtimes(&mut self, root: &str) -> Result<Vec<String>> {
        let mut runtimes = Vec::new();
        let was_mounted = self.mounted.contains_key("/");
        if !was_mounted {
            if self.mount_ro(root, "/").is_err() {
                return Ok(runtimes);
            }
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
        if !was_mounted {
            self.umount("/").ok();
        }
        Ok(runtimes)
    }

    /// List cron jobs
    pub fn inspect_cron(&mut self, root: &str) -> Result<Vec<String>> {
        let mut cron_jobs = Vec::new();
        let was_mounted = self.mounted.contains_key("/");
        if !was_mounted {
            if self.mount_ro(root, "/").is_err() {
                return Ok(cron_jobs);
            }
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
        if !was_mounted {
            self.umount("/").ok();
        }
        Ok(cron_jobs)
    }

    /// List systemd timers
    pub fn inspect_systemd_timers(&mut self, root: &str) -> Result<Vec<String>> {
        let mut timers = Vec::new();
        let was_mounted = self.mounted.contains_key("/");
        if !was_mounted {
            if self.mount_ro(root, "/").is_err() {
                return Ok(timers);
            }
        }
        if let Ok(links) = self.ls("/etc/systemd/system/timers.target.wants") {
            timers = links
                .into_iter()
                .filter(|f| f.ends_with(".timer"))
                .collect();
        }
        if !was_mounted {
            self.umount("/").ok();
        }
        Ok(timers)
    }

    /// List SSL certificates
    pub fn inspect_certificates(&mut self, root: &str) -> Result<Vec<Certificate>> {
        let mut certs = Vec::new();
        let was_mounted = self.mounted.contains_key("/");
        if !was_mounted {
            if self.mount_ro(root, "/").is_err() {
                return Ok(certs);
            }
        }
        // Common certificate locations
        let mut paths = Vec::new();
        for dir in &["/etc/ssl/certs", "/etc/pki/tls/certs", "/etc/pki/ca-trust"] {
            if let Ok(files) = self.ls(dir) {
                for file in files
                    .iter()
                    .filter(|f| f.ends_with(".crt") || f.ends_with(".pem"))
                {
                    paths.push(format!("{}/{}", dir, file));
                }
            }
        }
        // Parse certificates if openssl is available in the guest
        let has_openssl = self.exists("/usr/bin/openssl").unwrap_or(false);
        for path in paths {
            let mut subject = "Unknown".to_string();
            let mut issuer = "Unknown".to_string();
            let mut expiry = "Unknown".to_string();
            if has_openssl {
                let cmd = format!("openssl x509 -in {} -noout -subject -issuer -enddate", path);
                if let Ok(output) = self.command(&["sh", "-c", &cmd]) {
                    for line in output.lines() {
                        let trimmed = line.trim();
                        if trimmed.starts_with("subject=") {
                            subject = trimmed.strip_prefix("subject=").unwrap_or(&subject).to_string();
                        } else if trimmed.starts_with("issuer=") {
                            issuer = trimmed.strip_prefix("issuer=").unwrap_or(&issuer).to_string();
                        } else if trimmed.starts_with("notAfter=") {
                            expiry = trimmed.strip_prefix("notAfter=").unwrap_or(&expiry).to_string();
                        }
                    }
                }
            }
            certs.push(Certificate {
                path: path.clone(),
                subject,
                issuer,
                expiry,
            });
        }
        if !was_mounted {
            self.umount("/").ok();
        }
        Ok(certs)
    }

    /// Get kernel parameters
    pub fn inspect_kernel_params(&mut self, root: &str) -> Result<HashMap<String, String>> {
        let mut params = HashMap::new();
        let was_mounted = self.mounted.contains_key("/");
        if !was_mounted {
            if self.mount_ro(root, "/").is_err() {
                return Ok(params);
            }
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
        if !was_mounted {
            self.umount("/").ok();
        }
        Ok(params)
    }

    /// Detect virtualization guest tools
    pub fn inspect_vm_tools(&mut self, root: &str) -> Result<Vec<String>> {
        let mut tools = Vec::new();
        let was_mounted = self.mounted.contains_key("/");
        if !was_mounted {
            if self.mount_ro(root, "/").is_err() {
                return Ok(tools);
            }
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
        if !was_mounted {
            self.umount("/").ok();
        }
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
        let was_mounted = self.mounted.contains_key("/");
        if !was_mounted {
            if self.mount_ro(root, "/").is_err() {
                return Ok(config);
            }
        }
        // Check GRUB2
        let grub_paths = vec!["/boot/grub2/grub.cfg", "/boot/grub/grub.cfg", "/etc/grub2.cfg"];
        for grub_cfg in grub_paths {
            if self.exists(&grub_cfg).unwrap_or(false) {
                config.bootloader = "GRUB2".to_string();
                if let Ok(content) = self.cat(&grub_cfg) {
                    // Parse basic GRUB config
                    let mut default_index: Option<usize> = None;
                    let mut current_entry = 0;
                    let mut in_menuentry = false;
                    let mut current_title = String::new();
                    for line in content.lines() {
                        let trimmed = line.trim();
                        if trimmed.starts_with("set timeout=") {
                            config.timeout = trimmed.split('=').nth(1).unwrap_or("unknown").trim_matches('"').to_string();
                        } else if trimmed.starts_with("set default=") {
                            let default_str = trimmed.split('=').nth(1).unwrap_or("0").trim_matches('"');
                            default_index = default_str.parse::<usize>().ok();
                        } else if trimmed.starts_with("menuentry ") || trimmed.starts_with("submenu ") {
                            if in_menuentry {
                                current_entry += 1;
                            }
                            in_menuentry = true;
                            let title_parts: Vec<&str> = trimmed.splitn(3, '\'').collect();
                            if title_parts.len() >= 2 {
                                current_title = title_parts[1].to_string();
                            } else {
                                current_title = "unknown".to_string();
                            }
                            if Some(current_entry) == default_index {
                                config.default_entry = current_title.clone();
                            }
                        } else if in_menuentry && (trimmed.starts_with("linux") || trimmed.starts_with("linux16") || trimmed.starts_with("linuxefi")) {
                            let parts: Vec<&str> = trimmed.split_whitespace().collect();
                            if parts.len() > 1 && Some(current_entry) == default_index {
                                config.kernel_cmdline = parts[1..].join(" ");
                            }
                        } else if trimmed == "}" {
                            if in_menuentry {
                                in_menuentry = false;
                                current_entry += 1;
                            }
                        }
                    }
                }
                break;
            }
        }
        if !was_mounted {
            self.umount("/").ok();
        }
        Ok(config)
    }

    /// Get swap information
    pub fn inspect_swap(&mut self, root: &str) -> Result<Vec<String>> {
        let mut swap_devices = Vec::new();
        let was_mounted = self.mounted.contains_key("/");
        if !was_mounted {
            if self.mount_ro(root, "/").is_err() {
                return Ok(swap_devices);
            }
        }
        if let Ok(content) = self.cat("/etc/fstab") {
            for line in content.lines() {
                let line = line.trim();
                if line.is_empty() || line.starts_with('#') {
                    continue;
                }
                let parts: Vec<&str> = line.split_whitespace().collect();
                if parts.len() >= 3 && parts[2] == "swap" {
                    swap_devices.push(parts[0].to_string());
                }
            }
        }
        if !was_mounted {
            self.umount("/").ok();
        }
        Ok(swap_devices)
    }

    /// Get fstab mounts
    pub fn inspect_fstab(&mut self, root: &str) -> Result<Vec<(String, String, String)>> {
        let mut mounts = Vec::new();
        let was_mounted = self.mounted.contains_key("/");
        if !was_mounted {
            if self.mount_ro(root, "/").is_err() {
                return Ok(mounts);
            }
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
        if !was_mounted {
            self.umount("/").ok();
        }
        Ok(mounts)
    }

    // ==================== Windows-Specific Inspection ====================
    /// Inspect Windows software from registry
    pub fn inspect_windows_software(&mut self, root: &str) -> Result<Vec<WindowsApplication>> {
        let mut applications = Vec::new();
        let was_mounted = self.mounted.contains_key("/");
        if !was_mounted {
            if self.mount_ro(root, "/").is_err() {
                return Ok(applications);
            }
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
        if !was_mounted {
            self.umount("/").ok();
        }
        Ok(applications)
    }

    /// Inspect Windows services from registry
    pub fn inspect_windows_services(&mut self, root: &str) -> Result<Vec<WindowsService>> {
        let mut services = Vec::new();
        let was_mounted = self.mounted.contains_key("/");
        if !was_mounted {
            if self.mount_ro(root, "/").is_err() {
                return Ok(services);
            }
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
        if !was_mounted {
            self.umount("/").ok();
        }
        Ok(services)
    }

    /// Inspect Windows network configuration
    pub fn inspect_windows_network(&mut self, root: &str) -> Result<Vec<WindowsNetworkAdapter>> {
        let mut adapters = Vec::new();
        let was_mounted = self.mounted.contains_key("/");
        if !was_mounted {
            if self.mount_ro(root, "/").is_err() {
                return Ok(adapters);
            }
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
        if !was_mounted {
            self.umount("/").ok();
        }
        Ok(adapters)
    }

    /// Inspect Windows updates and patches
    pub fn inspect_windows_updates(&mut self, root: &str) -> Result<Vec<WindowsUpdate>> {
        let mut updates = Vec::new();
        let was_mounted = self.mounted.contains_key("/");
        if !was_mounted {
            if self.mount_ro(root, "/").is_err() {
                return Ok(updates);
            }
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
        if !was_mounted {
            self.umount("/").ok();
        }
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
        let was_mounted = self.mounted.contains_key("/");
        if !was_mounted {
            if self.mount_ro(root, "/").is_err() {
                return Ok(events);
            }
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
        if !was_mounted {
            self.umount("/").ok();
        }
        Ok(events)
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