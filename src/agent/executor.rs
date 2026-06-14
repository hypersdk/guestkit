// SPDX-License-Identifier: Apache-2.0
//! Privileged executor for policy-controlled remediation.

use crate::agent::policy::AgentPolicy;
use anyhow::{bail, Context, Result};
use guestkit_agent_protocol::{
    RemediationActionResult, RemediationResult,
};
use std::fs::OpenOptions;
use std::io::Write;
use std::process::Command;

const AUDIT_LOG: &str = "/var/log/zyvor/agent-audit.log";

pub struct Executor {
    policy: AgentPolicy,
}

impl Executor {
    pub fn new() -> Self {
        Self {
            policy: AgentPolicy::load(),
        }
    }

    pub fn restart_unit(&self, unit: &str) -> Result<String> {
        if !self.policy.can_restart_unit(unit) {
            bail!("restart_unit denied by policy for {unit}");
        }
        if crate::agent::executor_ipc::executor_available() {
            let result = crate::agent::executor_ipc::call_executor(
                "restart_unit",
                serde_json::json!({ "unit": unit }),
            )?;
            return result
                .as_str()
                .map(String::from)
                .ok_or_else(|| anyhow::anyhow!("unexpected executor response"));
        }
        #[cfg(target_os = "linux")]
        {
            let msg = crate::collectors::dbus::systemd1::restart_unit(unit)?;
            self.audit("restart_unit", unit, true, &msg);
            Ok(msg)
        }
        #[cfg(not(target_os = "linux"))]
        {
            bail!("restart_unit only supported on Linux")
        }
    }

    pub fn execute_remediation_plan(
        &self,
        plan_id: &str,
        actions: &[RemediationActionSpec],
    ) -> RemediationResult {
        if crate::agent::executor_ipc::executor_available() {
            if let Ok(result) = crate::agent::executor_ipc::call_executor(
                "execute_remediation_plan",
                serde_json::json!({
                    "plan_id": plan_id,
                    "actions": actions,
                }),
            ) {
                if let Ok(parsed) = serde_json::from_value::<RemediationResult>(result) {
                    return parsed;
                }
            }
        }
        let mut results = Vec::new();
        for action in actions {
            let result = match action.action.as_str() {
                "restart_unit" => self
                    .restart_unit(&action.target)
                    .map(|m| m)
                    .unwrap_or_else(|e| e.to_string()),
                "fix_dns" => self
                    .run_systemctl("restart", "systemd-resolved.service")
                    .unwrap_or_else(|e| e.to_string()),
                "rotate_journal" => Command::new("journalctl")
                    .args(["--vacuum-size=500M"])
                    .output()
                    .map(|_| "journal vacuumed".into())
                    .unwrap_or_else(|e| e.to_string()),
                other => format!("unsupported action: {other}"),
            };
            let success = !result.starts_with("denied") && !result.contains("unsupported");
            self.audit(&action.action, &action.target, success, &result);
            results.push(RemediationActionResult {
                action: action.action.clone(),
                success,
                message: result,
            });
        }
        let success = results.iter().all(|r| r.success);
        RemediationResult {
            plan_id: plan_id.to_string(),
            success,
            actions: results,
        }
    }

    pub fn collect_support_bundle(&self) -> Result<Vec<u8>> {
        if crate::agent::executor_ipc::executor_available() {
            let result = crate::agent::executor_ipc::call_executor(
                "collect_support_bundle",
                serde_json::json!({}),
            )?;
            use base64::{engine::general_purpose::STANDARD, Engine};
            if let Some(data) = result.get("data").and_then(|v| v.as_str()) {
                return Ok(STANDARD.decode(data).context("decode support bundle")?);
            }
        }
        crate::agent::support_bundle::build_support_bundle_bytes()
    }

    fn run_systemctl(&self, op: &str, unit: &str) -> Result<String> {
        let result = Command::new("systemctl")
            .arg(op)
            .arg(unit)
            .output()
            .map(|o| {
                if o.status.success() {
                    format!("{op} {unit} ok")
                } else {
                    String::from_utf8_lossy(&o.stderr).to_string()
                }
            })
            .map_err(|e| anyhow::anyhow!("{e}"));
        let success = result
            .as_ref()
            .map(|r| r.ends_with(" ok"))
            .unwrap_or(false);
        self.audit(
            &format!("systemctl_{op}"),
            unit,
            success,
            result.as_ref().map(String::as_str).unwrap_or(""),
        );
        result
    }

    fn audit(&self, action: &str, target: &str, success: bool, detail: &str) {
        let line = format!(
            "{} action={} target={} success={} detail={}\n",
            chrono::Utc::now().to_rfc3339(),
            action,
            target,
            success,
            detail.replace('\n', " ")
        );
        if let Ok(mut file) = OpenOptions::new().create(true).append(true).open(AUDIT_LOG) {
            let _ = file.write_all(line.as_bytes());
        }
    }
}

#[derive(Debug, Clone, serde::Deserialize, serde::Serialize)]
pub struct RemediationActionSpec {
    pub action: String,
    pub target: String,
}
