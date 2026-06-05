// SPDX-License-Identifier: LGPL-3.0-or-later
//! Live guest runtime metrics for VMRogue monitoring and Copilot.

#[cfg(feature = "agent")]
pub mod live;

#[cfg(feature = "agent")]
pub use live::{collect_metrics_live, GuestMetricsSnapshot};
