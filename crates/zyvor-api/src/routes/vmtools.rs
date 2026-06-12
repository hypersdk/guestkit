// SPDX-License-Identifier: Apache-2.0
//! Zeus VM Tools — bundle, fleet coverage, install orchestration.

use axum::extract::{Path, Query, State};
use axum::Json;
use kube::api::{Api, Patch, PatchParams};
use kube::discovery::ApiResource;
use kube::Client;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::collections::HashMap;

use crate::error::{ApiError, ApiResult};
use crate::kubevirt_guest_agent::{install_guest_agent, vm_is_windows, GuestAgentInstallResult};
use crate::models::ApiResponse;
use crate::routes::kubevirt::{
    fetch_vm, get_guest_agent_info, list_dynamic_all, vm_resource, vmi_resource, GuestAgentInfo,
};
use crate::state::AppState;

pub const VMTOOLS_VERSION: &str = "0.1.0";

#[derive(Debug, Clone, Serialize)]
pub struct VMToolsBundleInfo {
    pub version: String,
    pub channel: String,
    pub linux_rpm_url: Option<String>,
    pub linux_deb_url: Option<String>,
    pub linux_tar_url: Option<String>,
    pub iso_url: Option<String>,
    pub agent_binary_url: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct VMToolsCoverage {
    pub total_vms: usize,
    pub installed: usize,
    pub connected: usize,
    pub missing: usize,
    pub outdated: usize,
    pub windows_virtio_win: usize,
}

#[derive(Debug, Clone, Serialize)]
pub struct VMToolsVmStatus {
    pub namespace: String,
    pub name: String,
    pub product: String,
    pub installed: bool,
    pub connected: bool,
    pub version: Option<String>,
    pub recommended_method: String,
    pub os_name: Option<String>,
    pub os_family: String,
    pub ip_address: Option<String>,
    pub qga_ready: bool,
    pub zyvor_agent_ready: bool,
    pub snapshot_quiesce: bool,
    pub message: String,
    pub guest_agent: GuestAgentInfo,
}

#[derive(Debug, Deserialize, Default)]
pub struct InstallQuery {
    #[serde(default)]
    pub restart: Option<bool>,
}

pub fn bundle_info(state: &AppState) -> VMToolsBundleInfo {
    let base = std::env::var("VMTOOLS_BUNDLE_URL")
        .ok()
        .filter(|u| !u.trim().is_empty())
        .or_else(|| {
            state
                .config
                .zeus_public_url
                .as_ref()
                .map(|u| format!("{}/api/v1/engines/vmtools", u.trim_end_matches('/')))
        })
        .unwrap_or_else(|| "http://zyvor-api:8080/api/v1/vmtools/bundle".into());
    let agent = crate::kubevirt_guest_agent::resolve_guestkit_binary_url();
    VMToolsBundleInfo {
        version: VMTOOLS_VERSION.into(),
        channel: std::env::var("VMTOOLS_CHANNEL").unwrap_or_else(|_| "stable".into()),
        linux_rpm_url: Some(format!("{base}/linux/zyvor-vm-tools-{VMTOOLS_VERSION}.rpm")),
        linux_deb_url: Some(format!("{base}/linux/zyvor-vm-tools_{VMTOOLS_VERSION}_amd64.deb")),
        linux_tar_url: Some(format!("{base}/linux/zyvor-vm-tools-linux-amd64.tar.gz")),
        iso_url: Some(format!("{base}/zyvor-vm-tools.iso")),
        agent_binary_url: agent,
    }
}

pub async fn get_bundle(
    State(state): State<AppState>,
) -> Json<ApiResponse<VMToolsBundleInfo>> {
    Json(ApiResponse::ok(bundle_info(&state)))
}

pub async fn get_coverage(
    State(state): State<AppState>,
) -> ApiResult<Json<ApiResponse<VMToolsCoverage>>> {
    let client = kube_client(&state)?;
    let vms = list_dynamic_all(&client, &vm_resource()).await?;
    let vmis = list_dynamic_all(&client, &vmi_resource()).await?;
    let mut vmi_map: HashMap<(String, String), Value> = HashMap::new();
    for vmi in vmis {
        let ns = vmi
            .pointer("/metadata/namespace")
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string();
        let name = vmi
            .pointer("/metadata/labels/kubevirt.io/vm")
            .and_then(|v| v.as_str())
            .map(String::from)
            .or_else(|| {
                vmi.pointer("/metadata/name")
                    .and_then(|v| v.as_str())
                    .map(String::from)
            })
            .unwrap_or_default();
        vmi_map.insert((ns, name), vmi);
    }

    let mut coverage = VMToolsCoverage {
        total_vms: 0,
        installed: 0,
        connected: 0,
        missing: 0,
        outdated: 0,
        windows_virtio_win: 0,
    };

    for vm in vms {
        let name = vm.pointer("/metadata/name").and_then(|v| v.as_str()).unwrap_or("");
        let namespace = vm
            .pointer("/metadata/namespace")
            .and_then(|v| v.as_str())
            .unwrap_or("");
        if name.is_empty() || namespace.is_empty() {
            continue;
        }
        coverage.total_vms += 1;
        let vmi = vmi_map.get(&(namespace.to_string(), name.to_string()));
        let is_win = vm_is_windows(Some(&vm), vmi);
        if is_win {
            coverage.windows_virtio_win += 1;
        }
        let label = vm
            .pointer("/metadata/labels/zeus.zyvor.dev/guest-tools")
            .and_then(|v| v.as_str())
            .unwrap_or("missing");
        let agent_ok = vmi.map(agent_connected).unwrap_or(false);
        let tools_installed = agent_ok
            || matches!(label, "installed" | "connected");
        if agent_ok {
            coverage.connected += 1;
        }
        if tools_installed {
            coverage.installed += 1;
        } else {
            coverage.missing += 1;
        }
        let version = vm
            .pointer("/metadata/annotations/zeus.zyvor.dev/tools-version")
            .and_then(|v| v.as_str());
        if tools_installed {
            if let Some(ver) = version {
                if ver != VMTOOLS_VERSION {
                    coverage.outdated += 1;
                }
            } else if !agent_ok {
                coverage.outdated += 1;
            }
        }
    }

    Ok(Json(ApiResponse::ok(coverage)))
}

pub async fn get_vm_vmtools(
    State(state): State<AppState>,
    Path((namespace, name)): Path<(String, String)>,
) -> ApiResult<Json<ApiResponse<VMToolsVmStatus>>> {
    let guest = get_guest_agent_info(
        State(state.clone()),
        Path((namespace.clone(), name.clone())),
    )
    .await?
    .0
    .data;
    let status = build_vm_vmtools_status(&state, &namespace, &name, guest).await?;
    Ok(Json(ApiResponse::ok(status)))
}

pub async fn install_vm_vmtools(
    State(state): State<AppState>,
    Path((namespace, name)): Path<(String, String)>,
    Query(query): Query<InstallQuery>,
) -> ApiResult<Json<ApiResponse<GuestAgentInstallResult>>> {
    let client = kube_client(&state)?;
    let restart = query.restart.unwrap_or(true);
    let result = install_guest_agent(client.clone(), &namespace, &name, restart).await?;
    if result.success && !result.is_windows {
        sync_vm_tools_labels(&client, &namespace, &name, "installed", VMTOOLS_VERSION).await;
    }
    Ok(Json(ApiResponse::ok(result)))
}

pub async fn run_vm_diagnostics(
    State(state): State<AppState>,
    Path((namespace, name)): Path<(String, String)>,
) -> ApiResult<Json<ApiResponse<Value>>> {
    let guest = get_guest_agent_info(
        State(state.clone()),
        Path((namespace.clone(), name.clone())),
    )
    .await?
    .0
    .data;
    if !guest.agent_connected {
        return Err(ApiError::bad_request(
            "Guest agent not connected — install Zeus VM Tools first",
        ));
    }
    sync_vm_tools_labels(
        state.kube.as_ref().unwrap(),
        &namespace,
        &name,
        "connected",
        VMTOOLS_VERSION,
    )
    .await;
    Ok(Json(ApiResponse::ok(json!({
        "health": guest.health,
        "message": guest.message,
        "os_name": guest.os_name,
        "agent_version": guest.guest_agent_version,
        "diagnostics": "Live guest agent reachable — run Doctor via agent RPC for full score"
    }))))
}

async fn build_vm_vmtools_status(
    state: &AppState,
    namespace: &str,
    name: &str,
    guest: GuestAgentInfo,
) -> ApiResult<VMToolsVmStatus> {
    let client = state.kube.as_ref();
    let vm = if let Some(c) = client {
        fetch_vm(c, namespace, name).await
    } else {
        None
    };
    let is_win = vm_is_windows(vm.as_ref(), None);
    let os_family = if is_win { "windows" } else { "linux" }.to_string();
    let installed_label = vm
        .as_ref()
        .and_then(|v| v.pointer("/metadata/labels/zeus.zyvor.dev/guest-tools"))
        .and_then(|v| v.as_str());
    let version = vm
        .as_ref()
        .and_then(|v| v.pointer("/metadata/annotations/zeus.zyvor.dev/tools-version"))
        .and_then(|v| v.as_str())
        .map(String::from);
    let installed = guest.agent_connected
        || installed_label == Some("installed")
        || installed_label == Some("connected");
    let recommended_method = if is_win {
        "virtio-win".into()
    } else if guest.agent_connected {
        "connected".into()
    } else if guest.vmi_running {
        "cloud-init".into()
    } else {
        "offline-inject".into()
    };
    let message = if is_win {
        "Install QEMU Guest Agent from Zeus OS Guest Tools (virtio-win ISO).".into()
    } else if guest.agent_connected {
        format!(
            "Zeus VM Tools connected{}",
            guest
                .guest_agent_version
                .as_ref()
                .map(|v| format!(" — v{v}"))
                .unwrap_or_default()
        )
    } else if guest.vmi_running {
        "Install Zeus VM Tools via cloud-init and restart the VM.".into()
    } else {
        "Start the VM or use offline GuestKit injection to bootstrap the agent.".into()
    };

    if guest.agent_connected {
        if let Some(c) = client {
            sync_vm_tools_labels(c, namespace, name, "connected", version.as_deref().unwrap_or(VMTOOLS_VERSION))
                .await;
        }
    }

    Ok(VMToolsVmStatus {
        namespace: namespace.into(),
        name: name.into(),
        product: "Zeus VM Tools".into(),
        installed,
        connected: guest.agent_connected,
        version,
        recommended_method,
        os_name: guest.os_name.clone(),
        os_family,
        ip_address: guest
            .interfaces
            .as_ref()
            .and_then(|ifs| {
                ifs.iter()
                    .find_map(|i| i.get("ipAddress").and_then(|v| v.as_str()))
            })
            .map(String::from),
        qga_ready: guest.agent_connected,
        zyvor_agent_ready: guest.agent_connected,
        snapshot_quiesce: guest.agent_connected && !is_win,
        message,
        guest_agent: guest,
    })
}

pub async fn sync_vm_tools_labels(
    client: &Client,
    namespace: &str,
    name: &str,
    tools_state: &str,
    version: &str,
) {
    let ar = vm_resource();
    let api: Api<kube::api::DynamicObject> = Api::namespaced_with(client.clone(), namespace, &ar);
    let patch = json!({
        "metadata": {
            "labels": {
                "zeus.zyvor.dev/guest-tools": tools_state,
            },
            "annotations": {
                "zeus.zyvor.dev/tools-version": version,
                "zeus.zyvor.dev/last-heartbeat": chrono::Utc::now().to_rfc3339(),
            }
        }
    });
    let _ = api
        .patch(name, &PatchParams::default(), &Patch::Merge(&patch))
        .await;
}

fn kube_client(state: &AppState) -> ApiResult<Client> {
    state
        .kube
        .clone()
        .ok_or_else(|| ApiError::bad_request("VM Tools requires in-cluster Kubernetes access"))
}

fn agent_connected(vmi: &Value) -> bool {
    vmi.pointer("/status/conditions")
        .and_then(|c| c.as_array())
        .map(|conds| {
            conds.iter().any(|c| {
                c.get("type").and_then(|t| t.as_str()) == Some("AgentConnected")
                    && c.get("status").and_then(|s| s.as_str()) == Some("True")
            })
        })
        .unwrap_or(false)
}
