// SPDX-License-Identifier: LGPL-3.0-or-later
//! GuestKit agent JSON-RPC protocol (v1).
//!
//! Framing: 4-byte big-endian length prefix + UTF-8 JSON body.

pub mod capabilities;
pub mod error;
pub mod frame;
pub mod rpc;

pub use capabilities::{
    AgentCapabilities, PROTOCOL_VERSION, VIRTIO_CHANNEL_NAME, VIRTIO_DEVICE_PATH,
};
pub use error::{AgentError, RpcErrorCode};
pub use frame::{read_frame, write_frame};
pub use rpc::{JsonRpcRequest, JsonRpcResponse, RpcMethod};
