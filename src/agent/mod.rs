// SPDX-License-Identifier: Apache-2.0
//! In-guest agent daemon and host-side proxy.

#[cfg(unix)]
pub mod agent_call;
pub mod audit;
pub mod certificates;
pub mod cli;
pub mod containers;
pub mod customization;
pub mod inventory_cache;
pub mod daemon;
pub mod exec;
pub mod executor;
pub mod executor_ipc;
pub mod file_ops;
pub mod handler;
pub mod heartbeat;
pub mod inject;
pub mod integrity;
#[cfg(unix)]
pub mod local_client;
pub mod netintel;
pub mod nettest;
pub mod packages;
pub mod policy;
pub mod posture;
pub mod users;
pub mod state;
pub mod storage_ops;
pub mod telemetry;
pub mod proxy;
pub mod qga;
pub mod rdp;
pub mod snapshot;
pub mod snapshot_hooks;
pub mod support_bundle;
pub mod transport;
pub mod update_sign;
pub mod updater;

pub use agent_call::call_agent_socket;
pub use cli::{
    run_agent, run_agent_call, run_agent_proxy, AgentArgs, AgentCallArgs, AgentChannel,
    AgentProxyArgs,
};
pub use daemon::AgentDaemon;

/// Ping guest agent via libvirt channel unix socket.
pub fn ping_agent_socket(socket_path: &str) -> bool {
    use guestkit_agent_protocol::{read_frame, write_frame};
    use std::os::unix::net::UnixStream;

    let Ok(mut stream) = UnixStream::connect(socket_path) else {
        return false;
    };
    let req = br#"{"execute":"guest-ping"}"#;
    if write_frame(&mut stream, req).is_err() {
        return false;
    }
    let Ok(frame) = read_frame(&mut stream) else {
        return false;
    };
    serde_json::from_slice::<serde_json::Value>(&frame)
        .ok()
        .and_then(|v| v.get("return").cloned())
        .is_some()
}
