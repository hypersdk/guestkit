// SPDX-License-Identifier: Apache-2.0
//! Agent daemon main loop.

use crate::agent::handler::RequestHandler;
use crate::agent::transport::{FramedTransport, TransportConfig};
use anyhow::{Context, Result};
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::signal;
use tokio::task;

const MIN_REQUEST_INTERVAL: Duration = Duration::from_millis(10);

pub struct AgentDaemon {
    config: TransportConfig,
    handler: Arc<RequestHandler>,
}

impl AgentDaemon {
    pub fn new(config: TransportConfig) -> Self {
        Self {
            config,
            handler: Arc::new(RequestHandler::new()),
        }
    }

    pub async fn run(self) -> Result<()> {
        log::info!(
            "GuestKit agent starting (protocol {}, channel {:?})",
            guestkit_agent_protocol::PROTOCOL_VERSION,
            self.config.kind
        );

        // Local Unix socket API (read-only + policy-gated writes).
        crate::agent::transport::unix_listen::spawn_local_socket(
            Arc::clone(&self.handler),
            None,
        )
        .ok();

        // Outbound Zeus mTLS push worker.
        task::spawn(async {
            if let Err(e) = crate::agent::transport::zeus_push::run_push_worker().await {
                log::warn!("Zeus push worker stopped: {e}");
            }
        });

        #[cfg(target_os = "linux")]
        crate::collectors::dbus::systemd_events::spawn_subscriber();

        let config = self.config.clone();
        let handler = Arc::clone(&self.handler);

        let loop_handle = task::spawn_blocking(move || -> Result<()> {
            let mut transport = FramedTransport::open(&config)?;
            let mut last_request = Instant::now() - MIN_REQUEST_INTERVAL;

            loop {
                let frame = match transport.read_message() {
                    Ok(f) => f,
                    Err(e) => {
                        log::debug!("waiting for host frame: {e}");
                        std::thread::sleep(Duration::from_millis(250));
                        continue;
                    }
                };

                let elapsed = last_request.elapsed();
                if elapsed < MIN_REQUEST_INTERVAL {
                    std::thread::sleep(MIN_REQUEST_INTERVAL - elapsed);
                }
                last_request = Instant::now();

                let payload = handler.handle_frame(&frame);
                if let Err(e) = if crate::agent::qga::take_delimited_response() {
                    transport.write_delimited_frame(&payload)
                } else {
                    transport.write_frame(&payload)
                } {
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
