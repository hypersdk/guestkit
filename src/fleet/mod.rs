// SPDX-License-Identifier: LGPL-3.0-or-later
//! Fleet clustering and anomaly detection.

pub mod analyzer;
pub mod report;

pub use analyzer::analyze_fleet;
pub use report::FleetAnalysisReport;
