// SPDX-License-Identifier: LGPL-3.0-or-later
//! GuestCtl CLI - Command-line interface for disk image inspection and manipulation

use clap::{Parser, Subcommand};
use guestkit::guestfs::Guestfs;
use guestkit::core::ProgressReporter;
use std::path::PathBuf;
use anyhow::{Result, Context};
use serde_json::json;
use owo_colors::OwoColorize;

#[derive(Parser)]
#[command(
    name = "guestctl",
    version,
    about = "Inspect and manipulate disk images",
    long_about = "A command-line tool for inspecting and manipulating virtual machine disk images.\n\
                  Supports QCOW2, VMDK, RAW, and other formats."
)]
struct Cli {
    #[command(subcommand)]
    command: Commands,

    /// Enable verbose output
    #[arg(short, long, global = true)]
    verbose: bool,
}

#[derive(Subcommand)]
enum Commands {
    /// Inspect a disk image and show OS information
    #[command(alias = "info")]
    Inspect {
        /// Path to disk image
        disk: PathBuf,

        /// Output as JSON
        #[arg(short, long)]
        json: bool,
    },

    /// List filesystems and partitions
    #[command(alias = "fs")]
    Filesystems {
        /// Path to disk image
        disk: PathBuf,

        /// Show detailed information
        #[arg(short, long)]
        detailed: bool,
    },

    /// List installed packages
    #[command(alias = "pkg")]
    Packages {
        /// Path to disk image
        disk: PathBuf,

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

    /// Copy file from disk image
    Cp {
        /// Source in format: disk.img:/path/to/file
        source: String,

        /// Destination path
        dest: PathBuf,
    },

    /// List files in a directory
    Ls {
        /// Path to disk image
        disk: PathBuf,

        /// Path to list (default: /)
        #[arg(default_value = "/")]
        path: String,

        /// Long listing format
        #[arg(short, long)]
        long: bool,
    },

    /// Read file content from disk image
    Cat {
        /// Path to disk image
        disk: PathBuf,

        /// Path to file
        path: String,
    },
}

fn main() -> Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Commands::Inspect { disk, json } => cmd_inspect(disk, json, cli.verbose)?,
        Commands::Filesystems { disk, detailed } => cmd_filesystems(disk, detailed, cli.verbose)?,
        Commands::Packages { disk, filter, limit, json } => {
            cmd_packages(disk, filter, limit, json, cli.verbose)?
        }
        Commands::Cp { source, dest } => cmd_cp(source, dest, cli.verbose)?,
        Commands::Ls { disk, path, long } => cmd_ls(disk, path, long, cli.verbose)?,
        Commands::Cat { disk, path } => cmd_cat(disk, path, cli.verbose)?,
    }

    Ok(())
}

fn cmd_inspect(disk: PathBuf, json_output: bool, verbose: bool) -> Result<()> {
    let mut g = Guestfs::new().context("Failed to create Guestfs handle")?;

    if verbose {
        g.set_verbose(true);
    }

    // Show progress for long operations
    let progress = if !json_output {
        let p = ProgressReporter::spinner("Loading disk image...");
        Some(p)
    } else {
        None
    };

    g.add_drive_ro(&disk)
        .with_context(|| format!("Failed to add disk: {}", disk.display()))?;

    if let Some(ref p) = progress {
        p.set_message("Launching appliance...");
    }

    g.launch().context("Failed to launch appliance")?;

    if let Some(ref p) = progress {
        p.set_message("Inspecting operating systems...");
    }

    let roots = g.inspect_os().context("Failed to inspect OS")?;

    if let Some(p) = progress {
        p.finish_and_clear();
    }

    if json_output {
        // JSON output
        let os_info: Vec<_> = roots
            .iter()
            .map(|root| {
                json!({
                    "root": root,
                    "type": g.inspect_get_type(root).ok(),
                    "distro": g.inspect_get_distro(root).ok(),
                    "major_version": g.inspect_get_major_version(root).ok(),
                    "minor_version": g.inspect_get_minor_version(root).ok(),
                    "product_name": g.inspect_get_product_name(root).ok(),
                    "hostname": g.inspect_get_hostname(root).ok(),
                    "arch": g.inspect_get_arch(root).ok(),
                    "package_format": g.inspect_get_package_format(root).ok(),
                    "package_management": g.inspect_get_package_management(root).ok(),
                })
            })
            .collect();

        println!("{}", serde_json::to_string_pretty(&json!({
            "disk": disk.display().to_string(),
            "os_count": roots.len(),
            "operating_systems": os_info,
        }))?);
    } else {
        // Human-readable output with colors
        println!("\n{}", "═".repeat(70).bright_blue());
        println!("{} {}", "📀 Disk Image:".bright_cyan().bold(), disk.display().to_string().bright_white());
        println!("{}\n", "═".repeat(70).bright_blue());

        if roots.is_empty() {
            println!("{}", "⚠️  No operating systems detected".bright_yellow().bold());
            println!("\n{}", "Possible reasons:".dimmed());
            println!("  {} Disk is not bootable", "•".bright_black());
            println!("  {} Disk is encrypted (try checking with LUKS tools)", "•".bright_black());
            println!("  {} Unsupported OS type", "•".bright_black());
            println!("  {} Corrupted disk image", "•".bright_black());
        } else {
            println!("{} {} {}\n",
                "✓".bright_green().bold(),
                "Found".bright_white(),
                format!("{} operating system(s)", roots.len()).bright_cyan().bold()
            );

            for (i, root) in roots.iter().enumerate() {
                if i > 0 {
                    println!("\n{}", "─".repeat(70).bright_black());
                }

                println!("\n{} {}",
                    format!("OS #{}", i + 1).bright_magenta().bold(),
                    format!("({})", root).dimmed()
                );
                println!("{}", "─".repeat(50).bright_black());

                if let Ok(os_type) = g.inspect_get_type(root) {
                    let icon = match os_type.as_str() {
                        "linux" => "🐧",
                        "windows" => "🪟",
                        _ => "💻",
                    };
                    println!("  {} {}  {}",
                        "Type:".bright_white().bold(),
                        icon,
                        os_type.bright_cyan()
                    );
                }

                if let Ok(distro) = g.inspect_get_distro(root) {
                    let display_distro = if distro == "unknown" || distro.is_empty() {
                        format!("{} (detection requires mounting)", "unknown".dimmed())
                    } else {
                        distro.bright_green().to_string()
                    };
                    println!("  {} {}",
                        "Distribution:".bright_white().bold(),
                        display_distro
                    );
                }

                if let Ok(major) = g.inspect_get_major_version(root) {
                    let minor = g.inspect_get_minor_version(root).unwrap_or(0);
                    if major > 0 || minor > 0 {
                        println!("  {} {}",
                            "Version:".bright_white().bold(),
                            format!("{}.{}", major, minor).bright_yellow()
                        );
                    }
                }

                if let Ok(product) = g.inspect_get_product_name(root) {
                    if product != "Linux" && !product.is_empty() {
                        println!("  {} {}",
                            "Product:".bright_white().bold(),
                            product.bright_white()
                        );
                    }
                }

                if let Ok(hostname) = g.inspect_get_hostname(root) {
                    let display_hostname = if hostname == "localhost" || hostname.is_empty() {
                        format!("{} (default)", hostname.dimmed())
                    } else {
                        hostname.bright_cyan().to_string()
                    };
                    println!("  {} {}",
                        "Hostname:".bright_white().bold(),
                        display_hostname
                    );
                }

                if let Ok(arch) = g.inspect_get_arch(root) {
                    println!("  {} {}",
                        "Architecture:".bright_white().bold(),
                        arch.bright_yellow()
                    );
                }

                if let Ok(pkg_fmt) = g.inspect_get_package_format(root) {
                    let display_pkg = if pkg_fmt == "unknown" || pkg_fmt.is_empty() {
                        format!("{} (requires mounting)", "unknown".dimmed())
                    } else {
                        pkg_fmt.bright_green().to_string()
                    };
                    println!("  {} {}",
                        "Package format:".bright_white().bold(),
                        display_pkg
                    );
                }

                if let Ok(pkg_mgmt) = g.inspect_get_package_management(root) {
                    let display_mgmt = if pkg_mgmt == "unknown" || pkg_mgmt.is_empty() {
                        format!("{} (requires mounting)", "unknown".dimmed())
                    } else {
                        pkg_mgmt.bright_magenta().to_string()
                    };
                    println!("  {} {}",
                        "Package management:".bright_white().bold(),
                        display_mgmt
                    );
                }
            }

            println!("\n{}", "═".repeat(70).bright_blue());
        }
    }

    g.shutdown().ok();
    Ok(())
}

fn cmd_filesystems(disk: PathBuf, detailed: bool, verbose: bool) -> Result<()> {
    let mut g = Guestfs::new().context("Failed to create Guestfs handle")?;

    if verbose {
        g.set_verbose(true);
    }

    let progress = ProgressReporter::spinner("Loading disk image...");

    g.add_drive_ro(&disk)
        .with_context(|| format!("Failed to add disk: {}", disk.display()))?;

    progress.set_message("Launching appliance...");
    g.launch().context("Failed to launch appliance")?;

    progress.set_message("Scanning filesystems...");

    let devices = g.list_devices().context("Failed to list devices")?;

    progress.finish_and_clear();

    println!("\n{}", "═".repeat(70).bright_blue());
    println!("{} {}", "💾 Disk Image:".bright_cyan().bold(), disk.display().to_string().bright_white());
    println!("{}\n", "═".repeat(70).bright_blue());

    // Devices
    println!("{}", "Block Devices".bright_white().bold());
    println!("{}", "─".repeat(50).bright_black());
    for device in devices {
        println!("  {} {}",
            "▪".bright_cyan(),
            device.bright_white().bold()
        );

        if detailed {
            if let Ok(size) = g.blockdev_getsize64(&device) {
                let gb = size as f64 / 1_073_741_824.0;  // 1024^3
                println!("    {} {} ({:.2} GiB)",
                    "Size:".dimmed(),
                    size.to_string().bright_yellow(),
                    gb
                );
            }

            if let Ok(parttype) = g.part_get_parttype(&device) {
                println!("    {} {}",
                    "Partition table:".dimmed(),
                    parttype.bright_green()
                );
            }
        }
    }

    // Partitions
    let partitions = g.list_partitions().context("Failed to list partitions")?;
    if !partitions.is_empty() {
        println!("\n{}", "Partitions".bright_white().bold());
        println!("{}", "─".repeat(50).bright_black());

        for partition in partitions {
            let fstype = g.vfs_type(&partition).unwrap_or_else(|_| "unknown".to_string());
            let size = g.blockdev_getsize64(&partition).unwrap_or(0);
            let gb = size as f64 / 1_073_741_824.0;

            let fs_icon = match fstype.as_str() {
                "ext2" | "ext3" | "ext4" => "📁",
                "ntfs" => "🪟",
                "vfat" | "fat" | "fat32" => "💾",
                "xfs" => "🗄",
                "btrfs" => "🌳",
                "swap" => "💫",
                _ => "❓",
            };

            println!("  {} {} {} {}",
                fs_icon,
                partition.bright_white().bold(),
                format!("({})", fstype).bright_cyan(),
                format!("{:.1} GiB", gb).bright_yellow()
            );

            if let Ok(label) = g.vfs_label(&partition) {
                if !label.is_empty() {
                    println!("    {} {}",
                        "Label:".dimmed(),
                        label.bright_green()
                    );
                }
            }

            if detailed {
                if let Ok(uuid) = g.vfs_uuid(&partition) {
                    if !uuid.is_empty() {
                        println!("    {} {}",
                            "UUID:".dimmed(),
                            uuid.dimmed()
                        );
                    }
                }

                if let Ok(partnum) = g.part_to_partnum(&partition) {
                    println!("    {} {}",
                        "Number:".dimmed(),
                        partnum.to_string().bright_magenta()
                    );
                }
            }
        }
    }

    // LVM information
    if let Ok(vgs) = g.vgs() {
        if !vgs.is_empty() {
            println!("\n{}", "LVM Volume Groups".bright_white().bold());
            println!("{}", "─".repeat(50).bright_black());
            for vg in vgs {
                println!("  {} {}",
                    "▸".bright_magenta(),
                    vg.bright_white().bold()
                );
            }
        }
    }

    if let Ok(lvs) = g.lvs() {
        if !lvs.is_empty() {
            println!("\n{}", "LVM Logical Volumes".bright_white().bold());
            println!("{}", "─".repeat(50).bright_black());
            for lv in lvs {
                let size = g.blockdev_getsize64(&lv).unwrap_or(0);
                let gb = size as f64 / 1_073_741_824.0;

                println!("  {} {} {}",
                    "▸".bright_magenta(),
                    lv.bright_white().bold(),
                    format!("{:.1} GiB", gb).bright_yellow()
                );
            }
        }
    }

    println!("\n{}", "═".repeat(70).bright_blue());

    g.shutdown().ok();
    Ok(())
}

fn cmd_packages(
    disk: PathBuf,
    filter: Option<String>,
    limit: Option<usize>,
    json_output: bool,
    verbose: bool,
) -> Result<()> {
    let mut g = Guestfs::new().context("Failed to create Guestfs handle")?;

    if verbose {
        g.set_verbose(true);
    }

    // Show progress for long operations
    let progress = if !json_output {
        let p = ProgressReporter::spinner("Loading disk image...");
        Some(p)
    } else {
        None
    };

    g.add_drive_ro(&disk)
        .with_context(|| format!("Failed to add disk: {}", disk.display()))?;

    if let Some(ref p) = progress {
        p.set_message("Launching appliance...");
    }

    g.launch().context("Failed to launch appliance")?;

    if let Some(ref p) = progress {
        p.set_message("Detecting operating system...");
    }

    let roots = g.inspect_os().context("Failed to inspect OS")?;
    if roots.is_empty() {
        if let Some(p) = progress {
            p.abandon_with_message("No operating system detected");
        }
        anyhow::bail!("No operating system detected in disk image");
    }

    let root = &roots[0];

    // Mount filesystems
    if let Some(ref p) = progress {
        p.set_message("Mounting filesystems...");
    }

    if let Ok(mountpoints) = g.inspect_get_mountpoints(root) {
        let mut mounts: Vec<_> = mountpoints.iter().collect();
        mounts.sort_by_key(|(mount, _)| std::cmp::Reverse(mount.len()));
        for (mount, device) in mounts {
            g.mount_ro(device, mount).ok();
        }
    }

    // List packages
    if let Some(ref p) = progress {
        p.set_message("Listing installed packages...");
    }

    let apps = g
        .inspect_list_applications(root)
        .context("Failed to list applications")?;

    if let Some(p) = progress {
        p.finish_and_clear();
    }

    // Apply filter
    let filtered: Vec<_> = apps
        .into_iter()
        .filter(|app| {
            if let Some(ref f) = filter {
                app.name.contains(f)
            } else {
                true
            }
        })
        .collect();

    // Apply limit
    let limited: Vec<_> = if let Some(lim) = limit {
        filtered.into_iter().take(lim).collect()
    } else {
        filtered
    };

    if json_output {
        let packages: Vec<_> = limited
            .iter()
            .map(|app| {
                json!({
                    "name": app.name,
                    "version": app.version,
                    "release": app.release,
                    "epoch": app.epoch,
                })
            })
            .collect();

        println!("{}", serde_json::to_string_pretty(&json!({
            "total": packages.len(),
            "packages": packages,
        }))?);
    } else {
        println!("Found {} package(s)\n", limited.len());

        if !limited.is_empty() {
            println!("{:<40} {:<20} {:<20}", "Package", "Version", "Release");
            println!("{}", "-".repeat(82));

            for app in limited {
                let name = if app.name.len() > 38 {
                    format!("{}...", &app.name[..35])
                } else {
                    app.name.clone()
                };

                let version = if app.version.len() > 18 {
                    format!("{}...", &app.version[..15])
                } else {
                    app.version.clone()
                };

                let release = if app.release.len() > 18 {
                    format!("{}...", &app.release[..15])
                } else {
                    app.release.clone()
                };

                println!("{:<40} {:<20} {:<20}", name, version, release);
            }
        }
    }

    g.umount_all().ok();
    g.shutdown().ok();
    Ok(())
}

fn cmd_cp(source: String, dest: PathBuf, verbose: bool) -> Result<()> {
    // Parse "disk.img:/path/to/file" format
    let parts: Vec<&str> = source.splitn(2, ':').collect();
    if parts.len() != 2 {
        anyhow::bail!(
            "Source must be in format: disk.img:/path/to/file\nGot: {}",
            source
        );
    }

    let disk = PathBuf::from(parts[0]);
    let src_path = parts[1];

    let mut g = Guestfs::new().context("Failed to create Guestfs handle")?;

    if verbose {
        g.set_verbose(true);
    }

    let progress = ProgressReporter::spinner("Loading disk image...");

    g.add_drive_ro(&disk)
        .with_context(|| format!("Failed to add disk: {}", disk.display()))?;

    progress.set_message("Launching appliance...");
    g.launch().context("Failed to launch appliance")?;

    // Try to mount automatically
    progress.set_message("Mounting filesystems...");

    let roots = g.inspect_os().unwrap_or_default();
    if !roots.is_empty() {
        let root = &roots[0];
        if let Ok(mountpoints) = g.inspect_get_mountpoints(root) {
            let mut mounts: Vec<_> = mountpoints.iter().collect();
        mounts.sort_by_key(|(mount, _)| std::cmp::Reverse(mount.len()));
        for (mount, device) in mounts {
                g.mount_ro(device, mount).ok();
            }
        }
    }

    // Check if file exists
    if !g.is_file(src_path).unwrap_or(false) {
        progress.abandon_with_message(&format!("File not found: {}", src_path));
        anyhow::bail!("File not found: {}", src_path);
    }

    // Copy file
    progress.set_message(&format!("Copying {}...", src_path));

    g.download(src_path, dest.to_str().unwrap())
        .with_context(|| format!("Failed to copy file: {}", src_path))?;

    progress.finish_and_clear();

    println!("✓ Copied {} -> {}", source, dest.display());

    g.umount_all().ok();
    g.shutdown().ok();
    Ok(())
}

fn cmd_ls(disk: PathBuf, path: String, long: bool, verbose: bool) -> Result<()> {
    let mut g = Guestfs::new().context("Failed to create Guestfs handle")?;

    if verbose {
        g.set_verbose(true);
    }

    let progress = ProgressReporter::spinner("Loading disk image...");

    g.add_drive_ro(&disk)
        .with_context(|| format!("Failed to add disk: {}", disk.display()))?;

    progress.set_message("Launching appliance...");
    g.launch().context("Failed to launch appliance")?;

    // Mount filesystems
    progress.set_message("Mounting filesystems...");

    let roots = g.inspect_os().unwrap_or_default();
    if !roots.is_empty() {
        let root = &roots[0];
        if let Ok(mountpoints) = g.inspect_get_mountpoints(root) {
            let mut mounts: Vec<_> = mountpoints.iter().collect();
        mounts.sort_by_key(|(mount, _)| std::cmp::Reverse(mount.len()));
        for (mount, device) in mounts {
                g.mount_ro(device, mount).ok();
            }
        }
    }

    // Check if directory exists
    if !g.is_dir(&path).unwrap_or(false) {
        progress.abandon_with_message(&format!("Not a directory: {}", path));
        anyhow::bail!("Not a directory: {}", path);
    }

    progress.set_message(&format!("Listing {}...", path));

    let result = if long {
        // Long listing
        g.ll(&path).context("Failed to list directory")
    } else {
        // Simple listing
        let entries = g.ls(&path).context("Failed to list directory")?;
        Ok(entries.join("\n"))
    };

    progress.finish_and_clear();

    match result {
        Ok(output) => println!("{}", output),
        Err(e) => return Err(e),
    }

    g.umount_all().ok();
    g.shutdown().ok();
    Ok(())
}

fn cmd_cat(disk: PathBuf, path: String, verbose: bool) -> Result<()> {
    let mut g = Guestfs::new().context("Failed to create Guestfs handle")?;

    if verbose {
        g.set_verbose(true);
    }

    let progress = ProgressReporter::spinner("Loading disk image...");

    g.add_drive_ro(&disk)
        .with_context(|| format!("Failed to add disk: {}", disk.display()))?;

    progress.set_message("Launching appliance...");
    g.launch().context("Failed to launch appliance")?;

    // Mount filesystems
    progress.set_message("Mounting filesystems...");

    let roots = g.inspect_os().unwrap_or_default();
    if !roots.is_empty() {
        let root = &roots[0];
        if let Ok(mountpoints) = g.inspect_get_mountpoints(root) {
            let mut mounts: Vec<_> = mountpoints.iter().collect();
        mounts.sort_by_key(|(mount, _)| std::cmp::Reverse(mount.len()));
        for (mount, device) in mounts {
                g.mount_ro(device, mount).ok();
            }
        }
    }

    // Check if file exists
    if !g.is_file(&path).unwrap_or(false) {
        progress.abandon_with_message(&format!("File not found: {}", path));
        anyhow::bail!("File not found: {}", path);
    }

    // Read and print file
    progress.set_message(&format!("Reading {}...", path));
    let content = g
        .read_file(&path)
        .with_context(|| format!("Failed to read file: {}", path))?;

    progress.finish_and_clear();

    // Try to print as UTF-8, fall back to hex if binary
    match String::from_utf8(content.clone()) {
        Ok(text) => print!("{}", text),
        Err(_) => {
            eprintln!("Warning: File contains binary data, displaying hex dump");
            for (i, chunk) in content.chunks(16).enumerate() {
                print!("{:08x}  ", i * 16);
                for byte in chunk {
                    print!("{:02x} ", byte);
                }
                println!();
            }
        }
    }

    g.umount_all().ok();
    g.shutdown().ok();
    Ok(())
}
