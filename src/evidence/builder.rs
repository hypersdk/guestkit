// SPDX-License-Identifier: LGPL-3.0-or-later
//! Build evidence snapshots from Guestfs inspection.

use super::snapshot::*;
use crate::Guestfs;
use anyhow::Result;
use chrono::Utc;
use std::path::Path;

pub struct EvidenceBuilder;

/// Collect a full evidence snapshot from a mounted guestfs instance.
pub fn build_evidence(g: &mut Guestfs, root: &str, image_path: &Path) -> Result<EvidenceSnapshot> {
    EvidenceBuilder::build(g, root, image_path)
}

impl EvidenceBuilder {
    pub fn build(g: &mut Guestfs, root: &str, image_path: &Path) -> Result<EvidenceSnapshot> {
        let os = Self::collect_os(g, root);
        let storage = Self::collect_storage(g, root);
        let boot = Self::collect_boot(g, root);
        let network = Self::collect_network(g, root);
        let packages = Self::collect_packages(g, root);
        let security = Self::collect_security(g, root);
        let vm_tools = Self::collect_vm_tools(g, root);
        let windows = if os.os_type.to_lowercase().contains("windows") {
            Some(Self::collect_windows(g, root))
        } else {
            None
        };

        Ok(EvidenceSnapshot {
            schema_version: SCHEMA_VERSION,
            image_path: image_path.display().to_string(),
            collected_at: Utc::now().to_rfc3339(),
            root: root.to_string(),
            os,
            storage,
            boot,
            network,
            packages,
            security,
            vm_tools,
            windows,
        })
    }

    fn collect_os(g: &mut Guestfs, root: &str) -> OsEvidence {
        let version = match (
            g.inspect_get_major_version(root),
            g.inspect_get_minor_version(root),
        ) {
            (Ok(major), Ok(minor)) => format!("{}.{}", major, minor),
            _ => String::new(),
        };

        OsEvidence {
            os_type: g.inspect_get_type(root).unwrap_or_default(),
            distribution: g.inspect_get_distro(root).unwrap_or_default(),
            version,
            architecture: g.inspect_get_arch(root).unwrap_or_default(),
            hostname: g.inspect_get_hostname(root).unwrap_or_default(),
            init_system: g.inspect_get_init_system(root).unwrap_or_default(),
            package_manager: g.inspect_get_package_management(root).unwrap_or_default(),
        }
    }

    fn collect_storage(g: &mut Guestfs, root: &str) -> StorageEvidence {
        let fstab_content = g
            .read_file("/etc/fstab")
            .ok()
            .map(|c| String::from_utf8_lossy(&c).into_owned())
            .unwrap_or_default();

        let fstab_entries: Vec<FstabEntry> = g
            .inspect_fstab(root)
            .unwrap_or_default()
            .into_iter()
            .map(|(device, mountpoint, fstype)| {
                let options = fstab_content
                    .lines()
                    .find(|l| l.contains(&device) && l.contains(&mountpoint))
                    .and_then(|l| l.split_whitespace().nth(3))
                    .unwrap_or("")
                    .to_string();
                FstabEntry {
                    device,
                    mountpoint,
                    fstype,
                    options,
                }
            })
            .collect();

        let crypttab_entries = Self::parse_crypttab(g);
        let swap_devices = g.inspect_swap(root).unwrap_or_default();
        let root_filesystem = fstab_entries
            .iter()
            .find(|e| e.mountpoint == "/")
            .map(|e| e.fstype.clone())
            .unwrap_or_default();

        let partition_uuids = Self::collect_partition_uuids(g, root);

        StorageEvidence {
            fstab_entries,
            crypttab_entries,
            swap_devices,
            root_filesystem,
            partition_uuids,
        }
    }

    fn parse_crypttab(g: &mut Guestfs) -> Vec<CrypttabEntry> {
        let mut entries = Vec::new();
        if let Ok(content) = g.read_file("/etc/crypttab") {
            let text = String::from_utf8_lossy(&content);
            for line in text.lines() {
                let line = line.trim();
                if line.is_empty() || line.starts_with('#') {
                    continue;
                }
                let parts: Vec<&str> = line.split_whitespace().collect();
                if parts.len() >= 2 {
                    entries.push(CrypttabEntry {
                        name: parts[0].to_string(),
                        device: parts[1].to_string(),
                        keyfile: parts.get(2).unwrap_or(&"none").to_string(),
                    });
                }
            }
        }
        entries
    }

    fn collect_partition_uuids(g: &mut Guestfs, _root: &str) -> Vec<PartitionUuid> {
        let mut uuids = Vec::new();
        if let Ok(blockdevs) = g.list_filesystems() {
            for (device, fstype) in blockdevs {
                if let Ok(output) = g.command(&["blkid", "-o", "export", &device]) {
                    let mut uuid = String::new();
                    for line in output.lines() {
                        if let Some(val) = line.strip_prefix("UUID=") {
                            uuid = val.to_string();
                            break;
                        }
                    }
                    if !uuid.is_empty() {
                        uuids.push(PartitionUuid {
                            device,
                            uuid,
                            fstype,
                        });
                    }
                }
            }
        }
        uuids
    }

    fn collect_boot(g: &mut Guestfs, root: &str) -> BootEvidence {
        let boot_config = g.inspect_boot_config(root).unwrap_or_default();
        let kernel_paths = Self::list_boot_files(g, "/boot", "vmlinuz");
        let initramfs_paths = Self::list_boot_files(g, "/boot", "initrd")
            .into_iter()
            .chain(Self::list_boot_files(g, "/boot", "initramfs"))
            .collect();

        let efi_present = g.exists("/boot/efi").unwrap_or(false)
            || g.exists("/sys/firmware/efi").unwrap_or(false);

        let grub_cfg_path = ["/boot/grub/grub.cfg", "/boot/grub2/grub.cfg"]
            .iter()
            .find(|p| g.exists(p).unwrap_or(false))
            .map(|s| s.to_string());

        let loaded_modules = g
            .read_file("/proc/modules")
            .ok()
            .map(|content| {
                String::from_utf8_lossy(&content)
                    .lines()
                    .filter_map(|l| l.split_whitespace().next())
                    .map(|s| s.to_string())
                    .collect()
            })
            .unwrap_or_else(|| Self::list_module_names(g));

        let pending_relabel = g.exists("/.autorelabel").unwrap_or(false);
        let cloud_init_present = g.exists("/etc/cloud").unwrap_or(false);

        BootEvidence {
            bootloader: boot_config.bootloader,
            default_entry: boot_config.default_entry,
            kernel_cmdline: boot_config.kernel_cmdline,
            kernel_paths,
            initramfs_paths,
            efi_present,
            grub_cfg_path,
            loaded_modules,
            pending_relabel,
            cloud_init_present,
        }
    }

    fn list_boot_files(g: &mut Guestfs, dir: &str, prefix: &str) -> Vec<String> {
        let mut paths = Vec::new();
        if let Ok(entries) = g.ls(dir) {
            for entry in entries {
                if entry.contains(prefix) {
                    paths.push(format!("{}/{}", dir, entry));
                }
            }
        }
        paths
    }

    fn list_module_names(g: &mut Guestfs) -> Vec<String> {
        let mut modules = Vec::new();
        if let Ok(entries) = g.ls("/lib/modules") {
            if let Some(latest) = entries.iter().max() {
                let mod_dir = format!("/lib/modules/{}/kernel", latest);
                Self::walk_modules(g, &mod_dir, &mut modules);
            }
        }
        modules
    }

    fn walk_modules(g: &mut Guestfs, dir: &str, out: &mut Vec<String>) {
        if let Ok(entries) = g.ls(dir) {
            for entry in entries {
                let path = format!("{}/{}", dir, entry);
                if entry.ends_with(".ko") || entry.ends_with(".ko.xz") {
                    out.push(
                        entry
                            .trim_end_matches(".ko.xz")
                            .trim_end_matches(".ko")
                            .to_string(),
                    );
                } else if g.is_dir(&path).unwrap_or(false) {
                    Self::walk_modules(g, &path, out);
                }
            }
        }
    }

    fn collect_network(g: &mut Guestfs, root: &str) -> NetworkEvidence {
        let interfaces = g
            .inspect_network(root)
            .unwrap_or_default()
            .into_iter()
            .map(|i| format!("{}:{}", i.name, i.ip_address.join(",")))
            .collect();
        let dns_servers = g.inspect_dns(root).unwrap_or_default();

        let mut udev_persistent_net = Vec::new();
        if let Ok(content) = g.read_file("/etc/udev/rules.d/70-persistent-net.rules") {
            let text = String::from_utf8_lossy(&content);
            for line in text.lines() {
                if line.contains("NAME=") {
                    udev_persistent_net.push(line.to_string());
                }
            }
        }

        NetworkEvidence {
            interfaces,
            dns_servers,
            udev_persistent_net,
        }
    }

    fn collect_packages(g: &mut Guestfs, root: &str) -> PackageEvidence {
        let pkg_info = g.inspect_packages(root).unwrap_or_default();
        let sample: Vec<String> = pkg_info
            .packages
            .iter()
            .take(50)
            .map(|p| format!("{}={}", p.name, p.version))
            .collect();
        let kernels: Vec<String> = pkg_info
            .packages
            .iter()
            .filter(|p| p.name.starts_with("linux-image") || p.name.starts_with("kernel"))
            .map(|p| format!("{}={}", p.name, p.version))
            .collect();

        PackageEvidence {
            count: pkg_info.packages.len(),
            kernels,
            sample_packages: sample,
        }
    }

    fn collect_security(g: &mut Guestfs, root: &str) -> SecurityEvidence {
        let sec = g.inspect_security(root).unwrap_or_default();
        let firewall_enabled = g.inspect_firewall(root).map(|f| f.enabled).unwrap_or(false);

        let ssh_root_login = g.inspect_ssh_config(root).ok().and_then(|cfg| {
            match cfg.get("PermitRootLogin").map(String::as_str) {
                Some("yes") | Some("without-password") | Some("prohibit-password") => Some(true),
                Some("no") => Some(false),
                _ => None,
            }
        });

        SecurityEvidence {
            selinux: sec.selinux,
            apparmor: sec.apparmor,
            firewall_enabled,
            ssh_root_login,
            auditd: sec.auditd,
        }
    }

    fn collect_vm_tools(g: &mut Guestfs, root: &str) -> VmToolsEvidence {
        VmToolsEvidence {
            detected: g.inspect_vm_tools(root).unwrap_or_default(),
        }
    }

    fn collect_windows(g: &mut Guestfs, root: &str) -> WindowsEvidence {
        use crate::guestfs::windows_registry;
        use std::path::PathBuf;

        let systemroot = g
            .inspect_get_windows_systemroot(root)
            .unwrap_or_else(|_| "/Windows".to_string());

        let software_hive = format!("{}/System32/config/SOFTWARE", systemroot);
        let system_hive = format!("{}/System32/config/SYSTEM", systemroot);

        let installed_apps_count =
            windows_registry::parse_installed_software(PathBuf::from(&software_hive).as_path())
                .map(|a| a.len())
                .unwrap_or(0);

        let services_count =
            windows_registry::parse_windows_services(PathBuf::from(&system_hive).as_path())
                .map(|s| s.len())
                .unwrap_or(0);

        let drivers_path = format!("{}/System32/drivers", systemroot);
        let drivers_count = g.ls(&drivers_path).map(|d| d.len()).unwrap_or(0);

        let (product_name, version) =
            windows_registry::get_windows_version(PathBuf::from(&software_hive).as_path())
                .map(|(n, v, _)| (n, v))
                .unwrap_or_else(|_| (String::new(), String::new()));

        let domain_info =
            windows_registry::parse_domain_info(PathBuf::from(&system_hive).as_path());
        let rdp_enabled =
            windows_registry::parse_rdp_enabled(PathBuf::from(&system_hive).as_path());
        let pending_reboot =
            windows_registry::parse_pending_reboot(PathBuf::from(&system_hive).as_path());
        let bitlocker_detected = g.exists("/$BitLocker").unwrap_or(false)
            || g.exists(&format!("{}/System32/drivers/fvevol.sys", systemroot))
                .unwrap_or(false);
        let hypervisor_remnants = windows_registry::detect_hypervisor_remnants(
            PathBuf::from(&system_hive).as_path(),
            &drivers_path,
            g,
        );
        let av_edr = windows_registry::detect_av_edr(
            PathBuf::from(&software_hive).as_path(),
            g,
            &systemroot,
        );
        let minidump_path = format!("{}/Minidump", systemroot);
        let minidump_count = g.ls(&minidump_path).map(|d| d.len()).unwrap_or(0);

        WindowsEvidence {
            systemroot,
            product_name,
            version,
            domain_joined: domain_info.0,
            domain_name: domain_info.1,
            rdp_enabled,
            pending_reboot,
            bitlocker_detected,
            installed_apps_count,
            services_count,
            drivers_count,
            hypervisor_remnants,
            av_edr,
            minidump_count,
        }
    }
}

// Default impls for guestfs types used above
impl Default for crate::guestfs::BootConfig {
    fn default() -> Self {
        Self {
            bootloader: "unknown".to_string(),
            default_entry: "unknown".to_string(),
            timeout: "unknown".to_string(),
            kernel_cmdline: String::new(),
        }
    }
}

impl Default for crate::guestfs::SecurityInfo {
    fn default() -> Self {
        Self {
            selinux: "unknown".to_string(),
            apparmor: false,
            fail2ban: false,
            aide: false,
            auditd: false,
            ssh_keys: Vec::new(),
        }
    }
}

impl Default for crate::guestfs::PackageInfo {
    fn default() -> Self {
        Self {
            manager: "unknown".to_string(),
            package_count: 0,
            packages: Vec::new(),
        }
    }
}
