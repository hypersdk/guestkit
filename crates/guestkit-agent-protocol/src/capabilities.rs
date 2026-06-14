// SPDX-License-Identifier: Apache-2.0
//! Agent capability negotiation.

use serde::{Deserialize, Serialize};

/// Protocol version string.
pub const PROTOCOL_VERSION: &str = "1.2";

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
pub const METHOD_GET_STATUS: &str = "guestkit.getStatus";
pub const METHOD_GET_METRICS: &str = "guestkit.getMetrics";
pub const METHOD_GET_FILESYSTEM: &str = "guestkit.getFilesystem";
pub const METHOD_EXEC: &str = "guestkit.exec";
pub const METHOD_ENABLE_RDP: &str = "guestkit.enableRdp";
pub const METHOD_DISABLE_RDP: &str = "guestkit.disableRdp";
pub const METHOD_GET_GUEST_HEALTH: &str = "guestkit.getGuestHealth";
pub const METHOD_GET_SYSTEMD_UNITS: &str = "guestkit.getSystemdUnits";
pub const METHOD_GET_FAILED_UNITS: &str = "guestkit.getFailedUnits";
pub const METHOD_GET_BOOT_ANALYSIS: &str = "guestkit.getBootAnalysis";
pub const METHOD_GET_JOURNAL_SLICE: &str = "guestkit.getJournalSlice";
pub const METHOD_GET_LOGIN_STATE: &str = "guestkit.getLoginState";
pub const METHOD_GET_DNS_STATE: &str = "guestkit.getDnsState";
pub const METHOD_GET_TIMEDATE_STATE: &str = "guestkit.getTimedateState";
pub const METHOD_GET_SNAPSHOT_READINESS: &str = "guestkit.getSnapshotReadiness";
pub const METHOD_FREEZE_FILESYSTEM: &str = "guestkit.freezeFilesystem";
pub const METHOD_THAW_FILESYSTEM: &str = "guestkit.thawFilesystem";
pub const METHOD_RESTART_UNIT: &str = "guestkit.restartUnit";
pub const METHOD_EXECUTE_REMEDIATION_PLAN: &str = "guestkit.executeRemediationPlan";
pub const METHOD_COLLECT_SUPPORT_BUNDLE: &str = "guestkit.collectSupportBundle";
pub const METHOD_GET_GUEST_INFO: &str = "guestkit.getGuestInfo";
pub const METHOD_GET_SYSTEMD_UNIT: &str = "guestkit.getSystemdUnit";
pub const METHOD_GET_SYSTEMD_EVENTS: &str = "guestkit.getSystemdEvents";
pub const METHOD_GET_PROCESSES: &str = "guestkit.getProcesses";

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
                METHOD_GET_STATUS.to_string(),
                METHOD_DOCTOR.to_string(),
                METHOD_MIGRATE_SCORE.to_string(),
                METHOD_GET_METRICS.to_string(),
                METHOD_GET_FILESYSTEM.to_string(),
                METHOD_EXEC.to_string(),
                METHOD_ENABLE_RDP.to_string(),
                METHOD_DISABLE_RDP.to_string(),
                METHOD_GET_GUEST_HEALTH.to_string(),
                METHOD_GET_SYSTEMD_UNITS.to_string(),
                METHOD_GET_FAILED_UNITS.to_string(),
                METHOD_GET_BOOT_ANALYSIS.to_string(),
                METHOD_GET_JOURNAL_SLICE.to_string(),
                METHOD_GET_LOGIN_STATE.to_string(),
                METHOD_GET_DNS_STATE.to_string(),
                METHOD_GET_TIMEDATE_STATE.to_string(),
                METHOD_GET_SNAPSHOT_READINESS.to_string(),
                METHOD_FREEZE_FILESYSTEM.to_string(),
                METHOD_THAW_FILESYSTEM.to_string(),
                METHOD_RESTART_UNIT.to_string(),
                METHOD_EXECUTE_REMEDIATION_PLAN.to_string(),
                METHOD_COLLECT_SUPPORT_BUNDLE.to_string(),
                METHOD_GET_GUEST_INFO.to_string(),
                METHOD_GET_SYSTEMD_UNIT.to_string(),
                METHOD_GET_SYSTEMD_EVENTS.to_string(),
                METHOD_GET_PROCESSES.to_string(),
                METHOD_RUN_FIX_PLAN.to_string(),
                METHOD_RUN_FIX_PLAN_ROLLBACK.to_string(),
            ],
            fix_apply: true,
            windows: cfg!(target_os = "windows"),
        }
    }
}
