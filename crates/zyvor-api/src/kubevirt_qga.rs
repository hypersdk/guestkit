// SPDX-License-Identifier: Apache-2.0
//! QEMU guest-agent commands via virt-launcher (KubeVirt has no guest-exec subresource).

use base64::Engine;
use k8s_openapi::api::core::v1::Pod;
use kube::api::{AttachParams, Api, ListParams};
use kube::Client;
use serde_json::{json, Value};
use tokio::io::AsyncReadExt;

use crate::error::{ApiError, ApiResult};

const VIRT_LAUNCHER_CONTAINER: &str = "compute";

#[derive(Debug, Clone)]
pub struct QgaExecResult {
    pub exit_code: i64,
    pub stdout: String,
    pub stderr: String,
}

pub fn libvirt_domain(namespace: &str, name: &str) -> String {
    format!("{namespace}_{name}")
}

pub async fn qga_available(client: &Client, namespace: &str, name: &str) -> bool {
    qga_ping(client, namespace, name).await.is_ok()
}

pub async fn qga_ping(client: &Client, namespace: &str, name: &str) -> ApiResult<()> {
    let domain = libvirt_domain(namespace, name);
    let pod = find_virt_launcher_pod(client, namespace, name).await?;
    let out = virsh_qga_json(client, namespace, &pod, &domain, json!({"execute": "guest-ping"}))
        .await?;
    if out.get("return").is_some() {
        Ok(())
    } else if let Some(err) = out.get("error") {
        Err(ApiError::bad_request(format!("guest-ping failed: {err}")))
    } else {
        Err(ApiError::internal(format!("unexpected guest-ping response: {out}")))
    }
}

pub async fn qga_exec(
    client: &Client,
    namespace: &str,
    name: &str,
    path: &str,
    args: &[String],
    timeout_secs: u64,
) -> ApiResult<QgaExecResult> {
    let domain = libvirt_domain(namespace, name);
    let pod = find_virt_launcher_pod(client, namespace, name).await?;
    let exec_body = json!({
        "execute": "guest-exec",
        "arguments": {
            "path": path,
            "arg": args,
            "capture-output": true,
            "env": []
        }
    });
    let resp = virsh_qga_json(client, namespace, &pod, &domain, exec_body).await?;
    let pid = resp
        .pointer("/return/pid")
        .and_then(|p| p.as_u64())
        .ok_or_else(|| ApiError::internal(format!("guest-exec missing pid: {resp}")))?;

    let deadline =
        std::time::Instant::now() + std::time::Duration::from_secs(timeout_secs.max(5));
    loop {
        let status_body = json!({
            "execute": "guest-exec-status",
            "arguments": { "pid": pid }
        });
        let status =
            virsh_qga_json(client, namespace, &pod, &domain, status_body).await?;
        let ret = status
            .get("return")
            .ok_or_else(|| ApiError::internal(format!("guest-exec-status: {status}")))?;
        if ret.get("exited").and_then(|v| v.as_bool()) == Some(true) {
            let stdout = decode_b64_field(ret.get("out-data"));
            let stderr = decode_b64_field(ret.get("err-data"));
            let exit_code = ret
                .get("exitcode")
                .and_then(|v| v.as_i64())
                .unwrap_or(1);
            return Ok(QgaExecResult {
                exit_code,
                stdout,
                stderr,
            });
        }
        if std::time::Instant::now() >= deadline {
            return Err(ApiError::internal(format!(
                "guest-exec pid {pid} timed out after {timeout_secs}s"
            )));
        }
        tokio::time::sleep(std::time::Duration::from_millis(500)).await;
    }
}

pub async fn qga_exec_shell(
    client: &Client,
    namespace: &str,
    name: &str,
    script: &str,
    timeout_secs: u64,
) -> ApiResult<QgaExecResult> {
    qga_exec(
        client,
        namespace,
        name,
        "/bin/sh",
        &["-c".into(), script.into()],
        timeout_secs,
    )
    .await
}

pub async fn qga_exec_powershell(
    client: &Client,
    namespace: &str,
    name: &str,
    script: &str,
    timeout_secs: u64,
) -> ApiResult<QgaExecResult> {
    qga_exec(
        client,
        namespace,
        name,
        "C:\\Windows\\System32\\WindowsPowerShell\\v1.0\\powershell.exe",
        &[
            "-NoProfile".into(),
            "-NonInteractive".into(),
            "-ExecutionPolicy".into(),
            "Bypass".into(),
            "-Command".into(),
            script.into(),
        ],
        timeout_secs,
    )
    .await
}

async fn find_virt_launcher_pod(client: &Client, namespace: &str, vm_name: &str) -> ApiResult<String> {
    let pods: Api<Pod> = Api::namespaced(client.clone(), namespace);
    for label in [
        format!("kubevirt.io/vm={vm_name}"),
        format!("kubevirt.io/domain={vm_name}"),
        format!("vm.kubevirt.io/name={vm_name}"),
    ] {
        let lp = ListParams::default().labels(&label);
        if let Ok(list) = pods.list(&lp).await {
            if let Some(pod) = list.items.into_iter().find(|p| {
                p.status
                    .as_ref()
                    .and_then(|s| s.phase.as_deref())
                    == Some("Running")
            }) {
                return pod
                    .metadata
                    .name
                    .clone()
                    .ok_or_else(|| ApiError::internal("virt-launcher pod missing name"));
            }
        }
    }
    Err(ApiError::bad_request(format!(
        "No running virt-launcher pod for VM {namespace}/{vm_name}"
    )))
}

async fn virsh_qga_json(
    client: &Client,
    namespace: &str,
    pod: &str,
    domain: &str,
    body: Value,
) -> ApiResult<Value> {
    let json_cmd = serde_json::to_string(&body)
        .map_err(|e| ApiError::internal(format!("serialize QGA command: {e}")))?;
    let stdout = virsh_qga_raw(client, namespace, pod, domain, &json_cmd).await?;
    let trimmed = stdout.trim();
    if trimmed.is_empty() {
        return Err(ApiError::internal("empty response from qemu-agent-command"));
    }
    serde_json::from_str(trimmed)
        .map_err(|e| ApiError::internal(format!("parse QGA response {trimmed}: {e}")))
}

async fn virsh_qga_raw(
    client: &Client,
    namespace: &str,
    pod: &str,
    domain: &str,
    json_cmd: &str,
) -> ApiResult<String> {
    let pods: Api<Pod> = Api::namespaced(client.clone(), namespace);
    let ap = AttachParams {
        container: Some(VIRT_LAUNCHER_CONTAINER.into()),
        stdout: true,
        stderr: true,
        ..Default::default()
    };
    let cmd = vec![
        "virsh".to_string(),
        "--quiet".to_string(),
        "qemu-agent-command".to_string(),
        domain.to_string(),
        json_cmd.to_string(),
    ];
    let mut attached = pods
        .exec(pod, cmd, &ap)
        .await
        .map_err(|e| ApiError::internal(format!("exec virsh in {pod}: {e}")))?;
    let mut stdout = Vec::new();
    if let Some(mut out) = attached.stdout() {
        out.read_to_end(&mut stdout)
            .await
            .map_err(|e| ApiError::internal(format!("read qemu-agent stdout: {e}")))?;
    }
    let mut stderr = Vec::new();
    if let Some(mut err) = attached.stderr() {
        err.read_to_end(&mut stderr)
            .await
            .map_err(|e| ApiError::internal(format!("read qemu-agent stderr: {e}")))?;
    }
    if !stderr.is_empty() && stdout.is_empty() {
        return Err(ApiError::internal(format!(
            "qemu-agent-command failed: {}",
            String::from_utf8_lossy(&stderr)
        )));
    }
    Ok(String::from_utf8_lossy(&stdout).into_owned())
}

fn decode_b64_field(value: Option<&Value>) -> String {
    value
        .and_then(|v| v.as_str())
        .map(|s| {
            base64::engine::general_purpose::STANDARD
                .decode(s)
                .map(|b| String::from_utf8_lossy(&b).into_owned())
                .unwrap_or_else(|_| s.to_string())
        })
        .unwrap_or_default()
}
