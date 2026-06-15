// SPDX-License-Identifier: Apache-2.0
//! Resolve guest agent pull path: per-VM virtio/QGA first, then global HTTP proxy.

use serde_json::Value;

use crate::error::ApiError;
use crate::routes::guest_agent::pull_guest_rpc;
use crate::state::AppState;

pub async fn pull_for_vm(
    state: &AppState,
    namespace: &str,
    name: &str,
    method: &str,
    params: Value,
) -> Result<Value, String> {
    if let Some(client) = state.kube.as_ref() {
        return crate::guest_agent_vm::vm_guestkit_rpc(client, namespace, name, method, params)
            .await
            .map_err(|e| e.message);
    }

    if let Some(proxy) = state.config.agent_proxy_url.as_ref() {
        return pull_guest_rpc(proxy, method, params).await;
    }

    Err(format!(
        "no guest pull path for {namespace}/{name} (kube unavailable and AGENT_PROXY_URL unset)"
    ))
}

pub async fn pull_for_vm_api(
    state: &AppState,
    namespace: &str,
    name: &str,
    method: &str,
    params: Value,
) -> Result<Value, ApiError> {
    pull_for_vm(state, namespace, name, method, params)
        .await
        .map_err(ApiError::bad_request)
}

pub fn rpc_result(resp: Value) -> Value {
    resp.get("result").cloned().unwrap_or(resp)
}
