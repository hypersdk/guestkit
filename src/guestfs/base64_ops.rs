// SPDX-License-Identifier: LGPL-3.0-or-later
//! Base64 encoding/decoding operations compatible with libguestfs
//!
//! This implementation provides Base64 encoding and decoding for file content.

use crate::core::{Error, Result};
use crate::guestfs::Guestfs;

impl Guestfs {
    /// Encode file content to base64
    ///
    /// Compatible with libguestfs g.base64_in()
    pub fn base64_in(&mut self, base64file: &str, filename: &str) -> Result<()> {
        self.ensure_ready()?;

        if self.verbose {
            eprintln!("guestfs: base64_in {} {}", base64file, filename);
        }

        // Read base64 content from host file
        let base64_content = std::fs::read_to_string(base64file)
            .map_err(|e| Error::Io(e))?;

        // Decode base64
        let decoded = base64::Engine::decode(
            &base64::engine::general_purpose::STANDARD,
            base64_content.trim()
        ).map_err(|e| Error::InvalidFormat(format!("Base64 decode failed: {}", e)))?;

        // Write decoded content to guest file
        let host_path = self.resolve_guest_path(filename)?;
        std::fs::write(&host_path, decoded)
            .map_err(|e| Error::Io(e))?;

        Ok(())
    }

    /// Decode file content from base64
    ///
    /// Compatible with libguestfs g.base64_out()
    pub fn base64_out(&mut self, filename: &str, base64file: &str) -> Result<()> {
        self.ensure_ready()?;

        if self.verbose {
            eprintln!("guestfs: base64_out {} {}", filename, base64file);
        }

        // Read guest file content
        let host_path = self.resolve_guest_path(filename)?;
        let content = std::fs::read(&host_path)
            .map_err(|e| Error::Io(e))?;

        // Encode to base64
        let encoded = base64::Engine::encode(
            &base64::engine::general_purpose::STANDARD,
            &content
        );

        // Write base64 content to host file
        std::fs::write(base64file, encoded)
            .map_err(|e| Error::Io(e))?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_base64_ops_api_exists() {
        let mut g = Guestfs::new().unwrap();
        // API structure tests
    }
}
