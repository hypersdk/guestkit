// SPDX-License-Identifier: LGPL-3.0-or-later
//! Evidence collectors — populate typed slices of [`EvidenceSnapshot`].

pub mod systemd;
pub mod windows;

pub use systemd::{collect_systemd_guest, collect_systemd_live};
pub use windows::collect_windows_details;
