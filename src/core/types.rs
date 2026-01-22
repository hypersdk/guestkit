// SPDX-License-Identifier: LGPL-3.0-or-later
//! Common types for guestkit

use serde::{Deserialize, Serialize};
use std::path::PathBuf;

/// Disk image format
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum DiskFormat {
    Qcow2,
    Raw,
    Vmdk,
    Vhd,
    Vhdx,
    Vdi,
    Unknown,
}

impl DiskFormat {
    pub fn as_str(&self) -> &str {
        match self {
            DiskFormat::Qcow2 => "qcow2",
            DiskFormat::Raw => "raw",
            DiskFormat::Vmdk => "vmdk",
            DiskFormat::Vhd => "vhd",
            DiskFormat::Vhdx => "vhdx",
            DiskFormat::Vdi => "vdi",
            DiskFormat::Unknown => "unknown",
        }
    }

    pub fn from_str(s: &str) -> Self {
        match s.to_lowercase().as_str() {
            "qcow2" => DiskFormat::Qcow2,
            "raw" => DiskFormat::Raw,
            "vmdk" => DiskFormat::Vmdk,
            "vhd" => DiskFormat::Vhd,
            "vhdx" => DiskFormat::Vhdx,
            "vdi" => DiskFormat::Vdi,
            _ => DiskFormat::Unknown,
        }
    }
}

/// Guest OS type
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum GuestType {
    Linux,
    Windows,
    FreeBSD,
    OpenBSD,
    NetBSD,
    Unknown,
}

/// Firmware type
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Firmware {
    Bios,
    Uefi,
    Unknown,
}

/// Guest identity information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GuestIdentity {
    pub os_type: GuestType,
    pub os_name: String,
    pub os_version: String,
    pub architecture: String,
    pub firmware: Firmware,
    pub init_system: Option<String>,
    pub distro: Option<String>,
}

/// Conversion result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConversionResult {
    pub source_path: PathBuf,
    pub output_path: PathBuf,
    pub source_format: DiskFormat,
    pub output_format: DiskFormat,
    pub output_size: u64,
    pub duration_secs: f64,
    pub success: bool,
    pub error: Option<String>,
}

/// Pipeline stage
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum PipelineStage {
    Fetch,
    Flatten,
    Inspect,
    Fix,
    Convert,
    Validate,
}

impl PipelineStage {
    pub fn as_str(&self) -> &str {
        match self {
            PipelineStage::Fetch => "fetch",
            PipelineStage::Flatten => "flatten",
            PipelineStage::Inspect => "inspect",
            PipelineStage::Fix => "fix",
            PipelineStage::Convert => "convert",
            PipelineStage::Validate => "validate",
        }
    }
}

/// Pipeline stage result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PipelineResult {
    pub stage: PipelineStage,
    pub success: bool,
    pub duration_secs: f64,
    pub error: Option<String>,
    pub data: Option<serde_json::Value>,
}
