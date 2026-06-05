// SPDX-License-Identifier: LGPL-3.0-or-later
//! Agent daemon main loop.

use crate::agent::handler::RequestHandler;
use crate::agent::transport::{FramedTransport, TransportConfig};
use anyhow::{Context, Result};
use std::time::{Duration, Instant};
use tokio::signal;
use tokio::task;

const MIN_REQUEST_INTERVAL: Duration = Duration::from_millis(10);

pub struct AgentDaemon {
    config: TransportConfig,
    handler: RequestHandler,
}

impl AgentDaemon {
    pub fn new(config: TransportConfig) -> Self {
        Self {
            config,
            handler: RequestHandler::new(),
        }
    }

    pub async fn run(self) -> Result<()> {
        log::info!(
            "GuestKit agent starting (protocol {}, channel {:?})",
            guestkit_agent_protocol::PROTOCOL_VERSION,
            self.config.kind
        );

        let config = self.config.clone();
        let handler = self.handler;

        let loop_handle = task::spawn_blocking(move || -> Result<()> {
            let mut transport = FramedTransport::open(&config)?;
            let mut last_request = Instant::now() - MIN_REQUEST_INTERVAL;

            loop {
                let frame = match transport.read_frame() {
                    Ok(f) => f,
                    Err(e) => {
                        log::error!("read frame: {e}");
                        break;
                    }
                };

                let elapsed = last_request.elapsed();
                if elapsed < MIN_REQUEST_INTERVAL {
                    std::thread::sleep(MIN_REQUEST_INTERVAL - elapsed);
                }
                last_request = Instant::now();

                let payload = handler.handle_frame(&frame);
                if let Err(e) = transport.write_frame(&payload) {
                    log::error!("write frame: {e}");
                    break;
                }
            }
            Ok(())
        });

        tokio::select! {
            res = loop_handle => res.context("agent loop panicked")??,
            _ = signal::ctrl_c() => {
                log::info!("GuestKit agent shutting down");
            }
        }

        Ok(())
    }
}
