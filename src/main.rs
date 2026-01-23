// SPDX-License-Identifier: LGPL-3.0-or-later
//! guestkit CLI - Guest VM toolkit

use clap::{Parser, Subcommand};
use guestkit::{converters::DiskConverter, VERSION};
use std::path::PathBuf;

mod cli;
use cli::commands::*;

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
    /// Inspect a disk image and display OS information
    Inspect {
        /// Disk image path
        image: PathBuf,
    },

    /// List files in a disk image
    #[command(alias = "ls")]
    List {
        /// Disk image path
        image: PathBuf,

        /// Path to list (default: /)
        #[arg(default_value = "/")]
        path: String,
    },

    /// Extract a file from disk image
    #[command(alias = "get")]
    Extract {
        /// Disk image path
        image: PathBuf,

        /// Path in guest filesystem
        guest_path: String,

        /// Output file path on host
        #[arg(short, long)]
        output: PathBuf,
    },

    /// Execute a command in the guest
    #[command(alias = "exec")]
    Execute {
        /// Disk image path
        image: PathBuf,

        /// Command and arguments to execute
        #[arg(trailing_var_arg = true, required = true)]
        command: Vec<String>,
    },

    /// Backup files from guest to tar archive
    Backup {
        /// Disk image path
        image: PathBuf,

        /// Path to backup in guest
        #[arg(default_value = "/")]
        path: String,

        /// Output tar.gz file
        #[arg(short, long)]
        output: PathBuf,
    },

    /// Convert disk image format
    Convert {
        /// Source disk image path
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
        #[arg(short = 'F', long)]
        flatten: bool,
    },

    /// Create a new disk image
    Create {
        /// Output disk image path
        path: PathBuf,

        /// Size in megabytes
        #[arg(short, long)]
        size: u64,

        /// Disk format (raw, qcow2, vmdk, vhd, vdi)
        #[arg(short, long, default_value = "raw")]
        format: String,
    },

    /// Check filesystem on a disk image
    #[command(alias = "fsck")]
    Check {
        /// Disk image path
        image: PathBuf,

        /// Specific device to check (optional)
        #[arg(short, long)]
        device: Option<String>,
    },

    /// Show disk usage statistics
    #[command(alias = "df")]
    Usage {
        /// Disk image path
        image: PathBuf,
    },

    /// Detect disk image format
    Detect {
        /// Disk image path
        image: PathBuf,
    },

    /// Get disk image information
    Info {
        /// Disk image path
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
        Commands::Inspect { image } => {
            inspect_image(&image, cli.verbose)?;
        }

        Commands::List { image, path } => {
            list_files(&image, &path, cli.verbose)?;
        }

        Commands::Extract { image, guest_path, output } => {
            extract_file(&image, &guest_path, &output, cli.verbose)?;
        }

        Commands::Execute { image, command } => {
            execute_command(&image, &command, cli.verbose)?;
        }

        Commands::Backup { image, path, output } => {
            backup_files(&image, &path, &output, cli.verbose)?;
        }

        Commands::Create { path, size, format } => {
            create_disk(&path, size, &format, cli.verbose)?;
        }

        Commands::Check { image, device } => {
            check_filesystem(&image, device, cli.verbose)?;
        }

        Commands::Usage { image } => {
            show_disk_usage(&image, cli.verbose)?;
        }

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
            println!("A pure Rust implementation of libguestfs-compatible APIs");
            println!();
            println!("Project: https://github.com/ssahani/guestkit");
            println!("License: LGPL-3.0-or-later");
        }
    }

    Ok(())
}
