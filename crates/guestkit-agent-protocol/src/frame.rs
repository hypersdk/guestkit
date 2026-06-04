// SPDX-License-Identifier: LGPL-3.0-or-later
//! Length-prefixed frame encoding/decoding.

use crate::error::AgentError;
use std::io::{Read, Write};

/// Maximum frame size (16 MiB) to prevent memory exhaustion.
pub const MAX_FRAME_SIZE: u32 = 16 * 1024 * 1024;

/// Read one length-prefixed frame from `reader`.
pub fn read_frame<R: Read>(reader: &mut R) -> Result<Vec<u8>, AgentError> {
    let mut len_buf = [0u8; 4];
    reader.read_exact(&mut len_buf).map_err(AgentError::Io)?;
    let len = u32::from_be_bytes(len_buf);
    if len == 0 {
        return Err(AgentError::InvalidRequest("empty frame".into()));
    }
    if len > MAX_FRAME_SIZE {
        return Err(AgentError::InvalidRequest(format!(
            "frame too large: {len} bytes (max {MAX_FRAME_SIZE})"
        )));
    }
    let mut buf = vec![0u8; len as usize];
    reader.read_exact(&mut buf).map_err(AgentError::Io)?;
    Ok(buf)
}

/// Write one length-prefixed frame to `writer`.
pub fn write_frame<W: Write>(writer: &mut W, payload: &[u8]) -> Result<(), AgentError> {
    if payload.is_empty() {
        return Err(AgentError::InvalidRequest("empty payload".into()));
    }
    if payload.len() as u32 > MAX_FRAME_SIZE {
        return Err(AgentError::InvalidRequest(format!(
            "payload too large: {} bytes",
            payload.len()
        )));
    }
    let len = (payload.len() as u32).to_be_bytes();
    writer.write_all(&len).map_err(AgentError::Io)?;
    writer.write_all(payload).map_err(AgentError::Io)?;
    writer.flush().map_err(AgentError::Io)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Cursor;

    #[test]
    fn round_trip_frame() {
        let payload = br#"{"jsonrpc":"2.0","method":"guestkit.ping"}"#;
        let mut buf = Vec::new();
        write_frame(&mut buf, payload).unwrap();
        let mut cursor = Cursor::new(buf);
        let decoded = read_frame(&mut cursor).unwrap();
        assert_eq!(decoded, payload);
    }
}
