// SPDX-License-Identifier: LGPL-3.0-or-later
//! CLI module for guestctl

pub mod batch;
pub mod cache;
pub mod commands;
pub mod diff;
pub mod errors;
pub mod exporters;
pub mod formatters;
pub mod interactive;
pub mod output;
pub mod parallel;
pub mod profiles;
pub mod tui;

pub use batch::*;
pub use interactive::*;
// Parallel inspection features - currently unused but available for future use
#[allow(unused_imports)]
pub use parallel::{
    inspect_batch, inspect_batch_with_workers, InspectionConfig, InspectionResult,
    ParallelInspector, ProgressiveInspector,
};
