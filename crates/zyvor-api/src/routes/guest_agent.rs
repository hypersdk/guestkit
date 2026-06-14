// SPDX-License-Identifier: Apache-2.0
//! Guest agent push registration, heartbeat, and report storage.

use axum::extract::{Path, State};
use axum::Json;
use redis::AsyncCommands;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::time::Duration;

use crate::error::{ApiError, ApiResult};
use crate::models::ApiResponse;
use crate::state::AppState;

#[derive(Debug, Deserialize)]
pub struct RegisterGuestAgentRequest {
    pub hostname: String,
    pub agent_version: String,
    #[serde(default)]
    pub bootstrap_token: Option<String>,
    #[serde(default)]
    pub namespace: Option<String>,
    #[serde(default)]
    pub vm_name: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct RegisterGuestAgentResponse {
    pub agent_id: String,
}

#[derive(Debug, Deserialize)]
pub struct GuestReportPayload {
    #[serde(default)]
    pub guest_health: Value,
    #[serde(default)]
    pub metrics: Value,
    #[serde(default)]
    pub recent_events: Value,
}

fn report_key(agent_id: &str) -> String {
    format!("guest-agent:report:{agent_id}")
}

fn vm_report_key(namespace: &str, name: &str) -> String {
    format!("guest-agent:vm:{namespace}:{name}")
}

fn heartbeat_key(agent_id: &str) -> String {
    format!("guest-agent:heartbeat:{agent_id}")
}

pub async fn register_guest_agent(
    State(state): State<AppState>,
    Json(body): Json<RegisterGuestAgentRequest>,
) -> ApiResult<Json<ApiResponse<RegisterGuestAgentResponse>>> {
    let agent_id = if body.hostname.is_empty() {
        uuid::Uuid::new_v4().to_string()
    } else {
        format!(
            "{}-{}",
            body.hostname.replace('.', "-"),
            uuid::Uuid::new_v4()
                .to_string()
                .split('-')
                .next()
                .unwrap_or("id")
        )
    };

    let mut redis = state.redis.clone();
    let meta = serde_json::json!({
        "hostname": body.hostname,
        "agent_version": body.agent_version,
        "namespace": body.namespace,
        "vm_name": body.vm_name,
        "registered_at": chrono::Utc::now().to_rfc3339(),
    });
    redis
        .set::<_, _, ()>(
            format!("guest-agent:meta:{agent_id}"),
            serde_json::to_string(&meta).unwrap_or_default(),
        )
        .await
        .map_err(|e| ApiError::internal(e.to_string()))?;

    if let (Some(ns), Some(vm)) = (&body.namespace, &body.vm_name) {
        redis
            .set::<_, _, ()>(vm_report_key(ns, vm), &agent_id)
            .await
            .map_err(|e| ApiError::internal(e.to_string()))?;
    }

    Ok(Json(ApiResponse::ok(RegisterGuestAgentResponse {
        agent_id,
    })))
}

fn events_key(agent_id: &str) -> String {
    format!("guest-agent:events:{agent_id}")
}

pub async fn guest_agent_heartbeat(
    State(state): State<AppState>,
    Path(agent_id): Path<String>,
    Json(body): Json<Value>,
) -> ApiResult<Json<ApiResponse<Value>>> {
    let mut redis = state.redis.clone();
    let payload = serde_json::json!({
        "agent_id": agent_id,
        "body": body,
        "at": chrono::Utc::now().to_rfc3339(),
    });
    if let Some(events) = body.get("recent_events") {
        redis
            .set_ex::<_, _, ()>(
                events_key(&agent_id),
                serde_json::to_string(events).unwrap_or_default(),
                3600,
            )
            .await
            .map_err(|e| ApiError::internal(e.to_string()))?;
    }
    redis
        .set_ex::<_, _, ()>(
            heartbeat_key(&agent_id),
            serde_json::to_string(&payload).unwrap_or_default(),
            300,
        )
        .await
        .map_err(|e| ApiError::internal(e.to_string()))?;
    Ok(Json(ApiResponse::ok(json!({ "accepted": true }))))
}

pub async fn guest_agent_report(
    State(state): State<AppState>,
    Path(agent_id): Path<String>,
    Json(body): Json<GuestReportPayload>,
) -> ApiResult<Json<ApiResponse<Value>>> {
    let mut redis = state.redis.clone();
    let report = serde_json::json!({
        "agent_id": agent_id,
        "guest_health": body.guest_health,
        "metrics": body.metrics,
        "recent_events": body.recent_events,
        "received_at": chrono::Utc::now().to_rfc3339(),
    });
    let raw = serde_json::to_string(&report).unwrap_or_default();
    redis
        .set_ex::<_, _, ()>(report_key(&agent_id), &raw, 86400)
        .await
        .map_err(|e| ApiError::internal(e.to_string()))?;

  // Map to VM if meta has namespace/name
    let meta_raw: Option<String> = redis
        .get(format!("guest-agent:meta:{agent_id}"))
        .await
        .unwrap_or(None);
    if let Some(meta_str) = meta_raw {
        if let Ok(meta) = serde_json::from_str::<Value>(&meta_str) {
            if let (Some(ns), Some(vm)) = (
                meta.get("namespace").and_then(|v| v.as_str()),
                meta.get("vm_name").and_then(|v| v.as_str()),
            ) {
                redis
                    .set_ex::<_, _, ()>(vm_report_key(ns, vm), &raw, 86400)
                    .await
                    .map_err(|e| ApiError::internal(e.to_string()))?;

                if let Some(client) = state.kube.clone() {
                    let version = meta
                        .get("agent_version")
                        .and_then(|v| v.as_str())
                        .unwrap_or("unknown");
                    crate::kubevirt_guest_cr::patch_vmguestagent_health(
                        &client,
                        ns,
                        vm,
                        &body.guest_health,
                        version,
                    )
                    .await;
                }
            }
        }
    }

    Ok(Json(ApiResponse::ok(json!({ "accepted": true }))))
}

pub async fn get_guest_agent_report(
    State(state): State<AppState>,
    Path(agent_id): Path<String>,
) -> ApiResult<Json<ApiResponse<Value>>> {
    let mut redis = state.redis.clone();
    let raw: Option<String> = redis
        .get(report_key(&agent_id))
        .await
        .map_err(|e| ApiError::internal(e.to_string()))?;
    let report = raw
        .and_then(|s| serde_json::from_str::<Value>(&s).ok())
        .unwrap_or(json!({}));
    Ok(Json(ApiResponse::ok(report)))
}

pub async fn fetch_vm_guest_report(
    redis: &mut redis::aio::ConnectionManager,
    namespace: &str,
    name: &str,
) -> Option<Value> {
    let raw: Option<String> = redis.get(vm_report_key(namespace, name)).await.ok().flatten();
    raw.and_then(|s| serde_json::from_str(&s).ok())
}

pub async fn pull_guest_rpc(
    proxy_url: &str,
    method: &str,
    params: Value,
) -> Result<Value, String> {
    let client = reqwest::Client::builder()
        .timeout(Duration::from_secs(30))
        .build()
        .map_err(|e| e.to_string())?;

    let params_empty = params.as_object().map(|o| o.is_empty()).unwrap_or(true);
    let path = match method {
        "guestkit.getGuestHealth" if params_empty => "/guest/health",
        "guestkit.getSystemdUnits" if params_empty => "/guest/systemd",
        "guestkit.getGuestInfo" if params_empty => "/guest/info",
        "guestkit.getProcesses" if params_empty => "/guest/processes",
        "guestkit.getEvidence" if params_empty => "/evidence",
        _ => "/rpc",
    };

    if path == "/rpc" {
        let resp = client
            .post(format!("{proxy_url}/rpc"))
            .json(&json!({ "jsonrpc": "2.0", "method": method, "params": params, "id": 1 }))
            .send()
            .await
            .map_err(|e| e.to_string())?;
        resp.json().await.map_err(|e| e.to_string())
    } else {
        let resp = client
            .get(format!("{proxy_url}{path}"))
            .send()
            .await
            .map_err(|e| e.to_string())?;
        resp.json().await.map_err(|e| e.to_string())
    }
}
