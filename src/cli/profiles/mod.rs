// SPDX-License-Identifier: LGPL-3.0-or-later
//! Inspection profiles for focused use cases

use anyhow::Result;
use guestctl::Guestfs;
use serde::{Deserialize, Serialize};

pub mod compliance;
pub mod hardening;
pub mod migration;
pub mod performance;
pub mod security;

pub use compliance::ComplianceProfile;
pub use hardening::HardeningProfile;
pub use migration::MigrationProfile;
pub use performance::PerformanceProfile;
pub use security::SecurityProfile;

/// Risk level for security findings
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum RiskLevel {
    Critical,
    High,
    Medium,
    Low,
    Info,
}

impl std::fmt::Display for RiskLevel {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            RiskLevel::Critical => write!(f, "CRITICAL"),
            RiskLevel::High => write!(f, "HIGH"),
            RiskLevel::Medium => write!(f, "MEDIUM"),
            RiskLevel::Low => write!(f, "LOW"),
            RiskLevel::Info => write!(f, "INFO"),
        }
    }
}

/// Report section with findings
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReportSection {
    pub title: String,
    pub findings: Vec<Finding>,
}

/// Individual finding in a report
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Finding {
    pub item: String,
    pub status: FindingStatus,
    pub message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub risk_level: Option<RiskLevel>,
}

/// Status of a finding
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum FindingStatus {
    Pass,
    Warning,
    Fail,
    Info,
}

impl std::fmt::Display for FindingStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            FindingStatus::Pass => write!(f, "✓"),
            FindingStatus::Warning => write!(f, "⚠"),
            FindingStatus::Fail => write!(f, "✗"),
            FindingStatus::Info => write!(f, "ℹ"),
        }
    }
}

/// Profile report structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProfileReport {
    pub profile_name: String,
    pub sections: Vec<ReportSection>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub overall_risk: Option<RiskLevel>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub summary: Option<String>,
}

/// Trait for inspection profiles
pub trait InspectionProfile {
    /// Get profile name
    #[allow(dead_code)]
    fn name(&self) -> &str;

    /// Get profile description
    fn description(&self) -> &str;

    /// Run inspection with this profile
    fn inspect(&self, g: &mut Guestfs, root: &str) -> Result<ProfileReport>;
}

/// Get profile by name
pub fn get_profile(name: &str) -> Option<Box<dyn InspectionProfile>> {
    match name.to_lowercase().as_str() {
        "security" => Some(Box::new(SecurityProfile)),
        "migration" => Some(Box::new(MigrationProfile)),
        "performance" => Some(Box::new(PerformanceProfile)),
        "compliance" => Some(Box::new(ComplianceProfile)),
        "hardening" => Some(Box::new(HardeningProfile)),
        _ => None,
    }
}

/// List available profiles
#[allow(dead_code)]
pub fn list_profiles() -> Vec<(&'static str, &'static str)> {
    vec![
        (
            "security",
            "Security posture assessment and hardening recommendations",
        ),
        ("migration", "Migration planning and compatibility analysis"),
        (
            "performance",
            "Performance tuning opportunities and bottleneck detection",
        ),
        (
            "compliance",
            "Regulatory compliance assessment (CIS, FIPS, HIPAA, PCI-DSS)",
        ),
        (
            "hardening",
            "System hardening recommendations with actionable remediation steps",
        ),
    ]
}
