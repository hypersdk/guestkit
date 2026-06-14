// SPDX-License-Identifier: Apache-2.0
//! Pending guest remediation actions (approval workflow stub).

use axum::extract::{Path, State};
use axum::Json;
use redis::AsyncCommands;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};

use crate::error::{ApiError, ApiResult};
use crate::guest_action_policy::{enforce_restart_unit, fetch_guest_action_policy};
use crate::models::ApiResponse;
use crate::routes::guest_agent::pull_guest_rpc;
use crate::state::AppState;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PendingGuestAction {
    pub id: String,
    pub action: String,
    pub namespace: String,
    pub vm_name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub unit: Option<String>,
    pub status: String,
    pub created_at: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub approved_at: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub result: Option<Value>,
}

fn pending_key(id: &str) -> String {
    format!("guest-action:pending:{id}")
}

const PENDING_INDEX: &str = "guest-action:pending:index";

pub async fn enqueue_restart_unit(
    redis: &mut redis::aio::ConnectionManager,
    namespace: &str,
    vm_name: &str,
    unit: &str,
) -> Result<String, ApiError> {
    let id = uuid::Uuid::new_v4().to_string();
    let action = PendingGuestAction {
        id: id.clone(),
        action: "restart_unit".into(),
        namespace: namespace.into(),
        vm_name: vm_name.into(),
        unit: Some(unit.into()),
        status: "pending".into(),
        created_at: chrono::Utc::now().to_rfc3339(),
        approved_at: None,
        result: None,
    };
    let raw = serde_json::to_string(&action).map_err(|e| ApiError::internal(e.to_string()))?;
    redis
        .set_ex::<_, _, ()>(pending_key(&id), &raw, 86400)
        .await
        .map_err(|e| ApiError::internal(e.to_string()))?;
    redis
        .sadd::<_, _, ()>(PENDING_INDEX, &id)
        .await
        .map_err(|e| ApiError::internal(e.to_string()))?;
    Ok(id)
}

pub async fn enqueue_support_bundle(
    redis: &mut redis::aio::ConnectionManager,
    namespace: &str,
    vm_name: &str,
) -> Result<String, ApiError> {
    let id = uuid::Uuid::new_v4().to_string();
    let action = PendingGuestAction {
        id: id.clone(),
        action: "collect_support_bundle".into(),
        namespace: namespace.into(),
        vm_name: vm_name.into(),
        unit: None,
        status: "pending".into(),
        created_at: chrono::Utc::now().to_rfc3339(),
        approved_at: None,
        result: None,
    };
    let raw = serde_json::to_string(&action).map_err(|e| ApiError::internal(e.to_string()))?;
    redis
        .set_ex::<_, _, ()>(pending_key(&id), &raw, 86400)
        .await
        .map_err(|e| ApiError::internal(e.to_string()))?;
    redis
        .sadd::<_, _, ()>(PENDING_INDEX, &id)
        .await
        .map_err(|e| ApiError::internal(e.to_string()))?;
    Ok(id)
}

async fn load_pending(
    redis: &mut redis::aio::ConnectionManager,
    id: &str,
) -> ApiResult<PendingGuestAction> {
    let raw: Option<String> = redis
        .get(pending_key(id))
        .await
        .map_err(|e| ApiError::internal(e.to_string()))?;
    let action = raw
        .and_then(|s| serde_json::from_str(&s).ok())
        .ok_or_else(|| ApiError::bad_request("pending action not found"))?;
    Ok(action)
}

async fn save_pending(
    redis: &mut redis::aio::ConnectionManager,
    action: &PendingGuestAction,
) -> ApiResult<()> {
    let raw = serde_json::to_string(action).map_err(|e| ApiError::internal(e.to_string()))?;
    redis
        .set_ex::<_, _, ()>(pending_key(&action.id), &raw, 86400)
        .await
        .map_err(|e| ApiError::internal(e.to_string()))?;
    Ok(())
}

async fn execute_pending(state: &AppState, action: &PendingGuestAction) -> ApiResult<Value> {
    let proxy = state
        .config
        .agent_proxy_url
        .as_ref()
        .ok_or_else(|| ApiError::bad_request("agent_proxy_url required"))?;

    match action.action.as_str() {
        "restart_unit" => {
            let unit = action
                .unit
                .as_deref()
                .ok_or_else(|| ApiError::bad_request("unit missing on pending action"))?;
            enforce_restart_unit(state.kube.as_ref(), unit, true).await?;
            let resp = pull_guest_rpc(proxy, "guestkit.restartUnit", json!({ "unit": unit }))
                .await
                .map_err(|e| ApiError::bad_request(e))?;
            Ok(json!({
                "namespace": action.namespace,
                "name": action.vm_name,
                "unit": unit,
                "result": resp,
            }))
        }
        "collect_support_bundle" => {
            crate::guest_action_policy::enforce_support_bundle(state.kube.as_ref(), true).await?;
            let resp = pull_guest_rpc(proxy, "guestkit.collectSupportBundle", json!({}))
                .await
                .map_err(|e| ApiError::bad_request(e))?;
            Ok(json!({
                "namespace": action.namespace,
                "name": action.vm_name,
                "bundle": resp.get("result").cloned().unwrap_or(json!({})),
            }))
        }
        other => Err(ApiError::bad_request(format!("unsupported action: {other}"))),
    }
}

pub async fn list_pending_guest_actions(
    State(state): State<AppState>,
) -> ApiResult<Json<ApiResponse<Vec<PendingGuestAction>>>> {
    let mut redis = state.redis.clone();
    let ids: Vec<String> = redis
        .smembers(PENDING_INDEX)
        .await
        .map_err(|e| ApiError::internal(e.to_string()))?;
    let mut out = Vec::new();
    for id in ids {
        if let Ok(action) = load_pending(&mut redis, &id).await {
            if action.status == "pending" {
                out.push(action);
            }
        }
    }
    out.sort_by(|a, b| b.created_at.cmp(&a.created_at));
    Ok(Json(ApiResponse::ok(out)))
}

pub async fn approve_guest_action(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> ApiResult<Json<ApiResponse<Value>>> {
    let mut redis = state.redis.clone();
    let mut action = load_pending(&mut redis, &id).await?;
    if action.status != "pending" {
        return Err(ApiError::bad_request("action is not pending"));
    }
    let result = execute_pending(&state, &action).await?;
    action.status = "approved".into();
    action.approved_at = Some(chrono::Utc::now().to_rfc3339());
    action.result = Some(result.clone());
    save_pending(&mut redis, &action).await?;
    redis
        .srem::<_, _, ()>(PENDING_INDEX, &id)
        .await
        .map_err(|e| ApiError::internal(e.to_string()))?;
    Ok(Json(ApiResponse::ok(json!({
        "action_id": id,
        "status": "approved",
        "result": result,
    }))))
}

pub async fn reject_guest_action(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> ApiResult<Json<ApiResponse<Value>>> {
    let mut redis = state.redis.clone();
    let mut action = load_pending(&mut redis, &id).await?;
    if action.status != "pending" {
        return Err(ApiError::bad_request("action is not pending"));
    }
    action.status = "rejected".into();
    save_pending(&mut redis, &action).await?;
    redis
        .srem::<_, _, ()>(PENDING_INDEX, &id)
        .await
        .map_err(|e| ApiError::internal(e.to_string()))?;
    Ok(Json(ApiResponse::ok(json!({
        "action_id": id,
        "status": "rejected",
    }))))
}

pub async fn policy_requires_approval(client: Option<&kube::Client>) -> bool {
    if let Some(client) = client {
        if let Some(policy) = fetch_guest_action_policy(client).await {
            return policy.require_approval;
        }
    }
    false
}
