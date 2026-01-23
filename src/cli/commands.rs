// SPDX-License-Identifier: LGPL-3.0-or-later
//! CLI commands implementation

use guestkit::Guestfs;
use guestkit::core::ProgressReporter;
use anyhow::{Context, Result};
use std::path::PathBuf;

/// Inspect a disk image and display OS information
pub fn inspect_image(image: &PathBuf, verbose: bool) -> Result<()> {
    let mut g = Guestfs::new()?;
    g.set_verbose(verbose);

    let progress = ProgressReporter::spinner(&format!("Inspecting: {}", image.display()));

    g.add_drive_ro(image.to_str().unwrap())?;

    progress.set_message("Launching appliance...");
    g.launch().context("Failed to launch")?;

    progress.set_message("Scanning disk...");

    // List devices
    println!("\n=== Block Devices ===");
    let devices = g.list_devices()?;
    for device in &devices {
        let size = g.blockdev_getsize64(device)?;
        println!("  {}: {} bytes ({:.2} GB)", device, size, size as f64 / 1e9);
    }

    // List partitions
    println!("\n=== Partitions ===");
    let partitions = g.list_partitions()?;
    for partition in &partitions {
        println!("  {}", partition);

        if let Ok(part_list) = g.part_list("/dev/sda") {
            let part_num = g.part_to_partnum(partition)?;
            if let Some(p) = part_list.iter().find(|p| p.part_num == part_num as i32) {
                println!("    Start: {} bytes", p.part_start);
                println!("    Size:  {} bytes ({:.2} GB)", p.part_size, p.part_size as f64 / 1e9);
            }
        }
    }

    // List filesystems
    println!("\n=== Filesystems ===");
    let filesystems = g.list_filesystems()?;
    for (device, fstype) in &filesystems {
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
    let roots = g.inspect_os()?;

    progress.finish_and_clear();

    println!("\n=== Operating Systems ===");

    if roots.is_empty() {
        println!("  No operating systems found");
    } else {
        for root in &roots {
            println!("  Root: {}", root);

            if let Ok(ostype) = g.inspect_get_type(root) {
                println!("    Type:         {}", ostype);
            }
            if let Ok(distro) = g.inspect_get_distro(root) {
                println!("    Distribution: {}", distro);
            }
            if let Ok(product) = g.inspect_get_product_name(root) {
                println!("    Product:      {}", product);
            }
            if let Ok(arch) = g.inspect_get_arch(root) {
                println!("    Architecture: {}", arch);
            }
            if let Ok(major) = g.inspect_get_major_version(root) {
                if let Ok(minor) = g.inspect_get_minor_version(root) {
                    println!("    Version:      {}.{}", major, minor);
                }
            }
            if let Ok(hostname) = g.inspect_get_hostname(root) {
                println!("    Hostname:     {}", hostname);
            }
            if let Ok(pkg_fmt) = g.inspect_get_package_format(root) {
                println!("    Packages:     {}", pkg_fmt);
            }
        }
    }

    g.shutdown()?;
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
    progress.set_message(&format!("Listing {}...", path));
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

            println!("  {} {:>10} {:o} {}",
                     file_type, stat.size, stat.mode & 0o7777, file);
        } else {
            println!("  ?    {}",  file);
        }
    }

    g.umount_all()?;
    g.shutdown()?;
    Ok(())
}

/// Extract a file from disk image
pub fn extract_file(image: &PathBuf, guest_path: &str, host_path: &PathBuf, verbose: bool) -> Result<()> {
    let mut g = Guestfs::new()?;
    g.set_verbose(verbose);

    let progress = ProgressReporter::spinner(&format!("Extracting {} from {}",
                                                      guest_path, image.display()));

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
        progress.abandon_with_message(&format!("File not found: {}", guest_path));
        anyhow::bail!("File not found: {}", guest_path);
    }

    // Download file
    progress.set_message(&format!("Downloading {}...", guest_path));
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
    progress.set_message(&format!("Executing command: {}", command.join(" ")));
    let cmd_args: Vec<&str> = command.iter().map(|s| s.as_str()).collect();
    let output = g.command(&cmd_args)?;

    progress.finish_and_clear();

    println!("{}", output);

    g.umount_all()?;
    g.shutdown()?;
    Ok(())
}

/// Backup files from guest to host
pub fn backup_files(image: &PathBuf, guest_path: &str, output_tar: &PathBuf, verbose: bool) -> Result<()> {
    let mut g = Guestfs::new()?;
    g.set_verbose(verbose);

    let progress = ProgressReporter::spinner(&format!("Backing up {} from {}",
                                                      guest_path, image.display()));

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
    progress.set_message(&format!("Creating archive from {}...", guest_path));
    let temp_tar = "/tmp/backup.tar.gz";
    g.tar_out_opts(guest_path, temp_tar, Some("gzip"), false, false, false, false)?;

    // Download to host
    progress.set_message("Downloading archive...");
    g.download(temp_tar, output_tar.to_str().unwrap())?;

    let size = g.filesize(temp_tar)?;

    progress.finish_and_clear();

    println!("✓ Backup complete: {} bytes to {}", size, output_tar.display());

    g.umount_all()?;
    g.shutdown()?;
    Ok(())
}

/// Create a new disk image
pub fn create_disk(path: &PathBuf, size_mb: u64, format: &str, verbose: bool) -> Result<()> {
    let mut g = Guestfs::new()?;
    g.set_verbose(verbose);

    println!("Creating {} MB {} disk: {}", size_mb, format, path.display());

    let size_bytes = (size_mb * 1024 * 1024) as i64;
    g.disk_create(path.to_str().unwrap(), format, size_bytes)?;

    println!("✓ Disk created successfully");

    Ok(())
}

/// Check filesystem on a disk image
pub fn check_filesystem(image: &PathBuf, device: Option<String>, verbose: bool) -> Result<()> {
    let mut g = Guestfs::new()?;
    g.set_verbose(verbose);

    let progress = ProgressReporter::spinner(&format!("Checking filesystem on {}", image.display()));

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

    progress.set_message(&format!("Running fsck on {} ({})...", check_device, fstype));
    g.fsck(&fstype, &check_device)?;

    progress.finish_and_clear();

    println!("✓ Filesystem check complete for {} ({})", check_device, fstype);

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
