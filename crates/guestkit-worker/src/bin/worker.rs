//! Guestkit Worker Daemon

use clap::Parser;
use guestkit_worker::{
    Worker, WorkerConfig, HandlerRegistry,
    handlers::{EchoHandler, InspectHandler, ProfileHandler},
    transport::file::{FileTransport, FileTransportConfig},
    capabilities::Capabilities,
};
use std::sync::Arc;
use std::path::PathBuf;

#[derive(Parser, Debug)]
#[command(name = "guestkit-worker")]
#[command(about = "Guestkit distributed worker daemon")]
#[command(version)]
struct Args {
    /// Worker ID (defaults to generated ULID)
    #[arg(short, long)]
    worker_id: Option<String>,

    /// Worker pool name
    #[arg(short, long, default_value = "default")]
    pool: String,

    /// Job directory to watch
    #[arg(short, long, default_value = "./jobs")]
    jobs_dir: PathBuf,

    /// Working directory
    #[arg(short, long, default_value = "/tmp/guestkit-worker")]
    work_dir: PathBuf,

    /// Results output directory
    #[arg(short, long, default_value = "./results")]
    results_dir: PathBuf,

    /// Maximum concurrent jobs
    #[arg(short, long, default_value = "4")]
    max_concurrent: usize,

    /// Log level
    #[arg(long, default_value = "info")]
    log_level: String,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let args = Args::parse();

    // Initialize logging
    env_logger::Builder::from_env(
        env_logger::Env::default()
            .default_filter_or(&args.log_level)
    ).init();

    log::info!("Starting guestkit worker");

    // Worker configuration
    let config = WorkerConfig {
        worker_id: args.worker_id.unwrap_or_else(|| format!("worker-{}", ulid::Ulid::new())),
        worker_pool: Some(args.pool),
        work_dir: args.work_dir,
        result_dir: args.results_dir.clone(),
        max_concurrent_jobs: args.max_concurrent,
        shutdown_timeout_secs: 30,
    };

    log::info!("Worker ID: {}", config.worker_id);
    log::info!("Working directory: {}", config.work_dir.display());
    log::info!("Results directory: {}", config.result_dir.display());

    // Setup handler registry
    let mut registry = HandlerRegistry::new();

    // Register built-in handlers
    registry.register(Arc::new(EchoHandler::new()));

    // Register guestkit operation handlers
    registry.register(Arc::new(InspectHandler::new()));
    registry.register(Arc::new(ProfileHandler::new()));

    // TODO: Add more handlers
    // registry.register(Arc::new(FixHandler::new()));
    // registry.register(Arc::new(ConvertHandler::new()));

    log::info!("Registered {} operation handlers", registry.len());
    log::info!("Supported operations: {:?}", registry.operations());

    // Worker capabilities
    let capabilities = Capabilities::new()
        .with_operation("system.echo")
        .with_operation("test.echo")
        .with_operation("guestkit.inspect")
        .with_operation("guestkit.profile")
        .with_feature("rust")
        .with_feature("lvm")
        .with_feature("nbd")
        .with_disk_format("qcow2")
        .with_disk_format("vmdk")
        .with_disk_format("vdi")
        .with_disk_format("vhdx")
        .with_disk_format("raw");

    // Setup file transport
    let transport_config = FileTransportConfig {
        watch_dir: args.jobs_dir.clone(),
        done_dir: args.jobs_dir.join("done"),
        failed_dir: args.jobs_dir.join("failed"),
        poll_interval_secs: 2,
    };

    log::info!("Watching for jobs in: {}", transport_config.watch_dir.display());

    let transport = FileTransport::new(transport_config).await?;

    // Create and run worker
    let mut worker = Worker::new(
        config,
        capabilities,
        registry,
        Box::new(transport),
    )?;

    log::info!("Worker ready, waiting for jobs...");

    worker.run().await?;

    log::info!("Worker shut down cleanly");

    Ok(())
}
