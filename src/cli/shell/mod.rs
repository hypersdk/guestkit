// SPDX-License-Identifier: LGPL-3.0-or-later
//! Interactive shell for VM inspection

pub mod commands;
pub mod completion;
pub mod explore;
pub mod repl;

pub use repl::run_interactive_shell;
