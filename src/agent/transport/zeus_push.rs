// SPDX-License-Identifier: Apache-2.0
//! Outbound mTLS push to Zeus control plane.

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::path::Path;
use std::time::Duration;

const DEFAULT_CONFIG: &str = "/etc/zyvor/guest-agent.toml";
const AGENT_STATE_PATH: &str = "/var/lib/zyvor/agent-state.json";

#[derive(Debug, Clone, Default, Deserialize, Serialize)]
struct AgentStateFile {
    agent_id: Option<String>,
}

#[derive(Debug, Clone, Default, Deserialize)]
pub struct ZeusPushConfig {
    pub zeus_url: Option<String>,
    pub agent_id: Option<String>,
    pub bootstrap_token: Option<String>,
    pub cert_path: Option<String>,
    pub key_path: Option<String>,
    pub ca_path: Option<String>,
    #[serde(default = "default_interval")]
    pub interval_secs: u64,
    #[serde(default)]
    pub namespace: Option<String>,
    #[serde(default)]
    pub vm_name: Option<String>,
}

fn default_interval() -> u64 {
    60
}

pub fn load_config() -> ZeusPushConfig {
    let path = std::env::var("ZYVOR_AGENT_CONFIG").unwrap_or_else(|_| DEFAULT_CONFIG.to_string());
    if Path::new(&path).exists() {
        std::fs::read_to_string(&path)
            .ok()
            .and_then(|s| toml::from_str(&s).ok())
            .unwrap_or_default()
    } else {
        ZeusPushConfig::default()
    }
}

#[derive(Debug, Serialize)]
struct RegisterRequest {
    hostname: String,
    agent_version: String,
    bootstrap_token: Option<String>,
    namespace: Option<String>,
    vm_name: Option<String>,
}

#[derive(Debug, Deserialize)]
struct RegisterResponse {
    agent_id: String,
}

pub async fn run_push_worker() -> Result<()> {
    let config = load_config();
    let zeus_url = config
        .zeus_url
        .clone()
        .filter(|u| !u.is_empty())
        .or_else(|| std::env::var("ZYVOR_ZEUS_URL").ok())
        .filter(|u| !u.is_empty());

    if zeus_url.is_none() {
        log::info!("Zeus push disabled (no zeus_url configured)");
        return Ok(());
    }

    let base = zeus_url.unwrap().trim_end_matches('/').to_string();
    let client = build_client(&config)?;

    if let Ok(info) = client
        .get(format!("{base}/api/v1/guest-agents/bootstrap-info"))
        .send()
        .await
    {
        if let Ok(body) = info.json::<serde_json::Value>().await {
            if body
                .pointer("/data/token_required")
                .and_then(|v| v.as_bool())
                .unwrap_or(false)
                && config.bootstrap_token.is_none()
            {
                log::warn!("Zeus requires AGENT_BOOTSTRAP_TOKEN but guest-agent.toml has no bootstrap_token");
            }
        }
    }

    let mut agent_id = config.agent_id.clone().or_else(load_persisted_agent_id);

    if agent_id.is_none() {
        let hostname = std::fs::read_to_string("/etc/hostname")
            .unwrap_or_default()
            .trim()
            .to_string();
        let body = RegisterRequest {
            hostname,
            agent_version: crate::VERSION.to_string(),
            bootstrap_token: config.bootstrap_token.clone(),
            namespace: config
                .namespace
                .clone()
                .or_else(|| std::env::var("ZYVOR_VM_NAMESPACE").ok()),
            vm_name: config
                .vm_name
                .clone()
                .or_else(|| std::env::var("ZYVOR_VM_NAME").ok()),
        };
        let url = format!("{base}/api/v1/guest-agents/register");
        if let Ok(resp) = client.post(&url).json(&body).send().await {
            if let Ok(reg) = resp.json::<RegisterResponse>().await {
                log::info!("registered with Zeus as agent {}", reg.agent_id);
                agent_id = Some(reg.agent_id.clone());
                persist_agent_id(&reg.agent_id);
            }
        }
    }

    let agent_id = agent_id.unwrap_or_else(|| "local".into());
    let interval = Duration::from_secs(config.interval_secs.max(15));

    loop {
        if let Err(e) = push_heartbeat(&client, &base, &agent_id).await {
            log::warn!("heartbeat push failed: {e}");
        }
        if let Err(e) = push_report(&client, &base, &agent_id).await {
            log::warn!("report push failed: {e}");
        }
        tokio::time::sleep(interval).await;
    }
}

async fn push_heartbeat(
    client: &reqwest::Client,
    base: &str,
    agent_id: &str,
) -> Result<()> {
    let url = format!("{base}/api/v1/guest-agents/{agent_id}/heartbeat");
    let status = crate::evidence::build_agent_status_live().unwrap_or_else(|_| {
        crate::evidence::AgentStatus {
            hostname: "unknown".into(),
            os_name: String::new(),
            os_version: String::new(),
            kernel: String::new(),
            architecture: String::new(),
            ips: vec![],
            boot_mode: String::new(),
            cloud_init_present: false,
            qga_ready: false,
            zyvor_agent_ready: true,
            virtio_modules_loaded: false,
            agent_version: crate::VERSION.to_string(),
            uptime_secs: 0,
            last_heartbeat: chrono::Utc::now().to_rfc3339(),
        }
    });
    let recent_events = recent_systemd_events(50);
    let body = serde_json::json!({
        "status": status,
        "recent_events": recent_events,
    });
    client
        .post(&url)
        .json(&body)
        .send()
        .await
        .context("heartbeat POST")?;
    Ok(())
}

async fn push_report(client: &reqwest::Client, base: &str, agent_id: &str) -> Result<()> {
    let evidence = crate::evidence::build_evidence_live()?;
    let health = crate::health::build_guest_health(&evidence);
    let metrics = crate::metrics::collect_metrics_live();
    let recent_events = recent_systemd_events(100);
    let body = serde_json::json!({
        "guest_health": health,
        "metrics": metrics,
        "recent_events": recent_events,
    });
    let url = format!("{base}/api/v1/guest-agents/{agent_id}/report");
    client.post(&url).json(&body).send().await.context("report POST")?;
    Ok(())
}

fn build_client(config: &ZeusPushConfig) -> Result<reqwest::Client> {
    let mut builder = reqwest::Client::builder().timeout(Duration::from_secs(30));

    if let (Some(cert), Some(key)) = (&config.cert_path, &config.key_path) {
        let identity = reqwest::Identity::from_pem(
            &[
                std::fs::read(cert).context("read cert")?,
                std::fs::read(key).context("read key")?,
            ]
            .concat(),
        )?;
        builder = builder.identity(identity);
    }

    if let Some(ca) = &config.ca_path {
        let pem = std::fs::read(ca).context("read ca")?;
        let cert = reqwest::Certificate::from_pem(&pem).context("parse ca pem")?;
        builder = builder.tls_built_in_root_certs(false).add_root_certificate(cert);
    }

    builder.build().context("build reqwest client")
}

fn load_persisted_agent_id() -> Option<String> {
    std::fs::read_to_string(AGENT_STATE_PATH)
        .ok()
        .and_then(|raw| serde_json::from_str::<AgentStateFile>(&raw).ok())
        .and_then(|s| s.agent_id)
        .filter(|id| !id.is_empty())
}

fn persist_agent_id(agent_id: &str) {
    let state = AgentStateFile {
        agent_id: Some(agent_id.to_string()),
    };
    if let Some(dir) = std::path::Path::new(AGENT_STATE_PATH).parent() {
        let _ = std::fs::create_dir_all(dir);
    }
    if let Ok(json) = serde_json::to_string_pretty(&state) {
        let _ = std::fs::write(AGENT_STATE_PATH, json);
    }
}

fn recent_systemd_events(limit: usize) -> Vec<guestkit_agent_protocol::SystemdEvent> {
    #[cfg(target_os = "linux")]
    {
        crate::collectors::dbus::systemd_events::recent_events(limit)
    }
    #[cfg(not(target_os = "linux"))]
    {
        let _ = limit;
        Vec::new()
    }
}
