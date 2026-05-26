// SPDX-License-Identifier: LGPL-3.0-or-later
//! Deterministic root-cause inference engine.

pub mod engine;
pub mod report;

pub use engine::infer_root_cause;
pub use report::RootCauseReport;
