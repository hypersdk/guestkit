// SPDX-License-Identifier: Apache-2.0
//! KubeVirt guest intelligence routes (pull + cached push reports).

use axum::extract::{Path, State};
use axum::Json;
use serde_json::{json, Value};

use crate::error::{ApiError, ApiResult};
use crate::models::ApiResponse;
use crate::routes::guest_agent::{fetch_vm_guest_report, pull_guest_rpc};
use crate::routes::kubevirt::{build_guest_info, fetch_vm, fetch_vmi};
use crate::state::AppState;

pub async fn get_guest_info(
    State(state): State<AppState>,
    Path((namespace, name)): Path<(String, String)>,
) -> ApiResult<Json<ApiResponse<Value>>> {
    let client = state
        .kube
        .clone()
        .ok_or_else(|| ApiError::bad_request("KubeVirt requires in-cluster access"))?;

    let vm = fetch_vm(&client, &namespace, &name).await;
    let vmi = fetch_vmi(&client, &namespace, &name).await;
    let vmi_running = vmi.is_some();
    let guest = build_guest_info(vm.as_ref(), vmi.as_ref(), vmi_running);

    let mut redis = state.redis.clone();
    let cached = fetch_vm_guest_report(&mut redis, &namespace, &name).await;

    let pulled = if let Some(proxy) = state.config.agent_proxy_url.as_ref() {
        pull_guest_rpc(proxy, "guestkit.getGuestHealth", json!({}))
            .await
            .ok()
    } else {
        None
    };

    let boot_analysis = if let Some(proxy) = state.config.agent_proxy_url.as_ref() {
        pull_guest_rpc(proxy, "guestkit.getBootAnalysis", json!({}))
            .await
            .ok()
            .and_then(|r| r.get("result").cloned())
    } else {
        None
    };

    let guest_health = pulled
        .as_ref()
        .and_then(|r| r.get("result").cloned())
        .or_else(|| cached.as_ref().and_then(|c| c.get("guest_health").cloned()));
    let metrics = cached.as_ref().and_then(|c| c.get("metrics").cloned());
    let pulled_events = if let Some(proxy) = state.config.agent_proxy_url.as_ref() {
        pull_guest_rpc(proxy, "guestkit.getSystemdEvents", json!({ "limit": 50 }))
            .await
            .ok()
            .and_then(|r| r.get("result").and_then(|v| v.get("events").cloned()))
    } else {
        None
    };
    let recent_events = pulled_events
        .or_else(|| cached.as_ref().and_then(|c| c.get("recent_events").cloned()));
    let report_source = if pulled.is_some() {
        "pull"
    } else if cached.is_some() {
        "push"
    } else {
        "none"
    };
    let received_at = cached.as_ref().and_then(|c| c.get("received_at").cloned());

    Ok(Json(ApiResponse::ok(json!({
        "guest": guest,
        "guest_health": guest_health,
        "boot_analysis": boot_analysis,
        "metrics": metrics,
        "recent_events": recent_events,
        "report_source": report_source,
        "received_at": received_at,
    }))))
}

pub async fn get_guest_systemd(
    State(state): State<AppState>,
    Path((namespace, name)): Path<(String, String)>,
) -> ApiResult<Json<ApiResponse<Value>>> {
    let proxy = state
        .config
        .agent_proxy_url
        .as_ref()
        .ok_or_else(|| ApiError::bad_request("agent_proxy_url required for live systemd pull"))?;

    let resp = pull_guest_rpc(proxy, "guestkit.getSystemdUnits", json!({}))
        .await
        .map_err(|e| ApiError::bad_request(e))?;

    let units = resp.get("result").cloned().unwrap_or(json!([]));
    Ok(Json(ApiResponse::ok(json!({
        "namespace": namespace,
        "name": name,
        "units": units,
    }))))
}

pub async fn post_restart_unit(
    State(state): State<AppState>,
    Path((namespace, name)): Path<(String, String)>,
    Json(body): Json<Value>,
) -> ApiResult<Json<ApiResponse<Value>>> {
    let unit = body
        .get("unit")
        .and_then(|v| v.as_str())
        .ok_or_else(|| ApiError::bad_request("unit is required"))?;
    let proxy = state
        .config
        .agent_proxy_url
        .as_ref()
        .ok_or_else(|| ApiError::bad_request("agent_proxy_url required"))?;

    let resp = pull_guest_rpc(
        proxy,
        "guestkit.restartUnit",
        json!({ "unit": unit }),
    )
    .await
    .map_err(|e| ApiError::bad_request(e))?;

    Ok(Json(ApiResponse::ok(json!({
        "namespace": namespace,
        "name": name,
        "unit": unit,
        "result": resp,
    }))))
}

pub async fn get_guest_logs(
    State(state): State<AppState>,
    Path((namespace, name)): Path<(String, String)>,
) -> ApiResult<Json<ApiResponse<Value>>> {
    let proxy = state
        .config
        .agent_proxy_url
        .as_ref()
        .ok_or_else(|| ApiError::bad_request("agent_proxy_url required for guest logs"))?;

    let resp = pull_guest_rpc(
        proxy,
        "guestkit.getJournalSlice",
        json!({ "unit": "", "limit": 200 }),
    )
    .await
    .map_err(|e| ApiError::bad_request(e))?;

    Ok(Json(ApiResponse::ok(json!({
        "namespace": namespace,
        "name": name,
        "journal": resp.get("result").cloned().unwrap_or(json!({})),
    }))))
}

pub async fn post_collect_support_bundle(
    State(state): State<AppState>,
    Path((namespace, name)): Path<(String, String)>,
) -> ApiResult<Json<ApiResponse<Value>>> {
    let proxy = state
        .config
        .agent_proxy_url
        .as_ref()
        .ok_or_else(|| ApiError::bad_request("agent_proxy_url required"))?;

    let resp = pull_guest_rpc(proxy, "guestkit.collectSupportBundle", json!({}))
        .await
        .map_err(|e| ApiError::bad_request(e))?;

    Ok(Json(ApiResponse::ok(json!({
        "namespace": namespace,
        "name": name,
        "bundle": resp.get("result").cloned().unwrap_or(json!({})),
    }))))
}

pub async fn post_pre_snapshot_freeze(
    State(state): State<AppState>,
    Path((namespace, name)): Path<(String, String)>,
) -> ApiResult<Json<ApiResponse<Value>>> {
    let proxy = state
        .config
        .agent_proxy_url
        .as_ref()
        .ok_or_else(|| ApiError::bad_request("agent_proxy_url required"))?;

    let resp = pull_guest_rpc(proxy, "guestkit.freezeFilesystem", json!({}))
        .await
        .map_err(|e| ApiError::bad_request(e))?;

    Ok(Json(ApiResponse::ok(json!({
        "namespace": namespace,
        "name": name,
        "action": "freeze",
        "result": resp,
    }))))
}

pub async fn post_post_snapshot_thaw(
    State(state): State<AppState>,
    Path((namespace, name)): Path<(String, String)>,
) -> ApiResult<Json<ApiResponse<Value>>> {
    let proxy = state
        .config
        .agent_proxy_url
        .as_ref()
        .ok_or_else(|| ApiError::bad_request("agent_proxy_url required"))?;

    let resp = pull_guest_rpc(proxy, "guestkit.thawFilesystem", json!({}))
        .await
        .map_err(|e| ApiError::bad_request(e))?;

    Ok(Json(ApiResponse::ok(json!({
        "namespace": namespace,
        "name": name,
        "action": "thaw",
        "result": resp,
    }))))
}

pub async fn get_guest_health(
    State(state): State<AppState>,
    Path((namespace, name)): Path<(String, String)>,
) -> ApiResult<Json<ApiResponse<Value>>> {
    let mut redis = state.redis.clone();
    let cached = fetch_vm_guest_report(&mut redis, &namespace, &name).await;

    if let Some(proxy) = state.config.agent_proxy_url.as_ref() {
        if let Ok(resp) = pull_guest_rpc(proxy, "guestkit.getGuestHealth", json!({})).await {
            return Ok(Json(ApiResponse::ok(json!({
                "namespace": namespace,
                "name": name,
                "guest_health": resp.get("result").cloned().unwrap_or(resp),
                "source": "pull",
            }))));
        }
    }

    let guest_health = cached.as_ref().and_then(|c| c.get("guest_health").cloned());
    Ok(Json(ApiResponse::ok(json!({
        "namespace": namespace,
        "name": name,
        "guest_health": guest_health,
        "source": if cached.is_some() { "push" } else { "none" },
    }))))
}

pub async fn get_guest_journal(
    State(state): State<AppState>,
    Path((namespace, name)): Path<(String, String)>,
    axum::extract::Query(query): axum::extract::Query<std::collections::HashMap<String, String>>,
) -> ApiResult<Json<ApiResponse<Value>>> {
    let proxy = state
        .config
        .agent_proxy_url
        .as_ref()
        .ok_or_else(|| ApiError::bad_request("agent_proxy_url required"))?;

    let unit = query.get("unit").cloned().unwrap_or_default();
    let boot = query.get("boot").cloned().unwrap_or_else(|| "current".into());
    let limit = query
        .get("limit")
        .and_then(|v| v.parse().ok())
        .unwrap_or(200);

    let resp = pull_guest_rpc(
        proxy,
        "guestkit.getJournalSlice",
        json!({ "unit": unit, "boot": boot, "limit": limit }),
    )
    .await
    .map_err(|e| ApiError::bad_request(e))?;

    Ok(Json(ApiResponse::ok(json!({
        "namespace": namespace,
        "name": name,
        "journal": resp.get("result").cloned().unwrap_or(resp),
    }))))
}

pub async fn get_guest_processes(
    State(state): State<AppState>,
    Path((namespace, name)): Path<(String, String)>,
) -> ApiResult<Json<ApiResponse<Value>>> {
    let proxy = state
        .config
        .agent_proxy_url
        .as_ref()
        .ok_or_else(|| ApiError::bad_request("agent_proxy_url required"))?;

    let resp = pull_guest_rpc(proxy, "guestkit.getProcesses", json!({}))
        .await
        .map_err(|e| ApiError::bad_request(e))?;

    Ok(Json(ApiResponse::ok(json!({
        "namespace": namespace,
        "name": name,
        "processes": resp.get("result").cloned().unwrap_or(resp),
    }))))
}

pub async fn get_guest_systemd_unit(
    State(state): State<AppState>,
    Path((namespace, name, unit)): Path<(String, String, String)>,
) -> ApiResult<Json<ApiResponse<Value>>> {
    let proxy = state
        .config
        .agent_proxy_url
        .as_ref()
        .ok_or_else(|| ApiError::bad_request("agent_proxy_url required"))?;

    let resp = pull_guest_rpc(
        proxy,
        "guestkit.getSystemdUnit",
        json!({ "unit": unit }),
    )
    .await
    .map_err(|e| ApiError::bad_request(e))?;

    Ok(Json(ApiResponse::ok(json!({
        "namespace": namespace,
        "name": name,
        "unit": unit,
        "detail": resp.get("result").cloned().unwrap_or(resp),
    }))))
}

pub async fn get_guest_systemd_events(
    State(state): State<AppState>,
    Path((namespace, name)): Path<(String, String)>,
) -> ApiResult<Json<ApiResponse<Value>>> {
    let mut redis = state.redis.clone();
    let cached = fetch_vm_guest_report(&mut redis, &namespace, &name).await;
    if let Some(events) = cached.as_ref().and_then(|c| c.get("recent_events").cloned()) {
        return Ok(Json(ApiResponse::ok(json!({
            "namespace": namespace,
            "name": name,
            "events": events,
            "source": "push",
        }))));
    }

    let proxy = state
        .config
        .agent_proxy_url
        .as_ref()
        .ok_or_else(|| ApiError::bad_request("agent_proxy_url required for live events"))?;

    let resp = pull_guest_rpc(proxy, "guestkit.getSystemdEvents", json!({ "limit": 100 }))
        .await
        .map_err(|e| ApiError::bad_request(e))?;

    Ok(Json(ApiResponse::ok(json!({
        "namespace": namespace,
        "name": name,
        "events": resp.get("result").cloned().unwrap_or(resp),
        "source": "pull",
    }))))
}
