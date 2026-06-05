// SPDX-License-Identifier: Apache-2.0
//! Bootability prediction engine.

pub mod checks;
pub mod engine;
pub mod report;

pub use engine::{analyze_bootability, BootTarget};
pub use report::{BootabilityReport, CheckResult, CheckSeverity, Finding};
