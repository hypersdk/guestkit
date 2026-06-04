// SPDX-License-Identifier: LGPL-3.0-or-later
//! JSON-RPC 2.0 message types.

use crate::error::{AgentError, RpcErrorCode};
use serde::{Deserialize, Serialize};
use serde_json::Value;

/// Parsed RPC method identifier.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RpcMethod {
    Ping,
    GetVersion,
    GetCapabilities,
    GetEvidence,
    Doctor,
    MigrateScore,
    RunFixPlan,
    RunFixPlanRollback,
    Unknown(String),
}

impl RpcMethod {
    pub fn parse(name: &str) -> Self {
        use crate::capabilities::*;
        match name {
            METHOD_PING => Self::Ping,
            METHOD_GET_VERSION => Self::GetVersion,
            METHOD_GET_CAPABILITIES => Self::GetCapabilities,
            METHOD_GET_EVIDENCE => Self::GetEvidence,
            METHOD_DOCTOR => Self::Doctor,
            METHOD_MIGRATE_SCORE => Self::MigrateScore,
            METHOD_RUN_FIX_PLAN => Self::RunFixPlan,
            METHOD_RUN_FIX_PLAN_ROLLBACK => Self::RunFixPlanRollback,
            other => Self::Unknown(other.to_string()),
        }
    }
}

#[derive(Debug, Clone, Deserialize)]
pub struct JsonRpcRequest {
    pub jsonrpc: String,
    pub method: String,
    #[serde(default)]
    pub params: Value,
    pub id: Option<Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JsonRpcErrorObject {
    pub code: i32,
    pub message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub data: Option<Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JsonRpcResponse {
    pub jsonrpc: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub result: Option<Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<JsonRpcErrorObject>,
    pub id: Option<Value>,
}

impl JsonRpcResponse {
    pub fn success(id: Option<Value>, result: Value) -> Self {
        Self {
            jsonrpc: "2.0".to_string(),
            result: Some(result),
            error: None,
            id,
        }
    }

    pub fn error(id: Option<Value>, code: RpcErrorCode, message: impl Into<String>) -> Self {
        Self {
            jsonrpc: "2.0".to_string(),
            result: None,
            error: Some(JsonRpcErrorObject {
                code: code.as_i32(),
                message: message.into(),
                data: None,
            }),
            id,
        }
    }

    pub fn from_agent_error(id: Option<Value>, err: AgentError) -> Self {
        Self::error(id, err.rpc_code(), err.message())
    }
}

impl JsonRpcRequest {
    pub fn parse(bytes: &[u8]) -> Result<Self, AgentError> {
        serde_json::from_slice(bytes).map_err(|e| AgentError::Parse(e.to_string()))
    }

    pub fn validate(&self) -> Result<(), AgentError> {
        if self.jsonrpc != "2.0" {
            return Err(AgentError::InvalidRequest(format!(
                "unsupported jsonrpc version: {}",
                self.jsonrpc
            )));
        }
        if self.method.is_empty() {
            return Err(AgentError::InvalidRequest("missing method".into()));
        }
        Ok(())
    }

    pub fn method(&self) -> RpcMethod {
        RpcMethod::parse(&self.method)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_ping_request() {
        let req =
            JsonRpcRequest::parse(br#"{"jsonrpc":"2.0","method":"guestkit.ping","id":1}"#).unwrap();
        assert_eq!(req.method(), RpcMethod::Ping);
    }
}
