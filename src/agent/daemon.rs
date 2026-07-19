// SPDX-License-Identifier: Apache-2.0
//! Agent daemon main loop.
//!
//! One blocking request/response loop runs per available channel: the
//! configured primary (usually the QGA-shared virtio channel), plus the
//! dedicated GuestKit channel and the legacy channel when their device
//! nodes exist. Server-push (heartbeat) only ever happens on channels
//! marked push-capable and explicitly subscribed.

use crate::agent::handler::RequestHandler;
use crate::agent::state::{AgentRuntime, ChannelHandle};
use crate::agent::transport::{ChannelKind, FramedTransport, TransportConfig};
use anyhow::{Context, Result};
use guestkit_agent_protocol::{VIRTIO_DEVICE_PATH, VIRTIO_DEVICE_PATH_GUESTKIT};
use std::sync::atomic::AtomicBool;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::signal;
use tokio::task;

const MIN_REQUEST_INTERVAL: Duration = Duration::from_millis(10);

/// Device path of the legacy GuestKit-only channel.
const VIRTIO_DEVICE_PATH_LEGACY: &str = "/dev/virtio-ports/com.zyvor.guestkit.0";

pub struct AgentDaemon {
    config: TransportConfig,
    handler: Arc<RequestHandler>,
    runtime: Arc<AgentRuntime>,
}

/// One channel to serve: its transport config, display name, and whether
/// notifications may be pushed on it after a subscribe.
struct ChannelPlan {
    config: TransportConfig,
    name: String,
    push_capable: bool,
    /// Primary channel: failure to open is fatal (matches historic
    /// single-channel behavior). Secondary channels log and drop out.
    required: bool,
}

impl AgentDaemon {
    pub fn new(config: TransportConfig) -> Self {
        let runtime = AgentRuntime::global();
        Self {
            config,
            handler: Arc::new(RequestHandler::with_runtime(Arc::clone(&runtime))),
            runtime,
        }
    }

    fn channel_plans(&self) -> Vec<ChannelPlan> {
        let primary_push_capable = match self.config.kind {
            // Pushing interleaved frames at a plain-QGA host would break it.
            ChannelKind::Virtio => self.config.device_path != VIRTIO_DEVICE_PATH,
            ChannelKind::Vsock | ChannelKind::Stdio => true,
        };
        let mut plans = vec![ChannelPlan {
            config: self.config.clone(),
            name: match self.config.kind {
                ChannelKind::Virtio => self.config.device_path.clone(),
                ChannelKind::Vsock => "vsock".to_string(),
                ChannelKind::Stdio => "stdio".to_string(),
            },
            push_capable: primary_push_capable,
            required: true,
        }];

        // Extra virtio channels, only when their device nodes exist and they
        // aren't already the primary.
        if self.config.kind == ChannelKind::Virtio {
            for (path, push_capable) in [
                (VIRTIO_DEVICE_PATH_GUESTKIT, true),
                (VIRTIO_DEVICE_PATH_LEGACY, true),
            ] {
                if path != self.config.device_path && std::path::Path::new(path).exists() {
                    plans.push(ChannelPlan {
                        config: TransportConfig {
                            kind: ChannelKind::Virtio,
                            device_path: path.to_string(),
                            vsock_cid: None,
                            vsock_port: None,
                        },
                        name: path.to_string(),
                        push_capable,
                        required: false,
                    });
                }
            }
        }
        plans
    }

    pub async fn run(self) -> Result<()> {
        log::info!(
            "GuestKit agent starting (protocol {}, channel {:?})",
            guestkit_agent_protocol::PROTOCOL_VERSION,
            self.config.kind
        );

        // Local Unix socket API (read-only + policy-gated writes).
        // GUESTKIT_LOCAL_SOCKET overrides the default paths (tests/e2e).
        crate::agent::transport::unix_listen::spawn_local_socket(
            Arc::clone(&self.handler),
            std::env::var("GUESTKIT_LOCAL_SOCKET").ok(),
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

        // If a previous run died mid-cutover with filesystems frozen, thaw.
        crate::migration::workflow::recover_cutover_state();

        // Periodic heartbeat build + push to subscribed channels.
        task::spawn(crate::agent::heartbeat::run_heartbeat_task(
            Arc::clone(&self.runtime),
            Duration::from_secs(crate::agent::heartbeat::DEFAULT_INTERVAL_SECS),
        ));

        // 1-second rolling telemetry sampler.
        task::spawn(crate::agent::telemetry::sampler::run_sampler(Arc::clone(
            &self.runtime.telemetry,
        )));

        let mut loop_handles = Vec::new();
        for plan in self.channel_plans() {
            let handler = Arc::clone(&self.handler);
            let runtime = Arc::clone(&self.runtime);
            let required = plan.required;
            let handle = task::spawn_blocking(move || run_channel(plan, handler, runtime));
            loop_handles.push((required, handle));
        }

        // The daemon lives until ctrl-c or the primary (required) loop ends.
        let (required_handles, optional_handles): (Vec<_>, Vec<_>) =
            loop_handles.into_iter().partition(|(req, _)| *req);
        drop(optional_handles); // detached: secondary channel failures are non-fatal

        let primary = required_handles
            .into_iter()
            .map(|(_, h)| h)
            .next()
            .expect("at least one required channel");

        tokio::select! {
            res = primary => res.context("agent loop panicked")??,
            _ = signal::ctrl_c() => {
                log::info!("GuestKit agent shutting down");
            }
        }

        Ok(())
    }
}

fn run_channel(
    plan: ChannelPlan,
    handler: Arc<RequestHandler>,
    runtime: Arc<AgentRuntime>,
) -> Result<()> {
    let transport = match FramedTransport::open(&plan.config) {
        Ok(t) => t,
        Err(e) if !plan.required => {
            log::warn!("skipping channel {}: {e}", plan.name);
            return Ok(());
        }
        Err(e) => return Err(e),
    };
    let (mut reader, writer) = transport.split();
    let channel = Arc::new(ChannelHandle {
        name: plan.name.clone(),
        push_capable: plan.push_capable,
        subscribed: AtomicBool::new(false),
        writer: writer.clone(),
    });
    runtime.register_channel(Arc::clone(&channel));
    log::info!(
        "serving channel {} (push_capable={})",
        plan.name,
        plan.push_capable
    );

    let mut last_request = Instant::now() - MIN_REQUEST_INTERVAL;
    loop {
        let frame = match reader.read_message() {
            Ok(f) => f,
            Err(e) => {
                log::debug!("waiting for host frame on {}: {e}", plan.name);
                std::thread::sleep(Duration::from_millis(250));
                continue;
            }
        };

        let elapsed = last_request.elapsed();
        if elapsed < MIN_REQUEST_INTERVAL {
            std::thread::sleep(MIN_REQUEST_INTERVAL - elapsed);
        }
        last_request = Instant::now();

        let payload = handler.handle_frame_on(&frame, Some(&channel));
        let delimited = crate::agent::qga::take_delimited_response();
        if let Err(e) = writer.write_message(&payload, delimited) {
            log::error!("write frame on {}: {e}", plan.name);
            runtime.unregister_channel(&channel);
            break;
        }
    }
    Ok(())
}
