// SPDX-License-Identifier: LGPL-3.0-or-later
//! Normalized evidence snapshot — digital twin primitive for migration assurance.

pub mod builder;
pub mod collectors;
pub mod snapshot;

#[cfg(feature = "agent")]
pub mod live;

pub use builder::{build_evidence, EvidenceBuilder};
#[cfg(feature = "agent")]
pub use live::build_evidence_live;
pub use snapshot::{
    BootEvidence, CloudInitEvidence, EvidenceSnapshot, KubevirtEvidence, NetworkEvidence,
    NetworkProbeEvidence, OsEvidence, PackageEvidence, SecurityEvidence, SnapshotReadinessEvidence,
    StorageEvidence, SystemdInfo, SystemdProblemHint, SystemdProblemSeverity, SystemdUnit,
    SystemdUnitState, VirtioDiskEntry, VmToolsEvidence, WindowsAppEntry, WindowsEvidence,
    WindowsServiceEntry, WindowsStartType, SCHEMA_VERSION,
};
