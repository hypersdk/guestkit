// SPDX-License-Identifier: Apache-2.0

use guestkit_agent_protocol::{
    AgentCapabilities, JsonRpcRequest, JsonRpcResponse, RpcErrorCode, RpcMethod, PROTOCOL_VERSION,
};
use serde_json::{json, Value};
use sysinfo::{Disks, Networks, System};

const VERSION: &str = env!("CARGO_PKG_VERSION");

pub struct RequestHandler {
    capabilities: AgentCapabilities,
}

impl RequestHandler {
    pub fn new() -> Self {
        Self {
            capabilities: AgentCapabilities::standard(VERSION),
        }
    }

    pub fn handle_frame(&self, bytes: &[u8]) -> Vec<u8> {
        if is_qga_request(bytes) {
            return handle_qga_ping(bytes);
        }
        serde_json::to_vec(&self.handle(bytes)).unwrap_or_default()
    }

    pub fn handle(&self, bytes: &[u8]) -> JsonRpcResponse {
        let req = match JsonRpcRequest::parse(bytes) {
            Ok(r) => r,
            Err(e) => return JsonRpcResponse::from_agent_error(None, e),
        };
        if let Err(e) = req.validate() {
            return JsonRpcResponse::from_agent_error(req.id, e);
        }
        match req.method() {
            RpcMethod::Ping => JsonRpcResponse::success(req.id, json!({ "pong": true })),
            RpcMethod::GetVersion => JsonRpcResponse::success(
                req.id,
                json!({ "version": VERSION, "protocol": PROTOCOL_VERSION, "product": "zyvor-guest-agent" }),
            ),
            RpcMethod::GetCapabilities => JsonRpcResponse::success(
                req.id,
                serde_json::to_value(&self.capabilities).unwrap_or(json!({})),
            ),
            RpcMethod::GetEvidence | RpcMethod::GetStatus | RpcMethod::Doctor => {
                JsonRpcResponse::success(req.id, self.live_summary())
            }
            RpcMethod::GetMetrics => JsonRpcResponse::success(req.id, self.metrics()),
            RpcMethod::Unknown(name) => JsonRpcResponse::error(
                req.id,
                RpcErrorCode::MethodNotFound,
                format!("unknown method: {name}"),
            ),
            _ => JsonRpcResponse::error(
                req.id,
                RpcErrorCode::MethodNotFound,
                "method not implemented on this platform build",
            ),
        }
    }

    fn live_summary(&self) -> Value {
        let mut sys = System::new_all();
        sys.refresh_all();
        let hostname = System::host_name().unwrap_or_else(|| "unknown".into());
        let os = System::name().unwrap_or_else(|| "unknown".into());
        let version = System::os_version().unwrap_or_default();
        json!({
            "schema_version": "live-v1",
            "collected_at": chrono_now(),
            "os": {
                "hostname": hostname,
                "distribution": os,
                "version": version,
                "architecture": System::cpu_arch(),
            },
            "vm_tools": {
                "product": "Zeus VM Tools",
                "agent_version": VERSION,
                "connected": true,
            }
        })
    }

    fn metrics(&self) -> Value {
        let mut sys = System::new_all();
        sys.refresh_cpu_usage();
        sys.refresh_memory();
        json!({
            "cpu_usage_percent": sys.global_cpu_usage(),
            "memory_total_bytes": sys.total_memory(),
            "memory_used_bytes": sys.used_memory(),
            "disks": Disks::new_with_refreshed_list()
                .iter()
                .map(|d| json!({ "name": d.name().to_string_lossy(), "total_bytes": d.total_space() }))
                .collect::<Vec<_>>(),
            "networks": Networks::new_with_refreshed_list()
                .iter()
                .map(|(name, data)| json!({ "name": name, "received": data.received(), "transmitted": data.transmitted() }))
                .collect::<Vec<_>>(),
        })
    }
}

fn chrono_now() -> String {
    use std::time::{SystemTime, UNIX_EPOCH};
    let secs = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0);
    format!("{secs}")
}

fn is_qga_request(bytes: &[u8]) -> bool {
    serde_json::from_slice::<Value>(bytes)
        .ok()
        .and_then(|v| v.get("execute").and_then(|e| e.as_str()).map(String::from))
        .is_some()
}

fn handle_qga_ping(bytes: &[u8]) -> Vec<u8> {
    let _ = bytes;
    br#"{"return":{}}"#.to_vec()
}
