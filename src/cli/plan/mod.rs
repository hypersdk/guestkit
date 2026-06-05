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

pub mod apply;
pub mod command;
pub mod export;
pub mod generator;
pub mod preview;
pub mod types;

#[cfg(feature = "agent")]
pub mod executor_live;
pub mod topo_sort;

pub use types::{
    FileEdit, FixPlan, Operation, OperationType, PackageInstall, PostApplyAction, Priority,
    RegistryEdit, SELinuxMode, ServiceOperation,
};

pub use apply::{ApplyResult, PlanApplicator};
pub use command::PlanCommand;
#[cfg(feature = "agent")]
pub use executor_live::LivePlanExecutor;
pub use export::PlanExporter;
pub use generator::PlanGenerator;
pub use preview::PlanPreview;
