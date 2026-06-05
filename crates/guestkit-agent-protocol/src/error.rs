// SPDX-License-Identifier: Apache-2.0
//! Protocol error types.

use serde::{Deserialize, Serialize};
use thiserror::Error;

/// JSON-RPC error codes (application-specific range -32000..-32099).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RpcErrorCode {
    ParseError = -32700,
    InvalidRequest = -32600,
    MethodNotFound = -32601,
    InvalidParams = -32602,
    InternalError = -32603,
    NotImplemented = -32000,
    CapabilityDenied = -32001,
    PlanApplyFailed = -32002,
}

impl RpcErrorCode {
    pub fn as_i32(self) -> i32 {
        self as i32
    }
}

#[derive(Debug, Error)]
pub enum AgentError {
    #[error("JSON parse error: {0}")]
    Parse(String),

    #[error("invalid request: {0}")]
    InvalidRequest(String),

    #[error("method not found: {0}")]
    MethodNotFound(String),

    #[error("invalid params: {0}")]
    InvalidParams(String),

    #[error("internal error: {0}")]
    Internal(String),

    #[error("not implemented: {0}")]
    NotImplemented(String),

    #[error("capability denied: {0}")]
    CapabilityDenied(String),

    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),
}

impl AgentError {
    pub fn rpc_code(&self) -> RpcErrorCode {
        match self {
            Self::Parse(_) => RpcErrorCode::ParseError,
            Self::InvalidRequest(_) => RpcErrorCode::InvalidRequest,
            Self::MethodNotFound(_) => RpcErrorCode::MethodNotFound,
            Self::InvalidParams(_) => RpcErrorCode::InvalidParams,
            Self::Internal(_) => RpcErrorCode::InternalError,
            Self::NotImplemented(_) => RpcErrorCode::NotImplemented,
            Self::CapabilityDenied(_) => RpcErrorCode::CapabilityDenied,
            Self::Io(e) => {
                if e.kind() == std::io::ErrorKind::UnexpectedEof {
                    RpcErrorCode::ParseError
                } else {
                    RpcErrorCode::InternalError
                }
            }
        }
    }

    pub fn message(&self) -> String {
        self.to_string()
    }
}
