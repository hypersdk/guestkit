// SPDX-License-Identifier: Apache-2.0
//! Azure Blob disk source.

use super::local::LocalDiskSource;
use super::uri::{DiskSource, DiskSourceMetadata};
use anyhow::{Context, Result};
use std::io::{Read, Seek, SeekFrom};
use std::process::Command;
use tempfile::NamedTempFile;

pub struct AzureDiskSource {
    local: LocalDiskSource,
    uri: String,
}

impl AzureDiskSource {
    pub fn open(uri: &str) -> Result<Self> {
        let tmp = NamedTempFile::new().context("Failed to create temp file")?;
        let tmp_path = tmp.path().to_path_buf();
        tmp.close()?;

        let status = Command::new("az")
            .args([
                "storage",
                "blob",
                "download",
                "--blob-url",
                uri,
                "--file",
                tmp_path.to_str().unwrap(),
            ])
            .status()
            .context("Failed to run az storage blob download — install Azure CLI")?;

        if !status.success() {
            anyhow::bail!("Azure blob download failed for {}", uri);
        }

        Ok(Self {
            local: LocalDiskSource::open(&tmp_path)?,
            uri: uri.to_string(),
        })
    }
}

impl Read for AzureDiskSource {
    fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
        self.local.read(buf)
    }
}

impl Seek for AzureDiskSource {
    fn seek(&mut self, pos: SeekFrom) -> std::io::Result<u64> {
        self.local.seek(pos)
    }
}

impl DiskSource for AzureDiskSource {
    fn metadata(&self) -> DiskSourceMetadata {
        DiskSourceMetadata {
            uri: self.uri.clone(),
            size_bytes: self.local.metadata().size_bytes,
            backend: "azure".to_string(),
        }
    }

    fn local_path(&self) -> Option<&std::path::Path> {
        self.local.local_path()
    }
}

impl Drop for AzureDiskSource {
    fn drop(&mut self) {
        if let Some(p) = self.local.local_path() {
            let _ = std::fs::remove_file(p);
        }
    }
}
