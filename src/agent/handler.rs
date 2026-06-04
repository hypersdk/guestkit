// SPDX-License-Identifier: LGPL-3.0-or-later
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
            RpcMethod::Doctor => self.doctor(req.id, &req.params),
            RpcMethod::MigrateScore => self.migrate_score(req.id, &req.params),
            RpcMethod::RunFixPlan => self.run_fix_plan(req.id, &req.params),
            RpcMethod::RunFixPlanRollback => self.run_fix_plan_rollback(req.id, &req.params),
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
            Ok(evidence) => match serde_json::to_value(evidence) {
                Ok(v) => JsonRpcResponse::success(id, v),
                Err(e) => JsonRpcResponse::error(
                    id,
                    RpcErrorCode::InternalError,
                    format!("serialize evidence: {e}"),
                ),
            },
            Err(e) => JsonRpcResponse::error(
                id,
                RpcErrorCode::InternalError,
                format!("collect evidence: {e}"),
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
                JsonRpcResponse::success(
                    id,
                    json!({
                        "evidence": evidence,
                        "boot_report": boot_report,
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
