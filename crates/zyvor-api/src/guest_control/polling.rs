// SPDX-License-Identifier: Apache-2.0
//! Host-mediated polling for AirgapLive VMs without push telemetry.

use guestkit_agent_protocol::capabilities::{
    METHOD_GET_CAPABILITIES, METHOD_GET_GUEST_HEALTH,
};
use redis::AsyncCommands;
use serde_json::{json, Value};
use std::time::Duration;

use crate::error::ApiResult;
use crate::routes::kubevirt::{list_dynamic_all, vm_resource};
use crate::state::AppState;

use super::capabilities::ControlState;
use super::transport::{probe_guest_context, pull_method};

const POLL_KEY_PREFIX: &str = "guest-agent:vm-poll";
const POLL_TTL_SECS: u64 = 600;

fn poll_key(namespace: &str, name: &str) -> String {
    format!("{POLL_KEY_PREFIX}:{namespace}:{name}")
}

pub async fn store_poll_result(
    redis: &mut redis::aio::ConnectionManager,
    namespace: &str,
    name: &str,
    payload: &Value,
) -> Result<(), crate::error::ApiError> {
    let raw = serde_json::to_string(payload).map_err(|e| crate::error::ApiError::internal(e.to_string()))?;
    redis
        .set_ex::<_, _, ()>(poll_key(namespace, name), &raw, POLL_TTL_SECS)
        .await
        .map_err(|e| crate::error::ApiError::internal(e.to_string()))?;
    Ok(())
}

pub async fn reconcile_airgap_polls(state: &AppState) -> ApiResult<Value> {
    let client = match state.kube.as_ref() {
        Some(c) => c,
        None => {
            return Ok(json!({
                "scanned": 0,
                "polled": 0,
                "skipped": 0,
                "errors": ["kubernetes client unavailable"],
            }));
        }
    };

    let vms = list_dynamic_all(client, &vm_resource())
        .await
        .unwrap_or_default();
    let mut scanned = 0usize;
    let mut polled = 0usize;
    let mut skipped = 0usize;
    let mut errors = Vec::new();

    for vm in vms {
        let namespace = vm
            .pointer("/metadata/namespace")
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string();
        let name = vm
            .pointer("/metadata/name")
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string();
        if namespace.is_empty() || name.is_empty() {
            continue;
        }
        scanned += 1;
        let ctx = probe_guest_context(state, &namespace, &name).await;
        if ctx.control_state != ControlState::AirgapLive || ctx.push_registered {
            skipped += 1;
            continue;
        }

        let mut poll_data = json!({
            "namespace": namespace,
            "name": name,
            "controlState": ctx.control_state.as_str(),
            "transport": ctx.active_transport.as_str(),
            "polledAt": chrono::Utc::now().to_rfc3339(),
        });

        match pull_method(state, &namespace, &name, "guestkit.ping", json!({})).await {
            Ok(r) => {
                poll_data["ping"] = r.value;
                poll_data["pingTransport"] = r.transport.as_str().into();
            }
            Err(e) => errors.push(format!("{namespace}/{name} ping: {e}")),
        }

        if let Ok(r) = pull_method(
            state,
            &namespace,
            &name,
            METHOD_GET_GUEST_HEALTH,
            json!({}),
        )
        .await
        {
            poll_data["guestHealth"] = r.value;
        }

        if let Ok(r) = pull_method(
            state,
            &namespace,
            &name,
            METHOD_GET_CAPABILITIES,
            json!({}),
        )
        .await
        {
            poll_data["capabilities"] = r.value;
        }

        if let Err(e) = store_poll_result(&mut state.redis.clone(), &namespace, &name, &poll_data).await
        {
            errors.push(format!("{namespace}/{name} redis: {}", e.message));
        } else {
            polled += 1;
        }
    }

    Ok(json!({
        "scanned": scanned,
        "polled": polled,
        "skipped": skipped,
        "errors": errors,
    }))
}

/// Background worker: poll airgap VMs on an interval (default 30s).
pub fn spawn_airgap_poll_worker(state: AppState) {
    let interval_secs = std::env::var("GUEST_AIRGAP_POLL_INTERVAL_SECS")
        .ok()
        .and_then(|v| v.parse().ok())
        .unwrap_or(30);
    if std::env::var("GUEST_AIRGAP_POLL_ENABLED")
        .map(|v| v == "0" || v.eq_ignore_ascii_case("false"))
        .unwrap_or(false)
    {
        tracing::info!("guest airgap poll worker disabled");
        return;
    }
    tokio::spawn(async move {
        let mut ticker = tokio::time::interval(Duration::from_secs(interval_secs));
        loop {
            ticker.tick().await;
            match reconcile_airgap_polls(&state).await {
                Ok(summary) => tracing::debug!("airgap poll reconcile: {summary}"),
                Err(e) => tracing::warn!("airgap poll reconcile failed: {}", e.message),
            }
        }
    });
    tracing::info!("guest airgap poll worker started (interval={interval_secs}s)");
}
