// SPDX-License-Identifier: Apache-2.0
//! GCS disk source.

use super::local::LocalDiskSource;
use super::uri::{DiskSource, DiskSourceMetadata};
use anyhow::{Context, Result};
use std::io::{Read, Seek, SeekFrom};
use std::process::Command;
use tempfile::NamedTempFile;

pub struct GcsDiskSource {
    local: LocalDiskSource,
    uri: String,
}

impl GcsDiskSource {
    pub fn open(uri: &str) -> Result<Self> {
        let tmp = NamedTempFile::new().context("Failed to create temp file")?;
        let tmp_path = tmp.path().to_path_buf();
        tmp.close()?;

        let status = Command::new("gsutil")
            .args(["cp", uri, tmp_path.to_str().unwrap()])
            .status()
            .context("Failed to run gsutil cp — install Google Cloud SDK")?;

        if !status.success() {
            anyhow::bail!("gsutil cp failed for {}", uri);
        }

        Ok(Self {
            local: LocalDiskSource::open(&tmp_path)?,
            uri: uri.to_string(),
        })
    }
}

impl Read for GcsDiskSource {
    fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
        self.local.read(buf)
    }
}

impl Seek for GcsDiskSource {
    fn seek(&mut self, pos: SeekFrom) -> std::io::Result<u64> {
        self.local.seek(pos)
    }
}

impl DiskSource for GcsDiskSource {
    fn metadata(&self) -> DiskSourceMetadata {
        DiskSourceMetadata {
            uri: self.uri.clone(),
            size_bytes: self.local.metadata().size_bytes,
            backend: "gcs".to_string(),
        }
    }

    fn local_path(&self) -> Option<&std::path::Path> {
        self.local.local_path()
    }
}

impl Drop for GcsDiskSource {
    fn drop(&mut self) {
        if let Some(p) = self.local.local_path() {
            let _ = std::fs::remove_file(p);
        }
    }
}
