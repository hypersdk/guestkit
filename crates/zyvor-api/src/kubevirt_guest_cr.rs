// SPDX-License-Identifier: Apache-2.0
//! Sync VMGuestAgent CR status from guest push reports.

use kube::api::{Api, ApiResource, Patch, PatchParams};
use kube::Client;
use serde_json::{json, Value};

fn vmguestagent_resource() -> ApiResource {
    ApiResource {
        group: "zeus.zyvor.dev".into(),
        version: "v1alpha1".into(),
        api_version: "zeus.zyvor.dev/v1alpha1".into(),
        kind: "VMGuestAgent".into(),
        plural: "vmguestagents".into(),
    }
}

/// Patch VMGuestAgent status with live guest health from agent push.
pub async fn patch_vmguestagent_health(
    client: &Client,
    namespace: &str,
    vm_name: &str,
    guest_health: &Value,
    agent_version: &str,
) {
    let cr_name = format!("{vm_name}-vmtools");
    let ar = vmguestagent_resource();
    let api: Api<kube::api::DynamicObject> = Api::namespaced_with(client.clone(), namespace, &ar);

    if api.get(&cr_name).await.is_err() {
        let spec_obj = json!({
            "apiVersion": "zeus.zyvor.dev/v1alpha1",
            "kind": "VMGuestAgent",
            "metadata": {
                "name": cr_name,
                "namespace": namespace,
                "labels": { "zeus.zyvor.dev/vm": vm_name }
            },
            "spec": {
                "vmRef": { "namespace": namespace, "name": vm_name },
                "desiredVersion": agent_version,
            }
        });
        if let Ok(obj) = serde_json::from_value::<kube::api::DynamicObject>(spec_obj) {
            let _ = api.create(&kube::api::PostParams::default(), &obj).await;
        }
    }

    let guest_level = guest_health
        .get("guest_health")
        .and_then(|v| v.as_str())
        .unwrap_or("unknown");
    let score = guest_health
        .get("score")
        .and_then(|v| v.as_u64())
        .unwrap_or(0) as u8;
    let failed_units = guest_health
        .get("failed_units")
        .and_then(|v| v.as_u64())
        .unwrap_or(0) as u32;
    let systemd_state = guest_health
        .get("systemd_state")
        .and_then(|v| v.as_str())
        .unwrap_or("unknown");
    let hostname = guest_health
        .get("vm_hostname")
        .and_then(|v| v.as_str())
        .unwrap_or("");
    let journal_hints = guest_health
        .get("journal_hints")
        .and_then(|v| v.as_array())
        .map(|a| a.len())
        .unwrap_or(0);

    let status = json!({
        "status": {
            "installed": true,
            "connected": true,
            "version": agent_version,
            "zyvorAgentReady": true,
            "lastHeartbeat": chrono::Utc::now().to_rfc3339(),
            "guestHealth": guest_level,
            "healthScore": score,
            "failedUnits": failed_units,
            "systemdState": systemd_state,
            "hostname": hostname,
            "journalHintCount": journal_hints,
        }
    });

    let _ = api
        .patch_status(&cr_name, &PatchParams::default(), &Patch::Merge(&status))
        .await;
}
