// SPDX-License-Identifier: Apache-2.0
//! Cluster GuestActionPolicy enforcement for Zeus remediation APIs.

use kube::api::{Api, ApiResource};
use kube::Client;
use serde::{Deserialize, Serialize};

use crate::error::{ApiError, ApiResult};

#[derive(Debug, Clone, Default, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct GuestActionPolicySpec {
    #[serde(default)]
    pub allowed_actions: Vec<String>,
    #[serde(default)]
    pub restart_unit_allowlist: Vec<String>,
    #[serde(default)]
    pub require_approval: bool,
}

fn guest_action_policy_resource() -> ApiResource {
    ApiResource {
        group: "zeus.zyvor.dev".into(),
        version: "v1alpha1".into(),
        api_version: "zeus.zyvor.dev/v1alpha1".into(),
        kind: "GuestActionPolicy".into(),
        plural: "guestactionpolicies".into(),
    }
}

pub async fn fetch_guest_action_policy(client: &Client) -> Option<GuestActionPolicySpec> {
    let ar = guest_action_policy_resource();
    let api: Api<kube::api::DynamicObject> = Api::all_with(client.clone(), &ar);
    let items = api.list(&kube::api::ListParams::default().limit(1)).await.ok();
    items?
        .items
        .first()
        .and_then(|obj| obj.data.get("spec"))
        .and_then(|spec| serde_json::from_value(spec.clone()).ok())
}

pub fn restart_unit_allowed(policy: &GuestActionPolicySpec, unit: &str) -> bool {
    if !policy.allowed_actions.is_empty()
        && !policy
            .allowed_actions
            .iter()
            .any(|a| a == "restart_unit" || a == "restart-unit")
    {
        return false;
    }
    if policy.restart_unit_allowlist.is_empty() {
        return true;
    }
    policy.restart_unit_allowlist.iter().any(|u| u == unit)
}

pub fn action_allowed(policy: &GuestActionPolicySpec, action: &str) -> bool {
    if policy.allowed_actions.is_empty() {
        return true;
    }
    policy.allowed_actions.iter().any(|a| a == action || a.replace('-', "_") == action.replace('-', "_"))
}

pub async fn enforce_restart_unit(
    client: Option<&Client>,
    unit: &str,
    skip_approval: bool,
) -> ApiResult<()> {
    if let Some(client) = client {
        if let Some(policy) = fetch_guest_action_policy(client).await {
            if !skip_approval && policy.require_approval {
                return Err(ApiError::bad_request(
                    "GuestActionPolicy requires approval for remediation actions",
                ));
            }
            if !restart_unit_allowed(&policy, unit) {
                return Err(ApiError::bad_request(format!(
                    "restart_unit denied by GuestActionPolicy for {unit}"
                )));
            }
        }
    }
    Ok(())
}

pub async fn enforce_support_bundle(client: Option<&Client>, skip_approval: bool) -> ApiResult<()> {
    if let Some(client) = client {
        if let Some(policy) = fetch_guest_action_policy(client).await {
            if !skip_approval && policy.require_approval {
                return Err(ApiError::bad_request(
                    "GuestActionPolicy requires approval for support bundle collection",
                ));
            }
            if !action_allowed(&policy, "collect_support_bundle") {
                return Err(ApiError::bad_request(
                    "collect_support_bundle denied by GuestActionPolicy",
                ));
            }
        }
    }
    Ok(())
}

/// Summarize guest health counts from VMGuestAgent CRs.
pub async fn count_guest_health(client: &Client) -> (usize, usize, usize) {
    let ar = crate::kubevirt_guest_cr::vmguestagent_resource();
    let api: Api<kube::api::DynamicObject> = Api::all_with(client.clone(), &ar);
    let list = api.list(&kube::api::ListParams::default()).await;
    let mut healthy = 0;
    let mut degraded = 0;
    let mut unhealthy = 0;
    if let Ok(items) = list {
        for obj in items.items {
            let level = obj
                .data
                .pointer("/status/guestHealth")
                .and_then(|v| v.as_str())
                .unwrap_or("");
            match level {
                "healthy" => healthy += 1,
                "degraded" => degraded += 1,
                "unhealthy" => unhealthy += 1,
                _ => {}
            }
        }
    }
    (healthy, degraded, unhealthy)
}
