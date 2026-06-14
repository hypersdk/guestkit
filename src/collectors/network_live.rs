// SPDX-License-Identifier: Apache-2.0
//! Live network interface and route collection (ip -j / procfs).

use crate::evidence::snapshot::{NetworkEvidence, NetworkInterfaceLive};
use std::fs;
use std::path::Path;
use std::process::Command;

pub fn enrich_network_evidence(network: &mut NetworkEvidence) {
    network.network_stack = detect_network_stack();
    collect_ip_json(network);
    if network.default_gateway.is_none() {
        network.default_gateway = read_default_gateway_proc();
    }
}

fn detect_network_stack() -> String {
    if fs::metadata("/run/NetworkManager").is_ok() || Command::new("which")
        .arg("nmcli")
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false)
    {
        return "NetworkManager".into();
    }
    if Path::new("/etc/netplan").is_dir() {
        return "netplan".into();
    }
    if Command::new("systemctl")
        .args(["is-active", "--quiet", "systemd-networkd"])
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false)
    {
        return "systemd-networkd".into();
    }
    "unknown".into()
}

fn collect_ip_json(network: &mut NetworkEvidence) {
    if let Ok(out) = Command::new("ip").args(["-j", "addr"]).output() {
        if let Ok(json) = serde_json::from_slice::<Vec<serde_json::Value>>(&out.stdout) {
            for iface in json {
                let name = iface["ifname"].as_str().unwrap_or("").to_string();
                if name.is_empty() || name == "lo" {
                    continue;
                }
                let state = iface["operstate"].as_str().unwrap_or("").to_string();
                let mac = iface
                    .get("address")
                    .and_then(|v| v.as_str())
                    .unwrap_or("")
                    .to_string();
                let mut addresses = Vec::new();
                if let Some(addrinfo) = iface.get("addr_info").and_then(|v| v.as_array()) {
                    for addr in addrinfo {
                        if let Some(local) = addr.get("local").and_then(|v| v.as_str()) {
                            let family = addr.get("family").and_then(|v| v.as_str()).unwrap_or("");
                            let prefix = addr.get("prefixlen").and_then(|v| v.as_u64());
                            if family == "inet" || family == "inet6" {
                                let suffix = prefix.map(|p| format!("/{p}")).unwrap_or_default();
                                addresses.push(format!("{local}{suffix}"));
                            }
                        }
                    }
                }
                if !network.interfaces.contains(&name) {
                    network.interfaces.push(name.clone());
                }
                network.live_interfaces.push(NetworkInterfaceLive {
                    name,
                    state,
                    mac,
                    addresses,
                    carrier: iface
                        .get("carrier")
                        .and_then(|v| v.as_bool())
                        .unwrap_or(true),
                });
            }
        }
    }

    if let Ok(out) = Command::new("ip").args(["-j", "route"]).output() {
        if let Ok(json) = serde_json::from_slice::<Vec<serde_json::Value>>(&out.stdout) {
            for route in json {
                if route.get("dst").and_then(|v| v.as_str()) == Some("default") {
                    network.default_gateway = route
                        .get("gateway")
                        .and_then(|v| v.as_str())
                        .map(String::from);
                    break;
                }
            }
        }
    }

    if let Ok(out) = Command::new("ip").args(["-j", "link"]).output() {
        if let Ok(json) = serde_json::from_slice::<Vec<serde_json::Value>>(&out.stdout) {
            for link in json {
                let name = link.get("ifname").and_then(|v| v.as_str()).unwrap_or("");
                if name.is_empty() || name == "lo" {
                    continue;
                }
                let stats = link.get("stats64").or_else(|| link.get("stats"));
                if let Some(stats) = stats {
                    let rx = stats.get("rx_bytes").and_then(|v| v.as_u64()).unwrap_or(0);
                    let tx = stats.get("tx_bytes").and_then(|v| v.as_u64()).unwrap_or(0);
                    for live in &mut network.live_interfaces {
                        if live.name == name {
                            live.addresses.push(format!("rx_bytes={rx}"));
                            live.addresses.push(format!("tx_bytes={tx}"));
                            break;
                        }
                    }
                }
            }
        }
    }
}

fn read_default_gateway_proc() -> Option<String> {
    let content = fs::read_to_string("/proc/net/route").ok()?;
    for line in content.lines().skip(1) {
        let parts: Vec<&str> = line.split_whitespace().collect();
        if parts.len() >= 3 && parts[1] == "00000000" {
            let hex = parts[2];
            if hex.len() == 8 {
                let a = u8::from_str_radix(&hex[6..8], 16).unwrap_or(0);
                let b = u8::from_str_radix(&hex[4..6], 16).unwrap_or(0);
                let c = u8::from_str_radix(&hex[2..4], 16).unwrap_or(0);
                let d = u8::from_str_radix(&hex[0..2], 16).unwrap_or(0);
                return Some(format!("{a}.{b}.{c}.{d}"));
            }
        }
    }
    None
}
