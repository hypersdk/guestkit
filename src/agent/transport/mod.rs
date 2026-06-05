// SPDX-License-Identifier: LGPL-3.0-or-later
//! Agent I/O transports.

pub mod stdio;
pub mod virtio;
pub mod vsock;

use anyhow::Result;
use guestkit_agent_protocol::{read_frame, write_frame};
use std::io::{Read, Write};

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
}

impl FramedTransport {
    pub fn open(config: &TransportConfig) -> Result<Self> {
        match config.kind {
            ChannelKind::Virtio => virtio::open(&config.device_path),
            ChannelKind::Vsock => vsock::open(config.vsock_cid, config.vsock_port),
            ChannelKind::Stdio => stdio::open(),
        }
    }

    pub fn read_frame(&mut self) -> Result<Vec<u8>> {
        read_frame(&mut self.reader).map_err(|e| anyhow::anyhow!("{e}"))
    }

    pub fn write_frame(&mut self, payload: &[u8]) -> Result<()> {
        write_frame(&mut self.writer, payload).map_err(|e| anyhow::anyhow!("{e}"))
    }
}
