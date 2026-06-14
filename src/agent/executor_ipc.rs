// SPDX-License-Identifier: Apache-2.0
//! Privileged executor IPC over Unix socket.

use anyhow::{Context, Result};
use guestkit_agent_protocol::{read_frame, write_frame};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::fs;
use std::io::{Read, Write};
use std::os::unix::net::{UnixListener, UnixStream};
use std::path::Path;
use std::thread;

pub const EXEC_SOCKET_PATH: &str = "/var/run/zyvor/guest-agent-exec.sock";

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecRequest {
    pub action: String,
    #[serde(default)]
    pub params: Value,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecResponse {
    pub ok: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub result: Option<Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
}

pub fn spawn_executor_server() -> Result<()> {
    let path = std::env::var("ZYVOR_EXEC_SOCKET").unwrap_or_else(|_| EXEC_SOCKET_PATH.to_string());
    if Path::new(&path).exists() {
        fs::remove_file(&path).ok();
    }
    let parent = Path::new(&path).parent().unwrap_or(Path::new("/var/run/zyvor"));
    fs::create_dir_all(parent).ok();

    let listener = UnixListener::bind(&path).with_context(|| format!("bind executor socket {path}"))?;
    log::info!("Zyvor guest agent executor listening on {path}");

    thread::spawn(move || {
        for conn in listener.incoming().flatten() {
            thread::spawn(move || {
                if let Err(e) = serve_connection(conn) {
                    log::debug!("executor client error: {e}");
                }
            });
        }
    });

    Ok(())
}

fn serve_connection(stream: UnixStream) -> Result<()> {
    let mut reader = stream.try_clone()?;
    let mut writer = stream;
    let frame = read_frame(&mut reader).map_err(|e| anyhow::anyhow!("{e}"))?;
    let req: ExecRequest = serde_json::from_slice(&frame).context("parse exec request")?;
    let executor = crate::agent::executor::Executor::new();
    let resp = dispatch(&executor, &req);
    let bytes = serde_json::to_vec(&resp)?;
    write_frame(&mut writer, &bytes).map_err(|e| anyhow::anyhow!("{e}"))?;
    Ok(())
}

fn dispatch(executor: &crate::agent::executor::Executor, req: &ExecRequest) -> ExecResponse {
    match req.action.as_str() {
        "restart_unit" => {
            let unit = req
                .params
                .get("unit")
                .and_then(|v| v.as_str())
                .unwrap_or("");
            match executor.restart_unit(unit) {
                Ok(msg) => ExecResponse {
                    ok: true,
                    result: Some(Value::String(msg)),
                    error: None,
                },
                Err(e) => ExecResponse {
                    ok: false,
                    result: None,
                    error: Some(e.to_string()),
                },
            }
        }
        "execute_remediation_plan" => {
            let plan_id = req
                .params
                .get("plan_id")
                .and_then(|v| v.as_str())
                .unwrap_or("local");
            let actions: Vec<crate::agent::executor::RemediationActionSpec> = req
                .params
                .get("actions")
                .and_then(|v| serde_json::from_value(v.clone()).ok())
                .unwrap_or_default();
            let result = executor.execute_remediation_plan(plan_id, &actions);
            ExecResponse {
                ok: result.success,
                result: Some(serde_json::to_value(result).unwrap_or(Value::Null)),
                error: None,
            }
        }
        "collect_support_bundle" => match executor.collect_support_bundle() {
            Ok(bytes) => {
                use base64::{engine::general_purpose::STANDARD, Engine};
                ExecResponse {
                    ok: true,
                    result: Some(serde_json::json!({
                        "format": "tar.zst",
                        "encoding": "base64",
                        "data": STANDARD.encode(bytes),
                    })),
                    error: None,
                }
            }
            Err(e) => ExecResponse {
                ok: false,
                result: None,
                error: Some(e.to_string()),
            },
        },
        other => ExecResponse {
            ok: false,
            result: None,
            error: Some(format!("unsupported executor action: {other}")),
        },
    }
}

pub fn call_executor(action: &str, params: Value) -> Result<Value> {
    let path = std::env::var("ZYVOR_EXEC_SOCKET").unwrap_or_else(|_| EXEC_SOCKET_PATH.to_string());
    if !Path::new(&path).exists() {
        return Err(anyhow::anyhow!("executor socket not available at {path}"));
    }
    let mut stream = UnixStream::connect(&path).with_context(|| format!("connect {path}"))?;
    let req = ExecRequest {
        action: action.to_string(),
        params,
    };
    let bytes = serde_json::to_vec(&req)?;
    write_frame(&mut stream, &bytes)?;
    let mut reader = stream.try_clone()?;
    let frame = read_frame(&mut reader).map_err(|e| anyhow::anyhow!("{e}"))?;
    let resp: ExecResponse = serde_json::from_slice(&frame).context("parse exec response")?;
    if resp.ok {
        Ok(resp.result.unwrap_or(Value::Null))
    } else {
        Err(anyhow::anyhow!(
            "{}",
            resp.error.unwrap_or_else(|| "executor failed".into())
        ))
    }
}

pub fn executor_available() -> bool {
    let path = std::env::var("ZYVOR_EXEC_SOCKET").unwrap_or_else(|_| EXEC_SOCKET_PATH.to_string());
    Path::new(&path).exists()
}
