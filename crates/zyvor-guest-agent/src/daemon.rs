// SPDX-License-Identifier: Apache-2.0

use crate::handler::RequestHandler;
use anyhow::{bail, Context, Result};
use guestkit_agent_protocol::{read_frame, write_frame};
use std::fs::OpenOptions;
use std::io::{Read, Write};
use std::path::Path;

pub fn run(channel: &str) -> Result<()> {
    log::info!("Zyvor guest agent starting (channel={channel})");
    let mut transport = open_transport(channel)?;
    let handler = RequestHandler::new();
    loop {
        let frame = read_frame(&mut transport.reader).map_err(|e| anyhow::anyhow!("read: {e}"))?;
        let response = handler.handle_frame(&frame);
        write_frame(&mut transport.writer, &response).map_err(|e| anyhow::anyhow!("write: {e}"))?;
    }
}

struct Transport {
    reader: Box<dyn Read>,
    writer: Box<dyn Write>,
}

fn open_transport(channel: &str) -> Result<Transport> {
    match channel {
        "stdio" => Ok(Transport {
            reader: Box::new(std::io::stdin()),
            writer: Box::new(std::io::stdout()),
        }),
        "virtio" => open_virtio(guestkit_agent_protocol::VIRTIO_DEVICE_PATH),
        path if !path.is_empty() => open_virtio(path),
        _ => bail_unknown(channel),
    }
}

fn open_virtio(path: &str) -> Result<Transport> {
    let file = OpenOptions::new()
        .read(true)
        .write(true)
        .open(Path::new(path))
        .with_context(|| format!("open virtio channel {path}"))?;
    Ok(Transport {
        reader: Box::new(file.try_clone()?),
        writer: Box::new(file),
    })
}

fn bail_unknown(channel: &str) -> Result<Transport> {
    bail!("unknown channel: {channel}")
}
