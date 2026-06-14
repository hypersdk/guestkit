// SPDX-License-Identifier: Apache-2.0
//! Channel update check with SHA256-verified staging for privileged apply.

use anyhow::{Context, Result};
use serde::Deserialize;
use sha2::{Digest, Sha256};
use std::fs;
use std::path::{Path, PathBuf};

#[cfg(unix)]
use std::os::unix::fs::PermissionsExt;

const STAGED_AGENT: &str = "/var/lib/zyvor/staged/zyvor-guest-agent";
const STAGED_META: &str = "/var/lib/zyvor/staged/update.json";

#[derive(Debug, Clone, Deserialize)]
struct ApiEnvelope<T> {
    data: T,
}

#[derive(Debug, Clone, Deserialize)]
struct BundleInfo {
    version: String,
    channel: String,
    linux_tar_url: Option<String>,
    linux_tar_sha256: Option<String>,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct StagedUpdateMeta {
    pub version: String,
    pub channel: String,
    pub artifact_url: String,
    pub sha256: String,
    pub staged_at: String,
}

#[derive(Debug, Clone)]
pub struct UpdateCheckResult {
    pub current_version: String,
    pub remote_version: Option<String>,
    pub channel: String,
    pub update_available: bool,
    pub artifact_url: Option<String>,
    pub artifact_sha256: Option<String>,
}

pub async fn check_update() -> Result<UpdateCheckResult> {
    let current = crate::VERSION.to_string();
    let bundle = fetch_bundle_info().await?;
    let update_available = version_greater(&bundle.version, &current);
    Ok(UpdateCheckResult {
        current_version: current,
        remote_version: Some(bundle.version.clone()),
        channel: bundle.channel,
        update_available,
        artifact_url: bundle.linux_tar_url,
        artifact_sha256: bundle.linux_tar_sha256,
    })
}

pub async fn stage_update(apply: bool) -> Result<String> {
    let check = check_update().await?;
    if !check.update_available {
        return Ok(format!(
            "Zyvor GuestAgent {} is current (channel {})",
            check.current_version, check.channel
        ));
    }

    let url = check
        .artifact_url
        .as_ref()
        .filter(|u| !u.is_empty())
        .context("bundle has no linux tar URL")?;
    let expected_sha = check
        .artifact_sha256
        .as_ref()
        .filter(|s| !s.is_empty())
        .context("bundle has no linux tar sha256; refusing unsigned download")?;

    let bytes = download_bytes(url).await?;
    let actual_sha = hex_sha256(&bytes);
    if !actual_sha.eq_ignore_ascii_case(expected_sha) {
        anyhow::bail!(
            "artifact sha256 mismatch (expected {expected_sha}, got {actual_sha})"
        );
    }

    let staged_dir = Path::new(STAGED_AGENT).parent().unwrap_or(Path::new("/var/lib/zyvor/staged"));
    fs::create_dir_all(staged_dir).context("create staged dir")?;
    extract_agent_binary(&bytes, Path::new(STAGED_AGENT))?;

    let meta = StagedUpdateMeta {
        version: check.remote_version.clone().unwrap_or_default(),
        channel: check.channel.clone(),
        artifact_url: url.clone(),
        sha256: actual_sha,
        staged_at: chrono::Utc::now().to_rfc3339(),
    };
    fs::write(STAGED_META, serde_json::to_string_pretty(&meta)?)?;

    if apply {
        if crate::agent::executor_ipc::executor_available() {
            let result = crate::agent::executor_ipc::call_executor(
                "apply_staged_update",
                serde_json::json!({}),
            )?;
            return Ok(result
                .as_str()
                .map(String::from)
                .unwrap_or_else(|| "staged update applied via executor".into()));
        }
        return Ok(format!(
            "staged {} to {STAGED_AGENT}; executor unavailable — run zyvor-guest-agent-exec to apply",
            meta.version
        ));
    }

    Ok(format!(
        "staged {} (sha256 verified) at {STAGED_AGENT}",
        meta.version
    ))
}

async fn fetch_bundle_info() -> Result<BundleInfo> {
    let base = resolve_bundle_base()?;
    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(30))
        .build()?;
    let url = format!("{base}/api/v1/vmtools/bundle");
    let resp = client.get(&url).send().await.context("fetch bundle")?;
    if !resp.status().is_success() {
        anyhow::bail!("bundle HTTP {}", resp.status());
    }
    let envelope = resp
        .json::<ApiEnvelope<BundleInfo>>()
        .await
        .context("parse bundle response")?;
    Ok(envelope.data)
}

fn resolve_bundle_base() -> Result<String> {
    if let Ok(url) = std::env::var("ZYVOR_UPDATE_URL") {
        if !url.trim().is_empty() {
            return Ok(url.trim_end_matches('/').to_string());
        }
    }
    let config = crate::agent::transport::zeus_push::load_config();
    if let Some(url) = config.zeus_url.filter(|u| !u.is_empty()) {
        return Ok(url.trim_end_matches('/').to_string());
    }
  if let Ok(url) = std::env::var("ZYVOR_ZEUS_URL") {
        if !url.trim().is_empty() {
            return Ok(url.trim_end_matches('/').to_string());
        }
    }
    anyhow::bail!("no ZYVOR_UPDATE_URL or zeus_url configured for update check")
}

async fn download_bytes(url: &str) -> Result<Vec<u8>> {
    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(120))
        .build()?;
    let resp = client.get(url).send().await.context("download artifact")?;
    if !resp.status().is_success() {
        anyhow::bail!("artifact HTTP {}", resp.status());
    }
    resp.bytes().await.context("read artifact bytes").map(|b| b.to_vec())
}

fn extract_agent_binary(tar_gz: &[u8], dest: &Path) -> Result<()> {
    use std::io::Cursor;
    let gz = flate2::read::GzDecoder::new(Cursor::new(tar_gz));
    let mut archive = tar::Archive::new(gz);
    let staged_dir = dest
        .parent()
        .unwrap_or(Path::new("/var/lib/zyvor/staged"));
    let temp_dir = staged_dir.join(format!(
        "extract-{}",
        uuid::Uuid::new_v4().to_string().split('-').next().unwrap_or("tmp")
    ));
    fs::create_dir_all(&temp_dir).context("create extract dir")?;
    archive.unpack(&temp_dir).context("unpack tar.gz")?;

    let candidate = find_file_recursive(&temp_dir, "zyvor-guest-agent")
        .context("zyvor-guest-agent binary not found in artifact")?;
    fs::copy(&candidate, dest).with_context(|| format!("copy staged binary to {}", dest.display()))?;
    fs::remove_dir_all(&temp_dir).ok();
    #[cfg(unix)]
    fs::set_permissions(dest, fs::Permissions::from_mode(0o755))?;
    Ok(())
}

fn find_file_recursive(dir: &Path, name: &str) -> Result<PathBuf> {
    for entry in fs::read_dir(dir)? {
        let entry = entry?;
        let path = entry.path();
        if path.is_dir() {
            if let Ok(found) = find_file_recursive(&path, name) {
                return Ok(found);
            }
        } else if path.file_name().and_then(|n| n.to_str()) == Some(name) {
            return Ok(path);
        }
    }
    anyhow::bail!("file {name} not found under {}", dir.display())
}

fn hex_sha256(bytes: &[u8]) -> String {
    let digest = Sha256::digest(bytes);
    digest.iter().map(|b| format!("{b:02x}")).collect()
}

fn version_greater(remote: &str, current: &str) -> bool {
    parse_version(remote) > parse_version(current)
}

fn parse_version(raw: &str) -> Vec<u64> {
    raw.split('.')
        .map(|part| part.parse::<u64>().unwrap_or(0))
        .collect()
}

pub fn apply_staged_update_privileged() -> Result<String> {
    let meta_raw = fs::read_to_string(STAGED_META).context("read staged update metadata")?;
    let meta: StagedUpdateMeta =
        serde_json::from_str(&meta_raw).context("parse staged update metadata")?;
    if !Path::new(STAGED_AGENT).exists() {
        anyhow::bail!("staged binary missing at {STAGED_AGENT}");
    }
    let bytes = fs::read(STAGED_AGENT).context("read staged binary")?;
    let actual_sha = hex_sha256(&bytes);
    if !actual_sha.eq_ignore_ascii_case(&meta.sha256) {
        anyhow::bail!("staged binary sha256 mismatch");
    }

    let targets = [
        "/usr/bin/zyvor-guest-agent",
        "/usr/local/bin/zyvor-guest-agent",
    ];
    let mut updated = Vec::new();
    for target in targets {
        if Path::new(target).parent().map(|p| p.exists()).unwrap_or(false) {
            fs::copy(STAGED_AGENT, target)
                .with_context(|| format!("install staged binary to {target}"))?;
            fs::set_permissions(target, fs::Permissions::from_mode(0o755))?;
            updated.push(target.to_string());
        }
    }
    if updated.is_empty() {
        anyhow::bail!("no install target found for staged agent binary");
    }

    Ok(format!(
        "applied {} to {} (sha256 verified)",
        meta.version,
        updated.join(", ")
    ))
}
