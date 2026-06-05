// SPDX-License-Identifier: Apache-2.0
//! Vsock transport stub (Windows / advanced fallback — phase 7).

use super::FramedTransport;
use anyhow::{bail, Result};

pub fn open(_cid: Option<u32>, _port: Option<u32>) -> Result<FramedTransport> {
    bail!("vsock transport is not yet implemented; use --channel virtio or stdio")
}
