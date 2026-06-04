// SPDX-License-Identifier: LGPL-3.0-or-later
//! Fix plan generation and application module
//!
//! This module provides offline patch & fix preview capabilities:
//! - Generate fix plans from security profiles
//! - Preview changes before applying
//! - Export plans as scripts (bash, ansible)
//! - Apply changes with safety checks
//! - Rollback capabilities

#![allow(unused_imports)]

pub mod types;
pub mod generator;
pub mod preview;
pub mod apply;
pub mod export;
pub mod command;

pub mod topo_sort;
#[cfg(feature = "agent")]
pub mod executor_live;

pub use types::{
    FixPlan,
    Operation,
    OperationType,
    Priority,
    FileEdit,
    PackageInstall,
    ServiceOperation,
    SELinuxMode,
    RegistryEdit,
    PostApplyAction,
};

pub use generator::PlanGenerator;
pub use preview::PlanPreview;
pub use apply::{ApplyResult, PlanApplicator};
#[cfg(feature = "agent")]
pub use executor_live::LivePlanExecutor;
pub use export::PlanExporter;
pub use command::PlanCommand;
