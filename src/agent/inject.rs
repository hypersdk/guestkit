// SPDX-License-Identifier: Apache-2.0
//! Offline agent injection into disk images via guestfs.

use crate::cli::plan::types::{
    CommandExec, FileCopy, FixPlan, Operation, OperationType, PostApplyAction, Priority,
};
use anyhow::{Context, Result};
use std::fs;
use std::path::{Path, PathBuf};

pub const GUEST_BINARY_DEST: &str = "/usr/bin/zyvor-guest-agent";
pub const GUEST_UNIT_DEST: &str = "/etc/systemd/system/zyvor-guest-agent.service";
const DEFAULT_UNIT: &str = include_str!("../../templates/agent/zyvor-guest-agent.service");

const KUBEVIRT_CHANNEL_HINT: &str = r#"Add virtio-serial channel (QGA-compatible) to VMI/libvirt domain:
spec:
  domain:
    devices:
      channels:
      - name: org.qemu.guest_agent.0
        target:
          type: virtio
          name: org.qemu.guest_agent.0
Guest device: /dev/virtio-ports/org.qemu.guest_agent.0"#;

/// Append agent install operations to a fix plan (for export / preview).
pub fn append_agent_ops(plan: &mut FixPlan, binary: &Path, unit_content: &str) -> Result<()> {
    plan.metadata.tags.push("inject-agent".to_string());
    let op_base = plan.operations.len() + 1;

    plan.operations.push(Operation {
        id: format!("agent-{op_base:03}"),
        op_type: OperationType::FileCopy(FileCopy {
            source: binary.display().to_string(),
            destination: GUEST_BINARY_DEST.to_string(),
            backup: false,
        }),
        priority: Priority::Medium,
        description: "Install guestkit binary for in-guest agent".to_string(),
        risk: Priority::Low,
        reversible: true,
        depends_on: vec![],
        validation: None,
        undo: None,
    });

    let unit_staging =
        std::env::temp_dir().join(format!("guestkit-agent-{}.service", std::process::id()));
    fs::write(&unit_staging, unit_content)?;

    plan.operations.push(Operation {
        id: format!("agent-{:03}", op_base + 1),
        op_type: OperationType::FileCopy(FileCopy {
            source: unit_staging.display().to_string(),
            destination: GUEST_UNIT_DEST.to_string(),
            backup: false,
        }),
        priority: Priority::Medium,
        description: "Install guestkit-agent systemd unit".to_string(),
        risk: Priority::Low,
        reversible: true,
        depends_on: vec![format!("agent-{op_base:03}")],
        validation: None,
        undo: None,
    });

    plan.operations.push(Operation {
        id: format!("agent-{:03}", op_base + 2),
        op_type: OperationType::CommandExec(CommandExec {
            command: "chmod 0755 /usr/local/bin/guestkit".to_string(),
            expected_exit: 0,
            timeout: Some(30),
        }),
        priority: Priority::Low,
        description: "Make guestkit binary executable".to_string(),
        risk: Priority::Low,
        reversible: false,
        depends_on: vec![format!("agent-{op_base:03}")],
        validation: None,
        undo: None,
    });

    plan.operations.push(Operation {
        id: format!("agent-{:03}", op_base + 3),
        op_type: OperationType::CommandExec(CommandExec {
            command: "systemctl enable guestkit-agent || true".to_string(),
            expected_exit: 0,
            timeout: Some(60),
        }),
        priority: Priority::Medium,
        description: "Enable guestkit-agent on boot".to_string(),
        risk: Priority::Low,
        reversible: true,
        depends_on: vec![format!("agent-{:03}", op_base + 1)],
        validation: None,
        undo: None,
    });

    plan.post_apply.push(PostApplyAction::Message {
        message: KUBEVIRT_CHANNEL_HINT.to_string(),
    });

    Ok(())
}

/// Inject agent binary and systemd unit into an offline disk image.
pub fn inject_agent_into_image(
    image: &Path,
    binary: &Path,
    unit_content: &str,
    dry_run: bool,
    verbose: bool,
) -> Result<()> {
    if !binary.exists() {
        anyhow::bail!("agent binary not found: {}", binary.display());
    }
    if dry_run {
        println!("Dry run — would inject agent:");
        println!("  binary: {} → {}", binary.display(), GUEST_BINARY_DEST);
        println!("  unit → {}", GUEST_UNIT_DEST);
        return Ok(());
    }

    if verbose {
        println!("Injecting GuestKit agent into {}", image.display());
    }

    let mut g = crate::Guestfs::new().context("create guestfs")?;
    g.add_drive(
        image
            .to_str()
            .ok_or_else(|| anyhow::anyhow!("invalid image path"))?,
    )?;
    g.launch()?;

    let roots = g.inspect_os()?;
    if roots.is_empty() {
        anyhow::bail!("no operating system found in disk image");
    }
    let root = &roots[0];
    if let Ok(mountpoints) = g.inspect_get_mountpoints(root) {
        let mut mounts: Vec<_> = mountpoints.into_iter().collect();
        mounts.sort_by_key(|(m, _)| m.len());
        for (mount, device) in &mounts {
            let _ = g.mount(device, mount);
        }
    }

    g.upload(
        binary
            .to_str()
            .ok_or_else(|| anyhow::anyhow!("binary path"))?,
        GUEST_BINARY_DEST,
    )?;
    g.command(&["chmod", "0755", GUEST_BINARY_DEST])?;

    let unit_tmp = tempfile::NamedTempFile::new()?;
    fs::write(unit_tmp.path(), unit_content)?;
    g.upload(
        unit_tmp
            .path()
            .to_str()
            .ok_or_else(|| anyhow::anyhow!("temp path"))?,
        GUEST_UNIT_DEST,
    )?;

    let _ = g.command(&["systemctl", "enable", "guestkit-agent"]);
    let _ = g.umount_all();
    g.shutdown()?;

    if verbose {
        println!("Agent injected successfully");
    }
    Ok(())
}

/// Resolve agent binary path (explicit or current executable).
pub fn resolve_agent_binary(explicit: Option<&Path>) -> Result<PathBuf> {
    if let Some(p) = explicit {
        return Ok(p.to_path_buf());
    }
    std::env::current_exe().context("resolve guestkit binary for injection")
}

/// Load systemd unit content from path or built-in template.
pub fn resolve_agent_unit(explicit: Option<&Path>) -> Result<String> {
    if let Some(p) = explicit {
        return fs::read_to_string(p).with_context(|| format!("read unit file {}", p.display()));
    }
    Ok(DEFAULT_UNIT.to_string())
}
