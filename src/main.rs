// SPDX-License-Identifier: LGPL-3.0-or-later
//! guestkit CLI - Guest VM toolkit

use clap::{Parser, Subcommand};
use guestkit::{converters::DiskConverter, VERSION};
use std::path::PathBuf;

/// guestkit - Guest VM toolkit for disk inspection and manipulation
#[derive(Parser)]
#[command(name = "guestkit")]
#[command(version = VERSION)]
#[command(about = "Guest VM toolkit for disk inspection and manipulation", long_about = None)]
struct Cli {
    /// Verbose output
    #[arg(short, long, global = true)]
    verbose: bool,

    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Convert disk image format
    Convert {
        /// Source disk image path
        #[arg(short, long)]
        source: PathBuf,

        /// Output disk image path
        #[arg(short, long)]
        output: PathBuf,

        /// Output format (qcow2, raw, vmdk, vhd, vdi)
        #[arg(short, long, default_value = "qcow2")]
        format: String,

        /// Enable compression (qcow2 only)
        #[arg(short, long)]
        compress: bool,

        /// Flatten snapshot chains
        #[arg(short, long)]
        flatten: bool,
    },

    /// Detect disk image format
    Detect {
        /// Disk image path
        #[arg(short, long)]
        image: PathBuf,
    },

    /// Get disk image information
    Info {
        /// Disk image path
        #[arg(short, long)]
        image: PathBuf,
    },

    /// Show version information
    Version,
}

fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();

    // Setup logging
    let log_level = if cli.verbose {
        log::LevelFilter::Debug
    } else {
        log::LevelFilter::Info
    };

    env_logger::Builder::new()
        .filter_level(log_level)
        .format_timestamp_secs()
        .init();

    match cli.command {
        Commands::Convert {
            source,
            output,
            format,
            compress,
            flatten,
        } => {
            log::info!("Converting {} -> {}", source.display(), output.display());

            let converter = DiskConverter::new();
            let result = converter.convert(
                &source,
                &output,
                &format,
                compress,
                flatten,
            )?;

            if result.success {
                println!("✓ Conversion successful!");
                println!("  Source:  {} ({})", source.display(), result.source_format.as_str());
                println!("  Output:  {} ({})", output.display(), result.output_format.as_str());
                println!("  Size:    {} bytes", result.output_size);
                println!("  Time:    {:.2}s", result.duration_secs);
            } else {
                eprintln!("✗ Conversion failed: {}", result.error.unwrap_or_default());
                std::process::exit(1);
            }
        }

        Commands::Detect { image } => {
            let converter = DiskConverter::new();
            let format = converter.detect_format(&image)?;

            println!("Detected format: {}", format.as_str());
        }

        Commands::Info { image } => {
            let converter = DiskConverter::new();
            let info = converter.get_info(&image)?;

            println!("{}", serde_json::to_string_pretty(&info)?);
        }

        Commands::Version => {
            println!("guestkit {}", VERSION);
        }
    }

    Ok(())
}
