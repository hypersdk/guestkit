// SPDX-License-Identifier: Apache-2.0
//! Live journal collection (journalctl fallback).

use crate::journal::sd_journal::{collect_journal_slice as collect_sd, BootSelector};
use guestkit_agent_protocol::JournalSlice;

pub fn collect_journal_slice(unit: &str, limit: usize) -> JournalSlice {
    collect_sd(unit, limit, BootSelector::Current)
}

pub fn collect_journal_slice_boot(unit: &str, limit: usize, boot: &str) -> JournalSlice {
    collect_sd(unit, limit, BootSelector::from_str(boot))
}
