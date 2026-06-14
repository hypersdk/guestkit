// SPDX-License-Identifier: Apache-2.0
//! Windows live evidence (WMI + Event Log when agent runs on Windows).

use crate::evidence::snapshot::WindowsEvidence;

#[cfg(target_os = "windows")]
pub fn collect_windows_live() -> Option<WindowsEvidence> {
    let live = crate::collectors::windows_live::collect_windows_live();
    let mut evidence = WindowsEvidence::default();
    if let Some(ev) = live {
        evidence.product_name = ev.os_caption;
        evidence.version = ev.os_version;
        evidence.systemroot = std::env::var("SystemRoot").unwrap_or_else(|_| "C:\\Windows".into());
        evidence.rdp_enabled = ev.rdp_enabled;
        evidence.pending_reboot = ev.pending_reboot;
        evidence.services_count = ev.services_count;
        evidence.event_logs.log_count = ev.event_log_channels.len();
        evidence.event_logs.forensic = Some(crate::evidence::snapshot::WindowsForensicProfile {
            service_failures: ev.critical_events_24h as u32,
            failed_logons: ev.error_events_24h as u32,
            ..Default::default()
        });
        evidence.av_edr = ev.av_products.clone();
    } else {
        evidence.systemroot = std::env::var("SystemRoot").unwrap_or_else(|_| "C:\\Windows".into());
    }
    Some(evidence)
}

#[cfg(not(target_os = "windows"))]
pub fn collect_windows_live() -> Option<WindowsEvidence> {
    None
}
