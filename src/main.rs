// SPDX-License-Identifier: LGPL-3.0-or-later
//! guestctl CLI - Guest VM toolkit

use clap::{CommandFactory, Parser, Subcommand};
use clap_complete::{generate, shells};
use guestkit::{converters::DiskConverter, VERSION};
use std::io;
use std::path::PathBuf;

mod cli;
use cli::commands::*;
use cli::plan::PlanCommand;

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

    /// Quiet mode (suppress non-error output)
    #[arg(short, long, global = true, conflicts_with = "verbose")]
    quiet: bool,

    /// Disable colored output
    #[arg(long, global = true)]
    no_color: bool,

    /// Read-only mode (prevent any write operations to disk images)
    #[arg(short = 'R', long, global = true)]
    read_only: bool,

    /// Operation timeout in seconds (0 = no timeout)
    #[arg(short = 'T', long, global = true, default_value = "0")]
    timeout: u64,

    /// Custom cache directory path
    #[arg(long, global = true, value_name = "DIR")]
    cache_dir: Option<PathBuf>,

    /// Number of parallel workers for operations that support parallelism
    #[arg(short = 'j', long, global = true, value_name = "N")]
    jobs: Option<usize>,

    /// Show timestamps in output
    #[arg(long, global = true)]
    timestamps: bool,

    /// Output in machine-readable format (implies --no-color)
    #[arg(long, global = true)]
    machine_readable: bool,

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

        /// Inspection profile (security, migration, performance, compliance, hardening)
        #[arg(short, long, value_name = "PROFILE")]
        profile: Option<String>,

        /// Export format (html, markdown, pdf)
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

        /// Show only summary information
        #[arg(short = 'S', long)]
        summary: bool,

        /// Include detailed package list in output
        #[arg(long)]
        include_packages: bool,

        /// Include full service list in output
        #[arg(long)]
        include_services: bool,

        /// Include network configuration details
        #[arg(long)]
        include_network: bool,

        /// Inspection depth (quick, standard, deep)
        #[arg(long, value_name = "DEPTH", default_value = "standard")]
        depth: String,

        /// Save inspection report to file
        #[arg(long, value_name = "FILE")]
        save_report: Option<PathBuf>,
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

        /// Recursive listing
        #[arg(short = 'r', long)]
        recursive: bool,

        /// Show detailed information (permissions, size, owner)
        #[arg(short, long)]
        long: bool,

        /// Show hidden files (starting with .)
        #[arg(short = 'a', long)]
        all: bool,

        /// Human-readable file sizes
        #[arg(short = 'H', long)]
        human_readable: bool,

        /// Sort by modification time
        #[arg(short = 't', long)]
        sort_time: bool,

        /// Reverse sort order
        #[arg(long)]
        reverse: bool,

        /// Filter by file pattern (glob)
        #[arg(short = 'f', long)]
        filter: Option<String>,

        /// Show only directories
        #[arg(short = 'D', long)]
        directories_only: bool,

        /// Limit number of results
        #[arg(short = 'n', long)]
        limit: Option<usize>,
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

        /// Preserve file permissions and timestamps
        #[arg(short, long)]
        preserve: bool,

        /// Extract multiple files (guest_path can be a directory)
        #[arg(short = 'r', long)]
        recursive: bool,

        /// Overwrite existing files without asking
        #[arg(short = 'f', long)]
        force: bool,

        /// Show progress during extraction
        #[arg(long)]
        progress: bool,

        /// Verify extracted file with checksum
        #[arg(long)]
        verify: bool,
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

        /// Show progress bar during conversion
        #[arg(short = 'P', long)]
        progress: bool,

        /// Verify conversion with checksum
        #[arg(long)]
        verify: bool,

        /// Sparse output (don't write zeros)
        #[arg(short = 'S', long)]
        sparse: bool,

        /// Preallocate disk space (faster but uses more space)
        #[arg(long)]
        preallocate: bool,

        /// Compression level (1-9, higher = better compression)
        #[arg(long, value_name = "LEVEL")]
        compression_level: Option<u8>,

        /// Buffer size in MB for I/O operations
        #[arg(long, value_name = "SIZE", default_value = "4")]
        buffer_size: usize,
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

        /// Show line numbers
        #[arg(short = 'n', long)]
        line_numbers: bool,

        /// Show non-printing characters
        #[arg(short = 'A', long)]
        show_all: bool,
    },

    /// Search for files by name or pattern
    #[command(alias = "find")]
    Search {
        /// Disk image path
        image: PathBuf,

        /// Search pattern (glob or regex)
        pattern: String,

        /// Starting directory for search
        #[arg(short, long, default_value = "/")]
        path: String,

        /// Use regex instead of glob
        #[arg(short = 'E', long)]
        regex: bool,

        /// Case-insensitive search
        #[arg(short = 'i', long)]
        ignore_case: bool,

        /// Search file content, not just names
        #[arg(short = 'c', long)]
        content: bool,

        /// File type filter (file, dir, link, socket, etc.)
        #[arg(short = 't', long)]
        file_type: Option<String>,

        /// Maximum search depth
        #[arg(short = 'D', long)]
        max_depth: Option<usize>,

        /// Limit number of results
        #[arg(short = 'l', long)]
        limit: Option<usize>,
    },

    /// Search file contents (like grep)
    Grep {
        /// Disk image path
        image: PathBuf,

        /// Search pattern
        pattern: String,

        /// File or directory to search in
        #[arg(default_value = "/")]
        path: String,

        /// Case-insensitive search
        #[arg(short = 'i', long)]
        ignore_case: bool,

        /// Show line numbers
        #[arg(short = 'n', long)]
        line_numbers: bool,

        /// Recursive search
        #[arg(short = 'r', long)]
        recursive: bool,

        /// Show only matching filenames
        #[arg(short = 'l', long)]
        files_only: bool,

        /// Invert match (show non-matching lines)
        #[arg(short = 'V', long)]
        invert: bool,

        /// Context lines before match
        #[arg(short = 'B', long, value_name = "NUM")]
        before_context: Option<usize>,

        /// Context lines after match
        #[arg(short = 'A', long, value_name = "NUM")]
        after_context: Option<usize>,

        /// Maximum results
        #[arg(short = 'm', long)]
        max_count: Option<usize>,
    },

    /// Calculate file checksums
    Hash {
        /// Disk image path
        image: PathBuf,

        /// Path to file in guest filesystem
        path: String,

        /// Hash algorithm (md5, sha1, sha256, sha512)
        #[arg(short = 'a', long, default_value = "sha256")]
        algorithm: String,

        /// Verify against expected hash
        #[arg(short = 'c', long)]
        check: Option<String>,

        /// Recursive hashing for directories
        #[arg(short = 'r', long)]
        recursive: bool,
    },

    /// Security vulnerability scan
    Scan {
        /// Disk image path
        image: PathBuf,

        /// Scan type (packages, config, permissions, all)
        #[arg(short = 't', long, default_value = "all")]
        scan_type: String,

        /// Severity threshold (low, medium, high, critical)
        #[arg(short = 's', long)]
        severity: Option<String>,

        /// Output format (text, json, sarif)
        #[arg(short, long, value_name = "FORMAT")]
        output: Option<String>,

        /// Generate detailed report
        #[arg(short = 'r', long)]
        report: bool,

        /// Check CVE database for vulnerabilities
        #[arg(long)]
        check_cve: bool,
    },

    /// Benchmark disk I/O performance
    Benchmark {
        /// Disk image path
        image: PathBuf,

        /// Test type (read, write, random, sequential)
        #[arg(short = 't', long, default_value = "all")]
        test_type: String,

        /// Block size for I/O operations (in KB)
        #[arg(short = 'b', long, default_value = "4")]
        block_size: usize,

        /// Duration of test in seconds
        #[arg(long, default_value = "10")]
        duration: u64,

        /// Number of iterations
        #[arg(short = 'n', long, default_value = "3")]
        iterations: usize,
    },

    /// Manage disk snapshots
    Snapshot {
        /// Disk image path
        image: PathBuf,

        /// Snapshot operation (create, list, delete, revert)
        #[arg(value_enum)]
        operation: SnapshotOperation,

        /// Snapshot name
        #[arg(short = 'n', long)]
        name: Option<String>,

        /// Snapshot description
        #[arg(long)]
        description: Option<String>,
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

    /// AI-powered diagnostics and assistance (requires --features ai and OPENAI_API_KEY)
    Ai {
        /// Disk image path
        image: PathBuf,

        /// Question or problem description
        #[arg(required = true)]
        query: String,
    },

    /// Generate shell completion scripts
    Completion {
        /// Shell type
        #[arg(value_enum)]
        shell: CompletionShell,
    },

    /// Manage fix plans (preview, validate, export, apply)
    Plan(PlanCommand),
}

#[derive(clap::ValueEnum, Clone)]
enum CompletionShell {
    Bash,
    Zsh,
    Fish,
    PowerShell,
    Elvish,
}

#[derive(clap::ValueEnum, Clone, Debug)]
enum SnapshotOperation {
    Create,
    List,
    Delete,
    Revert,
    Info,
}

fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();

    // Setup global environment variables
    if cli.debug {
        // SAFETY: Setting an environment variable in single-threaded initialization is safe
        unsafe {
            std::env::set_var("GUESTCTL_DEBUG", "1");
        }
    }

    if cli.no_color || cli.machine_readable {
        // SAFETY: Setting an environment variable in single-threaded initialization is safe
        unsafe {
            std::env::set_var("NO_COLOR", "1");
        }
    }

    if cli.read_only {
        // SAFETY: Setting an environment variable in single-threaded initialization is safe
        unsafe {
            std::env::set_var("GUESTCTL_READONLY", "1");
        }
    }

    if let Some(ref cache_dir) = cli.cache_dir {
        // SAFETY: Setting an environment variable in single-threaded initialization is safe
        unsafe {
            std::env::set_var("GUESTCTL_CACHE_DIR", cache_dir.to_str().unwrap_or_default());
        }
    }

    if cli.timeout > 0 {
        // SAFETY: Setting an environment variable in single-threaded initialization is safe
        unsafe {
            std::env::set_var("GUESTCTL_TIMEOUT", cli.timeout.to_string());
        }
    }

    // Setup logging
    let log_level = if cli.quiet {
        log::LevelFilter::Error
    } else if cli.verbose {
        log::LevelFilter::Debug
    } else {
        log::LevelFilter::Info
    };

    let mut logger = env_logger::Builder::new();
    logger.filter_level(log_level);

    if cli.timestamps {
        logger.format_timestamp_secs();
    } else {
        logger.format_timestamp(None);
    }

    logger.init();

    match cli.command {
        Commands::Inspect {
            image,
            output,
            profile,
            export,
            export_output,
            no_cache,
            cache_refresh,
            summary: _,
            include_packages: _,
            include_services: _,
            include_network: _,
            depth: _,
            save_report: _,
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

        Commands::List {
            image,
            path,
            recursive,
            long,
            all,
            human_readable,
            sort_time,
            reverse,
            filter,
            directories_only,
            limit,
        } => {
            list_files_enhanced(
                &image,
                &path,
                recursive,
                long,
                all,
                human_readable,
                sort_time,
                reverse,
                filter,
                directories_only,
                limit,
                cli.verbose,
            )?;
        }

        Commands::Extract {
            image,
            guest_path,
            output,
            preserve,
            recursive,
            force,
            progress,
            verify,
        } => {
            extract_file_enhanced(
                &image,
                &guest_path,
                &output,
                preserve,
                recursive,
                force,
                progress,
                verify,
                cli.verbose,
            )?;
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
            progress: _,
            verify: _,
            sparse: _,
            preallocate: _,
            compression_level: _,
            buffer_size: _,
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

        Commands::Cat {
            image,
            path,
            line_numbers,
            show_all,
        } => {
            cat_file_enhanced(&image, &path, line_numbers, show_all, cli.verbose)?;
        }

        Commands::Search {
            image,
            pattern,
            path,
            regex,
            ignore_case,
            content,
            file_type,
            max_depth,
            limit,
        } => {
            search_command(
                &image,
                &pattern,
                &path,
                regex,
                ignore_case,
                content,
                file_type,
                max_depth,
                limit,
                cli.verbose,
            )?;
        }

        Commands::Grep {
            image,
            pattern,
            path,
            ignore_case,
            line_numbers,
            recursive,
            files_only,
            invert,
            before_context,
            after_context,
            max_count,
        } => {
            grep_command(
                &image,
                &pattern,
                &path,
                ignore_case,
                line_numbers,
                recursive,
                files_only,
                invert,
                before_context,
                after_context,
                max_count,
                cli.verbose,
            )?;
        }

        Commands::Hash {
            image,
            path,
            algorithm,
            check,
            recursive,
        } => {
            hash_command(&image, &path, &algorithm, check, recursive, cli.verbose)?;
        }

        Commands::Scan {
            image,
            scan_type,
            severity,
            output,
            report,
            check_cve,
        } => {
            scan_command(&image, &scan_type, severity, output, report, check_cve, cli.verbose)?;
        }

        Commands::Benchmark {
            image,
            test_type,
            block_size,
            duration,
            iterations,
        } => {
            benchmark_command(&image, &test_type, block_size, duration, iterations, cli.verbose)?;
        }

        Commands::Snapshot {
            image,
            operation,
            name,
            description,
        } => {
            let op_str = match operation {
                SnapshotOperation::Create => "create",
                SnapshotOperation::List => "list",
                SnapshotOperation::Delete => "delete",
                SnapshotOperation::Revert => "revert",
                SnapshotOperation::Info => "info",
            };
            snapshot_command(&image, op_str, name, description, cli.verbose)?;
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

        Commands::Ai { image, query } => {
            cli::ai::run_ai_assistant(&image, &query)?;
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

        Commands::Plan(plan_cmd) => {
            plan_cmd.execute()?;
        }
    }

    Ok(())
}
