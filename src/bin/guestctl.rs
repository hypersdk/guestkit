// SPDX-License-Identifier: Apache-2.0
//! guestctl binary — alias entry point for guestkit.

fn main() -> anyhow::Result<()> {
    guestkit::cli::run()
}
