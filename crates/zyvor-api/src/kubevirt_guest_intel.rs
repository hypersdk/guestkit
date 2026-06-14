// SPDX-License-Identifier: Apache-2.0
//! KubeVirt guest intelligence routes (pull + cached push reports).

use axum::extract::{Path, State};
use axum::Json;
use axum::Extension;
use serde_json::{json, Value};

use crate::error::{ApiError, ApiResult};
use crate::auth::types::AuthUserClaims;
use crate::models::ApiResponse;
use crate::routes::guest_agent::fetch_vm_guest_report;
use crate::kubevirt_guest_pull::{pull_for_vm, pull_for_vm_api, rpc_result};
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

    let pulled = pull_for_vm(&state, &namespace, &name, "guestkit.getGuestHealth", json!({}))
        .await
        .ok();

    let boot_analysis = pull_for_vm(
        &state,
        &namespace,
        &name,
        "guestkit.getBootAnalysis",
        json!({}),
    )
        .await
        .ok()
        .map(|v| rpc_result(v));

    let guest_health = pulled
        .as_ref()
        .map(|v| rpc_result(v.clone()))
        .or_else(|| cached.as_ref().and_then(|c| c.get("guest_health").cloned()));
    let metrics = cached.as_ref().and_then(|c| c.get("metrics").cloned());
    let pulled_events = pull_for_vm(
        &state,
        &namespace,
        &name,
        "guestkit.getSystemdEvents",
        json!({ "limit": 50 }),
    )
        .await
        .ok()
        .map(|v| rpc_result(v))
        .and_then(|v| v.get("events").cloned());
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

    let packetwolf = vm.as_ref().map(packetwolf_from_vm).unwrap_or(json!({}));

    Ok(Json(ApiResponse::ok(json!({
        "guest": guest,
        "guest_health": guest_health,
        "boot_analysis": boot_analysis,
        "metrics": metrics,
        "recent_events": recent_events,
        "report_source": report_source,
        "received_at": received_at,
        "packetwolf": packetwolf,
    }))))
}

fn packetwolf_from_vm(vm: &serde_json::Value) -> serde_json::Value {
    let ann = vm.pointer("/metadata/annotations").cloned().unwrap_or(json!({}));
    json!({
        "correlation": ann.get("zeus.zyvor.dev/packetwolf-correlation").and_then(|v| v.as_str()),
        "correlation_at": ann.get("zeus.zyvor.dev/packetwolf-correlation-at").and_then(|v| v.as_str()),
        "fleet_correlation": ann.get("zeus.zyvor.dev/packetwolf-fleet-correlation").and_then(|v| v.as_str()),
        "fleet_at": ann.get("zeus.zyvor.dev/packetwolf-fleet-at").and_then(|v| v.as_str()),
        "fleet_count": ann.get("zeus.zyvor.dev/packetwolf-fleet-count").and_then(|v| v.as_str()),
    })
}

pub async fn get_guest_systemd(
    State(state): State<AppState>,
    Path((namespace, name)): Path<(String, String)>,
) -> ApiResult<Json<ApiResponse<Value>>> {
    let resp = pull_for_vm_api(
        &state,
        &namespace,
        &name,
        "guestkit.getSystemdUnits",
        json!({}),
    )
    .await?;

    let units = rpc_result(resp);
    Ok(Json(ApiResponse::ok(json!({
        "namespace": namespace,
        "name": name,
        "units": units,
    }))))
}

pub async fn post_restart_unit(
    State(state): State<AppState>,
    Path((namespace, name)): Path<(String, String)>,
    user: Option<Extension<AuthUserClaims>>,
    Json(body): Json<Value>,
) -> ApiResult<Json<ApiResponse<Value>>> {
    crate::guest_remediation_auth::require_guest_remediation_requester(
        &state,
        user.as_deref(),
    )?;
    let unit = body
        .get("unit")
        .and_then(|v| v.as_str())
        .ok_or_else(|| ApiError::bad_request("unit is required"))?;

    crate::guest_action_policy::enforce_restart_unit(state.kube.as_ref(), unit, true).await?;

    if crate::guest_actions::policy_requires_approval(state.kube.as_ref()).await {
        let mut redis = state.redis.clone();
        let requested_by = user.as_ref().map(|Extension(u)| {
            u.email
                .clone()
                .or_else(|| u.name.clone())
                .unwrap_or_else(|| u.sub.clone())
        });
        let action_id = crate::guest_actions::enqueue_restart_unit(
            &mut redis,
            &namespace,
            &name,
            unit,
            requested_by.as_deref(),
        )
        .await?;
        return Ok(Json(ApiResponse::ok(json!({
            "namespace": namespace,
            "name": name,
            "unit": unit,
            "status": "pending_approval",
            "action_id": action_id,
        }))));
    }

    let resp = pull_for_vm_api(
        &state,
        &namespace,
        &name,
        "guestkit.restartUnit",
        json!({ "unit": unit }),
    )
    .await?;

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
    let resp = pull_for_vm_api(
        &state,
        &namespace,
        &name,
        "guestkit.getJournalSlice",
        json!({ "unit": "", "limit": 200 }),
    )
    .await?;

    Ok(Json(ApiResponse::ok(json!({
        "namespace": namespace,
        "name": name,
        "journal": rpc_result(resp),
    }))))
}

pub async fn post_collect_support_bundle(
    State(state): State<AppState>,
    Path((namespace, name)): Path<(String, String)>,
    user: Option<Extension<AuthUserClaims>>,
) -> ApiResult<Json<ApiResponse<Value>>> {
    crate::guest_remediation_auth::require_guest_remediation_requester(
        &state,
        user.as_deref(),
    )?;
    crate::guest_action_policy::enforce_support_bundle(state.kube.as_ref(), true).await?;

    if crate::guest_actions::policy_requires_approval(state.kube.as_ref()).await {
        let mut redis = state.redis.clone();
        let requested_by = user.as_ref().map(|Extension(u)| {
            u.email
                .clone()
                .or_else(|| u.name.clone())
                .unwrap_or_else(|| u.sub.clone())
        });
        let action_id = crate::guest_actions::enqueue_support_bundle(
            &mut redis,
            &namespace,
            &name,
            requested_by.as_deref(),
        )
        .await?;
        return Ok(Json(ApiResponse::ok(json!({
            "namespace": namespace,
            "name": name,
            "status": "pending_approval",
            "action_id": action_id,
        }))));
    }

    let resp = pull_for_vm_api(
        &state,
        &namespace,
        &name,
        "guestkit.collectSupportBundle",
        json!({}),
    )
    .await?;

    Ok(Json(ApiResponse::ok(json!({
        "namespace": namespace,
        "name": name,
        "bundle": rpc_result(resp),
    }))))
}

pub async fn post_pre_snapshot_freeze(
    State(state): State<AppState>,
    Path((namespace, name)): Path<(String, String)>,
    user: Option<Extension<AuthUserClaims>>,
) -> ApiResult<Json<ApiResponse<Value>>> {
    crate::guest_remediation_auth::require_guest_remediation_requester(
        &state,
        user.as_deref(),
    )?;
    let resp = pull_for_vm_api(
        &state,
        &namespace,
        &name,
        "guestkit.freezeFilesystem",
        json!({}),
    )
    .await?;

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
    user: Option<Extension<AuthUserClaims>>,
) -> ApiResult<Json<ApiResponse<Value>>> {
    crate::guest_remediation_auth::require_guest_remediation_requester(
        &state,
        user.as_deref(),
    )?;
    let resp = pull_for_vm_api(
        &state,
        &namespace,
        &name,
        "guestkit.thawFilesystem",
        json!({}),
    )
    .await?;

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

    if let Ok(resp) = pull_for_vm(
        &state,
        &namespace,
        &name,
        "guestkit.getGuestHealth",
        json!({}),
    )
    .await
    {
        return Ok(Json(ApiResponse::ok(json!({
            "namespace": namespace,
            "name": name,
            "guest_health": rpc_result(resp),
            "source": "pull",
        }))));
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
    let unit = query.get("unit").cloned().unwrap_or_default();
    let boot = query.get("boot").cloned().unwrap_or_else(|| "current".into());
    let limit = query
        .get("limit")
        .and_then(|v| v.parse().ok())
        .unwrap_or(200);

    let resp = pull_for_vm_api(
        &state,
        &namespace,
        &name,
        "guestkit.getJournalSlice",
        json!({ "unit": unit, "boot": boot, "limit": limit }),
    )
    .await?;

    Ok(Json(ApiResponse::ok(json!({
        "namespace": namespace,
        "name": name,
        "journal": rpc_result(resp),
    }))))
}

pub async fn get_guest_processes(
    State(state): State<AppState>,
    Path((namespace, name)): Path<(String, String)>,
) -> ApiResult<Json<ApiResponse<Value>>> {
    let resp = pull_for_vm_api(
        &state,
        &namespace,
        &name,
        "guestkit.getProcesses",
        json!({}),
    )
    .await?;

    Ok(Json(ApiResponse::ok(json!({
        "namespace": namespace,
        "name": name,
        "processes": rpc_result(resp),
    }))))
}

pub async fn get_guest_systemd_unit(
    State(state): State<AppState>,
    Path((namespace, name, unit)): Path<(String, String, String)>,
) -> ApiResult<Json<ApiResponse<Value>>> {
    let resp = pull_for_vm_api(
        &state,
        &namespace,
        &name,
        "guestkit.getSystemdUnit",
        json!({ "unit": unit }),
    )
    .await?;

    Ok(Json(ApiResponse::ok(json!({
        "namespace": namespace,
        "name": name,
        "unit": unit,
        "detail": rpc_result(resp),
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

    let resp = pull_for_vm_api(
        &state,
        &namespace,
        &name,
        "guestkit.getSystemdEvents",
        json!({ "limit": 100 }),
    )
    .await?;

    Ok(Json(ApiResponse::ok(json!({
        "namespace": namespace,
        "name": name,
        "events": rpc_result(resp),
        "source": "pull",
    }))))
}
