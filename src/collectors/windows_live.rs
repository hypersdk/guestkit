// SPDX-License-Identifier: Apache-2.0
//! Windows WMI/Event Log collectors (Phase 5 scaffold).

use guestkit_agent_protocol::{GuestHealth, HealthLevel};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct WindowsLiveEvidence {
    pub services_count: usize,
    pub event_log_channels: Vec<String>,
    pub rdp_sessions: usize,
    pub pending_updates: bool,
}

pub fn collect_windows_live() -> Option<WindowsLiveEvidence> {
    #[cfg(target_os = "windows")]
    {
        Some(WindowsLiveEvidence {
            services_count: 0,
            event_log_channels: vec!["System".into(), "Application".into()],
            rdp_sessions: 0,
            pending_updates: false,
        })
    }
    #[cfg(not(target_os = "windows"))]
    {
        None
    }
}

pub fn build_windows_guest_health(hostname: &str) -> GuestHealth {
    GuestHealth {
        vm_hostname: hostname.to_string(),
        guest_health: HealthLevel::Unknown,
        boot_state: "running".into(),
        systemd_state: "windows".into(),
        failed_units: 0,
        critical_services: vec![],
        network: Default::default(),
        storage: Default::default(),
        security: Default::default(),
        recommendations: vec![],
        collected_at: chrono::Utc::now().to_rfc3339(),
        agent_version: crate::VERSION.to_string(),
        score: 50,
        reasons: vec![],
        components: Default::default(),
        journal_hints: vec![],
    }
}
