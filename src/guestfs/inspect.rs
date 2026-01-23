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
    /// Compatible with libguestfs g.inspect_os()
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use guestkit::guestfs::Guestfs;
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
        let partition_table = self.partition_table.as_ref().unwrap();

        // Examine each partition
        for partition in partition_table.partitions() {
            let device_name = format!("/dev/sda{}", partition.number);

            // Try to detect filesystem
            if let Ok(fs) = FileSystem::detect(self.reader.as_mut().unwrap(), partition) {
                // Check if this looks like a root filesystem
                match fs.fs_type() {
                    crate::disk::FileSystemType::Ext |
                    crate::disk::FileSystemType::Xfs |
                    crate::disk::FileSystemType::Btrfs |
                    crate::disk::FileSystemType::Ntfs => {
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
    /// Compatible with libguestfs g.inspect_get_type()
    pub fn inspect_get_type(&mut self, root: &str) -> Result<String> {
        self.ensure_ready()?;

        // Parse device name to get partition
        let partition_num = self.parse_device_name(root)?;
        let partition_table = self.partition_table.as_ref().unwrap();

        let partition = partition_table.partitions()
            .iter()
            .find(|p| p.number == partition_num)
            .ok_or_else(|| Error::NotFound(format!("Partition {} not found", partition_num)))?;

        // Detect filesystem
        let fs = FileSystem::detect(self.reader.as_mut().unwrap(), partition)?;

        match fs.fs_type() {
            crate::disk::FileSystemType::Ntfs => Ok("windows".to_string()),
            crate::disk::FileSystemType::Ext |
            crate::disk::FileSystemType::Xfs |
            crate::disk::FileSystemType::Btrfs => Ok("linux".to_string()),
            _ => Ok("unknown".to_string()),
        }
    }

    /// Get the distribution name
    ///
    /// Compatible with libguestfs g.inspect_get_distro()
    pub fn inspect_get_distro(&mut self, root: &str) -> Result<String> {
        self.ensure_ready()?;

        let partition_num = self.parse_device_name(root)?;
        let partition_table = self.partition_table.as_ref().unwrap();

        let partition = partition_table.partitions()
            .iter()
            .find(|p| p.number == partition_num)
            .ok_or_else(|| Error::NotFound(format!("Partition {} not found", partition_num)))?;

        // Detect filesystem
        let fs = FileSystem::detect(self.reader.as_mut().unwrap(), partition)?;

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
            }
        }

        Ok("unknown".to_string())
    }

    /// Get the product name
    ///
    /// Compatible with libguestfs g.inspect_get_product_name()
    pub fn inspect_get_product_name(&mut self, root: &str) -> Result<String> {
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
                _ => Ok("Linux".to_string()),
            }
        } else {
            Ok("Unknown".to_string())
        }
    }

    /// Get the architecture
    ///
    /// Compatible with libguestfs g.inspect_get_arch()
    pub fn inspect_get_arch(&mut self, _root: &str) -> Result<String> {
        self.ensure_ready()?;
        // For now, assume x86_64 (TODO: detect from ELF binaries or PE headers)
        Ok("x86_64".to_string())
    }

    /// Get the major version number
    ///
    /// Compatible with libguestfs g.inspect_get_major_version()
    pub fn inspect_get_major_version(&mut self, _root: &str) -> Result<i32> {
        self.ensure_ready()?;
        // TODO: Read from /etc/os-release or registry
        Ok(0)
    }

    /// Get the minor version number
    ///
    /// Compatible with libguestfs g.inspect_get_minor_version()
    pub fn inspect_get_minor_version(&mut self, _root: &str) -> Result<i32> {
        self.ensure_ready()?;
        // TODO: Read from /etc/os-release or registry
        Ok(0)
    }

    /// Get the hostname
    ///
    /// Compatible with libguestfs g.inspect_get_hostname()
    pub fn inspect_get_hostname(&mut self, _root: &str) -> Result<String> {
        self.ensure_ready()?;
        // TODO: Read from /etc/hostname or registry
        Ok("localhost".to_string())
    }

    /// Get the package format (rpm, deb, etc.)
    ///
    /// Compatible with libguestfs g.inspect_get_package_format()
    pub fn inspect_get_package_format(&mut self, root: &str) -> Result<String> {
        let distro = self.inspect_get_distro(root)?;

        match distro.as_str() {
            "fedora" | "rhel" | "centos" => Ok("rpm".to_string()),
            "ubuntu" | "debian" => Ok("deb".to_string()),
            _ => Ok("unknown".to_string()),
        }
    }

    /// Get mountpoints for the root device
    ///
    /// Compatible with libguestfs g.inspect_get_mountpoints()
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
    /// Compatible with libguestfs g.inspect_list_applications()
    pub fn inspect_list_applications(&mut self, _root: &str) -> Result<Vec<Application>> {
        self.ensure_ready()?;

        // TODO: Parse RPM database or dpkg status
        // For now, return empty list

        Ok(Vec::new())
    }

    /// Check if this is a live CD/USB
    ///
    /// Compatible with libguestfs g.inspect_is_live()
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_inspect_api_exists() {
        let g = Guestfs::new().unwrap();
        // API structure tests
        let _ = g;
    }
}
