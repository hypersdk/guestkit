// SPDX-License-Identifier: Apache-2.0
//! Agent I/O transports.

pub mod stdio;
pub mod virtio;
pub mod vsock;
pub mod vsock_host;

use anyhow::Result;
use guestkit_agent_protocol::{read_frame, read_line, write_frame, write_line};
use std::io::{BufReader, Read, Write};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ChannelKind {
    Virtio,
    Vsock,
    Stdio,
}

#[derive(Debug, Clone)]
pub struct TransportConfig {
    pub kind: ChannelKind,
    pub device_path: String,
    pub vsock_cid: Option<u32>,
    pub vsock_port: Option<u32>,
}

/// Bidirectional framed transport.
pub struct FramedTransport {
    reader: Box<dyn Read + Send>,
    writer: Box<dyn Write + Send>,
    line_framing: bool,
}

impl FramedTransport {
    pub fn open(config: &TransportConfig) -> Result<Self> {
        let mut transport = match config.kind {
            ChannelKind::Virtio => virtio::open(&config.device_path)?,
            ChannelKind::Vsock => vsock::open(config.vsock_cid, config.vsock_port)?,
            ChannelKind::Stdio => stdio::open()?,
        };
        transport.line_framing = config.kind == ChannelKind::Virtio;
        Ok(transport)
    }

    pub fn read_message(&mut self) -> Result<Vec<u8>> {
        if self.line_framing {
            let mut buf = BufReader::new(&mut self.reader);
            read_line(&mut buf).map_err(|e| anyhow::anyhow!("{e}"))
        } else {
            read_frame(&mut self.reader).map_err(|e| anyhow::anyhow!("{e}"))
        }
    }

    pub fn write_message(&mut self, payload: &[u8], delimited: bool) -> Result<()> {
        if self.line_framing {
            if delimited {
                guestkit_agent_protocol::write_delimited_line(&mut self.writer, payload)
                    .map_err(|e| anyhow::anyhow!("{e}"))
            } else {
                write_line(&mut self.writer, payload).map_err(|e| anyhow::anyhow!("{e}"))
            }
        } else if delimited {
            guestkit_agent_protocol::write_delimited_line(&mut self.writer, payload)
                .map_err(|e| anyhow::anyhow!("{e}"))
        } else {
            write_frame(&mut self.writer, payload).map_err(|e| anyhow::anyhow!("{e}"))
        }
    }

    pub fn read_frame(&mut self) -> Result<Vec<u8>> {
        self.read_message()
    }

    pub fn write_frame(&mut self, payload: &[u8]) -> Result<()> {
        self.write_message(payload, false)
    }

    pub fn write_delimited_frame(&mut self, payload: &[u8]) -> Result<()> {
        self.write_message(payload, true)
    }
}
