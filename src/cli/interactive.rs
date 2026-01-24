// SPDX-License-Identifier: LGPL-3.0-or-later
//! Interactive REPL mode for guestctl CLI

use super::errors::errors;
use anyhow::{Context, Result};
use guestctl::Guestfs;
use owo_colors::OwoColorize;
use rustyline::completion::{Completer, Pair};
use rustyline::error::ReadlineError;
use rustyline::highlight::Highlighter;
use rustyline::hint::Hinter;
use rustyline::validate::Validator;
use rustyline::{Context as RustyContext, Editor, Helper, Result as RustyResult};
use std::collections::hash_map::DefaultHasher;
use std::collections::HashMap;
use std::fs;
use std::hash::{Hash, Hasher};
use std::path::{Path, PathBuf};

/// Get the history directory path (~/.guestctl/history/)
fn get_history_dir() -> Result<PathBuf> {
    let home =
        dirs::home_dir().ok_or_else(|| anyhow::anyhow!("Could not determine home directory"))?;
    let history_dir = home.join(".guestctl").join("history");

    // Create directory if it doesn't exist
    if !history_dir.exists() {
        fs::create_dir_all(&history_dir).with_context(|| {
            format!(
                "Failed to create history directory: {}",
                history_dir.display()
            )
        })?;
    }

    Ok(history_dir)
}

/// Get history file path for a specific disk image
/// Uses a hash of the disk path to create a unique history file
fn get_history_file(disk_path: &Path) -> Result<PathBuf> {
    let history_dir = get_history_dir()?;

    // Create a hash of the disk path for a unique filename
    let mut hasher = DefaultHasher::new();
    disk_path.to_string_lossy().hash(&mut hasher);
    let hash = hasher.finish();

    let filename = format!("guestctl-{:x}.history", hash);
    Ok(history_dir.join(filename))
}

/// Helper for rustyline completion
struct GuestkitHelper;

impl Helper for GuestkitHelper {}
impl Hinter for GuestkitHelper {
    type Hint = String;
}
impl Highlighter for GuestkitHelper {}
impl Validator for GuestkitHelper {}

impl Completer for GuestkitHelper {
    type Candidate = Pair;

    fn complete(
        &self,
        line: &str,
        pos: usize,
        _ctx: &RustyContext<'_>,
    ) -> RustyResult<(usize, Vec<Pair>)> {
        let before_cursor = &line[..pos];
        let parts: Vec<&str> = before_cursor.split_whitespace().collect();

        // Command completion
        if parts.is_empty() || (parts.len() == 1 && !before_cursor.ends_with(' ')) {
            let commands = vec![
                "help",
                "info",
                "filesystems",
                "fs",
                "mount",
                "umount",
                "unmount",
                "mounts",
                "ls",
                "cat",
                "head",
                "find",
                "stat",
                "download",
                "dl",
                "packages",
                "pkg",
                "services",
                "svc",
                "users",
                "network",
                "net",
                "clear",
                "cls",
                "exit",
                "quit",
                "q",
            ];

            let prefix = parts.last().unwrap_or(&"");
            let matches: Vec<Pair> = commands
                .iter()
                .filter(|cmd| cmd.starts_with(prefix))
                .map(|cmd| Pair {
                    display: cmd.to_string(),
                    replacement: cmd.to_string(),
                })
                .collect();

            let start = if parts.is_empty() {
                0
            } else {
                before_cursor.len() - prefix.len()
            };

            return Ok((start, matches));
        }

        // Future: Path completion would go here
        // For now, just command completion is implemented

        Ok((0, vec![]))
    }
}

/// Interactive session state
pub struct InteractiveSession {
    handle: Guestfs,
    editor: Editor<GuestkitHelper, rustyline::history::DefaultHistory>,
    disk_path: PathBuf,
    mounted: HashMap<String, String>, // device -> mountpoint
    current_root: Option<String>,
}

impl InteractiveSession {
    /// Create a new interactive session for the given disk image
    pub fn new(disk_path: PathBuf) -> Result<Self> {
        println!(
            "{}",
            "Initializing GuestKit Interactive Mode...".cyan().bold()
        );
        println!();

        // Create handle
        let mut handle = Guestfs::new().context("Failed to create guestfs handle")?;

        // Add drive
        println!("  {} Loading disk: {}", "→".cyan(), disk_path.display());
        handle
            .add_drive_ro(disk_path.to_str().unwrap())
            .context("Failed to add drive")?;

        // Launch
        println!("  {} Launching appliance...", "→".cyan());
        handle.launch().context("Failed to launch guestfs")?;

        // Auto-inspect
        println!("  {} Inspecting disk...", "→".cyan());
        let roots = handle.inspect_os().unwrap_or_default();
        let current_root = roots.first().cloned();

        if let Some(ref root) = current_root {
            if let Ok(os_type) = handle.inspect_get_type(root) {
                if let Ok(distro) = handle.inspect_get_distro(root) {
                    if let (Ok(major), Ok(minor)) = (
                        handle.inspect_get_major_version(root),
                        handle.inspect_get_minor_version(root),
                    ) {
                        println!();
                        println!(
                            "  {} Found: {} {} {}.{}",
                            "✓".green().bold(),
                            os_type.bright_cyan(),
                            distro.bright_cyan(),
                            major.to_string().bright_white(),
                            minor.to_string().bright_white()
                        );
                    }
                }
            }
        }

        // Create editor with tab completion
        let mut editor = Editor::new().context("Failed to create line editor")?;
        editor.set_helper(Some(GuestkitHelper));

        // Load command history if available
        if let Ok(history_file) = get_history_file(&disk_path) {
            if history_file.exists()
                && editor.load_history(&history_file).is_ok() {
                    println!("  {} Loaded command history", "→".cyan());
                }
        }

        println!();
        println!(
            "{}",
            "Ready! Type 'help' for commands, 'exit' to quit.".bright_green()
        );
        println!("{}", "Tip: Press TAB for command completion".dimmed());
        println!(
            "{}",
            "Tip: Use ↑/↓ arrows to browse command history".dimmed()
        );
        println!();

        Ok(Self {
            handle,
            editor,
            disk_path,
            mounted: HashMap::new(),
            current_root,
        })
    }

    /// Run the interactive session
    pub fn run(&mut self) -> Result<()> {
        loop {
            let prompt = format!("{}> ", "guestctl".bright_cyan().bold());

            match self.editor.readline(&prompt) {
                Ok(line) => {
                    let line = line.trim();
                    if line.is_empty() {
                        continue;
                    }

                    // Add to history
                    let _ = self.editor.add_history_entry(line);

                    // Handle exit
                    if line == "exit" || line == "quit" || line == "q" {
                        println!("Goodbye!");
                        break;
                    }

                    // Execute command
                    if let Err(e) = self.execute_command(line) {
                        eprintln!("{} {}", "Error:".red().bold(), e);
                    }
                }
                Err(ReadlineError::Interrupted) => {
                    println!("^C");
                    println!("Use 'exit' or 'quit' to leave interactive mode");
                }
                Err(ReadlineError::Eof) => {
                    println!("Goodbye!");
                    break;
                }
                Err(err) => {
                    eprintln!("{} {:?}", "Error:".red().bold(), err);
                    break;
                }
            }
        }

        // Save command history before exiting
        if let Ok(history_file) = get_history_file(&self.disk_path) {
            if let Err(e) = self.editor.save_history(&history_file) {
                eprintln!(
                    "{} Failed to save command history: {}",
                    "Warning:".yellow(),
                    e
                );
            }
        }

        Ok(())
    }

    /// Execute a command
    fn execute_command(&mut self, line: &str) -> Result<()> {
        let parts: Vec<&str> = line.split_whitespace().collect();
        if parts.is_empty() {
            return Ok(());
        }

        match parts[0] {
            "help" | "?" => self.cmd_help(),
            "info" => self.cmd_info(),
            "filesystems" | "fs" => self.cmd_filesystems(),
            "mount" => self.cmd_mount(&parts[1..]),
            "umount" | "unmount" => self.cmd_umount(&parts[1..]),
            "mounts" => self.cmd_mounts(),
            "ls" => self.cmd_ls(&parts[1..]),
            "cat" => self.cmd_cat(&parts[1..]),
            "head" => self.cmd_head(&parts[1..]),
            "find" => self.cmd_find(&parts[1..]),
            "stat" => self.cmd_stat(&parts[1..]),
            "download" | "dl" => self.cmd_download(&parts[1..]),
            "packages" | "pkg" => self.cmd_packages(&parts[1..]),
            "services" | "svc" => self.cmd_services(),
            "users" => self.cmd_users(),
            "network" | "net" => self.cmd_network(),
            "clear" | "cls" => self.cmd_clear(),
            _ => {
                let available = vec![
                    "help",
                    "info",
                    "filesystems",
                    "mount",
                    "umount",
                    "mounts",
                    "ls",
                    "cat",
                    "head",
                    "find",
                    "stat",
                    "download",
                    "packages",
                    "services",
                    "users",
                    "network",
                    "clear",
                    "exit",
                ];
                let err = errors::unknown_command(parts[0], &available);
                err.display();
                Ok(())
            }
        }
    }

    /// Show help
    fn cmd_help(&self) -> Result<()> {
        println!();
        println!("{}", "GuestKit Interactive Commands:".bright_cyan().bold());
        println!();

        println!("{}", "  System Information:".bright_white().bold());
        println!("    {}  - Show disk and OS information", "info".cyan());
        println!();

        println!("{}", "  Filesystem Operations:".bright_white().bold());
        println!(
            "    {}  - List available filesystems",
            "filesystems, fs".cyan()
        );
        println!(
            "    {}  - Mount a filesystem",
            "mount <device> <path>".cyan()
        );
        println!("    {}  - Unmount a filesystem", "umount <path>".cyan());
        println!("    {}  - Show mounted filesystems", "mounts".cyan());
        println!();

        println!("{}", "  File Operations:".bright_white().bold());
        println!("    {}  - List directory contents", "ls [path]".cyan());
        println!("    {}  - Display file contents", "cat <path>".cyan());
        println!(
            "    {}  - Display first lines of file",
            "head <path> [lines]".cyan()
        );
        println!(
            "    {}  - Find files by name pattern",
            "find <pattern>".cyan()
        );
        println!("    {}  - Show file information", "stat <path>".cyan());
        println!(
            "    {}  - Download file from disk",
            "download <src> <dest>".cyan()
        );
        println!();

        println!("{}", "  System Inspection:".bright_white().bold());
        println!(
            "    {}  - List installed packages",
            "packages, pkg [filter]".cyan()
        );
        println!("    {}  - List system services", "services, svc".cyan());
        println!("    {}  - List user accounts", "users".cyan());
        println!(
            "    {}  - Show network configuration",
            "network, net".cyan()
        );
        println!();

        println!("{}", "  Other:".bright_white().bold());
        println!("    {}  - Clear screen", "clear, cls".cyan());
        println!("    {}  - Show this help", "help, ?".cyan());
        println!("    {}  - Exit interactive mode", "exit, quit, q".cyan());
        println!();

        Ok(())
    }

    /// Show disk and OS information
    fn cmd_info(&mut self) -> Result<()> {
        println!();
        println!("{}", "Disk Information:".bright_cyan().bold());
        println!("  Path: {}", self.disk_path.display());

        if let Some(ref root) = self.current_root {
            println!();
            println!("{}", "Operating System:".bright_cyan().bold());

            if let Ok(os_type) = self.handle.inspect_get_type(root) {
                println!("  Type: {}", os_type.bright_white());
            }

            if let Ok(distro) = self.handle.inspect_get_distro(root) {
                println!("  Distribution: {}", distro.bright_white());
            }

            if let (Ok(major), Ok(minor)) = (
                self.handle.inspect_get_major_version(root),
                self.handle.inspect_get_minor_version(root),
            ) {
                println!(
                    "  Version: {}.{}",
                    major.to_string().bright_white(),
                    minor.to_string().bright_white()
                );
            }

            if let Ok(hostname) = self.handle.inspect_get_hostname(root) {
                println!("  Hostname: {}", hostname.bright_white());
            }

            if let Ok(arch) = self.handle.inspect_get_arch(root) {
                println!("  Architecture: {}", arch.bright_white());
            }
        } else {
            println!("\n  No operating system detected");
        }

        println!();
        Ok(())
    }

    /// List filesystems
    fn cmd_filesystems(&mut self) -> Result<()> {
        let filesystems = self
            .handle
            .list_filesystems()
            .context("Failed to list filesystems")?;

        println!();
        println!("{}", "Available Filesystems:".bright_cyan().bold());
        println!();

        for (device, fstype) in filesystems {
            let mounted = if self.mounted.contains_key(&device) {
                format!(" {} at {}", "→".green(), self.mounted[&device])
            } else {
                String::new()
            };

            println!(
                "  {} {}{}",
                device.bright_white().bold(),
                fstype.yellow(),
                mounted
            );
        }

        println!();
        Ok(())
    }

    /// Mount a filesystem
    fn cmd_mount(&mut self, args: &[&str]) -> Result<()> {
        if args.len() < 2 {
            println!(
                "{} Usage: mount <device> <mountpoint>",
                "Error:".red().bold()
            );
            println!("Example: mount /dev/sda1 /");
            return Ok(());
        }

        let device = args[0];
        let mountpoint = args[1];

        self.handle
            .mount(device, mountpoint)
            .with_context(|| format!("Failed to mount {} at {}", device, mountpoint))?;

        self.mounted
            .insert(device.to_string(), mountpoint.to_string());

        println!(
            "{} Mounted {} at {}",
            "✓".green().bold(),
            device.bright_white(),
            mountpoint.bright_cyan()
        );

        Ok(())
    }

    /// Unmount a filesystem
    fn cmd_umount(&mut self, args: &[&str]) -> Result<()> {
        if args.is_empty() {
            println!("{} Usage: umount <mountpoint>", "Error:".red().bold());
            return Ok(());
        }

        let mountpoint = args[0];

        self.handle
            .umount(mountpoint)
            .with_context(|| format!("Failed to unmount {}", mountpoint))?;

        // Remove from mounted map
        self.mounted.retain(|_, mp| mp != mountpoint);

        println!(
            "{} Unmounted {}",
            "✓".green().bold(),
            mountpoint.bright_cyan()
        );

        Ok(())
    }

    /// Show mounted filesystems
    fn cmd_mounts(&self) -> Result<()> {
        if self.mounted.is_empty() {
            println!("No filesystems mounted");
            println!("Use 'mount <device> <path>' to mount a filesystem");
            return Ok(());
        }

        println!();
        println!("{}", "Mounted Filesystems:".bright_cyan().bold());
        println!();

        for (device, mountpoint) in &self.mounted {
            println!(
                "  {} {} {}",
                device.bright_white().bold(),
                "→".cyan(),
                mountpoint.yellow()
            );
        }

        println!();
        Ok(())
    }

    /// List directory contents
    fn cmd_ls(&mut self, args: &[&str]) -> Result<()> {
        let path = if args.is_empty() { "/" } else { args[0] };

        let entries = self
            .handle
            .ls(path)
            .with_context(|| format!("Failed to list directory: {}", path))?;

        println!();
        for entry in &entries {
            println!("  {}", entry);
        }
        println!();
        println!(
            "{} entries",
            entries.len().to_string().bright_white().bold()
        );
        println!();

        Ok(())
    }

    /// Display file contents
    fn cmd_cat(&mut self, args: &[&str]) -> Result<()> {
        if args.is_empty() {
            println!("{} Usage: cat <path>", "Error:".red().bold());
            return Ok(());
        }

        let path = args[0];
        let content = self
            .handle
            .cat(path)
            .with_context(|| format!("Failed to read file: {}", path))?;

        println!();
        print!("{}", content);
        if !content.ends_with('\n') {
            println!();
        }
        println!();

        Ok(())
    }

    /// Display first lines of file
    fn cmd_head(&mut self, args: &[&str]) -> Result<()> {
        if args.is_empty() {
            println!("{} Usage: head <path> [lines]", "Error:".red().bold());
            return Ok(());
        }

        let path = args[0];
        let lines = if args.len() > 1 {
            args[1].parse::<usize>().unwrap_or(10)
        } else {
            10
        };

        let content = self
            .handle
            .cat(path)
            .with_context(|| format!("Failed to read file: {}", path))?;

        println!();
        for (i, line) in content.lines().enumerate() {
            if i >= lines {
                break;
            }
            println!("{}", line);
        }
        println!();

        Ok(())
    }

    /// Find files by pattern
    fn cmd_find(&mut self, args: &[&str]) -> Result<()> {
        if args.is_empty() {
            println!("{} Usage: find <pattern>", "Error:".red().bold());
            println!("Example: find '*.conf'");
            return Ok(());
        }

        let pattern = args[0];

        // Use guestfs glob functionality
        let results = self
            .handle
            .glob_expand(pattern)
            .with_context(|| format!("Failed to find files matching: {}", pattern))?;

        println!();
        if results.is_empty() {
            println!("No files found matching '{}'", pattern);
        } else {
            for path in &results {
                println!("  {}", path.bright_white());
            }
            println!();
            println!(
                "{} matches",
                results.len().to_string().bright_white().bold()
            );
        }
        println!();

        Ok(())
    }

    /// Show file information
    fn cmd_stat(&mut self, args: &[&str]) -> Result<()> {
        if args.is_empty() {
            println!("{} Usage: stat <path>", "Error:".red().bold());
            return Ok(());
        }

        let path = args[0];
        let stat = self
            .handle
            .stat(path)
            .with_context(|| format!("Failed to stat: {}", path))?;

        println!();
        println!("{}", "File Information:".bright_cyan().bold());
        println!("  Path: {}", path.bright_white());
        println!("  Size: {} bytes", stat.size.to_string().bright_white());
        println!("  Mode: {:o}", stat.mode);
        println!("  UID: {}", stat.uid);
        println!("  GID: {}", stat.gid);
        println!();

        Ok(())
    }

    /// Download file from disk
    fn cmd_download(&mut self, args: &[&str]) -> Result<()> {
        if args.len() < 2 {
            println!(
                "{} Usage: download <source> <destination>",
                "Error:".red().bold()
            );
            println!("Example: download /etc/hostname ./hostname.txt");
            return Ok(());
        }

        let source = args[0];
        let dest = args[1];

        self.handle
            .download(source, dest)
            .with_context(|| format!("Failed to download {} to {}", source, dest))?;

        println!(
            "{} Downloaded {} to {}",
            "✓".green().bold(),
            source.bright_white(),
            dest.bright_cyan()
        );

        Ok(())
    }

    /// List packages
    fn cmd_packages(&mut self, args: &[&str]) -> Result<()> {
        let filter = if args.is_empty() { None } else { Some(args[0]) };

        if let Some(ref root) = self.current_root {
            let apps = self
                .handle
                .inspect_list_applications(root)
                .context("Failed to list packages")?;

            let filtered: Vec<_> = if let Some(f) = filter {
                apps.iter().filter(|app| app.name.contains(f)).collect()
            } else {
                apps.iter().collect()
            };

            println!();
            if filtered.is_empty() {
                if filter.is_some() {
                    println!("No packages found matching '{}'", filter.unwrap());
                } else {
                    println!("No packages found");
                }
            } else {
                for app in filtered.iter().take(50) {
                    println!(
                        "  {} {} {}",
                        app.name.bright_white().bold(),
                        app.version.yellow(),
                        app.description.dimmed()
                    );
                }

                if filtered.len() > 50 {
                    println!();
                    println!(
                        "  {} (showing first 50 of {})",
                        "...".dimmed(),
                        filtered.len().to_string().bright_white()
                    );
                }
            }
            println!();
            println!(
                "{} packages total",
                apps.len().to_string().bright_white().bold()
            );
            println!();
        } else {
            println!("No operating system detected");
        }

        Ok(())
    }

    /// List services
    fn cmd_services(&mut self) -> Result<()> {
        if let Some(ref root) = self.current_root {
            let services = self
                .handle
                .inspect_systemd_services(root)
                .context("Failed to list services")?;

            println!();
            println!("{}", "Enabled Services:".bright_cyan().bold());
            println!();

            for service in &services {
                println!("  {} {}", "▶".green(), service.name.bright_white());
            }

            println!();
            println!(
                "{} services enabled",
                services.len().to_string().bright_white().bold()
            );
            println!();
        } else {
            println!("No operating system detected");
        }

        Ok(())
    }

    /// List users
    fn cmd_users(&mut self) -> Result<()> {
        if let Some(ref root) = self.current_root {
            let users = self
                .handle
                .inspect_users(root)
                .context("Failed to list users")?;

            println!();
            println!("{}", "User Accounts:".bright_cyan().bold());
            println!();

            for user in &users {
                let uid: i32 = user.uid.parse().unwrap_or(0);
                let color = if uid == 0 {
                    owo_colors::Style::new().red().bold()
                } else if uid >= 1000 {
                    owo_colors::Style::new().bright_white()
                } else {
                    owo_colors::Style::new().yellow()
                };

                println!(
                    "  {} (uid: {}, shell: {})",
                    user.username.style(color),
                    user.uid,
                    user.shell
                );
            }

            println!();
            println!("{} users", users.len().to_string().bright_white().bold());
            println!();
        } else {
            println!("No operating system detected");
        }

        Ok(())
    }

    /// Show network configuration
    fn cmd_network(&mut self) -> Result<()> {
        if let Some(ref root) = self.current_root {
            let interfaces = self
                .handle
                .inspect_network(root)
                .context("Failed to get network configuration")?;

            println!();
            println!("{}", "Network Interfaces:".bright_cyan().bold());
            println!();

            for iface in &interfaces {
                println!(
                    "  {} {}",
                    iface.name.bright_white().bold(),
                    iface.mac_address.yellow()
                );
                for addr in &iface.ip_address {
                    println!("    {} {}", "→".cyan(), addr.bright_white());
                }
                if !iface.ip_address.is_empty() {
                    println!();
                }
            }

            if let Ok(dns_servers) = self.handle.inspect_dns(root) {
                if !dns_servers.is_empty() {
                    println!("{}", "DNS Servers:".bright_cyan().bold());
                    for dns in &dns_servers {
                        println!("  {}", dns.bright_white());
                    }
                    println!();
                }
            }
        } else {
            println!("No operating system detected");
        }

        Ok(())
    }

    /// Clear screen
    fn cmd_clear(&self) -> Result<()> {
        print!("\x1B[2J\x1B[1;1H");
        Ok(())
    }
}

impl Drop for InteractiveSession {
    fn drop(&mut self) {
        let _ = self.handle.shutdown();
    }
}
