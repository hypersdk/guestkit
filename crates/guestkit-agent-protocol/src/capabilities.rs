// SPDX-License-Identifier: Apache-2.0
//! Agent capability negotiation.

use serde::{Deserialize, Serialize};

/// Protocol version string.
pub const PROTOCOL_VERSION: &str = "1.0";

/// Virtio-serial channel name — same as QEMU guest agent (libvirt `qemu-agent-command`).
pub const VIRTIO_CHANNEL_NAME: &str = "org.qemu.guest_agent.0";

/// Legacy GuestKit-only channel (deprecated; use [`VIRTIO_CHANNEL_NAME`]).
pub const VIRTIO_CHANNEL_LEGACY: &str = "com.zyvor.guestkit.0";

/// Default guest device path for the virtio channel.
pub const VIRTIO_DEVICE_PATH: &str = "/dev/virtio-ports/org.qemu.guest_agent.0";

/// Known RPC methods exposed by the agent.
pub const METHOD_PING: &str = "guestkit.ping";
pub const METHOD_GET_VERSION: &str = "guestkit.getVersion";
pub const METHOD_GET_CAPABILITIES: &str = "guestkit.getCapabilities";
pub const METHOD_GET_EVIDENCE: &str = "guestkit.getEvidence";
pub const METHOD_DOCTOR: &str = "guestkit.doctor";
pub const METHOD_MIGRATE_SCORE: &str = "guestkit.migrateScore";
pub const METHOD_RUN_FIX_PLAN: &str = "guestkit.runFixPlan";
pub const METHOD_RUN_FIX_PLAN_ROLLBACK: &str = "guestkit.runFixPlanRollback";
pub const METHOD_GET_METRICS: &str = "guestkit.getMetrics";
pub const METHOD_GET_FILESYSTEM: &str = "guestkit.getFilesystem";
pub const METHOD_EXEC: &str = "guestkit.exec";
pub const METHOD_ENABLE_RDP: &str = "guestkit.enableRdp";
pub const METHOD_DISABLE_RDP: &str = "guestkit.disableRdp";

/// Capability flags returned during negotiation.
#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq, Eq)]
pub struct AgentCapabilities {
    pub protocol_version: String,
    pub agent_version: String,
    pub platform: String,
    pub methods: Vec<String>,
    pub fix_apply: bool,
    pub windows: bool,
}

impl AgentCapabilities {
    pub fn standard(agent_version: &str) -> Self {
        Self {
            protocol_version: PROTOCOL_VERSION.to_string(),
            agent_version: agent_version.to_string(),
            platform: std::env::consts::OS.to_string(),
            methods: vec![
                METHOD_PING.to_string(),
                METHOD_GET_VERSION.to_string(),
                METHOD_GET_CAPABILITIES.to_string(),
                METHOD_GET_EVIDENCE.to_string(),
                METHOD_DOCTOR.to_string(),
                METHOD_MIGRATE_SCORE.to_string(),
                METHOD_GET_METRICS.to_string(),
                METHOD_GET_FILESYSTEM.to_string(),
                METHOD_EXEC.to_string(),
                METHOD_ENABLE_RDP.to_string(),
                METHOD_DISABLE_RDP.to_string(),
                METHOD_RUN_FIX_PLAN.to_string(),
                METHOD_RUN_FIX_PLAN_ROLLBACK.to_string(),
            ],
            fix_apply: true,
            windows: cfg!(target_os = "windows"),
        }
    }
}
