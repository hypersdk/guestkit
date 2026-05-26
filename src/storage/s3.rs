// SPDX-License-Identifier: LGPL-3.0-or-later
//! S3 disk source (downloads to temp file via AWS CLI).

use super::local::LocalDiskSource;
use super::uri::{DiskSource, DiskSourceMetadata};
use anyhow::{Context, Result};
use std::io::{Read, Seek, SeekFrom};
use std::path::PathBuf;
use std::process::Command;
use tempfile::NamedTempFile;

pub struct S3DiskSource {
    local: LocalDiskSource,
    uri: String,
}

impl S3DiskSource {
    pub fn open(uri: &str) -> Result<Self> {
        let path = uri.strip_prefix("s3://").context("Invalid S3 URI")?;
        let tmp = NamedTempFile::new().context("Failed to create temp file")?;
        let tmp_path = tmp.path().to_path_buf();
        tmp.close()?;

        let status = Command::new("aws")
            .args(["s3", "cp", &format!("s3://{}", path), tmp_path.to_str().unwrap()])
            .status()
            .context("Failed to run aws s3 cp — install AWS CLI and configure credentials")?;

        if !status.success() {
            anyhow::bail!("aws s3 cp failed for s3://{}", path);
        }

        Ok(Self {
            local: LocalDiskSource::open(&tmp_path)?,
            uri: uri.to_string(),
        })
    }
}

impl Read for S3DiskSource {
    fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
        self.local.read(buf)
    }
}

impl Seek for S3DiskSource {
    fn seek(&mut self, pos: SeekFrom) -> std::io::Result<u64> {
        self.local.seek(pos)
    }
}

impl DiskSource for S3DiskSource {
    fn metadata(&self) -> DiskSourceMetadata {
        DiskSourceMetadata {
            uri: self.uri.clone(),
            size_bytes: self.local.metadata().size_bytes,
            backend: "s3".to_string(),
        }
    }

    fn local_path(&self) -> Option<&std::path::Path> {
        self.local.local_path()
    }
}

// Keep temp file alive
impl Drop for S3DiskSource {
    fn drop(&mut self) {
        if let Some(p) = self.local.local_path() {
            let _ = std::fs::remove_file(p);
        }
    }
}
