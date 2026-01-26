// SPDX-License-Identifier: LGPL-3.0-or-later
//! REPL (Read-Eval-Print Loop) for interactive shell

use anyhow::{Context, Result};
use colored::Colorize;
use rustyline::error::ReadlineError;
use rustyline::DefaultEditor;
use std::path::Path;

use super::commands::{self, ShellContext};
use guestctl::Guestfs;

/// Run interactive shell
pub fn run_interactive_shell<P: AsRef<Path>>(image_path: P) -> Result<()> {
    println!("\n{}", "╔═══════════════════════════════════════════════════════════╗".cyan());
    println!("{}", "║          GuestKit Interactive Shell v0.3.1              ║".cyan().bold());
    println!("{}", "╚═══════════════════════════════════════════════════════════╝".cyan());
    println!();
    println!("{} Loading VM image...", "→".cyan());

    // Initialize guestfs
    let mut guestfs = Guestfs::new().context("Failed to create Guestfs handle")?;
    guestfs.add_drive_opts(image_path.as_ref().to_str().unwrap(), false, None)
        .context("Failed to add drive")?;
    guestfs.launch().context("Failed to launch guestfs")?;

    // Inspect and mount
    let roots = guestfs.inspect_os().context("Failed to inspect OS")?;
    if roots.is_empty() {
        anyhow::bail!("No operating systems found");
    }

    let root = &roots[0];
    println!("{} Detected OS: {}", "→".cyan(), root.yellow());

    let mounts = guestfs.inspect_get_mountpoints(root)
        .context("Failed to get mountpoints")?;

    for (mountpoint, device) in mounts {
        if let Err(e) = guestfs.mount(&device, &mountpoint) {
            eprintln!("{} Failed to mount {}: {}", "⚠".yellow(), mountpoint, e);
        }
    }

    println!("{} VM filesystem mounted successfully", "✓".green());
    println!();
    println!("{} for available commands", "Type 'help'".yellow().bold());
    println!("{} to exit", "Type 'exit' or press Ctrl+D".yellow().bold());
    println!();

    // Create shell context
    let mut ctx = ShellContext::new(guestfs, root.to_string());

    // Create readline editor with history
    let mut rl = DefaultEditor::new()?;

    // Load history if exists
    let history_path = dirs::home_dir()
        .map(|p| p.join(".guestkit_history"))
        .unwrap_or_else(|| std::path::PathBuf::from(".guestkit_history"));

    let _ = rl.load_history(&history_path);

    // REPL loop
    loop {
        let prompt = format!("{}{}{}> ",
            "guestkit:".cyan().bold(),
            ctx.current_path.yellow(),
            "".clear());

        match rl.readline(&prompt) {
            Ok(line) => {
                let line = line.trim();

                if line.is_empty() {
                    continue;
                }

                // Add to history
                let _ = rl.add_history_entry(line);

                // Parse command
                let parts: Vec<&str> = line.split_whitespace().collect();
                if parts.is_empty() {
                    continue;
                }

                let cmd = parts[0];
                let args = &parts[1..];

                // Execute command
                match cmd {
                    "exit" | "quit" | "q" => {
                        println!("{} Shutting down...", "→".cyan());
                        break;
                    }
                    "help" | "?" => {
                        let _ = commands::cmd_help(&ctx, args);
                    }
                    "clear" | "cls" => {
                        print!("\x1B[2J\x1B[1;1H");
                    }
                    "history" => {
                        for (i, entry) in rl.history().iter().enumerate() {
                            println!("{:4}  {}", i + 1, entry);
                        }
                    }
                    "ls" => {
                        let _ = commands::cmd_ls(&mut ctx, args);
                    }
                    "cat" => {
                        let _ = commands::cmd_cat(&mut ctx, args);
                    }
                    "cd" => {
                        let _ = commands::cmd_cd(&mut ctx, args);
                    }
                    "pwd" => {
                        let _ = commands::cmd_pwd(&ctx, args);
                    }
                    "find" => {
                        let _ = commands::cmd_find(&mut ctx, args);
                    }
                    "grep" => {
                        let _ = commands::cmd_grep(&mut ctx, args);
                    }
                    "info" => {
                        let _ = commands::cmd_info(&mut ctx, args);
                    }
                    "mounts" => {
                        let _ = commands::cmd_mounts(&mut ctx, args);
                    }
                    "packages" => {
                        cmd_packages(&mut ctx, args);
                    }
                    "services" => {
                        cmd_services(&mut ctx, args);
                    }
                    "users" => {
                        cmd_users(&mut ctx);
                    }
                    "network" => {
                        cmd_network(&mut ctx);
                    }
                    "security" => {
                        cmd_security(&mut ctx);
                    }
                    "health" => {
                        cmd_health(&mut ctx);
                    }
                    "risks" => {
                        cmd_risks(&mut ctx);
                    }
                    "ai" => {
                        let _ = commands::cmd_ai(&mut ctx, args);
                    }
                    _ => {
                        eprintln!("{} Unknown command: {}. Type 'help' for available commands.",
                                "Error:".red(), cmd);
                    }
                }
            }
            Err(ReadlineError::Interrupted) => {
                println!("{} Use 'exit' to quit", "^C".yellow());
            }
            Err(ReadlineError::Eof) => {
                println!();
                println!("{} Shutting down...", "→".cyan());
                break;
            }
            Err(err) => {
                eprintln!("{} {}", "Error:".red(), err);
                break;
            }
        }
    }

    // Save history
    let _ = rl.save_history(&history_path);

    // Shutdown
    ctx.guestfs.shutdown()?;
    println!("{} Goodbye!", "✓".green());

    Ok(())
}

// Enhanced command implementations with analysis

fn cmd_packages(ctx: &mut ShellContext, args: &[&str]) {
    println!("{} Analyzing packages...", "→".cyan());

    if let Ok(pkg_info) = ctx.guestfs.inspect_packages(&ctx.root) {
        let packages = &pkg_info.packages;
        let filtered: Vec<_> = if args.is_empty() {
            packages.iter().take(20).collect()
        } else {
            let pattern = args[0].to_lowercase();
            packages.iter()
                .filter(|p| p.name.to_lowercase().contains(&pattern))
                .collect()
        };

        println!("\n{} ({} shown, {} total)",
                "Installed Packages".yellow().bold(),
                filtered.len(),
                packages.len());
        println!("{}", "─".repeat(60).cyan());

        for pkg in filtered {
            println!("{:30} {}",
                    pkg.name.green(),
                    pkg.version.to_string().bright_black());
        }

        if args.is_empty() && packages.len() > 20 {
            println!("\n{} Use 'packages <pattern>' to search", "Tip:".yellow());
        }
        println!();
    }
}

fn cmd_services(ctx: &mut ShellContext, args: &[&str]) {
    println!("{} Analyzing services...", "→".cyan());

    if let Ok(services) = ctx.guestfs.inspect_systemd_services(&ctx.root) {
        let filtered: Vec<_> = if args.is_empty() {
            services.iter().collect()
        } else {
            let pattern = args[0].to_lowercase();
            services.iter()
                .filter(|s| s.name.to_lowercase().contains(&pattern))
                .collect()
        };

        println!("\n{} ({} shown)",
                "System Services".yellow().bold(),
                filtered.len());
        println!("{}", "─".repeat(60).cyan());

        for svc in filtered {
            let status = if svc.enabled { "enabled".green() } else { "disabled".bright_black() };
            println!("{:40} {}", svc.name, status);
        }
        println!();
    }
}

fn cmd_users(ctx: &mut ShellContext) {
    println!("{} Analyzing user accounts...", "→".cyan());

    if let Ok(users) = ctx.guestfs.inspect_users(&ctx.root) {
        println!("\n{} ({} total)",
                "User Accounts".yellow().bold(),
                users.len());
        println!("{}", "─".repeat(80).cyan());
        println!("{:20} {:10} {:10} {:30}",
                "Username".bold(), "UID".bold(), "GID".bold(), "Home".bold());
        println!("{}", "─".repeat(80).cyan());

        for user in users {
            let uid_color = if user.uid == "0" {
                user.uid.red()
            } else {
                user.uid.normal()
            };

            println!("{:20} {:10} {:10} {}",
                    user.username.green(),
                    uid_color,
                    user.gid.bright_black(),
                    user.home.bright_black());
        }
        println!();
    }
}

fn cmd_network(ctx: &mut ShellContext) {
    println!("{} Analyzing network configuration...", "→".cyan());

    if let Ok(interfaces) = ctx.guestfs.inspect_network(&ctx.root) {
        println!("\n{} ({} total)",
                "Network Interfaces".yellow().bold(),
                interfaces.len());
        println!("{}", "─".repeat(70).cyan());

        for iface in interfaces {
            println!("\n{}", iface.name.green().bold());
            // Note: NetworkInterface struct may have different fields
            // Adjust based on actual struct definition
        }

        if let Ok(dns) = ctx.guestfs.inspect_dns(&ctx.root) {
            if !dns.is_empty() {
                println!("\n{}", "DNS Servers".yellow().bold());
                for server in dns {
                    println!("  • {}", server.cyan());
                }
            }
        }
        println!();
    }
}

fn cmd_security(ctx: &mut ShellContext) {
    println!("{} Analyzing security configuration...", "→".cyan());

    if let Ok(sec) = ctx.guestfs.inspect_security(&ctx.root) {
        println!("\n{}", "Security Features".yellow().bold());
        println!("{}", "─".repeat(60).cyan());

        let selinux_status = if &sec.selinux != "disabled" {
            sec.selinux.green()
        } else {
            sec.selinux.red()
        };
        println!("  SELinux:  {}", selinux_status);

        let apparmor = if sec.apparmor { "enabled".green() } else { "disabled".red() };
        println!("  AppArmor: {}", apparmor);

        let fail2ban = if sec.fail2ban { "installed".green() } else { "not installed".bright_black() };
        println!("  fail2ban: {}", fail2ban);

        let aide = if sec.aide { "installed".green() } else { "not installed".bright_black() };
        println!("  AIDE:     {}", aide);

        let auditd = if sec.auditd { "enabled".green() } else { "disabled".red() };
        println!("  auditd:   {}", auditd);

        if let Ok(fw) = ctx.guestfs.inspect_firewall(&ctx.root) {
            let fw_status = if fw.enabled {
                format!("{} ({})", "enabled".green(), fw.firewall_type)
            } else {
                format!("{} ({})", "disabled".red(), fw.firewall_type)
            };
            println!("  Firewall: {}", fw_status);
        }

        println!();
    }
}

fn cmd_health(_ctx: &mut ShellContext) {
    println!("\n{}", "System Health Score".yellow().bold());
    println!("{}", "─".repeat(60).cyan());
    println!("\n{} This command requires full TUI mode.", "Note:".yellow());
    println!("{} Run 'guestctl tui <image>' for complete health analysis", "Tip:".cyan());
    println!();
}

fn cmd_risks(_ctx: &mut ShellContext) {
    println!("\n{}", "Security Risks".yellow().bold());
    println!("{}", "─".repeat(60).cyan());
    println!("\n{} This command requires full TUI mode.", "Note:".yellow());
    println!("{} Run 'guestctl tui <image>' to view security issues", "Tip:".cyan());
    println!();
}
