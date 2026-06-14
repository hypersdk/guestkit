// SPDX-License-Identifier: Apache-2.0
//! Per-VM guest agent JSON-RPC via QEMU guest-agent guest-exec.

use base64::{engine::general_purpose::STANDARD, Engine};
use kube::Client;
use serde_json::{json, Value};

use crate::error::{ApiError, ApiResult};
use crate::kubevirt_qga::{qga_available, qga_exec_shell};

/// Invoke GuestKit JSON-RPC inside the guest via `zyvor-guest-agent rpc`.
pub async fn vm_guestkit_rpc(
    client: &Client,
    namespace: &str,
    name: &str,
    method: &str,
    params: Value,
) -> ApiResult<Value> {
    if !qga_available(client, namespace, name).await {
        return Err(ApiError::bad_request(
            "QEMU guest agent not available for per-VM pull",
        ));
    }

    let req = json!({
        "jsonrpc": "2.0",
        "method": method,
        "params": params,
        "id": 1,
    });
    let req_bytes = serde_json::to_vec(&req)
        .map_err(|e| ApiError::internal(format!("serialize rpc: {e}")))?;
    let b64 = STANDARD.encode(&req_bytes);
    let script = format!(
        "B64='{b64}'; echo \"$B64\" | base64 -d | /usr/bin/zyvor-guest-agent rpc 2>/dev/null || \
         echo \"$B64\" | base64 -d | /usr/bin/zyvor-guest-agent --rpc 2>/dev/null"
    );

    let exec = qga_exec_shell(client, namespace, name, &script, 120).await?;
    if exec.exit_code != 0 {
        return Err(ApiError::bad_request(format!(
            "guest rpc failed (exit {}): {}",
            exec.exit_code,
            exec.stderr.trim()
        )));
    }

    let stdout = exec.stdout.trim();
    if stdout.is_empty() {
        return Err(ApiError::internal("empty guest rpc response"));
    }

    let envelope: Value = serde_json::from_str(stdout)
        .map_err(|e| ApiError::internal(format!("parse guest rpc response: {e} — {stdout}")))?;

    if envelope.get("error").is_some() {
        return Ok(envelope);
    }

    Ok(envelope)
}
