// SPDX-License-Identifier: Apache-2.0
//! Host-side proxy: libvirt unix socket ↔ optional HTTP bridge.

use crate::agent::handler::RequestHandler;
use anyhow::{Context, Result};
use guestkit_agent_protocol::{read_frame, write_frame, JsonRpcResponse};
use std::net::SocketAddr;
use std::os::unix::net::UnixStream;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::{TcpListener, TcpStream};

struct AgentClient {
    stream: UnixStream,
}

impl AgentClient {
    fn connect(socket_path: &str) -> Result<Self> {
        let stream = UnixStream::connect(socket_path)
            .with_context(|| format!("connect to agent socket {socket_path}"))?;
        stream.set_read_timeout(Some(std::time::Duration::from_secs(120)))?;
        stream.set_write_timeout(Some(std::time::Duration::from_secs(30)))?;
        Ok(Self { stream })
    }

    fn call_raw(&mut self, payload: Vec<u8>) -> Result<Vec<u8>> {
        write_frame(&mut self.stream, &payload)?;
        read_frame(&mut self.stream).map_err(|e| anyhow::anyhow!("{e}"))
    }

    fn call(&mut self, method: &str, params: serde_json::Value) -> Result<serde_json::Value> {
        let req = serde_json::json!({
            "jsonrpc": "2.0",
            "method": method,
            "params": params,
            "id": 1
        });
        let frame = self.call_raw(serde_json::to_vec(&req)?)?;
        serde_json::from_slice(&frame).context("parse agent response")
    }
}

pub async fn run_proxy(socket_path: &str, listen: Option<&str>) -> Result<()> {
    if let Some(addr) = listen {
        let addr: SocketAddr = addr.parse().context("parse --listen address")?;
        log::info!("GuestKit agent-proxy HTTP listening on {addr} (socket: {socket_path})");
        let listener = TcpListener::bind(addr).await?;
        let socket_path = socket_path.to_string();
        loop {
            let (stream, peer) = listener.accept().await?;
            log::debug!("HTTP connection from {peer}");
            let path = socket_path.clone();
            tokio::spawn(async move {
                if let Err(e) = handle_http(stream, &path).await {
                    log::error!("HTTP handler error: {e}");
                }
            });
        }
    } else {
        log::info!("GuestKit agent-proxy relay on {socket_path} (stdin/stdout frames)");
        let path = socket_path.to_string();
        tokio::task::spawn_blocking(move || -> Result<()> {
            let mut agent = UnixStream::connect(&path)?;
            loop {
                let frame_in = read_frame(&mut std::io::stdin())?;
                write_frame(&mut agent, &frame_in)?;
                let frame_out = read_frame(&mut agent)?;
                write_frame(&mut std::io::stdout(), &frame_out)?;
            }
        })
        .await??;
        Ok(())
    }
}

async fn handle_http(mut stream: TcpStream, socket_path: &str) -> Result<()> {
    let mut buf = vec![0u8; 8192];
    let n = stream.read(&mut buf).await?;
    let request = String::from_utf8_lossy(&buf[..n]);
    let mut lines = request.lines();
    let request_line = lines.next().unwrap_or("");
    let parts: Vec<&str> = request_line.split_whitespace().collect();
    if parts.len() < 2 {
        return write_http_error(&mut stream, 400, "bad request").await;
    }
    let method = parts[0];
    let path = parts[1];

    let body_start = request.find("\r\n\r\n").map(|i| i + 4).unwrap_or(n);
    let body = &buf[body_start..n];

    let path_only = path.split('?').next().unwrap_or(path);
    let query = path.split('?').nth(1).unwrap_or("");
    let query_target = query
        .split('&')
        .find_map(|pair| {
            let mut kv = pair.splitn(2, '=');
            let k = kv.next()?;
            let v = kv.next().unwrap_or("");
            if k == "target" {
                Some(v)
            } else {
                None
            }
        })
        .unwrap_or("kvm");

    let rpc_method = match (method, path_only) {
        ("GET", "/evidence") | ("GET", "/api/evidence") => "guestkit.getEvidence",
        ("GET", "/doctor") | ("GET", "/api/doctor") => "guestkit.doctor",
        ("GET", "/ping") | ("GET", "/api/ping") => "guestkit.ping",
        ("GET", "/capabilities") | ("GET", "/api/capabilities") => "guestkit.getCapabilities",
        ("POST", "/fix-plan") | ("POST", "/api/fix-plan") => "guestkit.runFixPlan",
        ("POST", "/rpc") | ("POST", "/api/rpc") => "__passthrough__",
        _ => {
            return write_http_error(&mut stream, 404, "not found").await;
        }
    };

    let (call_method, params) = if rpc_method == "__passthrough__" {
        let body_json: serde_json::Value =
            serde_json::from_slice(body).unwrap_or_else(|_| serde_json::json!({}));
        let m = body_json
            .get("method")
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string();
        if m.is_empty() {
            return write_http_error(&mut stream, 400, "method required").await;
        }
        let p = body_json
            .get("params")
            .cloned()
            .unwrap_or_else(|| serde_json::json!({}));
        (m, p)
    } else if rpc_method == "guestkit.runFixPlan" {
        (
            rpc_method.to_string(),
            serde_json::from_slice::<serde_json::Value>(body)
                .unwrap_or_else(|_| serde_json::json!({})),
        )
    } else if rpc_method == "guestkit.doctor" {
        (
            rpc_method.to_string(),
            serde_json::json!({ "target": query_target }),
        )
    } else {
        (rpc_method.to_string(), serde_json::json!({}))
    };

    let response = tokio::task::spawn_blocking({
        let socket_path = socket_path.to_string();
        let call_method = call_method.clone();
        move || -> Result<serde_json::Value> {
            let mut client = AgentClient::connect(&socket_path)?;
            client.call(&call_method, params)
        }
    })
    .await??;

    let status = if response.get("error").is_some() {
        500
    } else {
        200
    };
    let body = serde_json::to_string(&response)?;
    write_http_json(&mut stream, status, &body).await
}

async fn write_http_json(stream: &mut TcpStream, status: u16, body: &str) -> Result<()> {
    let status_text = match status {
        200 => "OK",
        400 => "Bad Request",
        404 => "Not Found",
        500 => "Internal Server Error",
        _ => "Unknown",
    };
    let response = format!(
        "HTTP/1.1 {status} {status_text}\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{body}",
        body.len()
    );
    stream.write_all(response.as_bytes()).await?;
    stream.flush().await?;
    Ok(())
}

async fn write_http_error(stream: &mut TcpStream, status: u16, message: &str) -> Result<()> {
    let body = serde_json::json!({ "error": message }).to_string();
    write_http_json(stream, status, &body).await
}

/// Local in-process handler for unit tests without a unix socket.
pub fn handle_local_request(bytes: &[u8]) -> JsonRpcResponse {
    let handler = RequestHandler::new();
    handler.handle(bytes)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn local_ping() {
        let resp = handle_local_request(br#"{"jsonrpc":"2.0","method":"guestkit.ping","id":1}"#);
        assert!(resp.result.is_some());
    }
}
