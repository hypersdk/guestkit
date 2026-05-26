// SPDX-License-Identifier: LGPL-3.0-or-later
//! Bootability prediction engine.

pub mod checks;
pub mod engine;
pub mod report;

pub use engine::{analyze_bootability, BootTarget};
pub use report::{BootabilityReport, CheckResult, CheckSeverity, Finding};
