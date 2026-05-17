// SPDX-License-Identifier: LGPL-3.0-or-later
//! Enterprise contact messaging for Community Edition builds.

pub const REPO_UTM: &str = "guestkit";

pub const CLI_AFTER_HELP: &str = "\
Enterprise (VMware exit, SLAs, fleet migrations, full platform):
  Watch demo: https://zyvor.dev/demo?utm_source=github&utm_medium=guestkit
  Platform:   https://zyvor.dev/?utm_source=github&utm_medium=guestkit
  Contact:    https://zyvor.dev/contact?utm_source=github&utm_medium=guestkit
  Sales:      sales@zyvor.dev
  Info:       info@zyvor.dev\n";

/// Shown after successful CLI commands (unless --quiet / --machine-readable).
pub fn print_success_footer() {
    eprintln!();
    eprintln!(
        "▶ Watch migration demo → https://zyvor.dev/demo?utm_source=github&utm_medium={}",
        REPO_UTM
    );
    eprintln!(
        "🏢 Production / VMware exit / SLAs → sales@zyvor.dev · https://zyvor.dev/contact?utm_source=github&utm_medium={}",
        REPO_UTM
    );
}
