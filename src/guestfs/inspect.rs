// SPDX-License-Identifier: LGPL-3.0-or-later
//! OS inspection APIs compatible with libguestfs

use crate::core::{Error, Result};
use crate::disk::FileSystem;
use crate::guestfs::Guestfs;
use std::collections::HashMap;

/// OS inspection information
#[derive(Debug, Clone)]
pub struct InspectedOS {
    pub root: String,
    pub os_type: String,
    pub distro: String,
    pub product_name: String,
    pub major_version: i32,
    pub minor_version: i32,
    pub arch: String,
    pub hostname: String,
    pub package_format: String,
    pub mountpoints: HashMap<String, String>,
}

impl Guestfs {
    /// Inspect operating systems in the disk image
    ///
    /// Returns a list of root devices where operating systems were found.
    /// GuestFS API: inspect_os()
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use guestctl::guestfs::Guestfs;
    ///
    /// let mut g = Guestfs::new().unwrap();
    /// g.add_drive_ro("/path/to/disk.qcow2").unwrap();
    /// g.launch().unwrap();
    ///
    /// let roots = g.inspect_os().unwrap();
    /// for root in roots {
    ///     println!("Found OS at: {}", root);
    /// }
    /// ```
    pub fn inspect_os(&mut self) -> Result<Vec<String>> {
        self.ensure_ready()?;

        let mut roots = Vec::new();

        // Clone partition data to avoid borrow checker issues
        let partitions: Vec<_> = {
            let partition_table = self.partition_table()?;
            partition_table.partitions().to_vec()
        };

        // Examine each partition
        for partition in &partitions {
            let device_name = format!("/dev/sda{}", partition.number);

            // Try to detect filesystem
            let reader = self
                .reader
                .as_mut()
                .ok_or_else(|| Error::InvalidState("Reader not initialized".to_string()))?;
            if let Ok(fs) = FileSystem::detect(reader, partition) {
                // Check if this looks like a root filesystem
                match fs.fs_type() {
                    crate::disk::FileSystemType::Ext
                    | crate::disk::FileSystemType::Xfs
                    | crate::disk::FileSystemType::Btrfs
                    | crate::disk::FileSystemType::Ntfs => {
                        // This could be a root partition
                        roots.push(device_name);
                    }
                    _ => {}
                }
            }
        }

        Ok(roots)
    }

    /// Get the type of operating system
    ///
    /// GuestFS API: inspect_get_type()
    pub fn inspect_get_type(&mut self, root: &str) -> Result<String> {
        self.ensure_ready()?;

        // Parse device name to get partition
        let partition_num = self.parse_device_name(root)?;

        // Clone partition to avoid borrow checker issues
        let partition = {
            let partition_table = self.partition_table()?;
            partition_table
                .partitions()
                .iter()
                .find(|p| p.number == partition_num)
                .cloned()
                .ok_or_else(|| Error::NotFound(format!("Partition {} not found", partition_num)))?
        };

        // Detect filesystem
        let reader = self
            .reader
            .as_mut()
            .ok_or_else(|| Error::InvalidState("Reader not initialized".to_string()))?;
        let fs = FileSystem::detect(reader, &partition)?;

        match fs.fs_type() {
            crate::disk::FileSystemType::Ntfs => Ok("windows".to_string()),
            crate::disk::FileSystemType::Ext
            | crate::disk::FileSystemType::Xfs
            | crate::disk::FileSystemType::Btrfs => Ok("linux".to_string()),
            _ => Ok("unknown".to_string()),
        }
    }

    /// Get the distribution name
    ///
    /// GuestFS API: inspect_get_distro()
    pub fn inspect_get_distro(&mut self, root: &str) -> Result<String> {
        self.ensure_ready()?;

        // Try to read /etc/os-release first
        if let Ok(os_release) = self.read_os_release(root) {
            return Ok(os_release.id);
        }

        let partition_num = self.parse_device_name(root)?;

        // Clone partition to avoid borrow checker issues
        let partition = {
            let partition_table = self.partition_table()?;
            partition_table
                .partitions()
                .iter()
                .find(|p| p.number == partition_num)
                .cloned()
                .ok_or_else(|| Error::NotFound(format!("Partition {} not found", partition_num)))?
        };

        // Detect filesystem
        let reader = self
            .reader
            .as_mut()
            .ok_or_else(|| Error::InvalidState("Reader not initialized".to_string()))?;
        let fs = FileSystem::detect(reader, &partition)?;

        // Try to infer distribution from filesystem label
        if let Some(label) = fs.label() {
            if label.contains("fedora") || label.contains("Fedora") {
                return Ok("fedora".to_string());
            } else if label.contains("ubuntu") || label.contains("Ubuntu") {
                return Ok("ubuntu".to_string());
            } else if label.contains("debian") || label.contains("Debian") {
                return Ok("debian".to_string());
            } else if label.contains("rhel") || label.contains("RHEL") {
                return Ok("rhel".to_string());
            } else if label.contains("centos") || label.contains("CentOS") {
                return Ok("centos".to_string());
            } else if label.contains("photon") || label.contains("Photon") {
                return Ok("photon".to_string());
            }
        }

        Ok("unknown".to_string())
    }

    /// Read and parse /etc/os-release
    fn read_os_release(&mut self, root: &str) -> Result<OsRelease> {
        // Try to mount the root partition first
        let was_mounted = self.mounted.contains_key("/");

        if !was_mounted {
            // Try to mount root temporarily
            self.mount(root, "/").ok();
        }

        let os_release_content = self
            .cat("/etc/os-release")
            .or_else(|_| self.cat("/usr/lib/os-release"))?;

        if !was_mounted {
            self.umount("/").ok();
        }

        OsRelease::parse(&os_release_content)
    }

    /// Get the product name
    ///
    /// GuestFS API: inspect_get_product_name()
    pub fn inspect_get_product_name(&mut self, root: &str) -> Result<String> {
        // Try to get from /etc/os-release first
        if let Ok(os_release) = self.read_os_release(root) {
            return Ok(os_release.pretty_name);
        }

        let os_type = self.inspect_get_type(root)?;
        let distro = self.inspect_get_distro(root)?;

        if os_type == "windows" {
            Ok("Windows".to_string())
        } else if os_type == "linux" {
            match distro.as_str() {
                "fedora" => Ok("Fedora Linux".to_string()),
                "ubuntu" => Ok("Ubuntu".to_string()),
                "debian" => Ok("Debian GNU/Linux".to_string()),
                "rhel" => Ok("Red Hat Enterprise Linux".to_string()),
                "centos" => Ok("CentOS Linux".to_string()),
                "photon" => Ok("VMware Photon OS".to_string()),
                _ => Ok("Linux".to_string()),
            }
        } else {
            Ok("Unknown".to_string())
        }
    }

    /// Get the architecture
    ///
    /// GuestFS API: inspect_get_arch()
    pub fn inspect_get_arch(&mut self, _root: &str) -> Result<String> {
        self.ensure_ready()?;
        // For now, assume x86_64 (TODO: detect from ELF binaries or PE headers)
        Ok("x86_64".to_string())
    }

    /// Get the major version number
    ///
    /// GuestFS API: inspect_get_major_version()
    pub fn inspect_get_major_version(&mut self, root: &str) -> Result<i32> {
        self.ensure_ready()?;

        if let Ok(os_release) = self.read_os_release(root) {
            return Ok(os_release.version_major);
        }

        Ok(0)
    }

    /// Get the minor version number
    ///
    /// GuestFS API: inspect_get_minor_version()
    pub fn inspect_get_minor_version(&mut self, root: &str) -> Result<i32> {
        self.ensure_ready()?;

        if let Ok(os_release) = self.read_os_release(root) {
            return Ok(os_release.version_minor);
        }

        Ok(0)
    }

    /// Get the hostname
    ///
    /// GuestFS API: inspect_get_hostname()
    pub fn inspect_get_hostname(&mut self, root: &str) -> Result<String> {
        self.ensure_ready()?;

        // Try to mount and read /etc/hostname
        let was_mounted = self.mounted.contains_key("/");

        if !was_mounted {
            self.mount(root, "/").ok();
        }

        let hostname = self
            .cat("/etc/hostname")
            .map(|content| content.trim().to_string())
            .unwrap_or_else(|_| "localhost".to_string());

        if !was_mounted {
            self.umount("/").ok();
        }

        Ok(hostname)
    }

    /// Get the package format (rpm, deb, etc.)
    ///
    /// GuestFS API: inspect_get_package_format()
    pub fn inspect_get_package_format(&mut self, root: &str) -> Result<String> {
        let distro = self.inspect_get_distro(root)?;

        match distro.as_str() {
            "fedora" | "rhel" | "centos" | "photon" => Ok("rpm".to_string()),
            "ubuntu" | "debian" => Ok("deb".to_string()),
            _ => Ok("unknown".to_string()),
        }
    }

    /// Get mountpoints for the root device
    ///
    /// GuestFS API: inspect_get_mountpoints()
    pub fn inspect_get_mountpoints(&mut self, root: &str) -> Result<HashMap<String, String>> {
        self.ensure_ready()?;

        let mut mountpoints = HashMap::new();

        // Root is always mounted at /
        mountpoints.insert("/".to_string(), root.to_string());

        // TODO: Parse fstab or other mount configuration
        // For now, just return root

        Ok(mountpoints)
    }

    /// List installed applications
    ///
    /// GuestFS API: inspect_list_applications()
    pub fn inspect_list_applications(&mut self, _root: &str) -> Result<Vec<Application>> {
        self.ensure_ready()?;

        // TODO: Parse RPM database or dpkg status
        // For now, return empty list

        Ok(Vec::new())
    }

    /// Check if this is a live CD/USB
    ///
    /// GuestFS API: inspect_is_live()
    pub fn inspect_is_live(&mut self, _root: &str) -> Result<bool> {
        self.ensure_ready()?;
        // TODO: Check for live indicators
        Ok(false)
    }
}

/// Installed application information
#[derive(Debug, Clone)]
pub struct Application {
    pub name: String,
    pub display_name: String,
    pub epoch: i32,
    pub version: String,
    pub release: String,
    pub arch: String,
    pub install_path: String,
    pub publisher: String,
    pub url: String,
    pub description: String,
}

/// Parsed /etc/os-release information
#[derive(Debug, Clone)]
#[allow(dead_code)]
struct OsRelease {
    pub id: String,
    pub pretty_name: String,
    pub version_id: String,
    pub version_major: i32,
    pub version_minor: i32,
    pub cpe_name: String,
    pub support_end: String,
    pub home_url: String,
    pub bug_report_url: String,
}

impl OsRelease {
    fn parse(content: &str) -> Result<Self> {
        let mut id = String::new();
        let mut pretty_name = String::new();
        let mut version_id = String::new();
        let mut cpe_name = String::new();
        let mut support_end = String::new();
        let mut home_url = String::new();
        let mut bug_report_url = String::new();

        for line in content.lines() {
            let line = line.trim();
            if line.is_empty() || line.starts_with('#') {
                continue;
            }

            if let Some((key, value)) = line.split_once('=') {
                let value = value.trim_matches('"').trim();

                match key {
                    "ID" => id = value.to_lowercase(),
                    "PRETTY_NAME" => pretty_name = value.to_string(),
                    "VERSION_ID" => version_id = value.to_string(),
                    "CPE_NAME" => cpe_name = value.to_string(),
                    "SUPPORT_END" => support_end = value.to_string(),
                    "HOME_URL" => home_url = value.to_string(),
                    "BUG_REPORT_URL" => bug_report_url = value.to_string(),
                    _ => {}
                }
            }
        }

        // Parse version into major.minor
        let (version_major, version_minor) = if !version_id.is_empty() {
            let parts: Vec<&str> = version_id.split('.').collect();
            let major = parts.first().and_then(|s| s.parse().ok()).unwrap_or(0);
            let minor = parts.get(1).and_then(|s| s.parse().ok()).unwrap_or(0);
            (major, minor)
        } else {
            (0, 0)
        };

        if id.is_empty() {
            return Err(Error::NotFound("ID not found in os-release".to_string()));
        }

        Ok(OsRelease {
            id,
            pretty_name,
            version_id,
            version_major,
            version_minor,
            cpe_name,
            support_end,
            home_url,
            bug_report_url,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_inspect_api_exists() {
        let g = Guestfs::new().unwrap();
        // API structure tests
        let _ = g;
    }

    #[test]
    fn test_os_release_parse_photon() {
        let content = r#"
NAME="VMware Photon OS"
VERSION="5.0"
ID=photon
VERSION_ID=5.0
PRETTY_NAME="VMware Photon OS/Linux"
"#;
        let os_release = OsRelease::parse(content).unwrap();
        assert_eq!(os_release.id, "photon");
        assert_eq!(os_release.version_major, 5);
        assert_eq!(os_release.version_minor, 0);
    }

    #[test]
    fn test_os_release_parse_fedora() {
        let content = r#"
NAME="Fedora Linux"
VERSION="39 (Server Edition)"
ID=fedora
VERSION_ID=39
PRETTY_NAME="Fedora Linux 39 (Server Edition)"
"#;
        let os_release = OsRelease::parse(content).unwrap();
        assert_eq!(os_release.id, "fedora");
        assert_eq!(os_release.version_major, 39);
        assert_eq!(os_release.version_minor, 0);
    }
}
