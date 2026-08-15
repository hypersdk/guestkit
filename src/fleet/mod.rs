// SPDX-License-Identifier: Apache-2.0
//! Fleet clustering and anomaly detection.

pub mod analyzer;
pub mod baseline;
pub mod report;
pub mod wave;

pub use analyzer::analyze_fleet;
pub use baseline::FleetBaseline;
pub use report::FleetAnalysisReport;
pub use wave::{plan_waves, MigrationWave, VmRole, WaveMember, WavePlan};
