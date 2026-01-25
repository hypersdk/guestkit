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
/// TODO: Full implementation using nt_hive2 SubPath API
/// This stub returns basic detection that hive file exists
pub fn parse_installed_software(hive_path: &Path) -> Result<Vec<WindowsApp>> {
    // Verify hive exists
    if !hive_path.exists() {
        return Err(Error::NotFound(format!(
            "SOFTWARE hive not found: {}",
            hive_path.display()
        )));
    }

    // TODO: Implement full registry parsing with nt_hive2
    // For now, return indication that hive was found
    Ok(vec![WindowsApp {
        name: "Registry Hive Detected".to_string(),
        version: "Parsing not yet implemented".to_string(),
        publisher: "guestctl".to_string(),
        install_location: None,
    }])
}

/// Parse Windows services from SYSTEM hive
///
/// TODO: Full implementation using nt_hive2 SubPath API
pub fn parse_windows_services(hive_path: &Path) -> Result<Vec<WindowsSvc>> {
    // Verify hive exists
    if !hive_path.exists() {
        return Err(Error::NotFound(format!(
            "SYSTEM hive not found: {}",
            hive_path.display()
        )));
    }

    // TODO: Implement full registry parsing
    Ok(vec![WindowsSvc {
        name: "RegistryDetected".to_string(),
        display_name: "SYSTEM Hive Detected".to_string(),
        start_type: "Automatic".to_string(),
        image_path: hive_path.display().to_string(),
    }])
}

/// Parse network configuration from SYSTEM hive
///
/// TODO: Full implementation using nt_hive2 SubPath API
pub fn parse_network_adapters(hive_path: &Path) -> Result<Vec<WindowsNetAdapter>> {
    // Verify hive exists
    if !hive_path.exists() {
        return Err(Error::NotFound(format!(
            "SYSTEM hive not found: {}",
            hive_path.display()
        )));
    }

    // TODO: Implement full network adapter parsing
    Ok(vec![WindowsNetAdapter {
        name: "DetectedAdapter".to_string(),
        description: "Network configuration in SYSTEM hive".to_string(),
        dhcp_enabled: true,
        ip_address: Vec::new(),
        mac_address: String::new(),
        dns_servers: Vec::new(),
    }])
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
