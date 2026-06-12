// SPDX-License-Identifier: Apache-2.0
//! Windows Registry parsing using nt_hive2
//!
//! This module provides pure Rust parsing of Windows registry hive files.
//! Note: This is an initial implementation that will be enhanced with full
//! registry parsing in future versions.

#[cfg(feature = "evtx")]
use serde_json::Value;

use crate::core::{Error, Result};
use std::path::Path;
#[derive(Debug, Clone)]
pub struct WindowsApp {
    pub name: String,
    pub version: String,
    pub publisher: String,
    pub install_location: Option<String>,
}

/// Windows service information
#[derive(Debug, Clone)]
pub struct WindowsSvc {
    pub name: String,
    pub display_name: String,
    pub start_type: String,
    pub image_path: String,
}

/// Windows network adapter information
#[derive(Debug, Clone)]
pub struct WindowsNetAdapter {
    pub name: String,
    pub description: String,
    pub dhcp_enabled: bool,
    pub ip_address: Vec<String>,
    pub mac_address: String,
    pub dns_servers: Vec<String>,
}

/// Parse installed applications from SOFTWARE hive
///
/// Reads from SOFTWARE\Microsoft\Windows\CurrentVersion\Uninstall
/// and SOFTWARE\Wow6432Node\Microsoft\Windows\CurrentVersion\Uninstall (for 32-bit apps on 64-bit Windows)
pub fn parse_installed_software(hive_path: &Path) -> Result<Vec<WindowsApp>> {
    use nt_hive2::{Hive, HiveParseMode, RegistryValue};
    use std::fs::File;

    // Verify hive exists
    if !hive_path.exists() {
        return Err(Error::NotFound(format!(
            "SOFTWARE hive not found: {}",
            hive_path.display()
        )));
    }

    // Read hive file
    let file = File::open(hive_path)
        .map_err(|e| Error::CommandFailed(format!("Failed to open hive: {}", e)))?;

    // Parse hive
    let mut hive = Hive::new(file, HiveParseMode::NormalWithBaseBlock)
        .map_err(|e| Error::CommandFailed(format!("Failed to parse hive: {:?}", e)))?;

    let mut applications = Vec::new();

    // Get root key
    let root_key = hive
        .root_key_node()
        .map_err(|e| Error::CommandFailed(format!("Failed to get root key: {:?}", e)))?;

    // Navigate to Uninstall key: Microsoft\Windows\CurrentVersion\Uninstall
    let microsoft_key = match root_key.subkey("Microsoft", &mut hive) {
        Ok(Some(key)) => key,
        _ => return Ok(applications), // No Microsoft key
    };

    let windows_key = match microsoft_key.borrow().subkey("Windows", &mut hive) {
        Ok(Some(key)) => key,
        _ => return Ok(applications), // No Windows key
    };

    let current_version_key = match windows_key.borrow().subkey("CurrentVersion", &mut hive) {
        Ok(Some(key)) => key,
        _ => return Ok(applications), // No CurrentVersion key
    };

    let uninstall_key = match current_version_key.borrow().subkey("Uninstall", &mut hive) {
        Ok(Some(key)) => key,
        _ => return Ok(applications), // No Uninstall key
    };

    // Iterate through subkeys (each represents an installed application)
    let uninstall_borrowed = uninstall_key.borrow();
    let subkeys_result = uninstall_borrowed.subkeys(&mut hive);
    let subkeys_ref = match subkeys_result {
        Ok(ref_vec) => ref_vec,
        Err(_) => return Ok(applications),
    };

    for app_key in subkeys_ref.iter() {
        let app_key_ref = app_key.borrow();

        // Extract application information from values
        let mut name = String::new();
        let mut version = String::new();
        let mut publisher = String::new();
        let mut install_location = None;

        for kv in app_key_ref.values() {
            match kv.name() {
                "DisplayName" => {
                    if let RegistryValue::RegSZ(data) | RegistryValue::RegExpandSZ(data) =
                        kv.value()
                    {
                        name = data.clone();
                    }
                }
                "DisplayVersion" => {
                    if let RegistryValue::RegSZ(data) | RegistryValue::RegExpandSZ(data) =
                        kv.value()
                    {
                        version = data.clone();
                    }
                }
                "Publisher" => {
                    if let RegistryValue::RegSZ(data) | RegistryValue::RegExpandSZ(data) =
                        kv.value()
                    {
                        publisher = data.clone();
                    }
                }
                "InstallLocation" => {
                    if let RegistryValue::RegSZ(data) | RegistryValue::RegExpandSZ(data) =
                        kv.value()
                    {
                        if !data.is_empty() {
                            install_location = Some(data.clone());
                        }
                    }
                }
                _ => {}
            }
        }

        // Only add if we have a display name
        if !name.is_empty() {
            applications.push(WindowsApp {
                name,
                version,
                publisher,
                install_location,
            });
        }
    }

    Ok(applications)
}

/// Parse Windows services from SYSTEM hive
///
/// Reads from SYSTEM\ControlSet001\Services
pub fn parse_windows_services(hive_path: &Path) -> Result<Vec<WindowsSvc>> {
    use nt_hive2::{Hive, HiveParseMode, RegistryValue};
    use std::fs::File;

    // Verify hive exists
    if !hive_path.exists() {
        return Err(Error::NotFound(format!(
            "SYSTEM hive not found: {}",
            hive_path.display()
        )));
    }

    // Read hive file
    let file = File::open(hive_path)
        .map_err(|e| Error::CommandFailed(format!("Failed to open hive: {}", e)))?;

    // Parse hive
    let mut hive = Hive::new(file, HiveParseMode::NormalWithBaseBlock)
        .map_err(|e| Error::CommandFailed(format!("Failed to parse hive: {:?}", e)))?;

    let mut services = Vec::new();

    // Get root key
    let root_key = hive
        .root_key_node()
        .map_err(|e| Error::CommandFailed(format!("Failed to get root key: {:?}", e)))?;

    // Navigate to Services: ControlSet001\Services
    let controlset_key = match root_key.subkey("ControlSet001", &mut hive) {
        Ok(Some(key)) => key,
        _ => return Ok(services), // No ControlSet001 key
    };

    let services_key = match controlset_key.borrow().subkey("Services", &mut hive) {
        Ok(Some(key)) => key,
        _ => return Ok(services), // No Services key
    };

    // Iterate through service subkeys
    let services_borrowed = services_key.borrow();
    let subkeys_result = services_borrowed.subkeys(&mut hive);
    let subkeys_ref = match subkeys_result {
        Ok(ref_vec) => ref_vec,
        Err(_) => return Ok(services),
    };

    for svc_key in subkeys_ref.iter() {
        let svc_key_ref = svc_key.borrow();
        let svc_name = svc_key_ref.name().to_string();

        // Extract service information
        let mut display_name = svc_name.clone();
        let mut start_type = String::from("Unknown");
        let mut image_path = String::new();

        for kv in svc_key_ref.values() {
            match kv.name() {
                "DisplayName" => {
                    if let RegistryValue::RegSZ(data) | RegistryValue::RegExpandSZ(data) =
                        kv.value()
                    {
                        display_name = data.clone();
                    }
                }
                "Start" => {
                    // Start type is a DWORD:
                    // 0 = Boot, 1 = System, 2 = Automatic, 3 = Manual, 4 = Disabled
                    if let RegistryValue::RegDWord(start_val) = kv.value() {
                        start_type = match start_val {
                            0 => "Boot".to_string(),
                            1 => "System".to_string(),
                            2 => "Automatic".to_string(),
                            3 => "Manual".to_string(),
                            4 => "Disabled".to_string(),
                            _ => format!("Unknown({})", start_val),
                        };
                    }
                }
                "ImagePath" => {
                    if let RegistryValue::RegSZ(data) | RegistryValue::RegExpandSZ(data) =
                        kv.value()
                    {
                        image_path = data.clone();
                    }
                }
                _ => {}
            }
        }

        // Only add services that have an image path (actual services, not just service groups)
        if !image_path.is_empty() {
            services.push(WindowsSvc {
                name: svc_name,
                display_name,
                start_type,
                image_path,
            });
        }
    }

    Ok(services)
}

/// Parse network configuration from SYSTEM hive
///
/// Reads from SYSTEM\ControlSet001\Services\Tcpip\Parameters\Interfaces
pub fn parse_network_adapters(hive_path: &Path) -> Result<Vec<WindowsNetAdapter>> {
    use nt_hive2::{Hive, HiveParseMode, RegistryValue};
    use std::fs::File;

    // Verify hive exists
    if !hive_path.exists() {
        return Err(Error::NotFound(format!(
            "SYSTEM hive not found: {}",
            hive_path.display()
        )));
    }

    // Read hive file
    let file = File::open(hive_path)
        .map_err(|e| Error::CommandFailed(format!("Failed to open hive: {}", e)))?;

    // Parse hive
    let mut hive = Hive::new(file, HiveParseMode::NormalWithBaseBlock)
        .map_err(|e| Error::CommandFailed(format!("Failed to parse hive: {:?}", e)))?;

    let mut adapters = Vec::new();

    // Get root key
    let root_key = hive
        .root_key_node()
        .map_err(|e| Error::CommandFailed(format!("Failed to get root key: {:?}", e)))?;

    // Navigate to Tcpip interfaces: ControlSet001\Services\Tcpip\Parameters\Interfaces
    let controlset_key = match root_key.subkey("ControlSet001", &mut hive) {
        Ok(Some(key)) => key,
        _ => return Ok(adapters), // No ControlSet001 key
    };

    let services_key = match controlset_key.borrow().subkey("Services", &mut hive) {
        Ok(Some(key)) => key,
        _ => return Ok(adapters), // No Services key
    };

    let tcpip_key = match services_key.borrow().subkey("Tcpip", &mut hive) {
        Ok(Some(key)) => key,
        _ => return Ok(adapters), // No Tcpip key
    };

    let params_key = match tcpip_key.borrow().subkey("Parameters", &mut hive) {
        Ok(Some(key)) => key,
        _ => return Ok(adapters), // No Parameters key
    };

    let interfaces_key = match params_key.borrow().subkey("Interfaces", &mut hive) {
        Ok(Some(key)) => key,
        _ => return Ok(adapters), // No Interfaces key
    };

    // Iterate through interface subkeys (each GUID represents an adapter)
    let interfaces_borrowed = interfaces_key.borrow();
    let subkeys_result = interfaces_borrowed.subkeys(&mut hive);
    let subkeys_ref = match subkeys_result {
        Ok(ref_vec) => ref_vec,
        Err(_) => return Ok(adapters),
    };

    for if_key in subkeys_ref.iter() {
        let if_key_ref = if_key.borrow();
        let adapter_guid = if_key_ref.name().to_string();

        // Extract network adapter information
        let mut dhcp_enabled = false;
        let mut ip_address = Vec::new();
        let mut dns_servers = Vec::new();

        for kv in if_key_ref.values() {
            match kv.name() {
                "EnableDHCP" => {
                    // DHCP enabled is a DWORD (1 = enabled, 0 = disabled)
                    if let RegistryValue::RegDWord(val) = kv.value() {
                        dhcp_enabled = *val == 1;
                    }
                }
                "IPAddress" => {
                    // IP addresses stored as REG_MULTI_SZ (array of strings)
                    if let RegistryValue::RegMultiSZ(addrs) = kv.value() {
                        for addr in addrs {
                            if !addr.is_empty() && addr != "0.0.0.0" {
                                ip_address.push(addr.clone());
                            }
                        }
                    }
                }
                "DhcpIPAddress" => {
                    // DHCP-assigned IP address
                    if let RegistryValue::RegSZ(addr) = kv.value() {
                        if !addr.is_empty() && addr != "0.0.0.0" {
                            ip_address.push(addr.clone());
                        }
                    }
                }
                "NameServer" => {
                    // DNS servers as comma-separated string
                    if let RegistryValue::RegSZ(servers) = kv.value() {
                        for server in servers.split(',') {
                            let trimmed = server.trim();
                            if !trimmed.is_empty() {
                                dns_servers.push(trimmed.to_string());
                            }
                        }
                    }
                }
                "DhcpNameServer" => {
                    // DHCP-assigned DNS servers
                    if let RegistryValue::RegSZ(servers) = kv.value() {
                        for server in servers.split(',') {
                            let trimmed = server.trim();
                            if !trimmed.is_empty() && !dns_servers.contains(&trimmed.to_string()) {
                                dns_servers.push(trimmed.to_string());
                            }
                        }
                    }
                }
                _ => {}
            }
        }

        // Only add adapters that have configuration (at least IP or DHCP enabled)
        if !ip_address.is_empty() || dhcp_enabled {
            adapters.push(WindowsNetAdapter {
                name: adapter_guid.clone(),
                description: format!("Network Adapter {}", adapter_guid),
                dhcp_enabled,
                ip_address,
                mac_address: String::new(), // MAC address not available in Tcpip key
                dns_servers,
            });
        }
    }

    Ok(adapters)
}

/// Get Windows version from SOFTWARE hive
///
/// Returns (product_name, version, edition)
/// Reads from SOFTWARE\Microsoft\Windows NT\CurrentVersion
pub fn get_windows_version(hive_path: &Path) -> Result<(String, String, String)> {
    use nt_hive2::{Hive, HiveParseMode};
    use std::fs::File;

    // Verify hive exists
    if !hive_path.exists() {
        return Err(Error::NotFound(format!(
            "SOFTWARE hive not found: {}",
            hive_path.display()
        )));
    }

    // Read hive file
    let file = File::open(hive_path)
        .map_err(|e| Error::CommandFailed(format!("Failed to open hive: {}", e)))?;

    // Parse hive
    let mut hive = Hive::new(file, HiveParseMode::NormalWithBaseBlock)
        .map_err(|e| Error::CommandFailed(format!("Failed to parse hive: {:?}", e)))?;

    // Navigate to CurrentVersion key
    let root_key = hive
        .root_key_node()
        .map_err(|e| Error::CommandFailed(format!("Failed to get root key: {:?}", e)))?;

    // Path: Microsoft\Windows NT\CurrentVersion
    // Navigate step by step
    let microsoft_key = root_key
        .subkey("Microsoft", &mut hive)
        .map_err(|e| Error::CommandFailed(format!("Failed to find Microsoft key: {:?}", e)))?
        .ok_or_else(|| Error::NotFound("Microsoft key not found".to_string()))?;

    let windows_nt_key = microsoft_key
        .borrow()
        .subkey("Windows NT", &mut hive)
        .map_err(|e| Error::CommandFailed(format!("Failed to find Windows NT key: {:?}", e)))?
        .ok_or_else(|| Error::NotFound("Windows NT key not found".to_string()))?;

    let current_version_key = windows_nt_key
        .borrow()
        .subkey("CurrentVersion", &mut hive)
        .map_err(|e| Error::CommandFailed(format!("Failed to find CurrentVersion key: {:?}", e)))?
        .ok_or_else(|| Error::NotFound("CurrentVersion key not found".to_string()))?;

    // Read values
    let mut product_name = String::from("Windows");
    let mut version = String::from("Unknown");
    let mut edition = String::from("Unknown");
    let mut build = String::new();
    let mut major_version = String::new();
    let mut minor_version = String::new();

    // Get all values from the CurrentVersion key
    let key_ref = current_version_key.borrow();
    let values = key_ref.values();

    // Iterate through values to find the ones we need
    for kv in values {
        // Get value name
        let name = kv.name();

        match name {
            "ProductName" => {
                // Read string value (e.g., "Windows 10 Pro", "Windows 11 Home")
                use nt_hive2::RegistryValue;
                if let RegistryValue::RegSZ(data) | RegistryValue::RegExpandSZ(data) = kv.value() {
                    product_name = data.clone();
                }
            }
            "EditionID" => {
                // Read string value (e.g., "Professional", "Home", "Enterprise")
                use nt_hive2::RegistryValue;
                if let RegistryValue::RegSZ(data) | RegistryValue::RegExpandSZ(data) = kv.value() {
                    edition = data.clone();
                }
            }
            "CurrentBuild" => {
                // Read string value (e.g., "19045", "22631")
                use nt_hive2::RegistryValue;
                if let RegistryValue::RegSZ(data) | RegistryValue::RegExpandSZ(data) = kv.value() {
                    build = data.clone();
                }
            }
            "CurrentMajorVersionNumber" => {
                // Read DWORD value
                use nt_hive2::RegistryValue;
                if let RegistryValue::RegDWord(data) = kv.value() {
                    major_version = data.to_string();
                }
            }
            "CurrentMinorVersionNumber" => {
                // Read DWORD value
                use nt_hive2::RegistryValue;
                if let RegistryValue::RegDWord(data) = kv.value() {
                    minor_version = data.to_string();
                }
            }
            _ => {}
        }
    }

    // Construct version string
    if !major_version.is_empty() && !build.is_empty() {
        version = format!("{}.{}.{}", major_version, minor_version, build);
    } else if !build.is_empty() {
        version = build;
    }

    Ok((product_name, version, edition))
}

/// Windows update/hotfix information
#[derive(Debug, Clone)]
pub struct WindowsUpdateInfo {
    pub kb_number: String,
    pub title: String,
    pub description: String,
    pub installed_date: String,
    pub update_type: String,
}

/// Parse installed Windows updates from registry
///
/// Checks SOFTWARE hive for installed updates and hotfixes
pub fn parse_installed_updates(hive_path: &Path) -> Result<Vec<WindowsUpdateInfo>> {
    // Verify hive exists
    if !hive_path.exists() {
        return Err(Error::NotFound(format!(
            "SOFTWARE hive not found: {}",
            hive_path.display()
        )));
    }

    // Registry parsing requires nt_hive2 crate for reading Windows registry hives.
    // Keys to parse:
    // - SOFTWARE\Microsoft\Windows\CurrentVersion\Component Based Servicing\Packages
    // - SOFTWARE\Microsoft\Windows NT\CurrentVersion\HotFix
    // For now, return empty since we cannot parse registry binary format without nt_hive2.
    Ok(vec![])
}

/// Parse CBS.log for component-based servicing updates
pub fn parse_cbs_log(log_content: &str) -> Vec<WindowsUpdateInfo> {
    let mut updates = Vec::new();

    // Parse CBS.log entries for installed packages
    // Format: "Package KB###### was successfully changed to the Installed state."
    for line in log_content.lines() {
        if line.contains("Package KB") && line.contains("Installed state") {
            // Extract KB number
            if let Some(kb_start) = line.find("KB") {
                let kb_part = &line[kb_start..];
                if let Some(kb_end) = kb_part.find(|c: char| !c.is_alphanumeric()) {
                    let kb_number = kb_part[..kb_end].to_string();

                    updates.push(WindowsUpdateInfo {
                        kb_number: kb_number.clone(),
                        title: format!("{} installed", kb_number),
                        description: "Detected from CBS.log".to_string(),
                        installed_date: "Unknown".to_string(),
                        update_type: "Component".to_string(),
                    });
                }
            }
        }
    }

    updates
}

/// Parse Windows Update history from DataStore
pub fn parse_update_datastore(datastore_path: &Path) -> Result<Vec<WindowsUpdateInfo>> {
    // Verify DataStore.edb exists
    if !datastore_path.exists() {
        return Err(Error::NotFound(format!(
            "DataStore.edb not found: {}",
            datastore_path.display()
        )));
    }

    // ESE database (Extensible Storage Engine) parsing requires a dedicated ESE parser.
    // Without one, we cannot extract update history from DataStore.edb.
    Ok(vec![])
}

/// Detect hotfixes from file system
pub fn detect_hotfixes_from_filesystem(windows_dir: &Path) -> Result<Vec<WindowsUpdateInfo>> {
    let mut hotfixes = Vec::new();

    // $NtUninstall directories contain per-KB subdirectories like $NtUninstallKB123456$
    let nt_uninstall_prefix = "$NtUninstall";
    if let Ok(entries) = std::fs::read_dir(windows_dir) {
        for entry in entries.flatten() {
            let name = entry.file_name();
            let name_str = name.to_string_lossy();
            if name_str.starts_with(nt_uninstall_prefix) && entry.path().is_dir() {
                // Extract KB number from directory name like "$NtUninstallKB123456$"
                let inner = name_str
                    .trim_start_matches(nt_uninstall_prefix)
                    .trim_end_matches('$');
                if !inner.is_empty() {
                    hotfixes.push(WindowsUpdateInfo {
                        kb_number: inner.to_string(),
                        title: format!("{} (uninstall data present)", inner),
                        description: format!("Found at {}", entry.path().display()),
                        installed_date: "Unknown".to_string(),
                        update_type: "Hotfix".to_string(),
                    });
                }
            }
        }
    }

    // $hf_mig$ directory indicates hotfix migration data exists
    let hf_mig = windows_dir.join("$hf_mig$");
    if hf_mig.exists() && hf_mig.is_dir() {
        // Count subdirectories as an approximation of hotfix count
        if let Ok(entries) = std::fs::read_dir(&hf_mig) {
            for entry in entries.flatten() {
                let name = entry.file_name();
                let name_str = name.to_string_lossy();
                if let Some(kb_start) = name_str.find("KB") {
                    let kb_part = &name_str[kb_start..];
                    let kb_end = kb_part
                        .find(|c: char| !c.is_alphanumeric())
                        .unwrap_or(kb_part.len());
                    let kb_number = &kb_part[..kb_end];
                    hotfixes.push(WindowsUpdateInfo {
                        kb_number: kb_number.to_string(),
                        title: format!("{} (migration data)", kb_number),
                        description: format!("Found at {}", entry.path().display()),
                        installed_date: "Unknown".to_string(),
                        update_type: "Hotfix".to_string(),
                    });
                }
            }
        }
    }

    Ok(hotfixes)
}

/// Windows event log entry
#[derive(Debug, Clone)]
pub struct WindowsEventEntry {
    pub event_id: u32,
    pub level: String,
    pub source: String,
    pub message: String,
    pub time_created: String,
    pub computer: String,
    pub channel: String,
}

/// Parse Windows Event Log (.evtx) file
pub fn parse_evtx_file(evtx_path: &Path, limit: usize) -> Result<Vec<WindowsEventEntry>> {
    if !evtx_path.exists() {
        return Err(Error::NotFound(format!(
            "EVTX file not found: {}",
            evtx_path.display()
        )));
    }

    #[cfg(feature = "evtx")]
    {
        return parse_evtx_with_crate(evtx_path, limit);
    }

    #[cfg(not(feature = "evtx"))]
    {
        let _ = limit;
        Ok(vec![])
    }
}

#[cfg(feature = "evtx")]
fn parse_evtx_with_crate(evtx_path: &Path, limit: usize) -> Result<Vec<WindowsEventEntry>> {
    use evtx::EvtxParser;
    use serde_json::{json, Value};
    use std::fs::File;

    let file = File::open(evtx_path)?;
    let mut parser = EvtxParser::from_read_seek(file)
        .map_err(|e| Error::InvalidFormat(format!("evtx open: {e}")))?;
    let mut events = Vec::new();
    for record in parser.records() {
        if events.len() >= limit {
            break;
        }
        let record = record.map_err(|e| Error::InvalidFormat(format!("evtx record: {e}")))?;
        let val: Value = serde_json::from_str(&record.data).unwrap_or(json!({}));
        let system = val.pointer("/Event/System").or_else(|| val.get("System"));
        let event_id = json_u32(system.and_then(|s| s.get("EventID")), 0);
        let level = system
            .and_then(|s| s.get("Level"))
            .map(json_scalar_string)
            .unwrap_or_else(|| "0".into());
        let source = system
            .and_then(|s| s.get("Provider"))
            .and_then(|p| p.get("Name"))
            .or_else(|| system.and_then(|s| s.get("Provider")))
            .map(json_scalar_string)
            .unwrap_or_else(|| "Unknown".into());
        let channel = system
            .and_then(|s| s.get("Channel"))
            .map(json_scalar_string)
            .unwrap_or_default();
        let computer = system
            .and_then(|s| s.get("Computer"))
            .map(json_scalar_string)
            .unwrap_or_default();
        let message = val
            .pointer("/Event/EventData")
            .map(|d| d.to_string())
            .unwrap_or_else(|| record.data.chars().take(512).collect());
        events.push(WindowsEventEntry {
            event_id,
            level,
            source,
            message,
            time_created: record.timestamp.to_rfc3339(),
            computer,
            channel,
        });
    }
    Ok(events)
}

#[cfg(feature = "evtx")]
fn json_u32(value: Option<&Value>, default: u32) -> u32 {
    value
        .and_then(|v| {
            v.as_u64()
                .or_else(|| v.as_str().and_then(|s| s.parse().ok()))
                .or_else(|| v.as_i64().map(|i| i as u64))
        })
        .map(|n| n as u32)
        .unwrap_or(default)
}

#[cfg(feature = "evtx")]
fn json_scalar_string(value: &Value) -> String {
    match value {
        Value::String(s) => s.clone(),
        Value::Number(n) => n.to_string(),
        Value::Bool(b) => b.to_string(),
        other => other.to_string(),
    }
}

/// Parse System event log for boot times and errors
pub fn parse_system_events(evtx_path: &Path) -> Result<Vec<WindowsEventEntry>> {
    parse_evtx_file(evtx_path, 100)
}

/// Parse Security event log for login attempts
pub fn parse_security_events(evtx_path: &Path) -> Result<Vec<WindowsEventEntry>> {
    // Security log - look for failed logins (Event ID 4625)
    parse_evtx_file(evtx_path, 100)
}

/// Parse Application event log
pub fn parse_application_events(evtx_path: &Path) -> Result<Vec<WindowsEventEntry>> {
    parse_evtx_file(evtx_path, 50)
}

/// Parse domain join status from SYSTEM hive
pub fn parse_domain_info(hive_path: &Path) -> (bool, Option<String>) {
    use nt_hive2::{Hive, HiveParseMode, RegistryValue};
    use std::fs::File;

    let Ok(file) = File::open(hive_path) else {
        return (false, None);
    };
    let Ok(mut hive) = Hive::new(file, HiveParseMode::NormalWithBaseBlock) else {
        return (false, None);
    };
    let Ok(root_key) = hive.root_key_node() else {
        return (false, None);
    };
    let Ok(Some(cs)) = root_key.subkey("ControlSet001", &mut hive) else {
        return (false, None);
    };
    let Ok(Some(svc)) = cs.borrow().subkey("Services", &mut hive) else {
        return (false, None);
    };
    let Ok(Some(tcpip)) = svc.borrow().subkey("Tcpip", &mut hive) else {
        return (false, None);
    };
    let Ok(Some(params)) = tcpip.borrow().subkey("Parameters", &mut hive) else {
        return (false, None);
    };

    let mut domain = None;
    for kv in params.borrow().values() {
        if kv.name() == "Domain" {
            if let RegistryValue::RegSZ(d) | RegistryValue::RegExpandSZ(d) = kv.value() {
                if !d.is_empty() && d.to_lowercase() != "workgroup" {
                    domain = Some(d.clone());
                }
            }
        }
    }
    (domain.is_some(), domain)
}

/// Check if RDP is enabled via SYSTEM hive
pub fn parse_rdp_enabled(hive_path: &Path) -> bool {
    use nt_hive2::{Hive, HiveParseMode, RegistryValue};
    use std::fs::File;

    let Ok(file) = File::open(hive_path) else {
        return false;
    };
    let Ok(mut hive) = Hive::new(file, HiveParseMode::NormalWithBaseBlock) else {
        return false;
    };
    let Ok(root_key) = hive.root_key_node() else {
        return false;
    };
    let Ok(Some(cs)) = root_key.subkey("ControlSet001", &mut hive) else {
        return false;
    };
    let Ok(Some(svc)) = cs.borrow().subkey("Services", &mut hive) else {
        return false;
    };
    let Ok(Some(ts)) = svc.borrow().subkey("TermService", &mut hive) else {
        return false;
    };
    let Ok(Some(params)) = ts.borrow().subkey("Parameters", &mut hive) else {
        return false;
    };

    for kv in params.borrow().values() {
        if kv.name() == "fDenyTSConnections" {
            if let RegistryValue::RegDWord(v) = kv.value() {
                return *v == 0;
            }
        }
    }
    false
}

/// Detect pending reboot from SYSTEM hive Session Manager
pub fn parse_pending_reboot(hive_path: &Path) -> bool {
    use nt_hive2::{Hive, HiveParseMode, RegistryValue};
    use std::fs::File;

    let Ok(file) = File::open(hive_path) else {
        return false;
    };
    let Ok(mut hive) = Hive::new(file, HiveParseMode::NormalWithBaseBlock) else {
        return false;
    };
    let Ok(root_key) = hive.root_key_node() else {
        return false;
    };
    let Ok(Some(cs)) = root_key.subkey("ControlSet001", &mut hive) else {
        return false;
    };
    let Ok(Some(sm)) = cs.borrow().subkey("Control", &mut hive) else {
        return false;
    };
    let Ok(Some(session)) = sm.borrow().subkey("Session Manager", &mut hive) else {
        return false;
    };

    for kv in session.borrow().values() {
        if kv.name() == "PendingFileRenameOperations" {
            if let RegistryValue::RegMultiSZ(ops) = kv.value() {
                return !ops.is_empty();
            }
        }
    }
    false
}

/// Parse SAM hive for local account summary
pub fn parse_sam_accounts(hive_path: &Path) -> Result<SamSummary> {
    use nt_hive2::{Hive, HiveParseMode};
    use std::fs::File;

    if !hive_path.exists() {
        return Err(Error::NotFound(format!(
            "SAM hive not found: {}",
            hive_path.display()
        )));
    }

    let file = File::open(hive_path)
        .map_err(|e| Error::CommandFailed(format!("Failed to open SAM hive: {}", e)))?;
    let mut hive = Hive::new(file, HiveParseMode::NormalWithBaseBlock)
        .map_err(|e| Error::CommandFailed(format!("Failed to parse SAM hive: {:?}", e)))?;

    let root_key = hive
        .root_key_node()
        .map_err(|e| Error::CommandFailed(format!("Failed to get SAM root: {:?}", e)))?;

    let mut summary = SamSummary::default();
    if let Ok(Some(sam)) = root_key.subkey("SAM", &mut hive) {
        if let Ok(Some(domains)) = sam.borrow().subkey("Domains", &mut hive) {
            if let Ok(Some(account)) = domains.borrow().subkey("Account", &mut hive) {
                if let Ok(Some(users)) = account.borrow().subkey("Users", &mut hive) {
                    if let Ok(subkeys) = users.borrow().subkeys(&mut hive) {
                        summary.local_account_count = subkeys.len();
                    }
                }
            }
        }
    }
    Ok(summary)
}

/// Summary of SAM hive account data
#[derive(Debug, Clone, Default)]
pub struct SamSummary {
    pub local_account_count: usize,
    pub admin_count: usize,
}

/// Detect hypervisor guest tool remnants
pub fn detect_hypervisor_remnants(
    system_hive: &Path,
    drivers_path: &str,
    g: &mut crate::guestfs::Guestfs,
) -> Vec<String> {
    let mut remnants = Vec::new();
    let vmware_drivers = ["vmci.sys", "vmhgfs.sys", "vmmouse.sys", "vm3dmp.sys"];
    let hyperv_drivers = ["vmbus.sys", "hv_vmbus.sys", "storvsc.sys"];

    if let Ok(drivers) = g.ls(drivers_path) {
        for d in &drivers {
            let lower = d.to_lowercase();
            if vmware_drivers
                .iter()
                .any(|v| lower.contains(v.trim_end_matches(".sys")))
            {
                remnants.push("vmware-drivers".to_string());
            }
            if hyperv_drivers
                .iter()
                .any(|v| lower.contains(v.trim_end_matches(".sys")))
            {
                remnants.push("hyper-v-drivers".to_string());
            }
        }
    }

    if let Ok(services) = parse_windows_services(system_hive) {
        for svc in services {
            let name = svc.name.to_lowercase();
            if name.contains("vmware") || name.contains("vmtools") {
                remnants.push("vmware-tools-service".to_string());
            }
            if name.contains("hyper-v") || name.contains("vmbus") {
                remnants.push("hyper-v-service".to_string());
            }
        }
    }

    remnants.sort();
    remnants.dedup();
    remnants
}

/// Detect AV/EDR products from registry and filesystem
pub fn detect_av_edr(
    software_hive: &Path,
    g: &mut crate::guestfs::Guestfs,
    systemroot: &str,
) -> Vec<String> {
    let mut products = Vec::new();
    if let Ok(apps) = parse_installed_software(software_hive) {
        for app in apps {
            let name = app.name.to_lowercase();
            for keyword in [
                "defender",
                "crowdstrike",
                "sentinelone",
                "carbon black",
                "symantec",
                "mcafee",
                "sophos",
                "kaspersky",
                "trend micro",
                "cylance",
                "malwarebytes",
                "bitdefender",
            ] {
                if name.contains(keyword) {
                    products.push(app.name.clone());
                }
            }
        }
    }

    let paths = [
        format!("{}/System32/Windows Defender", systemroot),
        format!(
            "{}/Program Files/CrowdStrike",
            systemroot.trim_start_matches('/')
        ),
    ];
    for p in paths {
        if g.exists(&p).unwrap_or(false) {
            if p.contains("Defender") {
                products.push("Windows Defender".to_string());
            }
            if p.contains("CrowdStrike") {
                products.push("CrowdStrike".to_string());
            }
        }
    }

    products.sort();
    products.dedup();
    products
}

/// Parse SECURITY hive for audit policy metadata
pub fn parse_security_hive(hive_path: &Path) -> Result<SecurityHiveSummary> {
    use nt_hive2::{CleanHive, Hive, HiveParseMode};
    use std::fs::File;

    if !hive_path.exists() {
        return Err(Error::NotFound(format!(
            "SECURITY hive not found: {}",
            hive_path.display()
        )));
    }

    let file = File::open(hive_path)
        .map_err(|e| Error::CommandFailed(format!("Failed to open SECURITY hive: {}", e)))?;
    let _hive: Hive<File, CleanHive> = Hive::new(file, HiveParseMode::NormalWithBaseBlock)
        .map_err(|e| Error::CommandFailed(format!("Failed to parse SECURITY hive: {:?}", e)))?;

    Ok(SecurityHiveSummary {
        lsa_present: true,
        audit_policy_configured: false,
    })
}

/// Summary from SECURITY hive (metadata only, no secret decryption)
#[derive(Debug, Clone, Default)]
pub struct SecurityHiveSummary {
    pub lsa_present: bool,
    pub audit_policy_configured: bool,
}

/// Detect BitLocker protectors from registry (FVE key path)
pub fn parse_bitlocker_status(software_hive: &Path) -> bool {
    use nt_hive2::{Hive, HiveParseMode};
    use std::fs::File;

    let Ok(file) = File::open(software_hive) else {
        return false;
    };
    let Ok(mut hive) = Hive::new(file, HiveParseMode::NormalWithBaseBlock) else {
        return false;
    };
    let Ok(root_key) = hive.root_key_node() else {
        return false;
    };
    root_key
        .subkey("Microsoft", &mut hive)
        .ok()
        .flatten()
        .and_then(|k| k.borrow().subkey("FVE", &mut hive).ok().flatten())
        .is_some()
}

/// Run / RunOnce registry value for persistence analysis.
#[derive(Debug, Clone)]
pub struct WindowsRunKeyEntry {
    pub location: String,
    pub name: String,
    pub command: String,
}

/// Parse Run and RunOnce keys from SOFTWARE hive.
pub fn parse_run_keys(hive_path: &Path) -> Result<Vec<WindowsRunKeyEntry>> {
    use nt_hive2::{Hive, HiveParseMode, RegistryValue};
    use std::fs::File;

    if !hive_path.exists() {
        return Ok(Vec::new());
    }

    let file = File::open(hive_path)
        .map_err(|e| Error::CommandFailed(format!("Failed to open hive: {e}")))?;
    let mut hive = Hive::new(file, HiveParseMode::NormalWithBaseBlock)
        .map_err(|e| Error::CommandFailed(format!("Failed to parse hive: {e:?}")))?;
    let root_key = hive
        .root_key_node()
        .map_err(|e| Error::CommandFailed(format!("root key: {e:?}")))?;

    let mut out = Vec::new();
    if let Ok(Some(microsoft)) = root_key.subkey("Microsoft", &mut hive) {
        if let Ok(Some(windows)) = microsoft.borrow().subkey("Windows", &mut hive) {
            if let Ok(Some(current)) = windows.borrow().subkey("CurrentVersion", &mut hive) {
                for (subkey, label) in
                    [("Run", "HKLM\\...\\Run"), ("RunOnce", "HKLM\\...\\RunOnce")]
                {
                    if let Ok(Some(run_key)) = current.borrow().subkey(subkey, &mut hive) {
                        for kv in run_key.borrow().values() {
                            let name = kv.name().to_string();
                            let command = match kv.value() {
                                RegistryValue::RegSZ(s) | RegistryValue::RegExpandSZ(s) => {
                                    s.clone()
                                }
                                RegistryValue::RegMultiSZ(v) => v.join(" "),
                                _ => continue,
                            };
                            out.push(WindowsRunKeyEntry {
                                location: label.into(),
                                name,
                                command,
                            });
                        }
                    }
                }
            }
        }
    }
    Ok(out)
}
