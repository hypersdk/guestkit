// SPDX-License-Identifier: Apache-2.0
//! Fleet clustering and anomaly detection.

pub mod analyzer;
pub mod report;

pub use analyzer::analyze_fleet;
pub use report::FleetAnalysisReport;
