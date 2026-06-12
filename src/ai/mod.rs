// SPDX-License-Identifier: Apache-2.0
//! Guest Intelligence Agent — evidence-grounded analysis and optional LLM co-pilot.
//!
//! Phases 0–4 of the AI Guest Agent roadmap. Deterministic analysis modules work
//! without `--features ai`; LLM providers require the `ai` feature.

pub mod drift;
pub mod intelligence;
pub mod platform;
pub mod prompts;
pub mod recommendations;
pub mod reports;
pub mod security_profiles;
pub mod semantic;
pub mod whatif;

#[cfg(feature = "ai")]
pub mod agent;
#[cfg(feature = "ai")]
pub mod providers;
#[cfg(feature = "ai")]
pub mod tools;

pub use drift::{explain_fleet_drift, FleetDriftReport};
pub use intelligence::{build_intelligence, IntelligenceBundle};
pub use platform::{MachinaEvidenceExport, PlatformSummary};
pub use recommendations::{generate_recommendations, Recommendation, RecommendationCategory};
pub use reports::{build_report_narrative, ReportNarrative};
pub use security_profiles::{evaluate_cis_profile, SecurityProfileReport};
pub use semantic::{analyze_semantic, SemanticAnalysis};
pub use whatif::{simulate_unit_disable, WhatIfResult};

#[cfg(feature = "ai")]
pub use agent::{run_agent_on_evidence, AgentConfig, AgentResult};
#[cfg(feature = "ai")]
pub use providers::{completion, Provider, ProviderConfig};
