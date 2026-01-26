// SPDX-License-Identifier: LGPL-3.0-or-later
//! guestctl CLI - Guest VM toolkit

use clap::{CommandFactory, Parser, Subcommand};
use clap_complete::{generate, shells};
use guestctl::{converters::DiskConverter, VERSION};
use std::io;
use std::path::PathBuf;

mod cli;
use cli::commands::*;

/// guestctl - Guest VM toolkit for disk inspection and manipulation
#[derive(Parser)]
#[command(name = "guestctl")]
#[command(version = VERSION)]
#[command(about = "Guest VM toolkit for disk inspection and manipulation", long_about = None)]
struct Cli {
    /// Verbose output
    #[arg(short, long, global = true)]
    verbose: bool,

    /// Debug output (show internal operations)
    #[arg(short, long, global = true)]
    debug: bool,

    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Inspect a disk image and display OS information
    Inspect {
        /// Disk image path
        image: PathBuf,

        /// Output format (text, json, yaml, csv)
        #[arg(short, long, value_name = "FORMAT")]
        output: Option<String>,

        /// Inspection profile (security, migration, performance)
        #[arg(short, long, value_name = "PROFILE")]
        profile: Option<String>,

        /// Export format (html, markdown)
        #[arg(short, long, value_name = "EXPORT_FORMAT")]
        export: Option<String>,

        /// Export output path
        #[arg(long, value_name = "PATH")]
        export_output: Option<PathBuf>,

        /// Disable caching of inspection results (enabled by default)
        #[arg(long)]
        no_cache: bool,

        /// Force refresh cache (ignore existing cached results)
        #[arg(long)]
        cache_refresh: bool,
    },

    /// Diff two disk images to show configuration changes
    Diff {
        /// First disk image
        image1: PathBuf,

        /// Second disk image
        image2: PathBuf,

        /// Output format (text, json, yaml)
        #[arg(short, long, value_name = "FORMAT")]
        output: Option<String>,
    },

    /// Compare multiple VMs against a baseline
    Compare {
        /// Baseline disk image
        baseline: PathBuf,

        /// Disk images to compare
        #[arg(required = true)]
        images: Vec<PathBuf>,
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

    /// Inspect multiple disk images in batch
    #[command(name = "inspect-batch")]
    InspectBatch {
        /// Disk image paths (can use glob patterns)
        #[arg(required = true)]
        images: Vec<PathBuf>,

        /// Number of parallel workers (default: 4)
        #[arg(short, long, default_value = "4")]
        parallel: usize,

        /// Output format (text, json, yaml)
        #[arg(short, long, value_name = "FORMAT")]
        output: Option<String>,

        /// Disable caching of inspection results (enabled by default)
        #[arg(long)]
        no_cache: bool,
    },

    /// Clear inspection cache
    #[command(name = "cache-clear")]
    CacheClear,

    /// Show cache statistics
    #[command(name = "cache-stats")]
    CacheStats,

    /// List filesystems and partitions
    #[command(alias = "fs")]
    Filesystems {
        /// Disk image path
        image: PathBuf,

        /// Show detailed information
        #[arg(short, long)]
        detailed: bool,
    },

    /// List installed packages
    #[command(alias = "pkg")]
    Packages {
        /// Disk image path
        image: PathBuf,

        /// Filter packages by name
        #[arg(short, long)]
        filter: Option<String>,

        /// Limit number of results
        #[arg(short, long)]
        limit: Option<usize>,

        /// Output as JSON
        #[arg(short, long)]
        json: bool,
    },

    /// Read file content from disk image
    Cat {
        /// Disk image path
        image: PathBuf,

        /// Path to file in guest filesystem
        path: String,
    },

    /// Show version information
    Version,

    /// Start interactive mode for exploring disk image
    #[command(alias = "repl")]
    Interactive {
        /// Disk image path
        image: PathBuf,
    },

    /// Execute commands from a script file (batch mode)
    #[command(alias = "batch")]
    Script {
        /// Disk image path
        image: PathBuf,

        /// Script file with commands (one per line)
        script: PathBuf,

        /// Stop on first error
        #[arg(short, long)]
        fail_fast: bool,
    },

    /// Analyze systemd journal logs
    #[command(name = "systemd-journal")]
    SystemdJournal {
        /// Disk image path
        image: PathBuf,

        /// Filter by priority (0=emerg, 3=err, 4=warning, 6=info)
        #[arg(short, long)]
        priority: Option<u8>,

        /// Filter by unit name
        #[arg(short, long)]
        unit: Option<String>,

        /// Show only errors (priority 0-3)
        #[arg(short, long)]
        errors: bool,

        /// Show only warnings (priority 4)
        #[arg(short, long)]
        warnings: bool,

        /// Show statistics
        #[arg(short, long)]
        stats: bool,

        /// Limit number of entries
        #[arg(short, long)]
        limit: Option<usize>,
    },

    /// Analyze systemd services and dependencies
    #[command(name = "systemd-services")]
    SystemdServices {
        /// Disk image path
        image: PathBuf,

        /// Show dependency tree for specific service
        #[arg(short, long)]
        service: Option<String>,

        /// Show only failed services
        #[arg(short, long)]
        failed: bool,

        /// Generate Mermaid diagram for dependencies
        #[arg(short, long)]
        diagram: bool,

        /// Output format (text, json)
        #[arg(short, long, value_name = "FORMAT")]
        output: Option<String>,
    },

    /// Analyze systemd boot performance
    #[command(name = "systemd-boot")]
    SystemdBoot {
        /// Disk image path
        image: PathBuf,

        /// Show boot timeline diagram
        #[arg(short, long)]
        timeline: bool,

        /// Show optimization recommendations
        #[arg(short, long)]
        recommendations: bool,

        /// Show summary statistics
        #[arg(short, long)]
        summary: bool,

        /// Number of slowest services to show
        #[arg(short = 'n', long, default_value = "10")]
        top: usize,
    },

    /// Interactive TUI for VM inspection with orange color theme
    #[command(alias = "ui")]
    Tui {
        /// Disk image path
        image: PathBuf,
    },

    /// Interactive shell for VM inspection (REPL mode)
    #[command(alias = "sh")]
    Shell {
        /// Disk image path
        image: PathBuf,
    },

    /// Generate shell completion scripts
    Completion {
        /// Shell type
        #[arg(value_enum)]
        shell: CompletionShell,
    },
}

#[derive(clap::ValueEnum, Clone)]
enum CompletionShell {
    Bash,
    Zsh,
    Fish,
    PowerShell,
    Elvish,
}

fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();

    // Setup debug environment variable
    if cli.debug {
        std::env::set_var("GUESTCTL_DEBUG", "1");
    }

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
        Commands::Inspect {
            image,
            output,
            profile,
            export,
            export_output,
            no_cache,
            cache_refresh,
        } => {
            use cli::formatters::OutputFormat;
            let output_format = output
                .as_ref()
                .map(|s| s.parse::<OutputFormat>())
                .transpose()
                .map_err(|e| anyhow::anyhow!("{}", e))?;

            inspect_image(
                &image,
                cli.verbose,
                cli.debug,
                output_format,
                profile,
                export,
                export_output,
                !no_cache,  // Cache enabled by default, disabled with --no-cache
                cache_refresh,
            )?;
        }

        Commands::Diff {
            image1,
            image2,
            output,
        } => {
            use cli::formatters::OutputFormat;
            let output_format = output
                .as_ref()
                .map(|s| s.parse::<OutputFormat>())
                .transpose()
                .map_err(|e| anyhow::anyhow!("{}", e))?;

            diff_images(&image1, &image2, cli.verbose, output_format)?;
        }

        Commands::Compare { baseline, images } => {
            compare_images(&baseline, &images, cli.verbose)?;
        }

        Commands::List { image, path } => {
            list_files(&image, &path, cli.verbose)?;
        }

        Commands::Extract {
            image,
            guest_path,
            output,
        } => {
            extract_file(&image, &guest_path, &output, cli.verbose)?;
        }

        Commands::Execute { image, command } => {
            execute_command(&image, &command, cli.verbose)?;
        }

        Commands::Backup {
            image,
            path,
            output,
        } => {
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
            let result = converter.convert(&source, &output, &format, compress, flatten)?;

            if result.success {
                println!("✓ Conversion successful!");
                println!(
                    "  Source:  {} ({})",
                    source.display(),
                    result.source_format.as_str()
                );
                println!(
                    "  Output:  {} ({})",
                    output.display(),
                    result.output_format.as_str()
                );
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

        Commands::InspectBatch {
            images,
            parallel,
            output,
            no_cache,
        } => {
            use cli::formatters::OutputFormat;
            let output_format = output
                .as_ref()
                .map(|s| s.parse::<OutputFormat>())
                .transpose()
                .map_err(|e| anyhow::anyhow!("{}", e))?;

            inspect_batch(&images, parallel, cli.verbose, output_format, !no_cache)?;  // Cache enabled by default
        }

        Commands::CacheClear => {
            use cli::cache::InspectionCache;
            let cache = InspectionCache::new()?;
            let count = cache.clear_all()?;

            println!("✓ Cleared {} cached inspection results", count);
        }

        Commands::CacheStats => {
            use cli::cache::InspectionCache;
            let cache = InspectionCache::new()?;
            let stats = cache.stats()?;

            println!("Cache Statistics:");
            println!("  Entries: {}", stats.entries);
            println!("  Total Size: {}", stats.size_human());
        }

        Commands::Filesystems { image, detailed } => {
            list_filesystems(&image, detailed, cli.verbose)?;
        }

        Commands::Packages {
            image,
            filter,
            limit,
            json,
        } => {
            list_packages(&image, filter, limit, json, cli.verbose)?;
        }

        Commands::Cat { image, path } => {
            cat_file(&image, &path, cli.verbose)?;
        }

        Commands::Version => {
            println!("guestctl {}", VERSION);
            println!("A modern VM disk inspection and manipulation toolkit");
            println!();
            println!("Project: https://github.com/ssahani/guestctl");
            println!("License: LGPL-3.0-or-later");
        }

        Commands::Interactive { image } => {
            let mut session = cli::InteractiveSession::new(image)?;
            session.run()?;
        }

        Commands::Script {
            image,
            script,
            fail_fast,
        } => {
            let mut executor = cli::BatchExecutor::new(image, fail_fast, cli.verbose)?;
            let report = executor.execute_script(&script)?;
            report.print();
            std::process::exit(report.exit_code());
        }

        Commands::SystemdJournal {
            image,
            priority,
            unit,
            errors,
            warnings,
            stats,
            limit,
        } => {
            systemd_journal_command(
                &image,
                priority,
                unit.as_deref(),
                errors,
                warnings,
                stats,
                limit,
                cli.verbose,
            )?;
        }

        Commands::SystemdServices {
            image,
            service,
            failed,
            diagram,
            output,
        } => {
            systemd_services_command(
                &image,
                service.as_deref(),
                failed,
                diagram,
                output.as_deref(),
                cli.verbose,
            )?;
        }

        Commands::SystemdBoot {
            image,
            timeline,
            recommendations,
            summary,
            top,
        } => {
            systemd_boot_command(&image, timeline, recommendations, summary, top, cli.verbose)?;
        }

        Commands::Tui { image } => {
            cli::tui::run_tui(&image)?;
        }

        Commands::Shell { image } => {
            cli::shell::run_interactive_shell(&image)?;
        }

        Commands::Completion { shell } => {
            let mut cmd = Cli::command();
            match shell {
                CompletionShell::Bash => generate(shells::Bash, &mut cmd, "guestctl", &mut io::stdout()),
                CompletionShell::Zsh => generate(shells::Zsh, &mut cmd, "guestctl", &mut io::stdout()),
                CompletionShell::Fish => generate(shells::Fish, &mut cmd, "guestctl", &mut io::stdout()),
                CompletionShell::PowerShell => {
                    generate(shells::PowerShell, &mut cmd, "guestctl", &mut io::stdout())
                }
                CompletionShell::Elvish => generate(shells::Elvish, &mut cmd, "guestctl", &mut io::stdout()),
            }
        }
    }

    Ok(())
}
