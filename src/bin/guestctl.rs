// SPDX-License-Identifier: LGPL-3.0-or-later
//! guestctl binary — alias entry point for guestkit.

fn main() -> anyhow::Result<()> {
    guestkit::cli::run()
}
