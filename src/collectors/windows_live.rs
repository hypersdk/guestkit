// SPDX-License-Identifier: Apache-2.0
//! Windows WMI/Event Log collectors (live guest).

use guestkit_agent_protocol::{
    GuestHealth, GuestHealthComponents, HealthLevel, NetworkHealth, SecurityHealthSummary,
};
use serde::{Deserialize, Serialize};

#[cfg(target_os = "windows")]
use std::process::Command;

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct WindowsLiveEvidence {
    pub os_caption: String,
    pub os_version: String,
    pub last_boot: String,
    pub services_count: usize,
    pub running_services: usize,
    pub stopped_services: usize,
    pub event_log_channels: Vec<String>,
    pub critical_events_24h: usize,
    pub error_events_24h: usize,
    pub rdp_sessions: usize,
    pub rdp_enabled: bool,
    pub pending_updates: bool,
    pub pending_reboot: bool,
    pub firewall_enabled: bool,
    pub av_products: Vec<String>,
    pub ip_addresses: Vec<String>,
    pub default_gateway: Option<String>,
}

pub fn collect_windows_live() -> Option<WindowsLiveEvidence> {
    #[cfg(target_os = "windows")]
    {
        Some(collect_windows_live_inner())
    }
    #[cfg(not(target_os = "windows"))]
    {
        None
    }
}

#[cfg(target_os = "windows")]
fn collect_windows_live_inner() -> WindowsLiveEvidence {
    let os_caption = wmi_string("Win32_OperatingSystem", "Caption").unwrap_or_default();
    let os_version = wmi_string("Win32_OperatingSystem", "Version").unwrap_or_default();
    let last_boot = wmi_string("Win32_OperatingSystem", "LastBootUpTime").unwrap_or_default();
    let services_count = powershell_usize("(Get-Service | Measure-Object).Count").unwrap_or(0);
    let running_services =
        powershell_usize("(Get-Service | Where-Object Status -eq Running | Measure-Object).Count")
            .unwrap_or(0);
    let stopped_services = services_count.saturating_sub(running_services);
    let critical_events_24h = powershell_usize(
        "(Get-WinEvent -FilterHashtable @{LogName='System'; Level=1; StartTime=(Get-Date).AddHours(-24)} -ErrorAction SilentlyContinue | Measure-Object).Count",
    )
    .unwrap_or(0);
    let error_events_24h = powershell_usize(
        "(Get-WinEvent -FilterHashtable @{LogName='System'; Level=2; StartTime=(Get-Date).AddHours(-24)} -ErrorAction SilentlyContinue | Measure-Object).Count",
    )
    .unwrap_or(0);
    let pending_reboot = powershell_bool(
        "Test-Path 'HKLM:\\SOFTWARE\\Microsoft\\Windows\\CurrentVersion\\WindowsUpdate\\Auto Update\\RebootRequired'",
    )
    .unwrap_or(false);
    let pending_updates = powershell_bool(
        "(Get-CimInstance -Namespace root/Microsoft/Windows/WindowsUpdate -ClassName MSFT_WUSettings -ErrorAction SilentlyContinue) -ne $null",
    )
    .unwrap_or(false);
    let rdp_sessions = powershell_usize("(quser 2>$null | Measure-Object -Line).Lines").unwrap_or(0);
    let rdp_enabled = powershell_bool(
        "(Get-ItemProperty 'HKLM:\\System\\CurrentControlSet\\Control\\Terminal Server').fDenyTSConnections -eq 0",
    )
    .unwrap_or(false);
    let firewall_enabled = powershell_bool(
        "(Get-NetFirewallProfile -ErrorAction SilentlyContinue | Where-Object Enabled -eq $true | Measure-Object).Count -gt 0",
    )
    .unwrap_or(false);
    let av_products = powershell_lines(
        "Get-CimInstance -Namespace root/SecurityCenter2 -ClassName AntiVirusProduct -ErrorAction SilentlyContinue | Select-Object -ExpandProperty displayName",
    );
    let ip_addresses = powershell_lines(
        "Get-NetIPAddress -AddressFamily IPv4 -ErrorAction SilentlyContinue | Where-Object { $_.IPAddress -notlike '127.*' } | Select-Object -ExpandProperty IPAddress",
    );
    let default_gateway = powershell_string(
        "(Get-NetRoute -DestinationPrefix '0.0.0.0/0' -ErrorAction SilentlyContinue | Select-Object -First 1 -ExpandProperty NextHop)",
    );

    WindowsLiveEvidence {
        os_caption,
        os_version,
        last_boot,
        services_count,
        running_services,
        stopped_services,
        event_log_channels: vec!["System".into(), "Application".into()],
        critical_events_24h,
        error_events_24h,
        rdp_sessions,
        rdp_enabled,
        pending_updates,
        pending_reboot,
        firewall_enabled,
        av_products,
        ip_addresses,
        default_gateway,
    }
}

#[cfg(target_os = "windows")]
fn wmi_string(class: &str, property: &str) -> Option<String> {
    powershell_string(&format!(
        "(Get-CimInstance -ClassName {class} -ErrorAction SilentlyContinue | Select-Object -First 1 -ExpandProperty {property})",
        class = class,
        property = property
    ))
}

#[cfg(target_os = "windows")]
fn powershell_usize(script: &str) -> Option<usize> {
    powershell_string(script)?.parse().ok()
}

#[cfg(target_os = "windows")]
fn powershell_bool(script: &str) -> Option<bool> {
    let raw = powershell_string(script)?;
    match raw.to_lowercase().as_str() {
        "true" | "1" => Some(true),
        "false" | "0" => Some(false),
        _ => raw.parse::<usize>().map(|n| n != 0),
    }
}

#[cfg(target_os = "windows")]
fn powershell_string(script: &str) -> Option<String> {
    let output = Command::new("powershell")
        .args([
            "-NoProfile",
            "-NonInteractive",
            "-ExecutionPolicy",
            "Bypass",
            "-Command",
            script,
        ])
        .output()
        .ok()?;
    if !output.status.success() {
        return None;
    }
    let text = String::from_utf8_lossy(&output.stdout).trim().to_string();
    if text.is_empty() {
        None
    } else {
        Some(text)
    }
}

#[cfg(target_os = "windows")]
fn powershell_lines(script: &str) -> Vec<String> {
    powershell_string(script)
        .map(|s| {
            s.lines()
                .map(|l| l.trim().to_string())
                .filter(|l| !l.is_empty())
                .collect()
        })
        .unwrap_or_default()
}

pub fn build_windows_guest_health(hostname: &str) -> GuestHealth {
    let evidence = collect_windows_live();
    let (level, score, reasons, components, journal_hints) = if let Some(ev) = &evidence {
        let mut reasons = Vec::new();
        let mut score = 90u8;
        if ev.pending_reboot {
            score = score.saturating_sub(15);
            reasons.push("Pending Windows reboot".into());
        }
        if ev.pending_updates {
            score = score.saturating_sub(5);
            reasons.push("Windows Update activity detected".into());
        }
        if ev.critical_events_24h > 0 {
            score = score.saturating_sub(25);
            reasons.push(format!(
                "{} critical System events in last 24h",
                ev.critical_events_24h
            ));
        }
        if ev.error_events_24h > 5 {
            score = score.saturating_sub(10);
            reasons.push(format!(
                "{} error System events in last 24h",
                ev.error_events_24h
            ));
        }
        if !ev.firewall_enabled {
            score = score.saturating_sub(10);
            reasons.push("Windows Firewall disabled on a profile".into());
        }
        let level = if score >= 80 {
            HealthLevel::Healthy
        } else if score >= 50 {
            HealthLevel::Degraded
        } else {
            HealthLevel::Unhealthy
        };
        let security_level = if ev.firewall_enabled && ev.av_products.is_empty() {
            HealthLevel::Degraded
        } else if ev.firewall_enabled {
            HealthLevel::Healthy
        } else {
            HealthLevel::Unhealthy
        };
        let components = GuestHealthComponents {
            boot: HealthLevel::Healthy,
            systemd: if ev.stopped_services > 5 {
                HealthLevel::Degraded
            } else {
                HealthLevel::Healthy
            },
            network: if ev.ip_addresses.is_empty() {
                HealthLevel::Degraded
            } else {
                HealthLevel::Healthy
            },
            dns: HealthLevel::Unknown,
            storage: HealthLevel::Unknown,
            security: security_level,
            agent: HealthLevel::Healthy,
        };
        let journal_hints = if ev.critical_events_24h > 0 {
            vec![format!(
                "system:critical_events_24h={}",
                ev.critical_events_24h
            )]
        } else {
            vec![]
        };
        (level, score, reasons, components, journal_hints)
    } else {
        (
            HealthLevel::Unknown,
            50,
            vec![],
            GuestHealthComponents::default(),
            vec![],
        )
    };

    let network = evidence
        .as_ref()
        .map(|ev| NetworkHealth {
            default_route: ev.default_gateway.is_some() || !ev.ip_addresses.is_empty(),
            dns_working: true,
            dns_error: None,
            interfaces_up: ev.ip_addresses.len(),
            cluster_dns_reachable: false,
        })
        .unwrap_or_default();

    let security = evidence
        .as_ref()
        .map(|ev| SecurityHealthSummary {
            pending_security_updates: ev.pending_updates,
            firewall_enabled: ev.firewall_enabled,
            selinux: ev
                .av_products
                .first()
                .cloned()
                .unwrap_or_else(|| "none".into()),
        })
        .unwrap_or_default();

    GuestHealth {
        vm_hostname: hostname.to_string(),
        guest_health: level,
        boot_state: evidence
            .as_ref()
            .map(|e| e.last_boot.clone())
            .unwrap_or_else(|| "running".into()),
        systemd_state: format!(
            "windows-services:{} running",
            evidence.as_ref().map(|e| e.running_services).unwrap_or(0)
        ),
        failed_units: evidence
            .as_ref()
            .map(|e| e.critical_events_24h + e.error_events_24h)
            .unwrap_or(0),
        critical_services: vec![],
        network,
        storage: Default::default(),
        security,
        recommendations: vec![],
        collected_at: chrono::Utc::now().to_rfc3339(),
        agent_version: crate::VERSION.to_string(),
        score,
        reasons,
        components,
        journal_hints,
    }
}
