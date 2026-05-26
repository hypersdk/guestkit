// SPDX-License-Identifier: LGPL-3.0-or-later
//! Evidence snapshot schema (v1).

use serde::{Deserialize, Serialize};

pub const SCHEMA_VERSION: u32 = 1;

/// Normalized evidence collected from an offline disk image.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EvidenceSnapshot {
    pub schema_version: u32,
    pub image_path: String,
    pub collected_at: String,
    pub root: String,
    pub os: OsEvidence,
    pub storage: StorageEvidence,
    pub boot: BootEvidence,
    pub network: NetworkEvidence,
    pub packages: PackageEvidence,
    pub security: SecurityEvidence,
    pub vm_tools: VmToolsEvidence,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub windows: Option<WindowsEvidence>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct OsEvidence {
    pub os_type: String,
    pub distribution: String,
    pub version: String,
    pub architecture: String,
    pub hostname: String,
    pub init_system: String,
    pub package_manager: String,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct StorageEvidence {
    pub fstab_entries: Vec<FstabEntry>,
    pub crypttab_entries: Vec<CrypttabEntry>,
    pub swap_devices: Vec<String>,
    pub root_filesystem: String,
    pub partition_uuids: Vec<PartitionUuid>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FstabEntry {
    pub device: String,
    pub mountpoint: String,
    pub fstype: String,
    pub options: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CrypttabEntry {
    pub name: String,
    pub device: String,
    pub keyfile: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PartitionUuid {
    pub device: String,
    pub uuid: String,
    pub fstype: String,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct BootEvidence {
    pub bootloader: String,
    pub default_entry: String,
    pub kernel_cmdline: String,
    pub kernel_paths: Vec<String>,
    pub initramfs_paths: Vec<String>,
    pub efi_present: bool,
    pub grub_cfg_path: Option<String>,
    pub loaded_modules: Vec<String>,
    pub pending_relabel: bool,
    pub cloud_init_present: bool,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct NetworkEvidence {
    pub interfaces: Vec<String>,
    pub dns_servers: Vec<String>,
    pub udev_persistent_net: Vec<String>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct PackageEvidence {
    pub count: usize,
    pub kernels: Vec<String>,
    pub sample_packages: Vec<String>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct SecurityEvidence {
    pub selinux: String,
    pub apparmor: bool,
    pub firewall_enabled: bool,
    pub ssh_root_login: Option<bool>,
    pub auditd: bool,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct VmToolsEvidence {
    pub detected: Vec<String>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct WindowsEvidence {
    pub systemroot: String,
    pub product_name: String,
    pub version: String,
    pub domain_joined: bool,
    pub domain_name: Option<String>,
    pub rdp_enabled: bool,
    pub pending_reboot: bool,
    pub bitlocker_detected: bool,
    pub installed_apps_count: usize,
    pub services_count: usize,
    pub drivers_count: usize,
    pub hypervisor_remnants: Vec<String>,
    pub av_edr: Vec<String>,
    pub minidump_count: usize,
}

impl EvidenceSnapshot {
    pub fn get_field(&self, path: &str) -> Option<String> {
        let parts: Vec<&str> = path.split('.').collect();
        match parts.as_slice() {
            ["os", "distribution"] => Some(self.os.distribution.clone()),
            ["os", "version"] => Some(self.os.version.clone()),
            ["os", "hostname"] => Some(self.os.hostname.clone()),
            ["security", "selinux"] => Some(self.security.selinux.clone()),
            ["security", "ssh", "root_login"] => {
                self.security.ssh_root_login.map(|v| v.to_string())
            }
            ["boot", "bootloader"] => Some(self.boot.bootloader.clone()),
            ["packages", "count"] => Some(self.packages.count.to_string()),
            _ => None,
        }
    }
}
