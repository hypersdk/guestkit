// SPDX-License-Identifier: Apache-2.0
//! Hardware inventory and guest identity from /sys and config.

use crate::evidence::snapshot::HardwareEvidence;
use std::fs;
use std::path::Path;

const DEFAULT_CONFIG: &str = "/etc/zyvor/guest-agent.toml";

pub fn collect_hardware_evidence() -> HardwareEvidence {
    let mut out = HardwareEvidence::default();
    out.dmi_manufacturer = read_sysfs("/sys/class/dmi/id/sys_vendor");
    out.dmi_product = read_sysfs("/sys/class/dmi/id/product_name");
    out.dmi_uuid = read_sysfs("/sys/class/dmi/id/product_uuid");
    out.machine_id = fs::read_to_string("/etc/machine-id")
        .map(|s| s.trim().to_string())
        .unwrap_or_else(|_| read_sysfs("/var/lib/dbus/machine-id"));
    out.virtio_net_count = count_dir_entries("/sys/class/net", "virtio");
    out.virtio_blk_count = count_dir_entries("/sys/block", "vd");
    if let Ok(entries) = fs::read_dir("/sys/block") {
        for entry in entries.flatten() {
            let name = entry.file_name().to_string_lossy().to_string();
            if !name.is_empty() {
                out.block_devices.push(name);
            }
        }
    }
    out.zeus_vm_uid = load_zeus_vm_uid();
    out
}

fn load_zeus_vm_uid() -> Option<String> {
    let path = std::env::var("ZYVOR_AGENT_CONFIG").unwrap_or_else(|_| DEFAULT_CONFIG.to_string());
    if !Path::new(&path).exists() {
        return std::env::var("ZYVOR_VM_UID").ok();
    }
    let content = fs::read_to_string(&path).ok()?;
    let table: toml::Table = toml::from_str(&content).ok()?;
    table
        .get("zeus_vm_uid")
        .and_then(|v| v.as_str())
        .map(String::from)
        .or_else(|| std::env::var("ZYVOR_VM_UID").ok())
}

fn read_sysfs(path: &str) -> String {
    fs::read_to_string(path)
        .map(|s| s.trim().to_string())
        .unwrap_or_default()
}

fn count_dir_entries(dir: &str, prefix: &str) -> usize {
    fs::read_dir(dir)
        .map(|entries| {
            entries
                .flatten()
                .filter(|e| e.file_name().to_string_lossy().starts_with(prefix))
                .count()
        })
        .unwrap_or(0)
}
