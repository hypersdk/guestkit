// SPDX-License-Identifier: Apache-2.0
//! JSON-RPC request dispatch for the in-guest agent.

use crate::VERSION;
use guestkit_agent_protocol::{
    AgentCapabilities, AgentError, JsonRpcRequest, JsonRpcResponse, RpcErrorCode, RpcMethod,
};
use serde_json::{json, Value};

pub struct RequestHandler {
    capabilities: AgentCapabilities,
}

impl Default for RequestHandler {
    fn default() -> Self {
        Self::new()
    }
}

impl RequestHandler {
    pub fn new() -> Self {
        Self {
            capabilities: AgentCapabilities::standard(VERSION),
        }
    }

    /// Dispatch QGA (`execute`) or GuestKit JSON-RPC (`method`) on one virtio channel.
    pub fn handle_frame(&self, bytes: &[u8]) -> Vec<u8> {
        if crate::agent::qga::is_qga_request(bytes) {
            return crate::agent::qga::handle(bytes);
        }
        serde_json::to_vec(&self.handle(bytes)).unwrap_or_else(|e| {
            serde_json::to_vec(&JsonRpcResponse::error(
                None,
                RpcErrorCode::InternalError,
                format!("serialize: {e}"),
            ))
            .unwrap_or_default()
        })
    }

    pub fn handle(&self, bytes: &[u8]) -> JsonRpcResponse {
        let req = match JsonRpcRequest::parse(bytes) {
            Ok(r) => r,
            Err(e) => return JsonRpcResponse::from_agent_error(None, e),
        };

        if let Err(e) = req.validate() {
            return JsonRpcResponse::from_agent_error(req.id, e);
        }

        match req.method() {
            RpcMethod::Ping => Self::ping(req.id),
            RpcMethod::GetVersion => Self::get_version(req.id),
            RpcMethod::GetCapabilities => self.get_capabilities(req.id),
            RpcMethod::GetEvidence => self.get_evidence(req.id),
            RpcMethod::GetStatus => self.get_status(req.id),
            RpcMethod::Doctor => self.doctor(req.id, &req.params),
            RpcMethod::MigrateScore => self.migrate_score(req.id, &req.params),
            RpcMethod::GetMetrics => self.get_metrics(req.id),
            RpcMethod::GetFilesystem => self.get_filesystem(req.id),
            RpcMethod::Exec => self.exec(req.id, &req.params),
            RpcMethod::EnableRdp => self.enable_rdp(req.id),
            RpcMethod::DisableRdp => self.disable_rdp(req.id),
            RpcMethod::RunFixPlan => self.run_fix_plan(req.id, &req.params),
            RpcMethod::RunFixPlanRollback => self.run_fix_plan_rollback(req.id, &req.params),
            RpcMethod::GetGuestHealth => self.get_guest_health(req.id),
            RpcMethod::GetSystemdUnits => self.get_systemd_units(req.id),
            RpcMethod::GetFailedUnits => self.get_failed_units(req.id),
            RpcMethod::GetBootAnalysis => self.get_boot_analysis(req.id),
            RpcMethod::GetJournalSlice => self.get_journal_slice(req.id, &req.params),
            RpcMethod::GetLoginState => self.get_login_state(req.id),
            RpcMethod::GetDnsState => self.get_dns_state(req.id),
            RpcMethod::GetTimedateState => self.get_timedate_state(req.id),
            RpcMethod::GetSnapshotReadiness => self.get_snapshot_readiness(req.id),
            RpcMethod::FreezeFilesystem => self.freeze_filesystem(req.id),
            RpcMethod::ThawFilesystem => self.thaw_filesystem(req.id),
            RpcMethod::RestartUnit => self.restart_unit(req.id, &req.params),
            RpcMethod::ExecuteRemediationPlan => {
                self.execute_remediation_plan(req.id, &req.params)
            }
            RpcMethod::CollectSupportBundle => self.collect_support_bundle(req.id),
            RpcMethod::GetGuestInfo => self.get_guest_info(req.id),
            RpcMethod::GetSystemdUnit => self.get_systemd_unit(req.id, &req.params),
            RpcMethod::GetSystemdEvents => self.get_systemd_events(req.id, &req.params),
            RpcMethod::GetProcesses => self.get_processes(req.id),
            RpcMethod::Unknown(name) => JsonRpcResponse::error(
                req.id,
                RpcErrorCode::MethodNotFound,
                format!("unknown method: {name}"),
            ),
        }
    }

    fn ping(id: Option<Value>) -> JsonRpcResponse {
        JsonRpcResponse::success(id, json!({ "pong": true }))
    }

    fn get_version(id: Option<Value>) -> JsonRpcResponse {
        JsonRpcResponse::success(
            id,
            json!({
                "version": VERSION,
                "protocol": guestkit_agent_protocol::PROTOCOL_VERSION,
            }),
        )
    }

    fn get_capabilities(&self, id: Option<Value>) -> JsonRpcResponse {
        JsonRpcResponse::success(
            id,
            serde_json::to_value(&self.capabilities).unwrap_or(json!({})),
        )
    }

    fn get_evidence(&self, id: Option<Value>) -> JsonRpcResponse {
        match crate::evidence::build_evidence_live() {
            Ok(evidence) => {
                let health = crate::health::build_guest_health(&evidence);
                match serde_json::to_value(evidence) {
                    Ok(mut v) => {
                        if let Some(obj) = v.as_object_mut() {
                            obj.insert(
                                "guest_health".to_string(),
                                serde_json::to_value(health).unwrap_or(json!({})),
                            );
                        }
                        JsonRpcResponse::success(id, v)
                    }
                    Err(e) => JsonRpcResponse::error(
                        id,
                        RpcErrorCode::InternalError,
                        format!("serialize evidence: {e}"),
                    ),
                }
            }
            Err(e) => JsonRpcResponse::error(
                id,
                RpcErrorCode::InternalError,
                format!("collect evidence: {e}"),
            ),
        }
    }

    fn get_status(&self, id: Option<Value>) -> JsonRpcResponse {
        match crate::evidence::build_agent_status_live() {
            Ok(status) => match serde_json::to_value(status) {
                Ok(v) => JsonRpcResponse::success(id, v),
                Err(e) => JsonRpcResponse::error(
                    id,
                    RpcErrorCode::InternalError,
                    format!("serialize status: {e}"),
                ),
            },
            Err(e) => JsonRpcResponse::error(
                id,
                RpcErrorCode::InternalError,
                format!("collect status: {e}"),
            ),
        }
    }

    fn doctor(&self, id: Option<Value>, params: &Value) -> JsonRpcResponse {
        let target = params
            .get("target")
            .and_then(|v| v.as_str())
            .unwrap_or("kvm");

        match crate::evidence::build_evidence_live() {
            Ok(evidence) => {
                let boot_target = crate::boot::BootTarget::parse(target);
                let boot_report = crate::boot::analyze_bootability(&evidence, boot_target);
                let semantic = crate::ai::semantic::analyze_semantic(&evidence);
                JsonRpcResponse::success(
                    id,
                    json!({
                        "evidence": evidence,
                        "boot_report": boot_report,
                        "semantic": semantic,
                    }),
                )
            }
            Err(e) => JsonRpcResponse::error(
                id,
                RpcErrorCode::InternalError,
                format!("doctor failed: {e}"),
            ),
        }
    }

    fn migrate_score(&self, id: Option<Value>, params: &Value) -> JsonRpcResponse {
        let target = params
            .get("target")
            .and_then(|v| v.as_str())
            .unwrap_or("kvm");

        match crate::evidence::build_evidence_live() {
            Ok(evidence) => {
                let boot_target = crate::boot::BootTarget::parse(target);
                let boot_report = crate::boot::analyze_bootability(&evidence, boot_target);
                let report = crate::cli::migrate::plan::compute_migration_score(
                    &evidence,
                    &boot_report,
                    target,
                );
                JsonRpcResponse::success(id, serde_json::to_value(report).unwrap_or(json!({})))
            }
            Err(e) => JsonRpcResponse::error(
                id,
                RpcErrorCode::InternalError,
                format!("migrate score failed: {e}"),
            ),
        }
    }

    fn get_metrics(&self, id: Option<Value>) -> JsonRpcResponse {
        let metrics = crate::metrics::collect_metrics_live();
        JsonRpcResponse::success(id, serde_json::to_value(metrics).unwrap_or(json!({})))
    }

    fn get_filesystem(&self, id: Option<Value>) -> JsonRpcResponse {
        match crate::agent::qga::filesystem_mounts_normalized() {
            Ok(v) => JsonRpcResponse::success(id, v),
            Err(e) => JsonRpcResponse::error(id, RpcErrorCode::InternalError, e),
        }
    }

    fn exec(&self, id: Option<Value>, params: &Value) -> JsonRpcResponse {
        let policy = crate::agent::policy::AgentPolicy::load();
        if !policy.shell_enabled() {
            return JsonRpcResponse::from_agent_error(
                id,
                AgentError::CapabilityDenied("shell exec disabled by policy".into()),
            );
        }
        match crate::agent::exec::exec_sync(params) {
            Ok(v) => JsonRpcResponse::success(id, v),
            Err(e) => JsonRpcResponse::error(id, RpcErrorCode::InternalError, e),
        }
    }

    fn enable_rdp(&self, id: Option<Value>) -> JsonRpcResponse {
        match crate::agent::rdp::enable_rdp() {
            Ok(v) => JsonRpcResponse::success(id, v),
            Err(e) => JsonRpcResponse::error(id, RpcErrorCode::InternalError, e),
        }
    }

    fn disable_rdp(&self, id: Option<Value>) -> JsonRpcResponse {
        match crate::agent::rdp::disable_rdp() {
            Ok(v) => JsonRpcResponse::success(id, v),
            Err(e) => JsonRpcResponse::error(id, RpcErrorCode::InternalError, e),
        }
    }

    fn run_fix_plan(&self, id: Option<Value>, params: &Value) -> JsonRpcResponse {
        if !self.capabilities.fix_apply {
            return JsonRpcResponse::from_agent_error(
                id,
                AgentError::CapabilityDenied("fix_apply not enabled".into()),
            );
        }

        let plan_value = match params.get("plan") {
            Some(v) => v.clone(),
            None => {
                return JsonRpcResponse::error(
                    id,
                    RpcErrorCode::InvalidParams,
                    "missing required param: plan",
                )
            }
        };

        let plan: crate::cli::plan::FixPlan = match serde_json::from_value(plan_value) {
            Ok(p) => p,
            Err(e) => {
                return JsonRpcResponse::error(
                    id,
                    RpcErrorCode::InvalidParams,
                    format!("invalid plan: {e}"),
                )
            }
        };

        let dry_run = params
            .get("dry_run")
            .and_then(|v| v.as_bool())
            .unwrap_or(false);

        let executor = crate::cli::plan::LivePlanExecutor::new(dry_run);
        match executor.apply(&plan) {
            Ok(result) => {
                let plan_id = format!(
                    "{}-{}",
                    plan.vm.replace('/', "_"),
                    plan.generated.timestamp()
                );
                JsonRpcResponse::success(
                    id,
                    json!({
                        "plan_id": plan_id,
                        "dry_run": dry_run,
                        "result": result,
                    }),
                )
            }
            Err(e) => JsonRpcResponse::error(
                id,
                RpcErrorCode::PlanApplyFailed,
                format!("apply failed: {e}"),
            ),
        }
    }

    fn run_fix_plan_rollback(&self, id: Option<Value>, params: &Value) -> JsonRpcResponse {
        let plan_id = match params.get("plan_id").and_then(|v| v.as_str()) {
            Some(id) => id,
            None => {
                return JsonRpcResponse::error(
                    id,
                    RpcErrorCode::InvalidParams,
                    "missing required param: plan_id",
                )
            }
        };

        let executor = crate::cli::plan::LivePlanExecutor::new(false);
        match executor.rollback(plan_id) {
            Ok(msg) => JsonRpcResponse::success(id, json!({ "message": msg })),
            Err(e) => JsonRpcResponse::error(
                id,
                RpcErrorCode::PlanApplyFailed,
                format!("rollback failed: {e}"),
            ),
        }
    }

    fn get_guest_health(&self, id: Option<Value>) -> JsonRpcResponse {
        match crate::health::build_guest_health_live() {
            Ok(health) => JsonRpcResponse::success(id, serde_json::to_value(health).unwrap_or(json!({}))),
            Err(e) => JsonRpcResponse::error(id, RpcErrorCode::InternalError, e.to_string()),
        }
    }

    fn get_systemd_units(&self, id: Option<Value>) -> JsonRpcResponse {
        match crate::evidence::build_evidence_live() {
            Ok(evidence) => {
                let units = evidence
                    .systemd
                    .as_ref()
                    .and_then(|s| s.runtime.as_ref())
                    .map(|r| r.units.clone())
                    .unwrap_or_default();
                JsonRpcResponse::success(id, serde_json::to_value(units).unwrap_or(json!([])))
            }
            Err(e) => JsonRpcResponse::error(id, RpcErrorCode::InternalError, e.to_string()),
        }
    }

    fn get_failed_units(&self, id: Option<Value>) -> JsonRpcResponse {
        match crate::evidence::build_evidence_live() {
            Ok(evidence) => {
                let failed = crate::health::list_failed_units_from_evidence(&evidence);
                JsonRpcResponse::success(id, serde_json::to_value(failed).unwrap_or(json!([])))
            }
            Err(e) => JsonRpcResponse::error(id, RpcErrorCode::InternalError, e.to_string()),
        }
    }

    fn get_boot_analysis(&self, id: Option<Value>) -> JsonRpcResponse {
        let analysis = crate::boot::live::collect_boot_analysis();
        JsonRpcResponse::success(id, serde_json::to_value(analysis).unwrap_or(json!({})))
    }

    fn get_journal_slice(&self, id: Option<Value>, params: &Value) -> JsonRpcResponse {
        let unit = params
            .get("unit")
            .and_then(|v| v.as_str())
            .unwrap_or("");
        let limit = params
            .get("limit")
            .and_then(|v| v.as_u64())
            .unwrap_or(200) as usize;
        let boot = params
            .get("boot")
            .and_then(|v| v.as_str())
            .unwrap_or("current");
        let slice = crate::journal::live::collect_journal_slice_boot(unit, limit, boot);
        JsonRpcResponse::success(id, serde_json::to_value(slice).unwrap_or(json!({})))
    }

    fn get_guest_info(&self, id: Option<Value>) -> JsonRpcResponse {
        match crate::evidence::build_evidence_live() {
            Ok(evidence) => {
                let info = crate::health::build_guest_info(&evidence);
                JsonRpcResponse::success(id, serde_json::to_value(info).unwrap_or(json!({})))
            }
            Err(e) => JsonRpcResponse::error(id, RpcErrorCode::InternalError, e.to_string()),
        }
    }

    fn get_systemd_unit(&self, id: Option<Value>, params: &Value) -> JsonRpcResponse {
        let unit = params
            .get("unit")
            .and_then(|v| v.as_str())
            .unwrap_or("");
        if unit.is_empty() {
            return JsonRpcResponse::error(id, RpcErrorCode::InvalidParams, "unit required");
        }
        match crate::evidence::build_evidence_live() {
            Ok(evidence) => {
                let detail = crate::health::build_service_health(unit, &evidence).or_else(|| {
                    #[cfg(target_os = "linux")]
                    {
                        crate::collectors::dbus::get_unit_detail(unit).map(|u| {
                            guestkit_agent_protocol::ServiceHealth {
                                name: u.name,
                                state: u.active_state,
                                sub_state: u.sub_state,
                                main_pid: u.main_pid,
                                exit_code: u.exec_main_status,
                                restart_count: u.n_restarts,
                                last_failure: None,
                                journal_cursor: None,
                                actions: vec!["view_logs".into(), "restart_unit".into()],
                            }
                        })
                    }
                    #[cfg(not(target_os = "linux"))]
                    {
                        None
                    }
                });
                JsonRpcResponse::success(id, serde_json::to_value(detail).unwrap_or(json!(null)))
            }
            Err(e) => JsonRpcResponse::error(id, RpcErrorCode::InternalError, e.to_string()),
        }
    }

    fn get_systemd_events(&self, id: Option<Value>, params: &Value) -> JsonRpcResponse {
        #[cfg(target_os = "linux")]
        {
            let cursor = params
                .get("cursor")
                .and_then(|v| v.as_u64())
                .unwrap_or(0);
            let limit = params
                .get("limit")
                .and_then(|v| v.as_u64())
                .unwrap_or(100) as usize;
            let (next_cursor, events) = if cursor > 0 {
                crate::collectors::dbus::systemd_events::get_events_since(cursor)
            } else {
                let events = crate::collectors::dbus::systemd_events::recent_events(limit);
                let (c, _) = crate::collectors::dbus::systemd_events::get_events_since(0);
                (c, events)
            };
            JsonRpcResponse::success(
                id,
                json!({
                    "cursor": next_cursor,
                    "events": events,
                }),
            )
        }
        #[cfg(not(target_os = "linux"))]
        {
            let _ = params;
            JsonRpcResponse::success(id, json!({ "cursor": 0, "events": [] }))
        }
    }

    fn get_processes(&self, id: Option<Value>) -> JsonRpcResponse {
        match crate::evidence::build_evidence_live() {
            Ok(evidence) => {
                let process = evidence
                    .process
                    .clone()
                    .unwrap_or_else(|| crate::collectors::process::collect_process_evidence());
                JsonRpcResponse::success(id, serde_json::to_value(process).unwrap_or(json!({})))
            }
            Err(e) => JsonRpcResponse::error(id, RpcErrorCode::InternalError, e.to_string()),
        }
    }

    fn get_login_state(&self, id: Option<Value>) -> JsonRpcResponse {
        let state = crate::collectors::dbus::collect_login_state_safe();
        JsonRpcResponse::success(id, serde_json::to_value(state).unwrap_or(json!({})))
    }

    fn get_dns_state(&self, id: Option<Value>) -> JsonRpcResponse {
        let dns = crate::collectors::dbus::collect_dns_health_safe();
        JsonRpcResponse::success(id, serde_json::to_value(dns).unwrap_or(json!({})))
    }

    fn get_timedate_state(&self, id: Option<Value>) -> JsonRpcResponse {
        let timedate = crate::collectors::dbus::collect_timedate_health_safe();
        JsonRpcResponse::success(id, serde_json::to_value(timedate).unwrap_or(json!({})))
    }

    fn get_snapshot_readiness(&self, id: Option<Value>) -> JsonRpcResponse {
        let report = crate::agent::snapshot_hooks::build_snapshot_readiness_report();
        JsonRpcResponse::success(id, serde_json::to_value(report).unwrap_or(json!({})))
    }

    fn freeze_filesystem(&self, id: Option<Value>) -> JsonRpcResponse {
        match crate::agent::snapshot_hooks::freeze_filesystems() {
            Ok(msg) => JsonRpcResponse::success(id, json!({ "message": msg })),
            Err(e) => JsonRpcResponse::error(id, RpcErrorCode::InternalError, e),
        }
    }

    fn thaw_filesystem(&self, id: Option<Value>) -> JsonRpcResponse {
        match crate::agent::snapshot_hooks::thaw_filesystems() {
            Ok(msg) => JsonRpcResponse::success(id, json!({ "message": msg })),
            Err(e) => JsonRpcResponse::error(id, RpcErrorCode::InternalError, e),
        }
    }

    fn restart_unit(&self, id: Option<Value>, params: &Value) -> JsonRpcResponse {
        let unit = match params.get("unit").and_then(|v| v.as_str()) {
            Some(u) => u,
            None => {
                return JsonRpcResponse::error(
                    id,
                    RpcErrorCode::InvalidParams,
                    "missing required param: unit",
                )
            }
        };
        let executor = crate::agent::executor::Executor::new();
        match executor.restart_unit(unit) {
            Ok(msg) => JsonRpcResponse::success(id, json!({ "message": msg })),
            Err(e) => JsonRpcResponse::error(id, RpcErrorCode::InternalError, e.to_string()),
        }
    }

    fn execute_remediation_plan(&self, id: Option<Value>, params: &Value) -> JsonRpcResponse {
        let plan_id = params
            .get("plan_id")
            .and_then(|v| v.as_str())
            .unwrap_or("local");
        let actions: Vec<crate::agent::executor::RemediationActionSpec> = params
            .get("actions")
            .and_then(|v| serde_json::from_value(v.clone()).ok())
            .unwrap_or_default();
        let executor = crate::agent::executor::Executor::new();
        let result = executor.execute_remediation_plan(plan_id, &actions);
        JsonRpcResponse::success(id, serde_json::to_value(result).unwrap_or(json!({})))
    }

    fn collect_support_bundle(&self, id: Option<Value>) -> JsonRpcResponse {
        let executor = crate::agent::executor::Executor::new();
        match executor.collect_support_bundle() {
            Ok(bytes) => {
                use base64::{engine::general_purpose::STANDARD, Engine};
                JsonRpcResponse::success(
                    id,
                    json!({
                        "format": "tar.zst",
                        "encoding": "base64",
                        "data": STANDARD.encode(bytes),
                    }),
                )
            }
            Err(e) => JsonRpcResponse::error(id, RpcErrorCode::InternalError, e.to_string()),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ping_round_trip() {
        let handler = RequestHandler::new();
        let resp = handler.handle(br#"{"jsonrpc":"2.0","method":"guestkit.ping","id":1}"#);
        assert!(resp.result.is_some());
        assert!(resp.error.is_none());
    }

    #[test]
    fn qga_guest_ping_round_trip() {
        let handler = RequestHandler::new();
        let raw = handler.handle_frame(br#"{"execute":"guest-ping"}"#);
        let v: serde_json::Value = serde_json::from_slice(&raw).unwrap();
        assert!(v.get("return").is_some());
    }
}
