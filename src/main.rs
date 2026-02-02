// SPDX-License-Identifier: LGPL-3.0-or-later
//! guestctl CLI - Guest VM toolkit

use anyhow::Context;
use clap::{CommandFactory, Parser, Subcommand};
use clap_complete::{generate, shells};
use colored::Colorize;
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

    /// Compare specific files between disk images
    DiffFiles {
        /// First disk image
        image1: PathBuf,

        /// Second disk image
        image2: PathBuf,

        /// Path to compare
        #[arg(default_value = "/")]
        path: String,

        /// Unified diff format
        #[arg(short = 'u', long)]
        unified: bool,

        /// Context lines
        #[arg(short = 'C', long, default_value = "3")]
        context: usize,

        /// Ignore whitespace differences
        #[arg(short = 'w', long)]
        ignore_whitespace: bool,
    },

    /// Find large files in disk image
    FindLarge {
        /// Disk image path
        image: PathBuf,

        /// Starting path
        #[arg(default_value = "/")]
        path: String,

        /// Minimum file size in bytes
        #[arg(short = 's', long, default_value = "10485760")]
        min_size: u64,

        /// Maximum number of results
        #[arg(short = 'n', long, default_value = "20")]
        max_results: usize,

        /// Human-readable sizes
        #[arg(short = 'H', long)]
        human_readable: bool,
    },

    /// Copy files between disk images
    Copy {
        /// Source disk image
        source_image: PathBuf,

        /// Source file path
        source_path: String,

        /// Destination disk image
        dest_image: PathBuf,

        /// Destination file path
        dest_path: String,

        /// Preserve permissions and timestamps
        #[arg(short = 'p', long)]
        preserve: bool,

        /// Force overwrite if destination exists
        #[arg(short = 'f', long)]
        force: bool,
    },

    /// Find duplicate files
    FindDuplicates {
        /// Disk image path
        image: PathBuf,

        /// Starting path
        #[arg(default_value = "/")]
        path: String,

        /// Minimum file size to consider
        #[arg(short = 's', long, default_value = "1048576")]
        min_size: u64,

        /// Hash algorithm
        #[arg(short = 'a', long, default_value = "sha256")]
        algorithm: String,
    },

    /// Analyze disk usage by directory
    DiskUsage {
        /// Disk image path
        image: PathBuf,

        /// Starting path
        #[arg(default_value = "/")]
        path: String,

        /// Maximum directory depth
        #[arg(short = 'D', long, default_value = "5")]
        max_depth: usize,

        /// Minimum size to display
        #[arg(short = 's', long, default_value = "1048576")]
        min_size: u64,

        /// Human-readable sizes
        #[arg(short = 'H', long)]
        human_readable: bool,
    },

    /// Build forensic timeline from multiple sources
    Timeline {
        /// Disk image path
        image: PathBuf,

        /// Start time filter (ISO 8601)
        #[arg(long)]
        start_time: Option<String>,

        /// End time filter (ISO 8601)
        #[arg(long)]
        end_time: Option<String>,

        /// Data sources (files, packages, logs)
        #[arg(short = 's', long, value_delimiter = ',')]
        sources: Vec<String>,

        /// Output format (text, json, csv)
        #[arg(short = 'f', long, default_value = "text")]
        format: String,
    },

    /// Create unique fingerprint for disk image
    Fingerprint {
        /// Disk image path
        image: PathBuf,

        /// Hash algorithm
        #[arg(short = 'a', long, default_value = "sha256")]
        algorithm: String,

        /// Include file content hashes
        #[arg(short = 'c', long)]
        include_content: bool,

        /// Output file path
        #[arg(short = 'o', long)]
        output: Option<PathBuf>,
    },

    /// Detect configuration drift from baseline
    Drift {
        /// Baseline disk image
        baseline: PathBuf,

        /// Current disk image to compare
        current: PathBuf,

        /// Paths to ignore (comma-separated)
        #[arg(long, value_delimiter = ',')]
        ignore_paths: Vec<String>,

        /// Drift threshold percentage (0-100)
        #[arg(short = 't', long, default_value = "20")]
        threshold: u8,

        /// Generate detailed report
        #[arg(short = 'r', long)]
        report: bool,
    },

    /// AI-powered deep analysis with insights
    Analyze {
        /// Disk image path
        image: PathBuf,

        /// Analysis focus areas (security, performance, compliance, maintainability)
        #[arg(short = 'f', long, value_delimiter = ',')]
        focus: Vec<String>,

        /// Analysis depth (quick, standard, deep)
        #[arg(long, default_value = "standard")]
        depth: String,

        /// Show actionable suggestions
        #[arg(short = 's', long)]
        suggestions: bool,
    },

    /// Scan for exposed secrets and credentials
    Secrets {
        /// Disk image path
        image: PathBuf,

        /// Paths to scan (comma-separated)
        #[arg(long, value_delimiter = ',')]
        scan_paths: Vec<String>,

        /// Custom regex patterns to search for
        #[arg(short = 'p', long, value_delimiter = ',')]
        patterns: Vec<String>,

        /// Paths to exclude (comma-separated)
        #[arg(short = 'e', long, value_delimiter = ',')]
        exclude: Vec<String>,

        /// Show actual secret content (WARNING: sensitive)
        #[arg(long)]
        show_content: bool,

        /// Export report to file
        #[arg(short = 'o', long)]
        export: Option<PathBuf>,
    },

    /// Automated rescue and recovery operations
    Rescue {
        /// Disk image path
        image: PathBuf,

        /// Rescue operation (reset-password, fix-fstab, fix-grub, enable-ssh)
        #[arg(short = 'o', long)]
        operation: String,

        /// Username (for reset-password)
        #[arg(short = 'u', long)]
        user: Option<String>,

        /// New password (for reset-password)
        #[arg(short = 'p', long)]
        password: Option<String>,

        /// Force operation even if risky
        #[arg(short = 'f', long)]
        force: bool,

        /// Backup files before modification
        #[arg(short = 'b', long)]
        backup: bool,
    },

    /// Optimize disk image (cleanup, compact)
    Optimize {
        /// Disk image path
        image: PathBuf,

        /// Operations to perform (temp, logs, cache, packages)
        #[arg(short = 'o', long, value_delimiter = ',')]
        operations: Vec<String>,

        /// Aggressive cleanup (may remove more files)
        #[arg(short = 'a', long)]
        aggressive: bool,

        /// Dry run (show what would be removed)
        #[arg(long)]
        dry_run: bool,
    },

    /// Analyze network configuration
    Network {
        /// Disk image path
        image: PathBuf,

        /// Show routing information
        #[arg(long)]
        show_routes: bool,

        /// Show network interfaces
        #[arg(long)]
        show_interfaces: bool,

        /// Show DNS configuration
        #[arg(long)]
        show_dns: bool,

        /// Export as JSON
        #[arg(short = 'j', long)]
        export_json: bool,
    },

    /// Compliance checking against security standards
    Compliance {
        /// Disk image path
        image: PathBuf,

        /// Security standard (cis, pci-dss, hipaa)
        #[arg(short = 's', long)]
        standard: String,

        /// Compliance profile (e.g., level1, level2)
        #[arg(short = 'p', long)]
        profile: Option<String>,

        /// Export report to file
        #[arg(short = 'e', long)]
        export: Option<PathBuf>,

        /// Attempt to fix issues
        #[arg(short = 'f', long)]
        fix: bool,
    },

    /// Malware and rootkit detection
    Malware {
        /// Disk image path
        image: PathBuf,

        /// Deep scan (more thorough but slower)
        #[arg(long)]
        deep_scan: bool,

        /// Check for rootkit indicators
        #[arg(long)]
        check_rootkits: bool,

        /// YARA rules file
        #[arg(long)]
        yara_rules: Option<PathBuf>,

        /// Quarantine suspicious files
        #[arg(short = 'q', long)]
        quarantine: bool,
    },

    /// System health and diagnostics
    Health {
        /// Disk image path
        image: PathBuf,

        /// Specific checks to run (disk, services, security, packages, logs)
        #[arg(short = 'c', long, value_delimiter = ',')]
        checks: Vec<String>,

        /// Show detailed information
        #[arg(long)]
        detailed: bool,

        /// Export as JSON
        #[arg(short = 'j', long)]
        export_json: Option<PathBuf>,
    },

    /// Clone disk image with customizations
    Clone {
        /// Source disk image
        source: PathBuf,

        /// Destination disk image
        dest: PathBuf,

        /// Run sysprep (generalize image)
        #[arg(short = 's', long)]
        sysprep: bool,

        /// Set new hostname
        #[arg(long)]
        hostname: Option<String>,

        /// Remove SSH host keys
        #[arg(long)]
        remove_keys: bool,

        /// Preserve user accounts and history
        #[arg(long)]
        preserve_users: bool,
    },

    /// Security patch analysis and CVE detection
    Patch {
        /// Disk image path
        image: PathBuf,

        /// Check for CVEs in installed packages
        #[arg(long)]
        check_cves: bool,

        /// Filter by severity (CRITICAL, HIGH, MEDIUM, LOW, ALL)
        #[arg(short = 's', long)]
        severity: Option<String>,

        /// Export report to file
        #[arg(short = 'e', long)]
        export: Option<PathBuf>,

        /// Simulate package updates
        #[arg(long)]
        simulate_update: bool,
    },

    /// Generate Software Bill of Materials (SBOM)
    Inventory {
        /// Disk image path
        image: PathBuf,

        /// Output format (spdx, cyclonedx, json, csv)
        #[arg(short = 'f', long, value_name = "FORMAT", default_value = "spdx")]
        format: String,

        /// Output file (stdout if not specified)
        #[arg(short, long, value_name = "FILE")]
        output: Option<PathBuf>,

        /// Include license information
        #[arg(long)]
        include_licenses: bool,

        /// Include file manifests
        #[arg(long)]
        include_files: bool,

        /// Include CVE mappings
        #[arg(long)]
        include_cves: bool,

        /// Filter CVEs by severity (critical, high, medium, low)
        #[arg(long, value_name = "SEVERITY")]
        severity: Option<String>,

        /// Show summary before export
        #[arg(short = 'S', long)]
        summary: bool,
    },

    /// Validate disk image against policy
    Validate {
        /// Disk image path
        image: PathBuf,

        /// Policy file path (YAML)
        #[arg(short, long, value_name = "FILE")]
        policy: Option<PathBuf>,

        /// Use industry benchmark (cis-ubuntu, cis-rhel, nist, pci, hipaa)
        #[arg(short, long, value_name = "BENCHMARK")]
        benchmark: Option<String>,

        /// Generate example policy file
        #[arg(long)]
        example_policy: bool,

        /// Output format (text, json)
        #[arg(short = 'f', long, value_name = "FORMAT", default_value = "text")]
        format: String,

        /// Output file (stdout if not specified)
        #[arg(short, long, value_name = "FILE")]
        output: Option<PathBuf>,

        /// Fail on any validation failure
        #[arg(long)]
        strict: bool,
    },

    /// Comprehensive security audit with detailed reporting
    Audit {
        /// Disk image path
        image: PathBuf,

        /// Audit categories (permissions, users, network, services)
        #[arg(short = 'c', long, value_delimiter = ',')]
        categories: Vec<String>,

        /// Output format (text, json)
        #[arg(short = 'f', long, default_value = "text")]
        output_format: String,

        /// Export report to file
        #[arg(short = 'e', long)]
        export: Option<PathBuf>,

        /// Attempt to fix issues automatically
        #[arg(long)]
        fix_issues: bool,
    },

    /// Automated system repair operations
    Repair {
        /// Disk image path
        image: PathBuf,

        /// Repair type (permissions, packages, network, bootloader, filesystem)
        #[arg(short = 't', long)]
        repair_type: String,

        /// Force repair even if risky
        #[arg(short = 'f', long)]
        force: bool,

        /// Backup before repair
        #[arg(short = 'b', long)]
        backup: bool,
    },

    /// System hardening configuration
    Harden {
        /// Disk image path
        image: PathBuf,

        /// Hardening profile (basic, moderate, strict)
        #[arg(short = 'p', long, default_value = "basic")]
        profile: String,

        /// Apply hardening (default is dry-run)
        #[arg(short = 'a', long)]
        apply: bool,

        /// Preview changes without applying
        #[arg(long)]
        preview: bool,
    },

    /// AI-powered anomaly detection
    Anomaly {
        /// Disk image path
        image: PathBuf,

        /// Baseline image for comparison
        #[arg(short = 'b', long)]
        baseline: Option<PathBuf>,

        /// Detection sensitivity (low, medium, high)
        #[arg(short = 's', long, default_value = "medium")]
        sensitivity: String,

        /// Categories to check (files, config, processes, network)
        #[arg(short = 'c', long, value_delimiter = ',')]
        categories: Vec<String>,

        /// Export report to file
        #[arg(short = 'e', long)]
        export: Option<PathBuf>,
    },

    /// Smart recommendations engine
    Recommend {
        /// Disk image path
        image: PathBuf,

        /// Focus areas (security, performance, reliability, cost)
        #[arg(short = 'f', long, value_delimiter = ',')]
        focus: Vec<String>,

        /// Priority filter (critical, high, medium, low)
        #[arg(short = 'p', long, default_value = "medium")]
        priority: String,

        /// Auto-apply safe recommendations
        #[arg(short = 'a', long)]
        apply: bool,
    },

    /// Dependency graph and impact analysis
    Dependencies {
        /// Disk image path
        image: PathBuf,

        /// Specific target to analyze
        #[arg(short = 't', long)]
        target: Option<String>,

        /// Graph type (packages, services, network)
        #[arg(short = 'g', long, default_value = "packages")]
        graph_type: String,

        /// Export to Graphviz DOT format
        #[arg(long)]
        export_dot: Option<PathBuf>,
    },

    /// Predictive analysis and capacity planning
    Predict {
        /// Disk image path
        image: PathBuf,

        /// Metric to predict (disk-growth, log-growth, package-updates)
        #[arg(short = 'm', long, default_value = "disk-growth")]
        metric: String,

        /// Forecast timeframe in days
        #[arg(short = 't', long, default_value = "30")]
        timeframe: u32,

        /// Export report to file
        #[arg(short = 'e', long)]
        export: Option<PathBuf>,
    },

    /// Threat intelligence correlation and IOC detection
    Intelligence {
        /// Disk image path
        image: PathBuf,

        /// Custom IOC file (STIX, OpenIOC, or CSV format)
        #[arg(short = 'i', long)]
        ioc_file: Option<PathBuf>,

        /// Threat level filter (critical, high, medium, low)
        #[arg(short = 'l', long, default_value = "medium")]
        threat_level: String,

        /// Enable correlation analysis
        #[arg(short = 'c', long)]
        correlate: bool,

        /// Export report to file
        #[arg(short = 'e', long)]
        export: Option<PathBuf>,
    },

    /// Change simulation and impact modeling
    Simulate {
        /// Disk image path
        image: PathBuf,

        /// Change type (remove-package, modify-config, disable-service, kernel-update)
        #[arg(short = 't', long)]
        change_type: String,

        /// Target (package name, config file, service name, etc.)
        #[arg(short = 'T', long)]
        target: String,

        /// Dry run - simulate without making changes
        #[arg(short = 'd', long, default_value = "true")]
        dry_run: bool,

        /// Include comprehensive risk assessment
        #[arg(short = 'r', long)]
        risk_assessment: bool,
    },

    /// Comprehensive multi-dimensional risk scoring
    Score {
        /// Disk image path
        image: PathBuf,

        /// Risk dimensions to check (security, compliance, reliability, performance, maintainability)
        #[arg(short = 'd', long, value_delimiter = ',')]
        dimensions: Vec<String>,

        /// Custom weights (format: security=40,compliance=30,...)
        #[arg(short = 'w', long)]
        weights: Option<String>,

        /// Compare against benchmark image
        #[arg(short = 'b', long)]
        benchmark: Option<PathBuf>,

        /// Export report to file
        #[arg(short = 'e', long)]
        export: Option<PathBuf>,
    },

    /// Golden image template validation
    Template {
        /// Disk image path
        image: PathBuf,

        /// Template type (web-server, database, docker-host, cis-level1)
        #[arg(short = 't', long)]
        template: String,

        /// Strict mode - fail on any violation
        #[arg(short = 's', long)]
        strict: bool,

        /// Automatically fix violations where possible
        #[arg(short = 'f', long)]
        fix: bool,

        /// Export template definition to file
        #[arg(short = 'e', long)]
        export_template: Option<PathBuf>,
    },

    /// Proactive threat hunting with hypothesis-driven investigation
    Hunt {
        /// Disk image path
        image: PathBuf,

        /// Threat hunting hypothesis
        #[arg(short = 'H', long)]
        hypothesis: String,

        /// Hunting framework (mitre-attack, custom)
        #[arg(short = 'f', long, default_value = "mitre-attack")]
        framework: String,

        /// Specific techniques to hunt (comma-separated tactics)
        #[arg(short = 't', long, value_delimiter = ',')]
        techniques: Vec<String>,

        /// Hunt depth (surface, shallow, deep, comprehensive)
        #[arg(short = 'D', long, default_value = "deep")]
        depth: String,

        /// Export hunt report to file
        #[arg(short = 'e', long)]
        export: Option<PathBuf>,
    },

    /// Forensic incident reconstruction and attack path visualization
    Reconstruct {
        /// Disk image path
        image: PathBuf,

        /// Incident type (compromise, data-exfiltration, ransomware, generic)
        #[arg(short = 't', long)]
        incident_type: String,

        /// Start time for analysis window
        #[arg(short = 's', long)]
        start_time: Option<String>,

        /// End time for analysis window
        #[arg(short = 'E', long)]
        end_time: Option<String>,

        /// Generate attack path visualization
        #[arg(short = 'V', long)]
        visualize: bool,

        /// Export reconstruction report to file
        #[arg(short = 'e', long)]
        export: Option<PathBuf>,
    },

    /// Automated progressive system evolution and self-improvement
    Evolve {
        /// Disk image path
        image: PathBuf,

        /// Target state (hardened, optimized, compliant, production-ready)
        #[arg(short = 't', long)]
        target_state: String,

        /// Evolution strategy (aggressive, balanced, conservative)
        #[arg(short = 's', long, default_value = "balanced")]
        strategy: String,

        /// Number of evolution stages
        #[arg(short = 'S', long, default_value = "3")]
        stages: u32,

        /// Enable safety checks and rollback plans
        #[arg(short = 'c', long)]
        safety_checks: bool,

        /// Export evolution plan to file
        #[arg(short = 'e', long)]
        export_plan: Option<PathBuf>,
    },

    /// Zero-trust continuous verification and supply chain integrity
    Verify {
        /// Disk image path
        image: PathBuf,

        /// Verification level (basic, standard, strict, paranoid)
        #[arg(short = 'l', long, default_value = "standard")]
        verification_level: String,

        /// Check supply chain integrity
        #[arg(short = 's', long)]
        check_supply_chain: bool,

        /// Verify identity and accounts
        #[arg(short = 'i', long)]
        check_identity: bool,

        /// Verify file integrity
        #[arg(short = 'I', long)]
        check_integrity: bool,

        /// Export verification report to file
        #[arg(short = 'e', long)]
        export: Option<PathBuf>,
    },

    /// Show version information
    Version,

    /// Start interactive mode for exploring disk image
    #[command(alias = "repl")]
    Interactive {
        /// Disk image path
        image: PathBuf,
    },

    /// Launch interactive file explorer (TUI mode)
    #[command(alias = "ex")]
    Explore {
        /// Disk image path
        image: PathBuf,

        /// Starting path in VM filesystem (default: /)
        #[arg(default_value = "/")]
        path: String,
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

/// Run standalone file explorer (direct from CLI)
fn run_standalone_explorer(image_path: &PathBuf, start_path: &str, verbose: bool) -> anyhow::Result<()> {
    use guestkit::Guestfs;
    use cli::shell::commands::ShellContext;
    use cli::shell::explore::run_explorer;

    if verbose {
        println!("{} Loading VM image: {}", "→".cyan(), image_path.display());
    }

    // Initialize guestfs
    let mut guestfs = Guestfs::new()
        .context("Failed to create Guestfs handle")?;

    guestfs.add_drive_opts(
        image_path.to_str().unwrap(),
        false,
        None
    ).context("Failed to add drive")?;

    guestfs.launch().context("Failed to launch guestfs")?;

    // Inspect and mount
    let roots = guestfs.inspect_os()
        .context("Failed to inspect OS")?;

    if roots.is_empty() {
        anyhow::bail!("No operating systems found in disk image");
    }

    let root = &roots[0];

    if verbose {
        println!("{} Detected OS: {}", "→".cyan(), root.yellow());
    }

    let mounts = guestfs.inspect_get_mountpoints(root)
        .context("Failed to get mountpoints")?;

    for (mountpoint, device) in mounts {
        if let Err(e) = guestfs.mount(&device, &mountpoint) {
            eprintln!("{} Failed to mount {}: {}", "⚠".yellow(), mountpoint, e);
        }
    }

    if verbose {
        println!("{} VM filesystem mounted successfully", "✓".green());
    }

    // Get OS information for context
    let os_product = guestfs.inspect_get_product_name(&root)
        .unwrap_or_else(|_| "Unknown OS".to_string());

    // Create shell context for explorer
    let mut ctx = ShellContext::new(guestfs, root.to_string());
    ctx.set_os_info(os_product);
    ctx.current_path = start_path.to_string();

    // Launch explorer
    println!("\n{}", "╔═══════════════════════════════════════════════════════════╗".cyan());
    println!("{}", "║          GuestKit File Explorer (TUI Mode)              ║".cyan().bold());
    println!("{}", "╚═══════════════════════════════════════════════════════════╝".cyan());
    println!();
    println!("{} Press 'h' for help, 'q' to quit", "ℹ".yellow());
    println!();

    std::thread::sleep(std::time::Duration::from_millis(800));

    run_explorer(&mut ctx, Some(start_path))?;

    println!("\n{} Explorer closed", "✓".green());

    Ok(())
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

        Commands::DiffFiles {
            image1,
            image2,
            path,
            unified,
            context,
            ignore_whitespace,
        } => {
            diff_command(&image1, &image2, &path, unified, context, ignore_whitespace, cli.verbose)?;
        }

        Commands::FindLarge {
            image,
            path,
            min_size,
            max_results,
            human_readable,
        } => {
            find_large_command(&image, &path, min_size, max_results, human_readable, cli.verbose)?;
        }

        Commands::Copy {
            source_image,
            source_path,
            dest_image,
            dest_path,
            preserve,
            force,
        } => {
            copy_command(&source_image, &source_path, &dest_image, &dest_path, preserve, force, cli.verbose)?;
        }

        Commands::FindDuplicates {
            image,
            path,
            min_size,
            algorithm,
        } => {
            find_duplicates_command(&image, &path, min_size, &algorithm, cli.verbose)?;
        }

        Commands::DiskUsage {
            image,
            path,
            max_depth,
            min_size,
            human_readable,
        } => {
            disk_usage_command(&image, &path, max_depth, min_size, human_readable, cli.verbose)?;
        }

        Commands::Timeline {
            image,
            start_time,
            end_time,
            sources,
            format,
        } => {
            timeline_command(&image, start_time, end_time, sources, &format, cli.verbose)?;
        }

        Commands::Fingerprint {
            image,
            algorithm,
            include_content,
            output,
        } => {
            fingerprint_command(&image, &algorithm, include_content, output, cli.verbose)?;
        }

        Commands::Drift {
            baseline,
            current,
            ignore_paths,
            threshold,
            report,
        } => {
            drift_command(&baseline, &current, ignore_paths, threshold, report, cli.verbose)?;
        }

        Commands::Analyze {
            image,
            focus,
            depth,
            suggestions,
        } => {
            analyze_command(&image, focus, &depth, suggestions, cli.verbose)?;
        }

        Commands::Secrets {
            image,
            scan_paths,
            patterns,
            exclude,
            show_content,
            export,
        } => {
            secrets_command(&image, scan_paths, patterns, exclude, show_content, export, cli.verbose)?;
        }

        Commands::Rescue {
            image,
            operation,
            user,
            password,
            force,
            backup,
        } => {
            rescue_command(&image, &operation, user, password, force, backup, cli.verbose)?;
        }

        Commands::Optimize {
            image,
            operations,
            aggressive,
            dry_run,
        } => {
            optimize_command(&image, operations, aggressive, dry_run, cli.verbose)?;
        }

        Commands::Network {
            image,
            show_routes,
            show_interfaces,
            show_dns,
            export_json,
        } => {
            network_command(&image, show_routes, show_interfaces, show_dns, export_json, cli.verbose)?;
        }

        Commands::Compliance {
            image,
            standard,
            profile,
            export,
            fix,
        } => {
            compliance_command(&image, &standard, profile, export, fix, cli.verbose)?;
        }

        Commands::Malware {
            image,
            deep_scan,
            check_rootkits,
            yara_rules,
            quarantine,
        } => {
            malware_command(&image, deep_scan, check_rootkits, yara_rules, quarantine, cli.verbose)?;
        }

        Commands::Health {
            image,
            checks,
            detailed,
            export_json,
        } => {
            health_command(&image, checks, detailed, export_json, cli.verbose)?;
        }

        Commands::Clone {
            source,
            dest,
            sysprep,
            hostname,
            remove_keys,
            preserve_users,
        } => {
            clone_command(&source, &dest, sysprep, hostname, remove_keys, preserve_users, cli.verbose)?;
        }

        Commands::Patch {
            image,
            check_cves,
            severity,
            export,
            simulate_update,
        } => {
            patch_command(&image, check_cves, severity, export, simulate_update, cli.verbose)?;
        }

        Commands::Inventory {
            image,
            format,
            output,
            include_licenses,
            include_files,
            include_cves,
            severity,
            summary,
        } => {
            inventory_command(
                &image,
                &format,
                output.as_deref().map(|p| p.to_str().unwrap()),
                include_licenses,
                include_files,
                include_cves,
                severity,
                summary,
                cli.verbose,
            )?;
        }

        Commands::Validate {
            image,
            policy,
            benchmark,
            example_policy,
            format,
            output,
            strict,
        } => {
            validate_command(
                &image,
                policy.as_deref(),
                benchmark,
                example_policy,
                &format,
                output.as_deref(),
                strict,
                cli.verbose,
            )?;
        }

        Commands::Audit {
            image,
            categories,
            output_format,
            export,
            fix_issues,
        } => {
            audit_command(&image, categories, &output_format, export, fix_issues, cli.verbose)?;
        }

        Commands::Repair {
            image,
            repair_type,
            force,
            backup,
        } => {
            repair_command(&image, &repair_type, force, backup, cli.verbose)?;
        }

        Commands::Harden {
            image,
            profile,
            apply,
            preview,
        } => {
            harden_command(&image, &profile, apply, preview, cli.verbose)?;
        }

        Commands::Anomaly {
            image,
            baseline,
            sensitivity,
            categories,
            export,
        } => {
            anomaly_command(&image, baseline, &sensitivity, categories, export, cli.verbose)?;
        }

        Commands::Recommend {
            image,
            focus,
            priority,
            apply,
        } => {
            recommend_command(&image, focus, &priority, apply, cli.verbose)?;
        }

        Commands::Dependencies {
            image,
            target,
            graph_type,
            export_dot,
        } => {
            dependencies_command(&image, target, &graph_type, export_dot, cli.verbose)?;
        }

        Commands::Predict {
            image,
            metric,
            timeframe,
            export,
        } => {
            predict_command(&image, &metric, timeframe, export, cli.verbose)?;
        }

        Commands::Intelligence {
            image,
            ioc_file,
            threat_level,
            correlate,
            export,
        } => {
            intelligence_command(&image, ioc_file, &threat_level, correlate, export, cli.verbose)?;
        }

        Commands::Simulate {
            image,
            change_type,
            target,
            dry_run,
            risk_assessment,
        } => {
            simulate_command(&image, &change_type, target, dry_run, risk_assessment, cli.verbose)?;
        }

        Commands::Score {
            image,
            dimensions,
            weights,
            benchmark,
            export,
        } => {
            score_command(&image, dimensions, weights, benchmark, export, cli.verbose)?;
        }

        Commands::Template {
            image,
            template,
            strict,
            fix,
            export_template,
        } => {
            template_command(&image, &template, strict, fix, export_template, cli.verbose)?;
        }

        Commands::Hunt {
            image,
            hypothesis,
            framework,
            techniques,
            depth,
            export,
        } => {
            hunt_command(&image, hypothesis, &framework, techniques, &depth, export, cli.verbose)?;
        }

        Commands::Reconstruct {
            image,
            incident_type,
            start_time,
            end_time,
            visualize,
            export,
        } => {
            reconstruct_command(&image, &incident_type, start_time, end_time, visualize, export, cli.verbose)?;
        }

        Commands::Evolve {
            image,
            target_state,
            strategy,
            stages,
            safety_checks,
            export_plan,
        } => {
            evolve_command(&image, &target_state, &strategy, stages, safety_checks, export_plan, cli.verbose)?;
        }

        Commands::Verify {
            image,
            verification_level,
            check_supply_chain,
            check_identity,
            check_integrity,
            export,
        } => {
            verify_command(&image, &verification_level, check_supply_chain, check_identity, check_integrity, export, cli.verbose)?;
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

        Commands::Explore { image, path } => {
            run_standalone_explorer(&image, &path, cli.verbose)?;
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
