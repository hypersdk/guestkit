// SPDX-License-Identifier: Apache-2.0
//! Local Unix socket client for in-guest zyvor-guestctl status.

use anyhow::{Context, Result};
use guestkit_agent_protocol::{read_frame, write_frame};
use std::os::unix::net::UnixStream;

pub const DEFAULT_SOCKET_PATH: &str = "/var/run/zyvor/guest-agent.sock";

pub fn print_local_status(socket_path: Option<&str>) -> Result<()> {
    let path = socket_path.unwrap_or(DEFAULT_SOCKET_PATH);
    let mut stream =
        UnixStream::connect(path).with_context(|| format!("connect to agent at {path}"))?;
    stream.set_read_timeout(Some(std::time::Duration::from_secs(30)))?;

    let health = call_method(&mut stream, "guestkit.getGuestHealth", serde_json::json!({}))?;
    let info = call_method(&mut stream, "guestkit.getGuestInfo", serde_json::json!({}))?;

    println!("Zyvor GuestAgent status");
    println!("========================");
    if let Some(hostname) = info.get("hostname").and_then(|v| v.as_str()) {
        println!("Hostname:     {hostname}");
    }
    if let Some(os) = info.get("os") {
        println!(
            "OS:           {} {}",
            os.get("id").and_then(|v| v.as_str()).unwrap_or(""),
            os.get("version").and_then(|v| v.as_str()).unwrap_or("")
        );
    }
    if let Some(level) = health.get("guest_health").and_then(|v| v.as_str()) {
        println!("Health:       {level}");
    }
    if let Some(score) = health.get("score").and_then(|v| v.as_u64()) {
        println!("Score:        {score}");
    }
    if let Some(state) = health.get("systemd_state").and_then(|v| v.as_str()) {
        println!("systemd:      {state}");
    }
    if let Some(failed) = health.get("failed_units").and_then(|v| v.as_u64()) {
        println!("Failed units: {failed}");
    }
  if let Some(services) = health.get("critical_services").and_then(|v| v.as_array()) {
        for svc in services.iter().take(5) {
            let name = svc.get("name").and_then(|v| v.as_str()).unwrap_or("");
            let state = svc.get("state").and_then(|v| v.as_str()).unwrap_or("");
            let failure = svc
                .get("last_failure")
                .and_then(|v| v.as_str())
                .unwrap_or("");
            println!("  - {name}: {state} {failure}");
        }
    }
    Ok(())
}

fn call_method(
    stream: &mut UnixStream,
    method: &str,
    params: serde_json::Value,
) -> Result<serde_json::Value> {
    let req = serde_json::json!({
        "jsonrpc": "2.0",
        "method": method,
        "params": params,
        "id": 1,
    });
    let mut writer = stream.try_clone()?;
    write_frame(&mut writer, &serde_json::to_vec(&req)?)?;
    let mut reader = stream.try_clone()?;
    let frame = read_frame(&mut reader).map_err(|e| anyhow::anyhow!("{e}"))?;
    let resp: serde_json::Value = serde_json::from_slice(&frame)?;
    if let Some(result) = resp.get("result") {
        Ok(result.clone())
    } else {
        Err(anyhow::anyhow!(
            "agent error: {}",
            resp.get("error")
                .and_then(|e| e.get("message"))
                .and_then(|m| m.as_str())
                .unwrap_or("unknown")
        ))
    }
}
