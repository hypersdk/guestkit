// SPDX-License-Identifier: Apache-2.0
//! Local Unix socket API for in-guest read-only queries.

use crate::agent::handler::RequestHandler;
use anyhow::{Context, Result};
use guestkit_agent_protocol::{read_frame, write_frame};
use std::fs;
use std::io::Write;
use std::os::unix::net::{UnixListener, UnixStream};
use std::path::Path;
use std::sync::Arc;
use std::thread;

pub const DEFAULT_SOCKET_PATH: &str = "/var/run/zyvor/guest-agent.sock";

pub fn spawn_local_socket(handler: Arc<RequestHandler>, socket_path: Option<String>) -> Result<()> {
    let path = socket_path.unwrap_or_else(|| DEFAULT_SOCKET_PATH.to_string());
    if Path::new(&path).exists() {
        fs::remove_file(&path).ok();
    }
    let parent = Path::new(&path).parent().unwrap_or(Path::new("/var/run/zyvor"));
    fs::create_dir_all(parent).ok();

    let listener = UnixListener::bind(&path).with_context(|| format!("bind {path}"))?;
    log::info!("Zyvor guest agent local API listening on {path}");

    thread::spawn(move || {
        for conn in listener.incoming().flatten() {
            let handler = Arc::clone(&handler);
            thread::spawn(move || {
                if let Err(e) = serve_connection(conn, &handler) {
                    log::debug!("local socket client error: {e}");
                }
            });
        }
    });

    Ok(())
}

fn serve_connection(stream: UnixStream, handler: &RequestHandler) -> Result<()> {
    let mut reader = stream.try_clone()?;
    let mut writer = stream;
    let frame = read_frame(&mut reader).map_err(|e| anyhow::anyhow!("{e}"))?;
    let response = handler.handle_frame(&frame);
    write_frame(&mut writer, &response).map_err(|e| anyhow::anyhow!("{e}"))?;
    Ok(())
}
