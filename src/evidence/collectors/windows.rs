// SPDX-License-Identifier: Apache-2.0
//! Windows registry and filesystem evidence enrichment.

use crate::evidence::snapshot::{
    WindowsAppEntry, WindowsEventLogSummary, WindowsPersistenceEvidence, WindowsServiceEntry,
    WindowsServiceType, WindowsStartType,
};
use crate::Guestfs;

/// Enrich Windows service/app/persistence details using guestfs-mounted hives.
pub fn collect_windows_details(g: &mut Guestfs, root: &str) -> WindowsDetails {
    let mut details = WindowsDetails::default();

    if let Ok(services) = g.inspect_windows_services(root) {
        details.services = services
            .into_iter()
            .map(|svc| {
                let auto_start = svc.start_type == "Automatic" || svc.start_type == "Boot";
                WindowsServiceEntry {
                    name: svc.name,
                    display_name: Some(svc.display_name),
                    image_path: None,
                    start_type: parse_start_type(&svc.start_type),
                    service_type: WindowsServiceType::Win32,
                    object_name: None,
                    description: None,
                    kernel_driver: false,
                    auto_start,
                }
            })
            .collect();
    }

    if let Ok(apps) = g.inspect_windows_software(root) {
        details.installed_apps = apps
            .into_iter()
            .take(50)
            .map(|app| WindowsAppEntry {
                name: app.name,
                version: app.version,
                publisher: app.publisher,
                install_location: None,
            })
            .collect();
    }

    let systemroot = g
        .inspect_get_windows_systemroot(root)
        .unwrap_or_else(|_| "/Windows".to_string());
    details.event_logs = summarize_event_logs(g, &systemroot);

    details
}

#[derive(Debug, Default)]
pub struct WindowsDetails {
    pub services: Vec<WindowsServiceEntry>,
    pub installed_apps: Vec<WindowsAppEntry>,
    pub persistence: WindowsPersistenceEvidence,
    pub event_logs: WindowsEventLogSummary,
}

fn parse_start_type(raw: &str) -> WindowsStartType {
    match raw {
        "Boot" => WindowsStartType::Boot,
        "System" => WindowsStartType::System,
        "Automatic" => WindowsStartType::Automatic,
        "Manual" => WindowsStartType::Manual,
        "Disabled" => WindowsStartType::Disabled,
        _ => WindowsStartType::Unknown,
    }
}

fn summarize_event_logs(g: &mut Guestfs, systemroot: &str) -> WindowsEventLogSummary {
    let log_dir = format!("{systemroot}/System32/winevt/Logs");
    let mut summary = WindowsEventLogSummary::default();

    if !g.exists(&log_dir).unwrap_or(false) {
        return summary;
    }

    if let Ok(entries) = g.ls(&log_dir) {
        for entry in entries {
            if !entry.ends_with(".evtx") {
                continue;
            }
            summary.log_count += 1;
            let path = format!("{log_dir}/{entry}");
            if let Ok(stat) = g.stat(&path) {
                summary.total_bytes = summary.total_bytes.saturating_add(stat.size.max(0) as u64);
            }
        }
    }

    summary
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn maps_start_types() {
        assert!(matches!(
            parse_start_type("Automatic"),
            WindowsStartType::Automatic
        ));
        assert!(matches!(
            parse_start_type("Disabled"),
            WindowsStartType::Disabled
        ));
    }
}
