# inspect_enhanced.rs Enhancement Recommendations

## Priority 1: Code Quality Improvements

### 1.1 Add Helper Method to Reduce Duplication ⭐⭐⭐

**Current Issue**: Every function repeats the same mount/unmount pattern (400+ lines of duplication)

```rust
// Add this helper method to Guestfs impl block:
impl Guestfs {
    /// Execute a function with automatic mount/unmount handling
    fn with_mount<F, T>(&mut self, root: &str, f: F) -> Result<T>
    where
        F: FnOnce(&mut Self) -> Result<T>,
    {
        let was_mounted = self.mounted.contains_key("/");
        if !was_mounted {
            self.mount_ro(root, "/")?;
        }

        let result = f(self);

        if !was_mounted {
            let _ = self.umount("/");
        }

        result
    }
}
```

**Benefits**:
- Reduces code from ~15 lines to ~3 lines per function
- Eliminates bugs from forgetting unmount in error paths
- Centralizes mount error handling

**Example Usage**:
```rust
// BEFORE (15 lines):
pub fn inspect_dns(&mut self, root: &str) -> Result<Vec<String>> {
    let mut dns_servers = Vec::new();
    let was_mounted = self.mounted.contains_key("/");
    if !was_mounted {
        if self.mount_ro(root, "/").is_err() {
            return Ok(dns_servers);
        }
    }
    // ... logic ...
    if !was_mounted {
        self.umount("/").ok();
    }
    Ok(dns_servers)
}

// AFTER (3 lines):
pub fn inspect_dns(&mut self, root: &str) -> Result<Vec<String>> {
    self.with_mount(root, |guestfs| {
        let mut dns_servers = Vec::new();
        if let Ok(content) = guestfs.cat("/etc/resolv.conf") {
            for line in content.lines() {
                let line = line.trim();
                if line.starts_with("nameserver ") {
                    if let Some(server) = line.split_whitespace().nth(1) {
                        dns_servers.push(server.to_string());
                    }
                }
            }
        }
        Ok(dns_servers)
    })
}
```

### 1.2 Fix Unused Variable Warning

**File**: src/guestfs/inspect_enhanced.rs:758

```rust
// REMOVE this unused assignment:
let mut current_title = String::new();

// It's overwritten in the loop without being read first
```

### 1.3 Add Constants for Common Paths

```rust
// Add at top of file:
const SYSTEMD_SERVICES_DIR: &str = "/etc/systemd/system/multi-user.target.wants";
const SYSTEMD_TIMERS_DIR: &str = "/etc/systemd/system/timers.target.wants";
const RESOLV_CONF: &str = "/etc/resolv.conf";
const PASSWD: &str = "/etc/passwd";
const SSHD_CONFIG: &str = "/etc/ssh/sshd_config";
const SELINUX_CONFIG: &str = "/etc/selinux/config";
// ... etc
```

---

## Priority 2: Missing Functionality

### 2.1 Add Package Manager Detection ⭐⭐⭐

```rust
/// Detect installed package manager and list packages
pub fn inspect_packages(&mut self, root: &str) -> Result<PackageInfo> {
    self.with_mount(root, |guestfs| {
        let mut pkg_info = PackageInfo {
            manager: "unknown".to_string(),
            package_count: 0,
            packages: Vec::new(),
        };

        // RPM-based (RHEL, Fedora, CentOS, openSUSE)
        if guestfs.exists("/usr/bin/rpm").unwrap_or(false) {
            pkg_info.manager = "rpm".to_string();
            if let Ok(output) = guestfs.command(&["rpm", "-qa", "--qf", "%{NAME}|%{VERSION}|%{RELEASE}\\n"]) {
                for line in output.lines() {
                    let parts: Vec<&str> = line.split('|').collect();
                    if parts.len() >= 3 {
                        pkg_info.packages.push(Package {
                            name: parts[0].to_string(),
                            version: format!("{}-{}", parts[1], parts[2]),
                            manager: "rpm".to_string(),
                        });
                    }
                }
            }
        }
        // DEB-based (Debian, Ubuntu)
        else if guestfs.exists("/usr/bin/dpkg").unwrap_or(false) {
            pkg_info.manager = "dpkg".to_string();
            if let Ok(output) = guestfs.command(&["dpkg-query", "-W", "-f=${Package}|${Version}\\n"]) {
                for line in output.lines() {
                    if let Some((name, version)) = line.split_once('|') {
                        pkg_info.packages.push(Package {
                            name: name.to_string(),
                            version: version.to_string(),
                            manager: "dpkg".to_string(),
                        });
                    }
                }
            }
        }
        // Arch Linux
        else if guestfs.exists("/usr/bin/pacman").unwrap_or(false) {
            pkg_info.manager = "pacman".to_string();
            if let Ok(output) = guestfs.command(&["pacman", "-Q"]) {
                for line in output.lines() {
                    let parts: Vec<&str> = line.split_whitespace().collect();
                    if parts.len() >= 2 {
                        pkg_info.packages.push(Package {
                            name: parts[0].to_string(),
                            version: parts[1].to_string(),
                            manager: "pacman".to_string(),
                        });
                    }
                }
            }
        }
        // Alpine Linux
        else if guestfs.exists("/sbin/apk").unwrap_or(false) {
            pkg_info.manager = "apk".to_string();
            if let Ok(output) = guestfs.command(&["apk", "info", "-v"]) {
                for line in output.lines() {
                    // Format: package-version
                    if let Some(pos) = line.rfind('-') {
                        pkg_info.packages.push(Package {
                            name: line[..pos].to_string(),
                            version: line[pos+1..].to_string(),
                            manager: "apk".to_string(),
                        });
                    }
                }
            }
        }

        pkg_info.package_count = pkg_info.packages.len();
        Ok(pkg_info)
    })
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PackageInfo {
    pub manager: String,
    pub package_count: usize,
    pub packages: Vec<Package>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Package {
    pub name: String,
    pub version: String,
    pub manager: String,
}
```

### 2.2 Add Firewall Detection ⭐⭐⭐

```rust
/// Detect firewall configuration
pub fn inspect_firewall(&mut self, root: &str) -> Result<FirewallInfo> {
    self.with_mount(root, |guestfs| {
        let mut fw_info = FirewallInfo {
            firewall_type: "none".to_string(),
            enabled: false,
            rules_count: 0,
            zones: Vec::new(),
        };

        // firewalld (RHEL/Fedora/CentOS)
        if guestfs.exists("/usr/sbin/firewalld").unwrap_or(false) {
            fw_info.firewall_type = "firewalld".to_string();

            // Check if enabled
            if let Ok(links) = guestfs.ls("/etc/systemd/system/multi-user.target.wants") {
                fw_info.enabled = links.iter().any(|l| l.contains("firewalld"));
            }

            // Get active zones
            if let Ok(zones) = guestfs.ls("/etc/firewalld/zones") {
                fw_info.zones = zones.into_iter()
                    .filter(|z| z.ends_with(".xml"))
                    .map(|z| z.strip_suffix(".xml").unwrap_or(&z).to_string())
                    .collect();
            }
        }
        // ufw (Ubuntu/Debian)
        else if guestfs.exists("/usr/sbin/ufw").unwrap_or(false) {
            fw_info.firewall_type = "ufw".to_string();

            if let Ok(content) = guestfs.cat("/etc/ufw/ufw.conf") {
                for line in content.lines() {
                    if line.starts_with("ENABLED=") {
                        fw_info.enabled = line.contains("yes");
                    }
                }
            }

            // Count rules
            if let Ok(rules) = guestfs.cat("/etc/ufw/user.rules") {
                fw_info.rules_count = rules.lines()
                    .filter(|l| !l.trim().is_empty() && !l.starts_with('#'))
                    .count();
            }
        }
        // iptables (legacy)
        else if guestfs.exists("/usr/sbin/iptables").unwrap_or(false) {
            fw_info.firewall_type = "iptables".to_string();

            // Check for rules files
            if let Ok(rules) = guestfs.cat("/etc/sysconfig/iptables")
                .or_else(|_| guestfs.cat("/etc/iptables/rules.v4")) {
                fw_info.rules_count = rules.lines()
                    .filter(|l| !l.trim().is_empty() && !l.starts_with('#'))
                    .count();
                fw_info.enabled = fw_info.rules_count > 0;
            }
        }

        Ok(fw_info)
    })
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FirewallInfo {
    pub firewall_type: String,
    pub enabled: bool,
    pub rules_count: usize,
    pub zones: Vec<String>,
}
```

### 2.3 Add Init System Detection ⭐⭐

```rust
/// Detect init system
pub fn inspect_init_system(&mut self, root: &str) -> Result<String> {
    self.with_mount(root, |guestfs| {
        // systemd
        if guestfs.is_dir("/run/systemd/system").unwrap_or(false)
            || guestfs.exists("/usr/lib/systemd/systemd").unwrap_or(false) {
            return Ok("systemd".to_string());
        }

        // upstart
        if guestfs.exists("/sbin/initctl").unwrap_or(false)
            && guestfs.is_dir("/etc/init").unwrap_or(false) {
            return Ok("upstart".to_string());
        }

        // OpenRC
        if guestfs.exists("/sbin/openrc").unwrap_or(false)
            || guestfs.exists("/sbin/rc-service").unwrap_or(false) {
            return Ok("openrc".to_string());
        }

        // runit
        if guestfs.is_dir("/etc/runit").unwrap_or(false) {
            return Ok("runit".to_string());
        }

        // SysVinit (fallback)
        if guestfs.exists("/sbin/init").unwrap_or(false) {
            return Ok("sysvinit".to_string());
        }

        Ok("unknown".to_string())
    })
}
```

### 2.4 Add Web Server Detection ⭐⭐

```rust
/// Detect installed web servers
pub fn inspect_web_servers(&mut self, root: &str) -> Result<Vec<WebServer>> {
    self.with_mount(root, |guestfs| {
        let mut servers = Vec::new();

        // Apache
        if guestfs.exists("/usr/sbin/httpd").unwrap_or(false)
            || guestfs.exists("/usr/sbin/apache2").unwrap_or(false) {
            let mut apache = WebServer {
                name: "apache".to_string(),
                version: "unknown".to_string(),
                config_path: String::new(),
                enabled: false,
            };

            // Detect config location
            if guestfs.is_dir("/etc/httpd").unwrap_or(false) {
                apache.config_path = "/etc/httpd/conf/httpd.conf".to_string();
            } else if guestfs.is_dir("/etc/apache2").unwrap_or(false) {
                apache.config_path = "/etc/apache2/apache2.conf".to_string();
            }

            // Check if enabled
            if let Ok(links) = guestfs.ls("/etc/systemd/system/multi-user.target.wants") {
                apache.enabled = links.iter().any(|l| l.contains("httpd") || l.contains("apache"));
            }

            servers.push(apache);
        }

        // Nginx
        if guestfs.exists("/usr/sbin/nginx").unwrap_or(false) {
            let mut nginx = WebServer {
                name: "nginx".to_string(),
                version: "unknown".to_string(),
                config_path: "/etc/nginx/nginx.conf".to_string(),
                enabled: false,
            };

            if let Ok(links) = guestfs.ls("/etc/systemd/system/multi-user.target.wants") {
                nginx.enabled = links.iter().any(|l| l.contains("nginx"));
            }

            servers.push(nginx);
        }

        // Lighttpd
        if guestfs.exists("/usr/sbin/lighttpd").unwrap_or(false) {
            servers.push(WebServer {
                name: "lighttpd".to_string(),
                version: "unknown".to_string(),
                config_path: "/etc/lighttpd/lighttpd.conf".to_string(),
                enabled: false,
            });
        }

        Ok(servers)
    })
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WebServer {
    pub name: String,
    pub version: String,
    pub config_path: String,
    pub enabled: bool,
}
```

### 2.5 Add Database Detection ⭐⭐

```rust
/// Detect installed databases
pub fn inspect_databases(&mut self, root: &str) -> Result<Vec<Database>> {
    self.with_mount(root, |guestfs| {
        let mut databases = Vec::new();

        // PostgreSQL
        if guestfs.exists("/usr/bin/postgres").unwrap_or(false)
            || guestfs.exists("/usr/lib/postgresql").unwrap_or(false) {
            databases.push(Database {
                name: "postgresql".to_string(),
                data_dir: "/var/lib/pgsql/data".to_string(),
                config_path: "/var/lib/pgsql/data/postgresql.conf".to_string(),
            });
        }

        // MySQL/MariaDB
        if guestfs.exists("/usr/bin/mysqld").unwrap_or(false)
            || guestfs.exists("/usr/sbin/mysqld").unwrap_or(false) {
            let name = if guestfs.exists("/usr/bin/mariadb").unwrap_or(false) {
                "mariadb"
            } else {
                "mysql"
            };

            databases.push(Database {
                name: name.to_string(),
                data_dir: "/var/lib/mysql".to_string(),
                config_path: "/etc/my.cnf".to_string(),
            });
        }

        // MongoDB
        if guestfs.exists("/usr/bin/mongod").unwrap_or(false) {
            databases.push(Database {
                name: "mongodb".to_string(),
                data_dir: "/var/lib/mongo".to_string(),
                config_path: "/etc/mongod.conf".to_string(),
            });
        }

        // Redis
        if guestfs.exists("/usr/bin/redis-server").unwrap_or(false) {
            databases.push(Database {
                name: "redis".to_string(),
                data_dir: "/var/lib/redis".to_string(),
                config_path: "/etc/redis.conf".to_string(),
            });
        }

        Ok(databases)
    })
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Database {
    pub name: String,
    pub data_dir: String,
    pub config_path: String,
}
```

### 2.6 Add Security Tool Detection ⭐

```rust
/// Detect security tools and hardening
pub fn inspect_security(&mut self, root: &str) -> Result<SecurityInfo> {
    self.with_mount(root, |guestfs| {
        let mut sec_info = SecurityInfo {
            selinux: "unknown".to_string(),
            apparmor: false,
            fail2ban: false,
            aide: false,
            auditd: false,
            ssh_keys: Vec::new(),
        };

        // SELinux
        if let Ok(content) = guestfs.cat("/etc/selinux/config") {
            for line in content.lines() {
                if line.starts_with("SELINUX=") {
                    sec_info.selinux = line.split('=').nth(1).unwrap_or("unknown").to_string();
                }
            }
        }

        // AppArmor
        sec_info.apparmor = guestfs.exists("/etc/apparmor.d").unwrap_or(false)
            || guestfs.exists("/sys/kernel/security/apparmor").unwrap_or(false);

        // fail2ban
        sec_info.fail2ban = guestfs.exists("/etc/fail2ban").unwrap_or(false);

        // AIDE
        sec_info.aide = guestfs.exists("/usr/sbin/aide").unwrap_or(false);

        // auditd
        sec_info.auditd = guestfs.exists("/sbin/auditd").unwrap_or(false);

        // SSH authorized keys
        if let Ok(users) = guestfs.ls("/home") {
            for user in users {
                let key_path = format!("/home/{}/.ssh/authorized_keys", user);
                if let Ok(content) = guestfs.cat(&key_path) {
                    let key_count = content.lines()
                        .filter(|l| !l.trim().is_empty() && !l.starts_with('#'))
                        .count();
                    if key_count > 0 {
                        sec_info.ssh_keys.push((user, key_count));
                    }
                }
            }
        }

        Ok(sec_info)
    })
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecurityInfo {
    pub selinux: String,
    pub apparmor: bool,
    pub fail2ban: bool,
    pub aide: bool,
    pub auditd: bool,
    pub ssh_keys: Vec<(String, usize)>,  // (username, key_count)
}
```

### 2.7 Add Kernel Module Detection ⭐

```rust
/// List loaded kernel modules
pub fn inspect_kernel_modules(&mut self, root: &str) -> Result<Vec<String>> {
    self.with_mount(root, |guestfs| {
        let mut modules = Vec::new();

        // Get list of modules to load at boot
        if let Ok(content) = guestfs.cat("/etc/modules") {
            for line in content.lines() {
                let line = line.trim();
                if !line.is_empty() && !line.starts_with('#') {
                    modules.push(line.to_string());
                }
            }
        }

        // RHEL/Fedora style
        if let Ok(files) = guestfs.ls("/etc/modules-load.d") {
            for file in files.iter().filter(|f| f.ends_with(".conf")) {
                let path = format!("/etc/modules-load.d/{}", file);
                if let Ok(content) = guestfs.cat(&path) {
                    for line in content.lines() {
                        let line = line.trim();
                        if !line.is_empty() && !line.starts_with('#') {
                            modules.push(line.to_string());
                        }
                    }
                }
            }
        }

        Ok(modules)
    })
}
```

### 2.8 Add Hosts File Parsing ⭐

```rust
/// Parse /etc/hosts file
pub fn inspect_hosts(&mut self, root: &str) -> Result<Vec<HostEntry>> {
    self.with_mount(root, |guestfs| {
        let mut entries = Vec::new();

        if let Ok(content) = guestfs.cat("/etc/hosts") {
            for line in content.lines() {
                let line = line.trim();
                if line.is_empty() || line.starts_with('#') {
                    continue;
                }

                let parts: Vec<&str> = line.split_whitespace().collect();
                if parts.len() >= 2 {
                    entries.push(HostEntry {
                        ip: parts[0].to_string(),
                        hostnames: parts[1..].iter().map(|s| s.to_string()).collect(),
                    });
                }
            }
        }

        Ok(entries)
    })
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HostEntry {
    pub ip: String,
    pub hostnames: Vec<String>,
}
```

---

## Priority 3: Error Handling Improvements

### 3.1 Better Error Context

Instead of silently returning empty results on mount failure, consider:

```rust
pub fn inspect_dns(&mut self, root: &str) -> Result<Vec<String>> {
    self.with_mount(root, |guestfs| {
        let content = guestfs.cat("/etc/resolv.conf")
            .map_err(|e| Error::InvalidState(format!("Failed to read resolv.conf: {}", e)))?;

        let mut dns_servers = Vec::new();
        for line in content.lines() {
            let line = line.trim();
            if line.starts_with("nameserver ") {
                if let Some(server) = line.split_whitespace().nth(1) {
                    dns_servers.push(server.to_string());
                }
            }
        }

        Ok(dns_servers)
    })
}
```

---

## Priority 4: Performance Optimizations

### 4.1 Reduce Multiple Mount Operations

For operations that read multiple files, batch them:

```rust
/// Get comprehensive system configuration in one mount
pub fn inspect_system_config(&mut self, root: &str) -> Result<SystemConfig> {
    self.with_mount(root, |guestfs| {
        let config = SystemConfig {
            hostname: guestfs.cat("/etc/hostname").ok(),
            timezone: guestfs.cat("/etc/timezone").ok(),
            locale: parse_locale(guestfs.cat("/etc/locale.conf").ok()),
            dns_servers: parse_resolv_conf(guestfs.cat("/etc/resolv.conf").ok()),
            hosts: parse_hosts(guestfs.cat("/etc/hosts").ok()),
        };
        Ok(config)
    })
}
```

---

## Priority 5: Documentation Improvements

Add doc comments with examples:

```rust
/// Inspect network configuration from the guest filesystem
///
/// Detects network interfaces from:
/// - Debian/Ubuntu: `/etc/network/interfaces`
/// - RHEL/CentOS/Fedora: `/etc/sysconfig/network-scripts/ifcfg-*`
/// - Ubuntu 17.10+: `/etc/netplan/*.yaml`
///
/// # Arguments
/// * `root` - Root device (e.g., "/dev/sda1")
///
/// # Returns
/// Vector of `NetworkInterface` structs containing name, IPs, MAC, and DHCP status
///
/// # Example
/// ```no_run
/// let interfaces = guestfs.inspect_network("/dev/sda1")?;
/// for iface in interfaces {
///     println!("{}: {}", iface.name, iface.ip_address.join(", "));
/// }
/// ```
pub fn inspect_network(&mut self, root: &str) -> Result<Vec<NetworkInterface>> {
    // ...
}
```

---

## Summary of Recommendations

**High Priority** (implement first):
1. ✅ Add `with_mount()` helper method - saves 400+ lines
2. ✅ Fix unused `current_title` variable
3. ✅ Add package manager detection (`inspect_packages()`)
4. ✅ Add firewall detection (`inspect_firewall()`)

**Medium Priority**:
5. Add init system detection (`inspect_init_system()`)
6. Add web server detection (`inspect_web_servers()`)
7. Add database detection (`inspect_databases()`)
8. Add security tool detection (`inspect_security()`)

**Low Priority** (nice to have):
9. Add kernel module listing
10. Add hosts file parsing
11. Add better error contexts
12. Add comprehensive doc comments
13. Extract constants for file paths

**Estimated Impact**:
- Lines reduced: ~400+ (via with_mount helper)
- New functionality: 8 major inspection capabilities
- Code maintainability: Significantly improved
- Error handling: More robust with proper context
