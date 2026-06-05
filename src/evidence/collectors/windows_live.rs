// SPDX-License-Identifier: LGPL-3.0-or-later
//! Windows live evidence (when agent runs on Windows).

use crate::evidence::snapshot::WindowsEvidence;

#[cfg(target_os = "windows")]
pub fn collect_windows_live() -> Option<WindowsEvidence> {
    use std::process::Command;
    let mut evidence = WindowsEvidence::default();

    if let Ok(out) = Command::new("cmd")
        .args(["/C", "ver"])
        .output()
    {
        evidence.version = String::from_utf8_lossy(&out.stdout).trim().to_string();
    }
    if let Ok(out) = Command::new("powershell")
        .args([
            "-NoProfile",
            "-Command",
            "(Get-CimInstance Win32_OperatingSystem).Caption",
        ])
        .output()
    {
        evidence.product_name = String::from_utf8_lossy(&out.stdout).trim().to_string();
    }
    evidence.systemroot = std::env::var("SystemRoot").unwrap_or_else(|_| "C:\\Windows".into());
    evidence.rdp_enabled = windows_rdp_enabled();
    evidence.pending_reboot = pending_reboot();
    Some(evidence)
}

#[cfg(not(target_os = "windows"))]
pub fn collect_windows_live() -> Option<WindowsEvidence> {
    None
}

#[cfg(target_os = "windows")]
fn windows_rdp_enabled() -> bool {
    use std::process::Command;
    Command::new("powershell")
        .args([
            "-NoProfile",
            "-Command",
            "(Get-ItemProperty 'HKLM:\\System\\CurrentControlSet\\Control\\Terminal Server').fDenyTSConnections -eq 0",
        ])
        .output()
        .map(|o| String::from_utf8_lossy(&o.stdout).trim() == "True")
        .unwrap_or(false)
}

#[cfg(target_os = "windows")]
fn pending_reboot() -> bool {
    use std::process::Command;
    Command::new("powershell")
        .args([
            "-NoProfile",
            "-Command",
            "Test-Path 'HKLM:\\SOFTWARE\\Microsoft\\Windows\\CurrentVersion\\WindowsUpdate\\Auto Update\\RebootRequired'",
        ])
        .output()
        .map(|o| String::from_utf8_lossy(&o.stdout).trim() == "True")
        .unwrap_or(false)
}
