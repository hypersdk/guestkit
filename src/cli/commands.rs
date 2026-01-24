// SPDX-License-Identifier: LGPL-3.0-or-later
//! CLI commands implementation

use super::formatters::*;
use super::profiles::{FindingStatus, ProfileReport};
use anyhow::{Context, Result};
use guestkit::core::ProgressReporter;
use guestkit::Guestfs;
use std::path::PathBuf;

/// Collect inspection data into a structured report
fn collect_inspection_data(
    g: &mut Guestfs,
    root: &str,
    _verbose: bool,
) -> Result<InspectionReport> {
    let mut report = InspectionReport {
        image_path: None,
        os: OsInfo {
            root: root.to_string(),
            os_type: g.inspect_get_type(root).ok(),
            distribution: g.inspect_get_distro(root).ok(),
            product_name: g.inspect_get_product_name(root).ok(),
            architecture: g.inspect_get_arch(root).ok(),
            version: {
                if let (Ok(major), Ok(minor)) = (
                    g.inspect_get_major_version(root),
                    g.inspect_get_minor_version(root),
                ) {
                    Some(VersionInfo { major, minor })
                } else {
                    None
                }
            },
            hostname: g.inspect_get_hostname(root).ok(),
            package_format: g.inspect_get_package_format(root).ok(),
            init_system: g.inspect_get_init_system(root).ok(),
            package_manager: g.inspect_get_package_management(root).ok(),
            format: g.inspect_get_format(root).ok(),
        },
        system_config: Some(SystemConfig {
            timezone: g.inspect_timezone(root).ok(),
            locale: g.inspect_locale(root).ok(),
            selinux: g.inspect_selinux(root).ok(),
            cloud_init: g.inspect_cloud_init(root).ok(),
            vm_tools: g.inspect_vm_tools(root).ok(),
        }),
        network: {
            let interfaces = g.inspect_network(root).ok();
            let dns_servers = g.inspect_dns(root).ok();
            if interfaces.is_some() || dns_servers.is_some() {
                Some(NetworkInfo {
                    interfaces,
                    dns_servers,
                })
            } else {
                None
            }
        },
        users: {
            if let Ok(all_users) = g.inspect_users(root) {
                let regular_users: Vec<_> = all_users
                    .iter()
                    .filter(|u| {
                        let uid: i32 = u.uid.parse().unwrap_or(0);
                        (1000..65534).contains(&uid)
                    })
                    .cloned()
                    .collect();

                let system_users_count = all_users
                    .iter()
                    .filter(|u| {
                        let uid: i32 = u.uid.parse().unwrap_or(0);
                        uid > 0 && uid < 1000
                    })
                    .count();

                Some(UsersInfo {
                    regular_users,
                    system_users_count,
                    total_users: all_users.len(),
                })
            } else {
                None
            }
        },
        ssh: g
            .inspect_ssh_config(root)
            .ok()
            .map(|config| SshConfig { config }),
        services: {
            let enabled_services = g.inspect_systemd_services(root).ok().unwrap_or_default();
            let timers = g.inspect_systemd_timers(root).ok().unwrap_or_default();
            if !enabled_services.is_empty() || !timers.is_empty() {
                Some(ServicesInfo {
                    enabled_services,
                    timers,
                })
            } else {
                None
            }
        },
        runtimes: {
            let language_runtimes = g.inspect_runtimes(root).ok().unwrap_or_default();
            let container_runtimes = g.inspect_container_runtimes(root).ok().unwrap_or_default();
            if !language_runtimes.is_empty() || !container_runtimes.is_empty() {
                Some(RuntimesInfo {
                    language_runtimes,
                    container_runtimes,
                })
            } else {
                None
            }
        },
        storage: {
            let lvm = g.inspect_lvm(root).ok().filter(|l| {
                !l.physical_volumes.is_empty()
                    || !l.volume_groups.is_empty()
                    || !l.logical_volumes.is_empty()
            });
            let swap_devices = g.inspect_swap(root).ok().filter(|s| !s.is_empty());
            let fstab_mounts = g.inspect_fstab(root).ok().map(|mounts| {
                mounts
                    .into_iter()
                    .map(|(device, mountpoint, fstype)| FstabMount {
                        device,
                        mountpoint,
                        fstype,
                    })
                    .collect()
            });

            if lvm.is_some() || swap_devices.is_some() || fstab_mounts.is_some() {
                Some(StorageInfo {
                    lvm,
                    swap_devices,
                    fstab_mounts,
                })
            } else {
                None
            }
        },
        boot: g
            .inspect_boot_config(root)
            .ok()
            .filter(|b| b.bootloader != "unknown"),
        scheduled_tasks: {
            let cron_jobs = g.inspect_cron(root).ok().unwrap_or_default();
            let systemd_timers = g.inspect_systemd_timers(root).ok().unwrap_or_default();
            if !cron_jobs.is_empty() || !systemd_timers.is_empty() {
                Some(ScheduledTasksInfo {
                    cron_jobs,
                    systemd_timers,
                })
            } else {
                None
            }
        },
        security: {
            if let Ok(certs) = g.inspect_certificates(root) {
                let kernel_params = g.inspect_kernel_params(root).ok().unwrap_or_default();
                Some(SecurityInfo {
                    certificates_count: certs.len(),
                    certificate_paths: certs.into_iter().take(5).collect(),
                    kernel_parameters_count: kernel_params.len(),
                })
            } else {
                None
            }
        },
        packages: None,   // Will be filled if we mount and check packages
        disk_usage: None, // Will be filled if we mount and get statvfs
        windows: None,    // Will be filled for Windows systems
    };

    // Try to mount and get additional info (packages, disk usage)
    if g.mount(root, "/").is_ok() {
        // Get disk usage
        if let Ok(usage_map) = g.statvfs("/") {
            let blocks = *usage_map.get("blocks").unwrap_or(&0);
            let bsize = *usage_map.get("bsize").unwrap_or(&4096);
            let bfree = *usage_map.get("bfree").unwrap_or(&0);

            let total_bytes = blocks * bsize;
            let free_bytes = bfree * bsize;
            let used_bytes = total_bytes - free_bytes;
            let used_percent = if total_bytes > 0 {
                (used_bytes as f64 / total_bytes as f64) * 100.0
            } else {
                0.0
            };

            report.disk_usage = Some(DiskUsageInfo {
                total_bytes,
                used_bytes,
                free_bytes,
                used_percent,
            });
        }

        // Get package info
        if let Ok(pkg_fmt) = g.inspect_get_package_format(root) {
            let count = match pkg_fmt.as_str() {
                "rpm" => g.rpm_list().ok().map(|p| p.len()).unwrap_or(0),
                "deb" => g.dpkg_list().ok().map(|p| p.len()).unwrap_or(0),
                _ => 0,
            };

            let kernels = g
                .ls("/boot")
                .ok()
                .map(|files| {
                    files
                        .iter()
                        .filter(|f| f.starts_with("vmlinuz-") || f.starts_with("vmlinux-"))
                        .map(|s| s.to_string())
                        .collect()
                })
                .unwrap_or_default();

            report.packages = Some(PackagesInfo {
                format: pkg_fmt,
                count,
                kernels,
            });
        }

        g.umount("/").ok();
    }

    // Windows-specific inspection
    if let Some(ref os_type) = report.os.os_type {
        if os_type == "windows" {
            let software = g.inspect_windows_software(root).ok();
            let services = g.inspect_windows_services(root).ok();
            let network_adapters = g.inspect_windows_network(root).ok();
            let updates = g.inspect_windows_updates(root).ok();
            let event_logs = g.inspect_windows_events(root, "System", 10).ok();

            if software.is_some()
                || services.is_some()
                || network_adapters.is_some()
                || updates.is_some()
                || event_logs.is_some()
            {
                report.windows = Some(WindowsInfo {
                    software,
                    services,
                    network_adapters,
                    updates,
                    event_logs,
                });
            }
        }
    }

    Ok(report)
}

/// Print profile report in text format
fn print_profile_report(report: &ProfileReport) {
    println!("Profile: {}", report.profile_name);
    println!();

    for section in &report.sections {
        println!("━━━ {} ━━━", section.title);
        println!();

        for finding in &section.findings {
            let status_symbol = match finding.status {
                FindingStatus::Pass => "✓",
                FindingStatus::Warning => "⚠",
                FindingStatus::Fail => "✗",
                FindingStatus::Info => "ℹ",
            };

            let risk_display = if let Some(risk) = finding.risk_level {
                format!(" [{}]", risk)
            } else {
                String::new()
            };

            println!(
                "  {} {}: {}{}",
                status_symbol, finding.item, finding.message, risk_display
            );
        }
        println!();
    }

    if let Some(summary) = &report.summary {
        println!("━━━ Summary ━━━");
        println!("{}", summary);
        println!();
    }

    if let Some(risk) = report.overall_risk {
        println!("Overall Risk: {}", risk);
    }
}

/// Print an inspection report using the specified format
fn print_inspection_report(
    report: &InspectionReport,
    output_format: Option<OutputFormat>,
    _verbose: bool,
) -> Result<()> {
    if let Some(format) = output_format {
        let formatter = get_formatter(format, true);
        let output = formatter.format(report)?;
        println!("{}", output);
    } else {
        // Use default text output for cached results
        let formatter = get_formatter(OutputFormat::Text, true);
        let output = formatter.format(report)?;
        println!("{}", output);
    }
    Ok(())
}

/// Inspect a disk image and display OS information
pub fn inspect_image(
    image: &PathBuf,
    verbose: bool,
    output_format: Option<OutputFormat>,
    profile: Option<String>,
    export_format: Option<String>,
    export_path: Option<PathBuf>,
    use_cache: bool,
    force_refresh: bool,
) -> Result<()> {
    use super::cache::InspectionCache;

    // Try to get cached result if caching is enabled
    if use_cache && !force_refresh {
        if let Ok(cache) = InspectionCache::new() {
            if let Ok(Some(cached_report)) = cache.get(image) {
                log::info!("✓ Using cached inspection result");

                // Handle export if requested
                if let (Some(export_fmt), Some(export_out)) = (export_format, export_path) {
                    use super::exporters::{export_report, ExportFormat};

                    let fmt = ExportFormat::from_str(&export_fmt)?;
                    export_report(&cached_report, fmt, &export_out)?;

                    println!("Report exported to: {}", export_out.display());
                    return Ok(());
                }

                // Handle profile output
                if profile.is_some() {
                    println!("⚠ Cannot use profiles with cached results. Use --cache-refresh to re-inspect.");
                    return Ok(());
                }

                // Print cached result
                print_inspection_report(&cached_report, output_format, verbose)?;
                return Ok(());
            }
        }
    }

    let mut g = Guestfs::new()?;
    g.set_verbose(verbose);

    let progress = ProgressReporter::spinner(&format!("Inspecting: {}", image.display()));

    if verbose {
        eprintln!("[VERBOSE] Adding drive: {}", image.display());
    }
    g.add_drive_ro(image.to_str().unwrap())?;

    progress.set_message("Launching appliance...");
    if verbose {
        eprintln!("[VERBOSE] Launching QEMU appliance...");
    }
    g.launch().context("Failed to launch")?;

    progress.set_message("Scanning disk...");

    // List devices
    if verbose {
        eprintln!("[VERBOSE] Enumerating block devices...");
    }
    println!("\n=== Block Devices ===");
    let devices = g.list_devices()?;
    for device in &devices {
        let size = g.blockdev_getsize64(device)?;
        if verbose {
            eprintln!("[VERBOSE] Found device: {} ({} bytes)", device, size);
        }
        println!("  {}: {} bytes ({:.2} GB)", device, size, size as f64 / 1e9);

        // Additional device information
        if let Ok(ro) = g.blockdev_getro(device) {
            println!("    Read-only: {}", if ro { "yes" } else { "no" });
        }
        if let Ok(ss) = g.blockdev_getss(device) {
            println!("    Sector size: {} bytes", ss);
        }
    }

    // List partitions
    if verbose {
        eprintln!("[VERBOSE] Analyzing partition table...");
    }
    println!("\n=== Partitions ===");
    let partitions = g.list_partitions()?;
    for partition in &partitions {
        if verbose {
            eprintln!("[VERBOSE] Examining partition: {}", partition);
        }
        println!("  {}", partition);

        if let Ok(part_list) = g.part_list("/dev/sda") {
            let part_num = g.part_to_partnum(partition)?;
            if let Some(p) = part_list.iter().find(|p| p.part_num == part_num) {
                println!("    Number: {}", p.part_num);
                println!("    Start:  {} bytes", p.part_start);
                println!(
                    "    Size:   {} bytes ({:.2} GB)",
                    p.part_size,
                    p.part_size as f64 / 1e9
                );
                println!("    End:    {} bytes", p.part_end);
            }
        }
    }

    // Partition scheme
    if verbose {
        eprintln!("[VERBOSE] Detecting partition scheme...");
    }
    if let Ok(scheme) = g.part_get_parttype("/dev/sda") {
        println!("\n=== Partition Scheme ===");
        println!("  Type: {}", scheme);
        if verbose {
            eprintln!("[VERBOSE] Partition scheme: {}", scheme);
        }
    }

    // List filesystems
    if verbose {
        eprintln!("[VERBOSE] Detecting filesystems...");
    }
    println!("\n=== Filesystems ===");
    let filesystems = g.list_filesystems()?;
    for (device, fstype) in &filesystems {
        if verbose {
            eprintln!("[VERBOSE] Filesystem on {}: {}", device, fstype);
        }
        println!("  {}: {}", device, fstype);

        if fstype != "unknown" && fstype != "swap" {
            if let Ok(label) = g.vfs_label(device) {
                if !label.is_empty() {
                    println!("    Label: {}", label);
                }
            }
            if let Ok(uuid) = g.vfs_uuid(device) {
                if !uuid.is_empty() {
                    println!("    UUID:  {}", uuid);
                }
            }
        }
    }

    // OS inspection
    progress.set_message("Detecting operating systems...");
    if verbose {
        eprintln!("[VERBOSE] Running OS detection algorithms...");
    }
    let roots = g.inspect_os()?;

    progress.finish_and_clear();

    // If profile is specified, run profile inspection
    if let Some(profile_name) = profile {
        use super::profiles::get_profile;

        if roots.is_empty() {
            eprintln!("No operating systems found in image");
            g.shutdown()?;
            return Ok(());
        }

        let root = &roots[0];

        if let Some(profile_impl) = get_profile(&profile_name) {
            println!("\n=== {} ===\n", profile_impl.description());

            let report = profile_impl.inspect(&mut g, root)?;

            // Output profile report
            if let Some(format) = output_format {
                let _formatter = get_formatter(format, true);
                let output = serde_json::to_string_pretty(&report)?;
                println!("{}", output);
            } else {
                // Text output for profile
                print_profile_report(&report);
            }

            g.shutdown()?;
            return Ok(());
        } else {
            eprintln!(
                "Unknown profile: {}. Available profiles: security, migration, performance",
                profile_name
            );
            g.shutdown()?;
            return Err(anyhow::anyhow!("Invalid profile"));
        }
    }

    // If structured output format is requested, collect data and format it
    if let Some(format) = output_format {
        if roots.is_empty() {
            eprintln!("No operating systems found in image");
            g.shutdown()?;
            return Ok(());
        }

        // Collect data for first root (or all roots if needed)
        let mut report = collect_inspection_data(&mut g, &roots[0], verbose)?;
        report.image_path = Some(image.to_string_lossy().to_string());

        g.shutdown()?;

        // Store in cache if caching is enabled
        if use_cache {
            if let Ok(cache) = InspectionCache::new() {
                if let Err(e) = cache.store(image, &report) {
                    log::warn!("Failed to cache inspection result: {}", e);
                } else {
                    log::info!("✓ Cached inspection result");
                }
            }
        }

        // Handle export if requested
        if let (Some(export_fmt), Some(export_out)) = (export_format, export_path) {
            use super::exporters::{export_report, ExportFormat};

            let fmt = ExportFormat::from_str(&export_fmt)?;
            export_report(&report, fmt, &export_out)?;

            println!("Report exported to: {}", export_out.display());
            return Ok(());
        }

        // Format and print output
        let formatter = get_formatter(format, true); // pretty=true for readability
        let output = formatter.format(&report)?;
        println!("{}", output);

        return Ok(());
    }

    // Otherwise, use traditional text output
    println!("\n=== Operating Systems ===");

    if roots.is_empty() {
        println!("  No operating systems found");
        if verbose {
            eprintln!("[VERBOSE] No bootable operating systems detected");
        }
    } else {
        for root in &roots {
            if verbose {
                eprintln!("[VERBOSE] Inspecting OS at root: {}", root);
            }
            println!("  Root: {}", root);

            if let Ok(ostype) = g.inspect_get_type(root) {
                if verbose {
                    eprintln!("[VERBOSE] OS type detected: {}", ostype);
                }
                println!("    Type:         {}", ostype);
            }
            if let Ok(distro) = g.inspect_get_distro(root) {
                if verbose {
                    eprintln!("[VERBOSE] Distribution: {}", distro);
                }
                println!("    Distribution: {}", distro);
            }
            if let Ok(product) = g.inspect_get_product_name(root) {
                if verbose {
                    eprintln!("[VERBOSE] Product name: {}", product);
                }
                println!("    Product:      {}", product);
            }
            if let Ok(arch) = g.inspect_get_arch(root) {
                if verbose {
                    eprintln!("[VERBOSE] Architecture: {}", arch);
                }
                println!("    Architecture: {}", arch);
            }
            if let Ok(major) = g.inspect_get_major_version(root) {
                if let Ok(minor) = g.inspect_get_minor_version(root) {
                    if verbose {
                        eprintln!("[VERBOSE] Version: {}.{}", major, minor);
                    }
                    println!("    Version:      {}.{}", major, minor);
                }
            }
            if let Ok(hostname) = g.inspect_get_hostname(root) {
                if verbose {
                    eprintln!("[VERBOSE] Hostname: {}", hostname);
                }
                println!("    Hostname:     {}", hostname);
            }
            if let Ok(pkg_fmt) = g.inspect_get_package_format(root) {
                if verbose {
                    eprintln!("[VERBOSE] Package format: {}", pkg_fmt);
                }
                println!("    Packages:     {}", pkg_fmt);
            }

            // Additional detailed information
            if verbose {
                eprintln!("[VERBOSE] Retrieving init system information...");
            }
            if let Ok(init) = g.inspect_get_init_system(root) {
                println!("    Init system:  {}", init);
            }

            if verbose {
                eprintln!("[VERBOSE] Detecting package management tool...");
            }
            if let Ok(pkg_mgr) = g.inspect_get_package_management(root) {
                println!("    Pkg Manager:  {}", pkg_mgr);
            }

            if verbose {
                eprintln!("[VERBOSE] Checking OS format...");
            }
            if let Ok(format) = g.inspect_get_format(root) {
                println!("    Format:       {}", format);
            }

            if verbose {
                eprintln!("[VERBOSE] Checking for product variant...");
            }
            if let Ok(variant) = g.inspect_get_product_variant(root) {
                if !variant.is_empty() {
                    println!("    Variant:      {}", variant);
                }
            }

            // Mount points
            if verbose {
                eprintln!("[VERBOSE] Analyzing mount points...");
            }
            if let Ok(mountpoints) = g.inspect_get_mountpoints(root) {
                if mountpoints.len() > 1 {
                    println!("    Mount points:");
                    for (mp, dev) in mountpoints {
                        println!("      {} -> {}", mp, dev);
                        if verbose {
                            eprintln!("[VERBOSE] Mountpoint: {} -> {}", mp, dev);
                        }
                    }
                }
            }

            // Flags and characteristics
            if verbose {
                eprintln!("[VERBOSE] Checking OS characteristics...");
            }
            if let Ok(multipart) = g.inspect_is_multipart(root) {
                if multipart {
                    println!("    Multipart:    yes");
                }
            }
            if let Ok(live) = g.inspect_is_live(root) {
                if live {
                    println!("    Live CD:      yes");
                }
            }
            if let Ok(netinst) = g.inspect_is_netinst(root) {
                if netinst {
                    println!("    NetInstall:   yes");
                }
            }

            // Try to mount and get additional info
            if verbose {
                eprintln!(
                    "[VERBOSE] Attempting to mount root filesystem for detailed inspection..."
                );
            }
            if g.mount(root, "/").is_ok() {
                // Filesystem usage
                if verbose {
                    eprintln!("[VERBOSE] Getting filesystem usage statistics...");
                }
                if let Ok(usage) = g.statvfs("/") {
                    let blocks = *usage.get("blocks").unwrap_or(&0);
                    let bsize = *usage.get("bsize").unwrap_or(&4096);
                    let bfree = *usage.get("bfree").unwrap_or(&0);

                    let total_bytes = blocks * bsize;
                    let free_bytes = bfree * bsize;
                    let used_bytes = total_bytes - free_bytes;
                    let used_percent = if total_bytes > 0 {
                        (used_bytes as f64 / total_bytes as f64) * 100.0
                    } else {
                        0.0
                    };

                    println!("    Disk usage:");
                    println!("      Total: {:.2} GB", total_bytes as f64 / 1e9);
                    println!(
                        "      Used:  {:.2} GB ({:.1}%)",
                        used_bytes as f64 / 1e9,
                        used_percent
                    );
                    println!("      Free:  {:.2} GB", free_bytes as f64 / 1e9);
                }

                // Count installed packages
                if verbose {
                    eprintln!("[VERBOSE] Counting installed packages...");
                }
                match g.inspect_get_package_format(root) {
                    Ok(pkg_fmt) if pkg_fmt == "rpm" => {
                        if let Ok(packages) = g.rpm_list() {
                            println!("    Installed RPM packages: {}", packages.len());
                            if verbose {
                                eprintln!("[VERBOSE] Found {} RPM packages", packages.len());
                            }
                        }
                    }
                    Ok(pkg_fmt) if pkg_fmt == "deb" => {
                        if let Ok(packages) = g.dpkg_list() {
                            println!("    Installed DEB packages: {}", packages.len());
                            if verbose {
                                eprintln!("[VERBOSE] Found {} DEB packages", packages.len());
                            }
                        }
                    }
                    _ => {}
                }

                // Kernel information
                if verbose {
                    eprintln!("[VERBOSE] Searching for kernel versions...");
                }
                if let Ok(files) = g.ls("/boot") {
                    let kernels: Vec<_> = files
                        .iter()
                        .filter(|f| f.starts_with("vmlinuz-") || f.starts_with("vmlinux-"))
                        .collect();
                    if !kernels.is_empty() {
                        println!("    Installed kernels:");
                        for kernel in kernels {
                            println!("      {}", kernel);
                            if verbose {
                                eprintln!("[VERBOSE] Found kernel: {}", kernel);
                            }
                        }
                    }
                }

                g.umount("/").ok();
            } else if verbose {
                eprintln!("[VERBOSE] Could not mount root filesystem for detailed inspection");
            }

            // System Configuration
            if verbose {
                eprintln!("[VERBOSE] Gathering system configuration...");
            }
            println!("\n    === System Configuration ===");

            if let Ok(timezone) = g.inspect_timezone(root) {
                println!("      Timezone: {}", timezone);
            }

            if let Ok(locale) = g.inspect_locale(root) {
                println!("      Locale:   {}", locale);
            }

            // SELinux
            if let Ok(selinux) = g.inspect_selinux(root) {
                if selinux != "disabled" {
                    println!("      SELinux:  {}", selinux);
                }
            }

            // Cloud-init
            if let Ok(has_cloud_init) = g.inspect_cloud_init(root) {
                if has_cloud_init {
                    println!("      Cloud-init: yes");
                }
            }

            // VM Tools
            if verbose {
                eprintln!("[VERBOSE] Detecting virtualization guest tools...");
            }
            if let Ok(vm_tools) = g.inspect_vm_tools(root) {
                if !vm_tools.is_empty() {
                    println!("      VM Tools: {}", vm_tools.join(", "));
                }
            }

            // Network Configuration
            if verbose {
                eprintln!("[VERBOSE] Analyzing network configuration...");
            }
            if let Ok(interfaces) = g.inspect_network(root) {
                if !interfaces.is_empty() {
                    println!("\n    === Network Configuration ===");
                    for iface in &interfaces {
                        println!("      Interface: {}", iface.name);
                        if !iface.ip_address.is_empty() {
                            println!("        IP: {}", iface.ip_address.join(", "));
                        }
                        if !iface.mac_address.is_empty() {
                            println!("        MAC: {}", iface.mac_address);
                        }
                        println!("        DHCP: {}", if iface.dhcp { "yes" } else { "no" });
                    }
                }
            }

            if let Ok(dns_servers) = g.inspect_dns(root) {
                if !dns_servers.is_empty() {
                    println!("      DNS Servers: {}", dns_servers.join(", "));
                }
            }

            // User Accounts
            if verbose {
                eprintln!("[VERBOSE] Listing user accounts...");
            }
            if let Ok(users) = g.inspect_users(root) {
                let regular_users: Vec<_> = users
                    .iter()
                    .filter(|u| {
                        let uid: i32 = u.uid.parse().unwrap_or(0);
                        (1000..65534).contains(&uid)
                    })
                    .collect();

                if !regular_users.is_empty() {
                    println!("\n    === User Accounts ===");
                    println!("      Regular users: {}", regular_users.len());
                    for user in regular_users.iter().take(10) {
                        println!(
                            "        {} (uid: {}, home: {})",
                            user.username, user.uid, user.home
                        );
                    }
                    if regular_users.len() > 10 {
                        println!("        ... and {} more", regular_users.len() - 10);
                    }
                }

                let system_users: Vec<_> = users
                    .iter()
                    .filter(|u| {
                        let uid: i32 = u.uid.parse().unwrap_or(0);
                        uid > 0 && uid < 1000
                    })
                    .collect();
                println!("      System users: {}", system_users.len());
            }

            // SSH Configuration
            if verbose {
                eprintln!("[VERBOSE] Checking SSH configuration...");
            }
            if let Ok(ssh_config) = g.inspect_ssh_config(root) {
                if !ssh_config.is_empty() {
                    println!("\n    === SSH Configuration ===");
                    if let Some(port) = ssh_config.get("Port") {
                        println!("      Port: {}", port);
                    }
                    if let Some(permit_root) = ssh_config.get("PermitRootLogin") {
                        println!("      PermitRootLogin: {}", permit_root);
                    }
                    if let Some(password_auth) = ssh_config.get("PasswordAuthentication") {
                        println!("      PasswordAuthentication: {}", password_auth);
                    }
                }
            }

            // Systemd Services
            if verbose {
                eprintln!("[VERBOSE] Listing systemd services...");
            }
            if let Ok(services) = g.inspect_systemd_services(root) {
                if !services.is_empty() {
                    println!("\n    === Systemd Services ===");
                    println!("      Enabled services: {}", services.len());
                    for service in services.iter().take(15) {
                        println!("        {}", service.name);
                    }
                    if services.len() > 15 {
                        println!("        ... and {} more", services.len() - 15);
                    }
                }
            }

            // Language Runtimes
            if verbose {
                eprintln!("[VERBOSE] Detecting language runtimes...");
            }
            if let Ok(runtimes) = g.inspect_runtimes(root) {
                if !runtimes.is_empty() {
                    println!("\n    === Language Runtimes ===");
                    for (runtime, version) in &runtimes {
                        println!("      {}: {}", runtime, version);
                    }
                }
            }

            // Container Runtimes
            if verbose {
                eprintln!("[VERBOSE] Detecting container runtimes...");
            }
            if let Ok(container_runtimes) = g.inspect_container_runtimes(root) {
                if !container_runtimes.is_empty() {
                    println!("\n    === Container Runtimes ===");
                    for runtime in &container_runtimes {
                        println!("      {}", runtime);
                    }
                }
            }

            // Storage Configuration
            if verbose {
                eprintln!("[VERBOSE] Analyzing storage configuration...");
            }
            if let Ok(lvm_info) = g.inspect_lvm(root) {
                if !lvm_info.physical_volumes.is_empty()
                    || !lvm_info.volume_groups.is_empty()
                    || !lvm_info.logical_volumes.is_empty()
                {
                    println!("\n    === LVM Configuration ===");
                    if !lvm_info.physical_volumes.is_empty() {
                        println!(
                            "      Physical Volumes: {}",
                            lvm_info.physical_volumes.join(", ")
                        );
                    }
                    if !lvm_info.volume_groups.is_empty() {
                        println!("      Volume Groups: {}", lvm_info.volume_groups.join(", "));
                    }
                    if !lvm_info.logical_volumes.is_empty() {
                        println!(
                            "      Logical Volumes: {}",
                            lvm_info.logical_volumes.join(", ")
                        );
                    }
                }
            }

            // Swap
            if let Ok(swap_devices) = g.inspect_swap(root) {
                if !swap_devices.is_empty() {
                    println!("\n    === Swap Configuration ===");
                    for swap in &swap_devices {
                        println!("      {}", swap);
                    }
                }
            }

            // fstab mounts
            if let Ok(fstab_mounts) = g.inspect_fstab(root) {
                if fstab_mounts.len() > 1 {
                    println!("\n    === Filesystem Mounts (fstab) ===");
                    for (device, mountpoint, fstype) in fstab_mounts.iter().take(10) {
                        println!("      {} on {} type {}", device, mountpoint, fstype);
                    }
                }
            }

            // Boot Configuration
            if verbose {
                eprintln!("[VERBOSE] Analyzing boot configuration...");
            }
            if let Ok(boot_config) = g.inspect_boot_config(root) {
                if boot_config.bootloader != "unknown" {
                    println!("\n    === Boot Configuration ===");
                    println!("      Bootloader: {}", boot_config.bootloader);
                    if boot_config.timeout != "unknown" {
                        println!("      Timeout: {}", boot_config.timeout);
                    }
                    if boot_config.default_entry != "unknown" {
                        println!("      Default: {}", boot_config.default_entry);
                    }
                }
            }

            // Scheduled Tasks
            if verbose {
                eprintln!("[VERBOSE] Checking scheduled tasks...");
            }
            if let Ok(cron_jobs) = g.inspect_cron(root) {
                if !cron_jobs.is_empty() {
                    println!("\n    === Cron Jobs ===");
                    println!("      Total: {}", cron_jobs.len());
                    for job in cron_jobs.iter().take(5) {
                        println!("        {}", job);
                    }
                    if cron_jobs.len() > 5 {
                        println!("        ... and {} more", cron_jobs.len() - 5);
                    }
                }
            }

            if let Ok(timers) = g.inspect_systemd_timers(root) {
                if !timers.is_empty() {
                    println!("\n    === Systemd Timers ===");
                    for timer in &timers {
                        println!("      {}", timer);
                    }
                }
            }

            // SSL Certificates
            if verbose {
                eprintln!("[VERBOSE] Scanning SSL certificates...");
            }
            if let Ok(certs) = g.inspect_certificates(root) {
                if !certs.is_empty() {
                    println!("\n    === SSL Certificates ===");
                    println!("      Found: {} certificates", certs.len());
                    for cert in certs.iter().take(5) {
                        println!("        {}", cert);
                    }
                    if certs.len() > 5 {
                        println!("        ... and {} more", certs.len() - 5);
                    }
                }
            }

            // Kernel Parameters
            if verbose {
                eprintln!("[VERBOSE] Reading kernel parameters...");
            }
            if let Ok(kernel_params) = g.inspect_kernel_params(root) {
                if !kernel_params.is_empty() {
                    println!("\n    === Kernel Parameters (sysctl) ===");
                    println!("      Total: {}", kernel_params.len());
                    let mut params_vec: Vec<_> = kernel_params.iter().collect();
                    params_vec.sort_by_key(|&(k, _)| k);
                    for (key, value) in params_vec.iter().take(10) {
                        println!("        {} = {}", key, value);
                    }
                    if kernel_params.len() > 10 {
                        println!("        ... and {} more", kernel_params.len() - 10);
                    }
                }
            }
        }
    }

    if verbose {
        eprintln!("[VERBOSE] Shutting down appliance...");
    }
    g.shutdown()?;

    if verbose {
        eprintln!("[VERBOSE] Inspection complete");
    }
    Ok(())
}

/// List files in a disk image at specified path
pub fn list_files(image: &PathBuf, path: &str, verbose: bool) -> Result<()> {
    let mut g = Guestfs::new()?;
    g.set_verbose(verbose);

    let progress = ProgressReporter::spinner("Loading disk image...");

    g.add_drive_ro(image.to_str().unwrap())?;

    progress.set_message("Launching appliance...");
    g.launch()?;

    // Auto-mount root filesystem
    progress.set_message("Detecting OS...");
    let roots = g.inspect_os()?;
    if roots.is_empty() {
        progress.abandon_with_message("No operating system found in image");
        anyhow::bail!("No operating system found in image");
    }

    progress.set_message("Mounting filesystems...");
    let mountpoints = g.inspect_get_mountpoints(&roots[0])?;
    for (mp, device) in mountpoints {
        g.mount(&device, &mp)?;
    }

    // List files
    progress.set_message(format!("Listing {}...", path));
    let files = g.ls(path)?;

    progress.finish_and_clear();

    println!("Files in {}:", path);

    for file in files {
        let full_path = if path == "/" {
            format!("/{}", file)
        } else {
            format!("{}/{}", path, file)
        };

        if let Ok(stat) = g.stat(&full_path) {
            let file_type = if (stat.mode & 0o170000) == 0o040000 {
                "dir "
            } else if (stat.mode & 0o170000) == 0o120000 {
                "link"
            } else {
                "file"
            };

            println!(
                "  {} {:>10} {:o} {}",
                file_type,
                stat.size,
                stat.mode & 0o7777,
                file
            );
        } else {
            println!("  ?    {}", file);
        }
    }

    g.umount_all()?;
    g.shutdown()?;
    Ok(())
}

/// Extract a file from disk image
pub fn extract_file(
    image: &PathBuf,
    guest_path: &str,
    host_path: &PathBuf,
    verbose: bool,
) -> Result<()> {
    let mut g = Guestfs::new()?;
    g.set_verbose(verbose);

    let progress = ProgressReporter::spinner(&format!(
        "Extracting {} from {}",
        guest_path,
        image.display()
    ));

    g.add_drive_ro(image.to_str().unwrap())?;

    progress.set_message("Launching appliance...");
    g.launch()?;

    // Auto-mount
    progress.set_message("Detecting OS...");
    let roots = g.inspect_os()?;
    if roots.is_empty() {
        progress.abandon_with_message("No operating system found in image");
        anyhow::bail!("No operating system found in image");
    }

    progress.set_message("Mounting filesystems...");
    let mountpoints = g.inspect_get_mountpoints(&roots[0])?;
    for (mp, device) in mountpoints {
        g.mount(&device, &mp)?;
    }

    // Check if file exists
    if !g.exists(guest_path)? {
        progress.abandon_with_message(format!("File not found: {}", guest_path));
        anyhow::bail!("File not found: {}", guest_path);
    }

    // Download file
    progress.set_message(format!("Downloading {}...", guest_path));
    g.download(guest_path, host_path.to_str().unwrap())?;

    let size = g.filesize(guest_path)?;

    progress.finish_and_clear();

    println!("✓ Extracted {} bytes to {}", size, host_path.display());

    g.umount_all()?;
    g.shutdown()?;
    Ok(())
}

/// Execute a command in the guest
pub fn execute_command(image: &PathBuf, command: &[String], verbose: bool) -> Result<()> {
    let mut g = Guestfs::new()?;
    g.set_verbose(verbose);

    let progress = ProgressReporter::spinner("Loading disk image...");

    g.add_drive_ro(image.to_str().unwrap())?;

    progress.set_message("Launching appliance...");
    g.launch()?;

    // Auto-mount
    progress.set_message("Detecting OS...");
    let roots = g.inspect_os()?;
    if roots.is_empty() {
        progress.abandon_with_message("No operating system found in image");
        anyhow::bail!("No operating system found in image");
    }

    progress.set_message("Mounting filesystems...");
    let mountpoints = g.inspect_get_mountpoints(&roots[0])?;
    for (mp, device) in mountpoints {
        g.mount(&device, &mp)?;
    }

    // Execute command
    progress.set_message(format!("Executing command: {}", command.join(" ")));
    let cmd_args: Vec<&str> = command.iter().map(|s| s.as_str()).collect();
    let output = g.command(&cmd_args)?;

    progress.finish_and_clear();

    println!("{}", output);

    g.umount_all()?;
    g.shutdown()?;
    Ok(())
}

/// Backup files from guest to host
pub fn backup_files(
    image: &PathBuf,
    guest_path: &str,
    output_tar: &PathBuf,
    verbose: bool,
) -> Result<()> {
    let mut g = Guestfs::new()?;
    g.set_verbose(verbose);

    let progress = ProgressReporter::spinner(&format!(
        "Backing up {} from {}",
        guest_path,
        image.display()
    ));

    g.add_drive_ro(image.to_str().unwrap())?;

    progress.set_message("Launching appliance...");
    g.launch()?;

    // Auto-mount
    progress.set_message("Detecting OS...");
    let roots = g.inspect_os()?;
    if roots.is_empty() {
        progress.abandon_with_message("No operating system found in image");
        anyhow::bail!("No operating system found in image");
    }

    progress.set_message("Mounting filesystems...");
    let mountpoints = g.inspect_get_mountpoints(&roots[0])?;
    for (mp, device) in mountpoints {
        g.mount(&device, &mp)?;
    }

    // Create tar archive in guest
    progress.set_message(format!("Creating archive from {}...", guest_path));
    let temp_tar = "/tmp/backup.tar.gz";
    g.tar_out_opts(
        guest_path,
        temp_tar,
        Some("gzip"),
        false,
        false,
        false,
        false,
    )?;

    // Download to host
    progress.set_message("Downloading archive...");
    g.download(temp_tar, output_tar.to_str().unwrap())?;

    let size = g.filesize(temp_tar)?;

    progress.finish_and_clear();

    println!(
        "✓ Backup complete: {} bytes to {}",
        size,
        output_tar.display()
    );

    g.umount_all()?;
    g.shutdown()?;
    Ok(())
}

/// Create a new disk image
pub fn create_disk(path: &PathBuf, size_mb: u64, format: &str, verbose: bool) -> Result<()> {
    let mut g = Guestfs::new()?;
    g.set_verbose(verbose);

    println!(
        "Creating {} MB {} disk: {}",
        size_mb,
        format,
        path.display()
    );

    let size_bytes = (size_mb * 1024 * 1024) as i64;
    g.disk_create(path.to_str().unwrap(), format, size_bytes)?;

    println!("✓ Disk created successfully");

    Ok(())
}

/// Check filesystem on a disk image
pub fn check_filesystem(image: &PathBuf, device: Option<String>, verbose: bool) -> Result<()> {
    let mut g = Guestfs::new()?;
    g.set_verbose(verbose);

    let progress =
        ProgressReporter::spinner(&format!("Checking filesystem on {}", image.display()));

    g.add_drive_ro(image.to_str().unwrap())?;

    progress.set_message("Launching appliance...");
    g.launch()?;

    progress.set_message("Detecting filesystems...");
    let check_device = if let Some(dev) = device {
        dev
    } else {
        // Use first partition
        let partitions = g.list_partitions()?;
        if partitions.is_empty() {
            progress.abandon_with_message("No partitions found");
            anyhow::bail!("No partitions found");
        }
        partitions[0].clone()
    };

    let fstype = g.vfs_type(&check_device)?;

    progress.set_message(format!("Running fsck on {} ({})...", check_device, fstype));
    g.fsck(&fstype, &check_device)?;

    progress.finish_and_clear();

    println!(
        "✓ Filesystem check complete for {} ({})",
        check_device, fstype
    );

    g.shutdown()?;
    Ok(())
}

/// Show disk usage statistics
pub fn show_disk_usage(image: &PathBuf, verbose: bool) -> Result<()> {
    let mut g = Guestfs::new()?;
    g.set_verbose(verbose);

    let progress = ProgressReporter::spinner("Loading disk image...");

    g.add_drive_ro(image.to_str().unwrap())?;

    progress.set_message("Launching appliance...");
    g.launch()?;

    // Auto-mount
    progress.set_message("Detecting OS...");
    let roots = g.inspect_os()?;
    if roots.is_empty() {
        progress.abandon_with_message("No operating system found in image");
        anyhow::bail!("No operating system found in image");
    }

    progress.set_message("Mounting filesystems...");
    let mountpoints = g.inspect_get_mountpoints(&roots[0])?;
    for (mp, device) in mountpoints {
        g.mount(&device, &mp)?;
    }

    // Get disk usage
    progress.set_message("Calculating disk usage...");
    let df = g.df()?;

    progress.finish_and_clear();

    println!("\n=== Disk Usage ===");
    println!("{}", df);

    g.umount_all()?;
    g.shutdown()?;
    Ok(())
}

/// Diff two disk images
pub fn diff_images(
    image1: &PathBuf,
    image2: &PathBuf,
    verbose: bool,
    output_format: Option<OutputFormat>,
) -> Result<()> {
    println!("Comparing: {} vs {}\n", image1.display(), image2.display());

    // Inspect first image
    let mut g1 = Guestfs::new()?;
    g1.set_verbose(verbose);
    g1.add_drive_ro(image1.to_str().unwrap())?;
    g1.launch()?;

    let roots1 = g1.inspect_os()?;
    if roots1.is_empty() {
        eprintln!("No operating system found in first image");
        g1.shutdown()?;
        return Ok(());
    }

    let report1 = collect_inspection_data(&mut g1, &roots1[0], verbose)?;
    g1.shutdown()?;

    // Inspect second image
    let mut g2 = Guestfs::new()?;
    g2.set_verbose(verbose);
    g2.add_drive_ro(image2.to_str().unwrap())?;
    g2.launch()?;

    let roots2 = g2.inspect_os()?;
    if roots2.is_empty() {
        eprintln!("No operating system found in second image");
        g2.shutdown()?;
        return Ok(());
    }

    let report2 = collect_inspection_data(&mut g2, &roots2[0], verbose)?;
    g2.shutdown()?;

    // Compute diff
    use super::diff::InspectionDiff;
    let diff = InspectionDiff::compute(&report1, &report2);

    // Output
    if let Some(format) = output_format {
        let _formatter = get_formatter(format, true);
        let output = serde_json::to_string_pretty(&diff)?;
        println!("{}", output);
    } else {
        diff.print();
    }

    Ok(())
}

/// Compare multiple VMs against a baseline
pub fn compare_images(baseline: &PathBuf, images: &[PathBuf], verbose: bool) -> Result<()> {
    println!(
        "Comparing {} images against baseline: {}\n",
        images.len(),
        baseline.display()
    );

    // Inspect baseline
    let mut g_baseline = Guestfs::new()?;
    g_baseline.set_verbose(verbose);
    g_baseline.add_drive_ro(baseline.to_str().unwrap())?;
    g_baseline.launch()?;

    let roots_baseline = g_baseline.inspect_os()?;
    if roots_baseline.is_empty() {
        eprintln!("No operating system found in baseline image");
        g_baseline.shutdown()?;
        return Ok(());
    }

    let baseline_report = collect_inspection_data(&mut g_baseline, &roots_baseline[0], verbose)?;
    g_baseline.shutdown()?;

    // Print header
    println!("=== Comparison Report ===\n");
    println!(
        "{:<20} {:<15} {:<15} {:<15}",
        "Metric", "Baseline", "VM1", "VM2"
    );
    println!("{:-<65}", "");

    // Compare each image
    for (idx, image) in images.iter().enumerate() {
        let mut g = Guestfs::new()?;
        g.set_verbose(verbose);
        g.add_drive_ro(image.to_str().unwrap())?;
        g.launch()?;

        let roots = g.inspect_os()?;
        if roots.is_empty() {
            eprintln!("No operating system found in {}", image.display());
            g.shutdown()?;
            continue;
        }

        let report = collect_inspection_data(&mut g, &roots[0], verbose)?;
        g.shutdown()?;

        // Print comparison row
        if idx == 0 {
            // Print baseline values
            let hostname = baseline_report.os.hostname.as_deref().unwrap_or("N/A");
            let version = baseline_report
                .os
                .version
                .as_ref()
                .map(|v| format!("{}.{}", v.major, v.minor))
                .unwrap_or_else(|| "N/A".to_string());
            let pkg_count = baseline_report
                .packages
                .as_ref()
                .map(|p| p.count.to_string())
                .unwrap_or_else(|| "N/A".to_string());

            println!(
                "{:<20} {:<15} {:<15}",
                "Hostname",
                hostname,
                report.os.hostname.as_deref().unwrap_or("N/A")
            );
            println!(
                "{:<20} {:<15} {:<15}",
                "OS Version",
                version,
                report
                    .os
                    .version
                    .as_ref()
                    .map(|v| format!("{}.{}", v.major, v.minor))
                    .unwrap_or_else(|| "N/A".to_string())
            );
            println!(
                "{:<20} {:<15} {:<15}",
                "Package Count",
                pkg_count,
                report
                    .packages
                    .as_ref()
                    .map(|p| p.count.to_string())
                    .unwrap_or_else(|| "N/A".to_string())
            );
        }
    }

    println!("\n");
    Ok(())
}

/// Inspect multiple disk images in batch mode
pub fn inspect_batch(
    images: &[PathBuf],
    parallel: usize,
    verbose: bool,
    output_format: Option<OutputFormat>,
    use_cache: bool,
) -> Result<()> {
    use super::cache::InspectionCache;
    use std::sync::{Arc, Mutex};
    use std::thread;

    println!("=== Batch Inspection ===");
    println!("Images: {}", images.len());
    println!("Parallel workers: {}", parallel);
    println!();

    // Shared results vector
    let results: Arc<Mutex<Vec<(String, Result<InspectionReport>)>>> =
        Arc::new(Mutex::new(Vec::new()));

    // Create work queue
    let work_queue: Arc<Mutex<Vec<PathBuf>>> = Arc::new(Mutex::new(images.to_vec()));

    // Progress tracking
    let total = images.len();
    let completed = Arc::new(Mutex::new(0usize));

    // Spawn worker threads
    let mut handles = vec![];

    for worker_id in 0..parallel {
        let work_queue = Arc::clone(&work_queue);
        let results = Arc::clone(&results);
        let completed = Arc::clone(&completed);

        let handle = thread::spawn(move || {
            loop {
                // Get next image from queue
                let image = {
                    let mut queue = work_queue.lock().unwrap();
                    if queue.is_empty() {
                        break;
                    }
                    queue.pop().unwrap()
                };

                if verbose {
                    eprintln!("[Worker {}] Processing: {}", worker_id, image.display());
                }

                // Try cache first if enabled
                let report_result = if use_cache {
                    if let Ok(cache) = InspectionCache::new() {
                        if let Ok(Some(cached)) = cache.get(&image) {
                            eprintln!("✓ [Worker {}] Cache hit: {}", worker_id, image.display());
                            Ok(cached)
                        } else {
                            inspect_single_image(&image, verbose, use_cache)
                        }
                    } else {
                        inspect_single_image(&image, verbose, use_cache)
                    }
                } else {
                    inspect_single_image(&image, verbose, use_cache)
                };

                // Store result
                {
                    let mut res = results.lock().unwrap();
                    res.push((image.to_string_lossy().to_string(), report_result));
                }

                // Update progress
                {
                    let mut count = completed.lock().unwrap();
                    *count += 1;
                    eprintln!("Progress: {}/{}", *count, total);
                }
            }
        });

        handles.push(handle);
    }

    // Wait for all workers to complete
    for handle in handles {
        handle.join().unwrap();
    }

    println!("\n=== Results ===\n");

    // Print results
    let final_results = results.lock().unwrap();
    let mut success_count = 0;
    let mut error_count = 0;

    for (image_path, result) in final_results.iter() {
        match result {
            Ok(report) => {
                success_count += 1;

                if let Some(format) = output_format {
                    // JSON/YAML output
                    let formatter = get_formatter(format, true);
                    let output = formatter.format(report)?;
                    println!("=== {} ===", image_path);
                    println!("{}", output);
                    println!();
                } else {
                    // Summary output
                    println!("✓ {}", image_path);
                    println!(
                        "  OS: {} {}",
                        report.os.distribution.as_deref().unwrap_or("Unknown"),
                        report
                            .os
                            .version
                            .as_ref()
                            .map(|v| format!("{}.{}", v.major, v.minor))
                            .unwrap_or_else(|| "N/A".to_string())
                    );
                    if let Some(hostname) = &report.os.hostname {
                        println!("  Hostname: {}", hostname);
                    }
                    if let Some(packages) = &report.packages {
                        println!("  Packages: {}", packages.count);
                    }
                    println!();
                }
            }
            Err(e) => {
                error_count += 1;
                println!("✗ {}", image_path);
                println!("  Error: {}", e);
                println!();
            }
        }
    }

    println!("=== Summary ===");
    println!("Total: {}", final_results.len());
    println!("Success: {}", success_count);
    println!("Errors: {}", error_count);

    Ok(())
}

/// Inspect a single image (helper for batch processing)
fn inspect_single_image(
    image: &PathBuf,
    verbose: bool,
    use_cache: bool,
) -> Result<InspectionReport> {
    use super::cache::InspectionCache;

    let mut g = Guestfs::new()?;
    g.set_verbose(false); // Disable verbose for batch mode to reduce noise

    g.add_drive_ro(image.to_str().unwrap())?;
    g.launch()?;

    let roots = g.inspect_os()?;
    if roots.is_empty() {
        g.shutdown()?;
        return Err(anyhow::anyhow!("No operating system found"));
    }

    let mut report = collect_inspection_data(&mut g, &roots[0], verbose)?;
    report.image_path = Some(image.to_string_lossy().to_string());

    g.shutdown()?;

    // Store in cache if enabled
    if use_cache {
        if let Ok(cache) = InspectionCache::new() {
            let _ = cache.store(image, &report);
        }
    }

    Ok(report)
}
