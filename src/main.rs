// SPDX-License-Identifier: Apache-2.0
//! guestkit binary entry point.

fn main() -> anyhow::Result<()> {
    guestkit::cli::run()
}
