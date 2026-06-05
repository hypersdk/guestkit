// SPDX-License-Identifier: Apache-2.0
//! Bootability report types.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BootabilityReport {
    pub score: f64,
    pub confidence: f64,
    pub target: String,
    pub blockers: Vec<Finding>,
    pub warnings: Vec<Finding>,
    pub checks: Vec<CheckResult>,
    pub summary: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CheckResult {
    pub id: String,
    pub name: String,
    pub passed: bool,
    pub severity: CheckSeverity,
    pub message: String,
    pub weight: f64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum CheckSeverity {
    Blocker,
    Warning,
    Info,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Finding {
    pub check_id: String,
    pub title: String,
    pub message: String,
    pub remediation: Option<String>,
}

impl BootabilityReport {
    pub fn boot_probability_message(&self) -> String {
        format!(
            "{:.0}% chance of successful first boot on {} (confidence: {:.0}%)",
            self.score,
            self.target,
            self.confidence * 100.0
        )
    }
}
