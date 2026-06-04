// SPDX-License-Identifier: LGPL-3.0-or-later
//! In-guest agent daemon and host-side proxy.

pub mod cli;
pub mod daemon;
pub mod handler;
pub mod inject;
pub mod proxy;
pub mod transport;

pub use cli::{run_agent, run_agent_proxy, AgentArgs, AgentProxyArgs};
pub use daemon::AgentDaemon;

/// Ping guest agent via libvirt channel unix socket.
pub fn ping_agent_socket(socket_path: &str) -> bool {
    use guestkit_agent_protocol::{read_frame, write_frame};
    use std::os::unix::net::UnixStream;

    let Ok(mut stream) = UnixStream::connect(socket_path) else {
        return false;
    };
    let req = br#"{"jsonrpc":"2.0","method":"guestkit.ping","id":1}"#;
    if write_frame(&mut stream, req).is_err() {
        return false;
    }
    let Ok(frame) = read_frame(&mut stream) else {
        return false;
    };
    serde_json::from_slice::<serde_json::Value>(&frame)
        .ok()
        .and_then(|v| v.get("result").cloned())
        .is_some()
}
