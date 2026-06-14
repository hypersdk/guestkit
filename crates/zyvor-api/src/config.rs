// SPDX-License-Identifier: Apache-2.0

use anyhow::Result;
use std::path::PathBuf;

#[derive(Debug, Clone)]
pub struct Config {
    pub bind_addr: String,
    pub database_url: String,
    pub redis_url: String,
    pub storage_path: PathBuf,
    pub storage_public_url: String,
    pub default_namespace: String,
    pub storage_class: String,
    /// Maximum multipart upload size in bytes (default 2 GiB).
    pub max_upload_bytes: usize,
    /// Default guestkit agent-proxy URL for live (online) VM inspection.
    pub agent_proxy_url: Option<String>,
    pub zeus_public_url: Option<String>,
    pub cluster_name: Option<String>,
    /// Extra directories the UI may browse for on-server disks (comma-separated env).
    pub storage_browse_paths: Vec<PathBuf>,
    pub auth_enabled: bool,
    pub jwt_secret: String,
    pub jwt_issuer: String,
    pub jwt_expiry_hours: u32,
    pub public_base_url: String,
    pub ui_base_url: String,
    pub default_role: String,
    pub nfs_mount_root: PathBuf,
    /// When set, guest agent registration must present this bootstrap token.
    pub agent_bootstrap_token: Option<String>,
}

impl Config {
    pub fn storage_browse_roots(&self) -> Vec<PathBuf> {
        let mut roots = vec![self.storage_path.clone()];
        for path in &self.storage_browse_paths {
            if !roots.iter().any(|r| r == path) {
                roots.push(path.clone());
            }
        }
        roots
    }

    pub fn jwt_expiry_secs(&self) -> usize {
        (self.jwt_expiry_hours as usize) * 3600
    }

    pub fn default_role(&self) -> &str {
        &self.default_role
    }

    pub fn oidc_redirect_uri(&self) -> String {
        format!(
            "{}/api/v1/auth/oidc/callback",
            self.public_base_url.trim_end_matches('/')
        )
    }

    pub fn from_env() -> Result<Self> {
        let public_base_url = std::env::var("PUBLIC_BASE_URL")
            .or_else(|_| std::env::var("API_PUBLIC_URL"))
            .unwrap_or_else(|_| "http://localhost:8080".into());
        let ui_base_url = std::env::var("UI_BASE_URL")
            .unwrap_or_else(|_| "http://localhost:30081".into());
        let auth_enabled = std::env::var("AUTH_ENABLED")
            .ok()
            .map(|v| matches!(v.to_lowercase().as_str(), "1" | "true" | "yes" | "on"))
            .unwrap_or(false);
        let jwt_secret = std::env::var("JWT_SECRET").unwrap_or_else(|_| {
            if auth_enabled {
                "change-me-in-production".into()
            } else {
                "dev-local-auth-disabled".into()
            }
        });

        Ok(Self {
            bind_addr: std::env::var("BIND_ADDR").unwrap_or_else(|_| "0.0.0.0:8080".into()),
            database_url: std::env::var("DATABASE_URL")
                .unwrap_or_else(|_| "postgres://zyvor:zyvor@localhost:5432/zyvor".into()),
            redis_url: std::env::var("REDIS_URL")
                .or_else(|_| std::env::var("QUEUE_URL"))
                .unwrap_or_else(|_| "redis://127.0.0.1:6379".into()),
            storage_path: PathBuf::from(
                std::env::var("STORAGE_PATH").unwrap_or_else(|_| "/var/lib/zyvor/images".into()),
            ),
            storage_public_url: std::env::var("STORAGE_PUBLIC_URL")
                .unwrap_or_else(|_| "http://minio:9000/vm-images".into()),
            default_namespace: std::env::var("DEFAULT_NAMESPACE")
                .unwrap_or_else(|_| "zyvor".into()),
            storage_class: std::env::var("STORAGE_CLASS")
                .unwrap_or_else(|_| "longhorn".into()),
            max_upload_bytes: std::env::var("MAX_UPLOAD_BYTES")
                .ok()
                .and_then(|v| v.parse().ok())
                .unwrap_or(2 * 1024 * 1024 * 1024),
            agent_proxy_url: std::env::var("AGENT_PROXY_URL")
                .ok()
                .filter(|u| !u.trim().is_empty()),
            zeus_public_url: std::env::var("ZEUS_PUBLIC_URL")
                .ok()
                .filter(|u| !u.trim().is_empty()),
            cluster_name: std::env::var("CLUSTER_NAME")
                .ok()
                .filter(|u| !u.trim().is_empty()),
            storage_browse_paths: std::env::var("STORAGE_BROWSE_PATHS")
                .ok()
                .map(|raw| {
                    raw.split(',')
                        .map(|s| s.trim())
                        .filter(|s| !s.is_empty())
                        .map(PathBuf::from)
                        .collect()
                })
                .unwrap_or_default(),
            auth_enabled,
            jwt_secret,
            jwt_issuer: std::env::var("JWT_ISSUER").unwrap_or_else(|_| "guestkit".into()),
            jwt_expiry_hours: std::env::var("JWT_EXPIRY_HOURS")
                .ok()
                .and_then(|v| v.parse().ok())
                .unwrap_or(24),
            public_base_url,
            ui_base_url,
            default_role: std::env::var("DEFAULT_ROLE").unwrap_or_else(|_| "operator".into()),
            nfs_mount_root: PathBuf::from(
                std::env::var("NFS_MOUNT_ROOT").unwrap_or_else(|_| "/mnt/nfs".into()),
            ),
            agent_bootstrap_token: std::env::var("AGENT_BOOTSTRAP_TOKEN")
                .ok()
                .filter(|t| !t.trim().is_empty()),
        })
    }
}
