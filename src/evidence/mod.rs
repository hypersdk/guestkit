// SPDX-License-Identifier: LGPL-3.0-or-later
//! Normalized evidence snapshot — digital twin primitive for migration assurance.

pub mod builder;
pub mod snapshot;

pub use builder::{build_evidence, EvidenceBuilder};
pub use snapshot::{
    BootEvidence, EvidenceSnapshot, NetworkEvidence, OsEvidence, PackageEvidence,
    SecurityEvidence, StorageEvidence, VmToolsEvidence, WindowsEvidence, SCHEMA_VERSION,
};
