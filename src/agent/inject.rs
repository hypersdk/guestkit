// SPDX-License-Identifier: Apache-2.0
//! Offline agent injection into disk images via guestfs.

use crate::cli::plan::types::{
    CommandExec, FileCopy, FixPlan, Operation, OperationType, PostApplyAction, Priority,
};
use anyhow::{Context, Result};
use std::fs;
use std::path::{Path, PathBuf};

pub const GUEST_BINARY_DEST: &str = "/usr/local/bin/zyvor-guest-agent";
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
        std::env::temp_dir().join(format!("zyvor-guest-agent-{}.service", std::process::id()));
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
            interpreter: None,
            command: "chmod 0755 /usr/local/bin/zyvor-guest-agent".to_string(),
            expected_exit: 0,
            timeout: Some(30),
        }),
        priority: Priority::Low,
        description: "Make zyvor-guest-agent binary executable".to_string(),
        risk: Priority::Low,
        reversible: false,
        depends_on: vec![format!("agent-{op_base:03}")],
        validation: None,
        undo: None,
    });

    plan.operations.push(Operation {
        id: format!("agent-{:03}", op_base + 3),
        op_type: OperationType::CommandExec(CommandExec {
            interpreter: None,
            command: "systemctl enable zyvor-guest-agent || true".to_string(),
            expected_exit: 0,
            timeout: Some(60),
        }),
        priority: Priority::Medium,
        description: "Enable zyvor-guest-agent on boot".to_string(),
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

    let _ = g.command(&["systemctl", "enable", "zyvor-guest-agent"]);
    let _ = g.umount_all();
    g.shutdown()?;

    if verbose {
        println!("Agent injected successfully");
    }
    Ok(())
}

/// Windows install directory for the agent (guest path, forward-slash form).
pub const WIN_AGENT_DIR: &str = "/guestkit";
/// Windows agent binary destination (guest path).
pub const WIN_AGENT_DEST: &str = "/guestkit/guestkitd.exe";
/// Windows service name registered for the agent (matches the service crate).
pub const WIN_SERVICE_NAME: &str = "GuestKitAgent";

/// Offline-install the GuestKit agent into a **Windows** disk image, entirely
/// through guestkit's own machinery: copy `guestkitd.exe` into `C:\guestkit\`
/// and register the `GuestKitAgent` Windows service (auto-start, LocalSystem,
/// `--service`) by writing the offline `SYSTEM` hive with guestkit's hivex
/// registry-write. The injected service answers the QEMU guest-agent
/// virtio-serial channel at boot, so the host can drive it via
/// `virsh qemu-agent-command … guestkit-rpc` exactly like the Linux agent.
///
/// This replaces hand-rolled `virt-win-reg` service registration and, per the
/// standing rule, keeps offline Windows provisioning inside guestkit (which
/// also exercises the hivex path on real disks).
#[cfg(feature = "registry-write")]
pub fn inject_windows_agent(
    image: &Path,
    binary: &Path,
    dry_run: bool,
    verbose: bool,
) -> Result<()> {
    use serde_json::json;

    if !binary.exists() {
        anyhow::bail!("agent binary not found: {}", binary.display());
    }
    if dry_run {
        println!("Dry run — would inject Windows agent:");
        println!("  binary: {} → C:\\guestkit\\guestkitd.exe", binary.display());
        println!(
            "  register service {WIN_SERVICE_NAME} (auto-start, LocalSystem, --service)"
        );
        return Ok(());
    }

    if verbose {
        println!("Injecting GuestKit Windows agent into {}", image.display());
    }

    let mut g = crate::Guestfs::new().context("create guestfs")?;
    g.add_drive(
        image
            .to_str()
            .ok_or_else(|| anyhow::anyhow!("invalid image path"))?,
    )?;
    g.launch()?;

    let roots = g.inspect_os()?;
    let root = roots
        .first()
        .cloned()
        .ok_or_else(|| anyhow::anyhow!("no operating system found in disk image"))?;
    if g.inspect_get_type(&root)? != "windows" {
        anyhow::bail!(
            "inject_windows_agent: image root {root} is not Windows (use inject_agent_into_image)"
        );
    }
    if let Ok(mountpoints) = g.inspect_get_mountpoints(&root) {
        let mut mounts: Vec<_> = mountpoints.into_iter().collect();
        mounts.sort_by_key(|(m, _)| m.len());
        for (mount, device) in &mounts {
            let _ = g.mount(device, mount);
        }
    }

    // 1. Copy the agent binary into C:\guestkit\.
    let _ = g.mkdir_p(WIN_AGENT_DIR);
    g.upload(
        binary
            .to_str()
            .ok_or_else(|| anyhow::anyhow!("binary path"))?,
        WIN_AGENT_DEST,
    )?;
    if verbose {
        println!("  copied agent → C:\\guestkit\\guestkitd.exe");
    }

    // 2. Register the service by writing the offline SYSTEM hive with hivex.
    let hive_guest_path = g.inspect_get_windows_system_hive(&root)?;
    let hive_tmp = tempfile::NamedTempFile::new()?;
    let hive_host = hive_tmp
        .path()
        .to_str()
        .ok_or_else(|| anyhow::anyhow!("temp hive path"))?;
    g.download_hive(&hive_guest_path, hive_host)?;

    // ControlSet001 is the on-disk control set; the running system aliases it as
    // CurrentControlSet. Registering there makes the service present at boot.
    let subpath: Vec<String> = ["ControlSet001", "Services", WIN_SERVICE_NAME]
        .iter()
        .map(|s| s.to_string())
        .collect();
    // SERVICE_WIN32_OWN_PROCESS (0x10), SERVICE_AUTO_START (2), errorctl NORMAL (1).
    let values: &[(&str, &str, serde_json::Value)] = &[
        ("Type", "dword", json!(0x10)),
        ("Start", "dword", json!(2)),
        ("ErrorControl", "dword", json!(1)),
        (
            "ImagePath",
            "expand_sz",
            json!("C:\\guestkit\\guestkitd.exe --service"),
        ),
        ("DisplayName", "sz", json!("Zyvor GuestKit Agent")),
        ("ObjectName", "sz", json!("LocalSystem")),
        (
            "Description",
            "sz",
            json!("GuestKit in-guest agent (QGA virtio-serial channel)."),
        ),
    ];
    for (name, ty, data) in values {
        crate::guestfs::hivex_ffi::set_registry_value(
            hive_tmp.path(),
            &subpath,
            name,
            ty,
            data,
        )
        .map_err(|e| anyhow::anyhow!("set {name}: {e}"))?;
    }
    g.upload_hive(hive_host, &hive_guest_path)?;
    if verbose {
        println!("  registered service {WIN_SERVICE_NAME} in SYSTEM hive");
    }

    let _ = g.umount_all();
    g.shutdown()?;

    if verbose {
        println!(
            "Windows agent injected; the {WIN_SERVICE_NAME} service answers the QGA virtio channel at boot."
        );
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
