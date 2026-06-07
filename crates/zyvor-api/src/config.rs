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
}

impl Config {
    pub fn from_env() -> Result<Self> {
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
        })
    }
}
