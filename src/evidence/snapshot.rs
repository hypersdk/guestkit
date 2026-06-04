// SPDX-License-Identifier: LGPL-3.0-or-later
//! Evidence snapshot schema (v2).

use serde::{Deserialize, Serialize};

pub const SCHEMA_VERSION: u32 = 2;

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
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub systemd: Option<SystemdInfo>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
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

/// Systemd unit and static analysis collected from disk.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct SystemdInfo {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub version: Option<String>,
    pub unit_count: usize,
    pub service_count: usize,
    pub timer_count: usize,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub units: Vec<SystemdUnit>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub problem_hints: Vec<SystemdProblemHint>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct SystemdUnit {
    pub name: String,
    pub unit_type: String,
    pub path: String,
    pub state: SystemdUnitState,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub exec_start: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub service_type: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub restart: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub remain_after_exit: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub user: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub group: Option<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub after: Vec<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub before: Vec<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub requires: Vec<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub wants: Vec<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub wanted_by: Vec<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub on_calendar: Option<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub sections: Vec<SystemdUnitSection>,
}

#[derive(Debug, Clone, Copy, Default, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum SystemdUnitState {
    #[default]
    Disabled,
    Enabled,
    Masked,
    Static,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct SystemdUnitSection {
    pub name: String,
    pub keys: std::collections::BTreeMap<String, String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SystemdProblemHint {
    pub unit: String,
    pub code: String,
    pub severity: SystemdProblemSeverity,
    pub message: String,
    pub path: String,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum SystemdProblemSeverity {
    Info,
    Warning,
    Critical,
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
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub services: Vec<WindowsServiceEntry>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub installed_apps: Vec<WindowsAppEntry>,
    #[serde(default)]
    pub persistence: WindowsPersistenceEvidence,
    #[serde(default)]
    pub event_logs: WindowsEventLogSummary,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct WindowsServiceEntry {
    pub name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub display_name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub image_path: Option<String>,
    pub start_type: WindowsStartType,
    pub service_type: WindowsServiceType,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub object_name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    pub kernel_driver: bool,
    pub auto_start: bool,
}

#[derive(Debug, Clone, Copy, Default, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum WindowsStartType {
    Boot,
    System,
    Automatic,
    Manual,
    Disabled,
    #[default]
    Unknown,
}

#[derive(Debug, Clone, Copy, Default, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum WindowsServiceType {
    KernelDriver,
    FileSystemDriver,
    Win32,
    #[default]
    Unknown,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct WindowsAppEntry {
    pub name: String,
    pub version: String,
    pub publisher: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub install_location: Option<String>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct WindowsPersistenceEvidence {
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub run_keys: Vec<WindowsPersistenceEntry>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub scheduled_tasks: Vec<String>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct WindowsPersistenceEntry {
    pub location: String,
    pub name: String,
    pub command: String,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct WindowsEventLogSummary {
    pub log_count: usize,
    pub total_bytes: u64,
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
            ["systemd", "service_count"] => self
                .systemd
                .as_ref()
                .map(|s| s.service_count.to_string()),
            ["systemd", "problem_count"] => self
                .systemd
                .as_ref()
                .map(|s| s.problem_hints.len().to_string()),
            _ => None,
        }
    }
}
