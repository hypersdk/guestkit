// SPDX-License-Identifier: LGPL-3.0-or-later
//! QEMU guest-agent protocol compatibility (`virsh qemu-agent-command` / libvirt).

use crate::VERSION;
use serde_json::{json, Value};
use std::collections::HashMap;
use std::process::{Command, Stdio};
use std::sync::{LazyLock, Mutex};
use std::time::{SystemTime, UNIX_EPOCH};

static EXEC_JOBS: LazyLock<Mutex<HashMap<u64, ExecJob>>> =
    LazyLock::new(|| Mutex::new(HashMap::new()));
static NEXT_PID: LazyLock<Mutex<u64>> = LazyLock::new(|| Mutex::new(1000));

#[derive(Debug)]
struct ExecJob {
    out: Vec<u8>,
    err: Vec<u8>,
    exited: bool,
    exitcode: i64,
}

/// True if payload is a QGA `{"execute":...}` command (not JSON-RPC).
pub fn is_qga_request(bytes: &[u8]) -> bool {
    let Ok(v) = serde_json::from_slice::<Value>(bytes) else {
        return false;
    };
    v.get("execute").and_then(|e| e.as_str()).is_some()
}

/// Handle one QGA request frame; returns full JSON response bytes.
pub fn handle(bytes: &[u8]) -> Vec<u8> {
    let req: Value = match serde_json::from_slice(bytes) {
        Ok(v) => v,
        Err(e) => return qga_error(None, &format!("invalid JSON: {e}")),
    };
    let execute = match req.get("execute").and_then(|e| e.as_str()) {
        Some(c) => c,
        None => return qga_error(req.get("id").cloned(), "missing execute"),
    };
    let args = req.get("arguments").cloned().unwrap_or(json!({}));
    let id = req.get("id").cloned();

    let result = match execute {
        "guest-ping" => Ok(json!({})),
        "guest-info" => Ok(guest_info()),
        "guest-get-osinfo" => guest_get_osinfo(),
        "guest-get-fsinfo" => guest_get_fsinfo(),
        "guest-get-users" => guest_get_users(),
        "guest-get-timezone" => guest_get_timezone(),
        "guest-get-time" => guest_get_time(),
        "guest-set-time" => guest_set_time(&args),
        "guest-fstrim" => guest_fstrim(),
        "guest-fsfreeze-status" => Ok(json!({ "frozen": 0 })),
        "guest-exec" => guest_exec(&args),
        "guest-exec-status" => guest_exec_status(&args),
        "guest-shutdown" => guest_shutdown(&args),
        "guest-reboot" => guest_reboot(),
        "guest-suspend-disk" | "guest-suspend-ram" | "guest-suspend-hybrid" => {
            Ok(json!({}))
        }
        other => Err(format!("unsupported QGA command: {other}")),
    };

    match result {
        Ok(ret) => qga_ok(id, ret),
        Err(msg) => qga_error(id, &msg),
    }
}

fn qga_ok(id: Option<Value>, ret: Value) -> Vec<u8> {
    let mut out = json!({ "return": ret });
    if let Some(i) = id {
        out["id"] = i;
    }
    serde_json::to_vec(&out).unwrap_or_else(|_| br#"{"return":{}}"#.to_vec())
}

fn qga_error(id: Option<Value>, desc: &str) -> Vec<u8> {
    let mut out = json!({
        "error": {
            "class": "GenericError",
            "desc": desc
        }
    });
    if let Some(i) = id {
        out["id"] = i;
    }
    serde_json::to_vec(&out).unwrap_or_else(|_| br#"{"error":{"class":"GenericError","desc":"error"}}"#.to_vec())
}

fn guest_info() -> Value {
    json!({
        "version": format!("guestkit-{VERSION}"),
        "supported_commands": [
            "guest-ping",
            "guest-info",
            "guest-get-osinfo",
            "guest-get-fsinfo",
            "guest-get-users",
            "guest-get-timezone",
            "guest-get-time",
            "guest-set-time",
            "guest-fstrim",
            "guest-fsfreeze-status",
            "guest-exec",
            "guest-exec-status",
            "guest-shutdown",
            "guest-reboot"
        ],
        "supported_events": []
    })
}

fn guest_get_osinfo() -> Result<Value, String> {
    let mut id = String::new();
    let mut name = String::new();
    let mut pretty = String::new();
    let mut version = String::new();
    if let Ok(content) = std::fs::read_to_string("/etc/os-release") {
        for line in content.lines() {
            let Some((k, v)) = line.split_once('=') else {
                continue;
            };
            let v = v.trim_matches('"');
            match k {
                "ID" => id = v.to_string(),
                "NAME" => name = v.to_string(),
                "PRETTY_NAME" => pretty = v.to_string(),
                "VERSION_ID" => version = v.to_string(),
                _ => {}
            }
        }
    }
    let kernel = Command::new("uname")
        .arg("-r")
        .output()
        .ok()
        .map(|o| String::from_utf8_lossy(&o.stdout).trim().to_string())
        .unwrap_or_default();
    let machine = Command::new("uname")
        .arg("-m")
        .output()
        .ok()
        .map(|o| String::from_utf8_lossy(&o.stdout).trim().to_string())
        .unwrap_or_default();
    Ok(json!({
        "id": id,
        "name": name,
        "pretty-name": if pretty.is_empty() { &name } else { &pretty },
        "version": version,
        "kernel-release": kernel,
        "machine": machine
    }))
}

fn guest_get_fsinfo() -> Result<Value, String> {
    let mut mounts = Vec::new();
    let Ok(content) = std::fs::read_to_string("/proc/mounts") else {
        return Ok(json!([]));
    };
    for line in content.lines() {
        let parts: Vec<&str> = line.split_whitespace().collect();
        if parts.len() < 3 {
            continue;
        }
        let mountpoint = parts[1];
        if mountpoint == "/proc"
            || mountpoint == "/sys"
            || mountpoint == "/dev"
            || mountpoint.starts_with("/proc/")
            || mountpoint.starts_with("/sys/")
        {
            continue;
        }
        let fstype = parts[2];
        if matches!(fstype, "proc" | "sysfs" | "devtmpfs" | "tmpfs" | "cgroup2" | "cgroup") {
            continue;
        }
        let (total_bytes, used_bytes) = statvfs_usage(mountpoint);
        mounts.push(json!({
            "name": parts[0],
            "mountpoint": mountpoint,
            "type": fstype,
            "total-bytes": total_bytes,
            "used-bytes": used_bytes,
            "disk": [{
                "bus-type": "virtio",
                "serial": "",
                "dev": parts[0],
                "total-bytes": total_bytes,
                "used-bytes": used_bytes
            }]
        }));
    }
    Ok(json!(mounts))
}

#[cfg(target_os = "linux")]
fn statvfs_usage(path: &str) -> (u64, u64) {
    use nix::sys::statvfs::statvfs;
    use std::path::Path;
    let Ok(st) = statvfs(Path::new(path)) else {
        return (0, 0);
    };
    let total = st.blocks() as u64 * st.fragment_size() as u64;
    let free = st.blocks_free() as u64 * st.fragment_size() as u64;
    (total, total.saturating_sub(free))
}

#[cfg(not(target_os = "linux"))]
fn statvfs_usage(_path: &str) -> (u64, u64) {
    (0, 0)
}

fn guest_get_users() -> Result<Value, String> {
    let out = Command::new("who")
        .output()
        .map_err(|e| format!("who: {e}"))?;
    let text = String::from_utf8_lossy(&out.stdout);
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0);
    let users: Vec<Value> = text
        .lines()
        .filter_map(|line| {
            let mut parts = line.split_whitespace();
            let user = parts.next()?;
            let _tty = parts.next()?;
            let _time = parts.next()?;
            Some(json!({
                "user": user,
                "login-time": now,
                "domain": ""
            }))
        })
        .collect();
    Ok(json!(users))
}

fn guest_get_timezone() -> Result<Value, String> {
    let tz = std::fs::read_link("/etc/localtime")
        .ok()
        .and_then(|p| {
            p.to_string_lossy()
                .strip_prefix("/usr/share/zoneinfo/")
                .map(|s| s.to_string())
        })
        .or_else(|| {
            std::fs::read_to_string("/etc/timezone")
                .ok()
                .map(|s| s.trim().to_string())
        })
        .unwrap_or_else(|| "UTC".into());
    Ok(json!({ "timezone": tz }))
}

fn guest_get_time() -> Result<Value, String> {
    let secs = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_err(|e| e.to_string())?
        .as_secs();
    Ok(json!({ "time": secs }))
}

fn guest_set_time(args: &Value) -> Result<Value, String> {
    let secs = args
        .get("time")
        .and_then(|v| v.as_u64())
        .ok_or_else(|| "missing time".to_string())?;
    let nanos = args.get("nanoseconds").and_then(|v| v.as_u64()).unwrap_or(0);
    let cmd = format!(
        "date -u --set=@{}.{:09}",
        secs,
        nanos % 1_000_000_000
    );
    let status = Command::new("sh").arg("-c").arg(&cmd).status();
    match status {
        Ok(s) if s.success() => Ok(json!({})),
        Ok(s) => Err(format!("date failed: {s}")),
        Err(e) => Err(format!("set time: {e}")),
    }
}

fn guest_fstrim() -> Result<Value, String> {
    let out = Command::new("fstrim")
        .arg("-a")
        .arg("-v")
        .output()
        .map_err(|e| format!("fstrim: {e}"))?;
    let text = String::from_utf8_lossy(&out.stdout);
    let mut results = Vec::new();
    for line in text.lines() {
        if line.contains(':') {
            let mount = line.split(':').next().unwrap_or("/").trim();
            results.push(json!({
                "path": mount,
                "minimum": 0,
                "error": if out.status.success() { 0 } else { 1 }
            }));
        }
    }
    if results.is_empty() && out.status.success() {
        results.push(json!({ "path": "/", "minimum": 0, "error": 0 }));
    }
    Ok(json!(results))
}

fn guest_exec(args: &Value) -> Result<Value, String> {
    let path = args
        .get("path")
        .and_then(|v| v.as_str())
        .ok_or_else(|| "missing path".to_string())?;
    let capture = args
        .get("capture-output")
        .and_then(|v| v.as_bool())
        .unwrap_or(false);
    let mut cmd = Command::new(path);
    if let Some(arr) = args.get("arg").and_then(|v| v.as_array()) {
        for a in arr {
            if let Some(s) = a.as_str() {
                cmd.arg(s);
            }
        }
    }
    if capture {
        cmd.stdout(Stdio::piped()).stderr(Stdio::piped());
    }
    let mut pid_guard = NEXT_PID.lock().map_err(|e| e.to_string())?;
    let pid = *pid_guard;
    *pid_guard += 1;
    drop(pid_guard);

    if capture {
        let output = cmd.output().map_err(|e| format!("exec: {e}"))?;
        let job = ExecJob {
            out: output.stdout,
            err: output.stderr,
            exited: true,
            exitcode: output.status.code().unwrap_or(1) as i64,
        };
        EXEC_JOBS.lock().map_err(|e| e.to_string())?.insert(pid, job);
    } else {
        cmd.spawn().map_err(|e| format!("spawn: {e}"))?;
        EXEC_JOBS.lock().map_err(|e| e.to_string())?.insert(
            pid,
            ExecJob {
                out: Vec::new(),
                err: Vec::new(),
                exited: true,
                exitcode: 0,
            },
        );
    }
    Ok(json!({ "pid": pid }))
}

fn guest_exec_status(args: &Value) -> Result<Value, String> {
    let pid = args
        .get("pid")
        .and_then(|v| v.as_u64())
        .ok_or_else(|| "missing pid".to_string())?;
    let jobs = EXEC_JOBS.lock().map_err(|e| e.to_string())?;
    let job = jobs
        .get(&pid)
        .ok_or_else(|| format!("unknown pid {pid}"))?;
    use base64::{engine::general_purpose::STANDARD, Engine};
    let mut ret = json!({
        "exited": job.exited,
        "exitcode": job.exitcode,
    });
    if !job.out.is_empty() {
        ret["out-data"] = Value::String(STANDARD.encode(&job.out));
    }
    if !job.err.is_empty() {
        ret["err-data"] = Value::String(STANDARD.encode(&job.err));
    }
    Ok(ret)
}

fn guest_shutdown(args: &Value) -> Result<Value, String> {
    let mode = args
        .get("mode")
        .and_then(|v| v.as_str())
        .unwrap_or("powerdown");
    let cmd = match mode {
        "halt" => "systemctl halt",
        "powerdown" | _ => "systemctl poweroff",
    };
    Command::new("sh")
        .arg("-c")
        .arg(cmd)
        .spawn()
        .map_err(|e| format!("shutdown: {e}"))?;
    Ok(json!({}))
}

fn guest_reboot() -> Result<Value, String> {
    Command::new("systemctl")
        .args(["reboot"])
        .spawn()
        .or_else(|_| Command::new("reboot").spawn())
        .map_err(|e| format!("reboot: {e}"))?;
    Ok(json!({}))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn guest_ping_round_trip() {
        let resp = handle(br#"{"execute":"guest-ping"}"#);
        let v: Value = serde_json::from_slice(&resp).unwrap();
        assert!(v.get("return").is_some());
    }

    #[test]
    fn detects_qga_vs_jsonrpc() {
        assert!(is_qga_request(br#"{"execute":"guest-ping"}"#));
        assert!(!is_qga_request(br#"{"jsonrpc":"2.0","method":"guestkit.ping","id":1}"#));
    }
}
