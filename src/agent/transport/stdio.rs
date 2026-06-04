// SPDX-License-Identifier: LGPL-3.0-or-later
//! Stdio transport for development and testing.

use super::FramedTransport;
use anyhow::Result;
use std::io::{stdin, stdout};

pub fn open() -> Result<FramedTransport> {
    Ok(FramedTransport {
        reader: Box::new(stdin()),
        writer: Box::new(stdout()),
    })
}
