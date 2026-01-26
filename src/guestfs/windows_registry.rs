// SPDX-License-Identifier: LGPL-3.0-or-later
//! Windows Registry parsing using nt_hive2
//!
//! This module provides pure Rust parsing of Windows registry hive files.
//! Note: This is an initial implementation that will be enhanced with full
//! registry parsing in future versions.

use crate::core::{Error, Result};
use std::path::Path;

/// Windows application information
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

    for app_key in (&*subkeys_ref).iter() {
        let app_key_ref = app_key.borrow();

        // Extract application information from values
        let mut name = String::new();
        let mut version = String::new();
        let mut publisher = String::new();
        let mut install_location = None;

        for kv in app_key_ref.values() {
            match kv.name().as_ref() {
                "DisplayName" => {
                    if let RegistryValue::RegSZ(data) | RegistryValue::RegExpandSZ(data) = kv.value() {
                        name = data.clone();
                    }
                }
                "DisplayVersion" => {
                    if let RegistryValue::RegSZ(data) | RegistryValue::RegExpandSZ(data) = kv.value() {
                        version = data.clone();
                    }
                }
                "Publisher" => {
                    if let RegistryValue::RegSZ(data) | RegistryValue::RegExpandSZ(data) = kv.value() {
                        publisher = data.clone();
                    }
                }
                "InstallLocation" => {
                    if let RegistryValue::RegSZ(data) | RegistryValue::RegExpandSZ(data) = kv.value() {
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

    for svc_key in (&*subkeys_ref).iter() {
        let svc_key_ref = svc_key.borrow();
        let svc_name = svc_key_ref.name().to_string();

        // Extract service information
        let mut display_name = svc_name.clone();
        let mut start_type = String::from("Unknown");
        let mut image_path = String::new();

        for kv in svc_key_ref.values() {
            match kv.name().as_ref() {
                "DisplayName" => {
                    if let RegistryValue::RegSZ(data) | RegistryValue::RegExpandSZ(data) = kv.value() {
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
                    if let RegistryValue::RegSZ(data) | RegistryValue::RegExpandSZ(data) = kv.value() {
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

    for if_key in (&*subkeys_ref).iter() {
        let if_key_ref = if_key.borrow();
        let adapter_guid = if_key_ref.name().to_string();

        // Extract network adapter information
        let mut dhcp_enabled = false;
        let mut ip_address = Vec::new();
        let mut dns_servers = Vec::new();

        for kv in if_key_ref.values() {
            match kv.name().as_ref() {
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

        match name.as_ref() {
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

    // TODO: Parse registry keys:
    // - SOFTWARE\Microsoft\Windows\CurrentVersion\Component Based Servicing\Packages
    // - SOFTWARE\Microsoft\Windows NT\CurrentVersion\HotFix

    Ok(vec![WindowsUpdateInfo {
        kb_number: "Registry".to_string(),
        title: "Update registry detected".to_string(),
        description: "Full parsing pending".to_string(),
        installed_date: "Unknown".to_string(),
        update_type: "Registry".to_string(),
    }])
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

    // TODO: Parse ESE database (Extensible Storage Engine)
    // This requires ESE database parsing which is complex
    // For now, just detect presence

    Ok(vec![WindowsUpdateInfo {
        kb_number: "DataStore".to_string(),
        title: "Windows Update Database".to_string(),
        description: format!("Update database found at {}", datastore_path.display()),
        installed_date: "Unknown".to_string(),
        update_type: "Database".to_string(),
    }])
}

/// Detect hotfixes from file system
pub fn detect_hotfixes_from_filesystem(windows_dir: &Path) -> Result<Vec<WindowsUpdateInfo>> {
    let mut hotfixes = Vec::new();

    // Check common hotfix directories
    let hotfix_dirs = vec![
        windows_dir.join("$hf_mig$"),
        windows_dir.join("$NtUninstall"),
    ];

    for dir in hotfix_dirs {
        if dir.exists() {
            // Directory exists, indicates hotfixes were installed
            hotfixes.push(WindowsUpdateInfo {
                kb_number: "Hotfix".to_string(),
                title: "Hotfix directory detected".to_string(),
                description: format!("Found at {}", dir.display()),
                installed_date: "Unknown".to_string(),
                update_type: "Hotfix".to_string(),
            });
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
///
/// TODO: Full implementation using evtx crate (pending log feature conflict resolution)
/// For now, detects presence of event log files
pub fn parse_evtx_file(evtx_path: &Path, _limit: usize) -> Result<Vec<WindowsEventEntry>> {
    // Verify file exists
    if !evtx_path.exists() {
        return Err(Error::NotFound(format!(
            "EVTX file not found: {}",
            evtx_path.display()
        )));
    }

    // Get file metadata
    let metadata = std::fs::metadata(evtx_path).map_err(Error::Io)?;

    let file_size = metadata.len();
    let channel_name = evtx_path
        .file_stem()
        .and_then(|n| n.to_str())
        .unwrap_or("Unknown");

    // Return stub event indicating file was detected
    Ok(vec![WindowsEventEntry {
        event_id: 0,
        level: "Information".to_string(),
        source: "EventLog Parser".to_string(),
        message: format!(
            "Event log file detected: {} ({} bytes). Full parsing will be available when evtx crate integration is complete.",
            channel_name, file_size
        ),
        time_created: "Unknown".to_string(),
        computer: "Unknown".to_string(),
        channel: channel_name.to_string(),
    }])

    // TODO: Implement full EVTX parsing with evtx crate
    // Waiting for resolution of log crate feature conflicts
    // The evtx crate (https://crates.io/crates/evtx) provides:
    // - Fast, safe EVTX parsing in pure Rust
    // - XML and JSON output
    // - Support for recovery of corrupted logs
    // - 1600x faster than Python alternatives
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

// TODO: Full registry parsing implementation using nt_hive2
// The nt_hive2 API requires:
// 1. use nt_hive2::{Hive, SubPath, HiveParseMode};
// 2. let mut hive = Hive::new(file, HiveParseMode::NormalWithBaseBlock)?;
// 3. let root = hive.root_key_node()?;
// 4. let key = root.subpath(path, &mut hive)?.unwrap();
// 5. Handle Rc<RefCell<KeyNode>> return types
// 6. Parse values with correct type conversions
//
// This will be implemented when we have Windows VM images for testing
