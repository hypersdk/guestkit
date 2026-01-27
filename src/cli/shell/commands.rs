// SPDX-License-Identifier: LGPL-3.0-or-later
//! Command implementations for interactive shell

use anyhow::Result;
use guestkit::Guestfs;
use colored::Colorize;
use std::collections::HashMap;
use std::time::Instant;

#[cfg(feature = "ai")]
use reqwest;

#[cfg(feature = "ai")]
use rig::{
    client::completion::CompletionClient,
    completion::{AssistantContent, CompletionModel},
    providers::openai,
};

pub struct ShellContext {
    pub guestfs: Guestfs,
    pub root: String,
    pub current_path: String,
    pub aliases: HashMap<String, String>,
    pub bookmarks: HashMap<String, String>,
    pub last_command_time: Option<std::time::Duration>,
    pub command_count: usize,
    pub os_info: String,
}

impl ShellContext {
    pub fn new(guestfs: Guestfs, root: String) -> Self {
        let mut aliases = HashMap::new();

        // Add default aliases
        aliases.insert("ll".to_string(), "ls -l".to_string());
        aliases.insert("la".to_string(), "ls -a".to_string());
        aliases.insert("..".to_string(), "cd ..".to_string());
        aliases.insert("~".to_string(), "cd /".to_string());
        aliases.insert("q".to_string(), "quit".to_string());

        Self {
            guestfs,
            root,
            current_path: "/".to_string(),
            aliases,
            bookmarks: HashMap::new(),
            last_command_time: None,
            command_count: 0,
            os_info: String::new(),
        }
    }

    /// Get OS information for display
    pub fn get_os_info(&self) -> &str {
        if self.os_info.is_empty() {
            "Unknown OS"
        } else {
            &self.os_info
        }
    }

    /// Set OS information
    pub fn set_os_info(&mut self, info: String) {
        self.os_info = info;
    }

    /// Add an alias
    pub fn add_alias(&mut self, name: String, command: String) {
        self.aliases.insert(name, command);
    }

    /// Get alias expansion
    pub fn get_alias(&self, name: &str) -> Option<&String> {
        self.aliases.get(name)
    }

    /// Add a bookmark
    pub fn add_bookmark(&mut self, name: String, path: String) {
        self.bookmarks.insert(name, path);
    }

    /// Get bookmark path
    pub fn get_bookmark(&self, name: &str) -> Option<&String> {
        self.bookmarks.get(name)
    }

    /// Start timing a command
    #[allow(dead_code)]
    pub fn start_timing(&mut self) -> Instant {
        Instant::now()
    }

    /// End timing and store duration
    pub fn end_timing(&mut self, start: Instant) {
        self.last_command_time = Some(start.elapsed());
        self.command_count += 1;
    }
}

/// List files in current or specified directory
pub fn cmd_ls(ctx: &mut ShellContext, args: &[&str]) -> Result<()> {
    let path = if args.is_empty() {
        &ctx.current_path
    } else {
        args[0]
    };

    let full_path = resolve_path(&ctx.current_path, path);

    match ctx.guestfs.ls(&full_path) {
        Ok(entries) => {
            for entry in entries {
                // Try to get file type
                let entry_path = format!("{}/{}", full_path.trim_end_matches('/'), entry);
                let is_dir = ctx.guestfs.is_dir(&entry_path).unwrap_or(false);

                if is_dir {
                    println!("{}/", entry.blue().bold());
                } else {
                    println!("{}", entry);
                }
            }
            Ok(())
        }
        Err(e) => {
            // Check if it's a file (common mistake: ls on a file instead of cat)
            if ctx.guestfs.is_file(&full_path).unwrap_or(false) {
                eprintln!("{} '{}' is a file, not a directory", "Error:".red(), full_path);
                eprintln!("{} Use 'cat {}' to view the file contents", "Hint:".yellow(), full_path);
            } else {
                eprintln!("{} {}", "Error:".red(), e);
            }
            Ok(())
        }
    }
}

/// Show file contents
pub fn cmd_cat(ctx: &mut ShellContext, args: &[&str]) -> Result<()> {
    if args.is_empty() {
        eprintln!("{} cat <file>", "Usage:".yellow());
        return Ok(());
    }

    let path = resolve_path(&ctx.current_path, args[0]);

    match ctx.guestfs.read_file(&path) {
        Ok(contents) => {
            print!("{}", String::from_utf8_lossy(&contents));
            Ok(())
        }
        Err(e) => {
            eprintln!("{} {}", "Error:".red(), e);
            Ok(())
        }
    }
}

/// Change directory
pub fn cmd_cd(ctx: &mut ShellContext, args: &[&str]) -> Result<()> {
    let path = if args.is_empty() {
        "/"
    } else {
        args[0]
    };

    let new_path = resolve_path(&ctx.current_path, path);

    // Verify directory exists
    if ctx.guestfs.is_dir(&new_path).unwrap_or(false) {
        ctx.current_path = new_path;
        Ok(())
    } else {
        eprintln!("{} Not a directory: {}", "Error:".red(), new_path);
        Ok(())
    }
}

/// Print working directory
pub fn cmd_pwd(ctx: &ShellContext, _args: &[&str]) -> Result<()> {
    println!("{}", ctx.current_path);
    Ok(())
}

/// Find files matching pattern
pub fn cmd_find(ctx: &mut ShellContext, args: &[&str]) -> Result<()> {
    if args.is_empty() {
        eprintln!("{} find <pattern>", "Usage:".yellow());
        return Ok(());
    }

    let pattern = args[0];
    let search_path = if args.len() > 1 {
        resolve_path(&ctx.current_path, args[1])
    } else {
        ctx.current_path.clone()
    };

    println!("{} files matching '{}' from '{}'...", "Searching".cyan(), pattern, search_path);

    // Recursive search implementation
    search_recursive(ctx, &search_path, pattern, 0)?;

    Ok(())
}

fn search_recursive(ctx: &mut ShellContext, path: &str, pattern: &str, depth: usize) -> Result<()> {
    if depth > 10 {
        return Ok(()); // Limit recursion depth
    }

    if let Ok(entries) = ctx.guestfs.ls(path) {
        for entry in entries {
            if entry.contains(pattern) {
                let full_path = format!("{}/{}", path.trim_end_matches('/'), entry);
                println!("{}", full_path.green());
            }

            let full_path = format!("{}/{}", path.trim_end_matches('/'), entry);
            if ctx.guestfs.is_dir(&full_path).unwrap_or(false) && entry != "." && entry != ".." {
                let _ = search_recursive(ctx, &full_path, pattern, depth + 1);
            }
        }
    }

    Ok(())
}

/// Search file contents
pub fn cmd_grep(ctx: &mut ShellContext, args: &[&str]) -> Result<()> {
    if args.len() < 2 {
        eprintln!("{} grep <pattern> <file>", "Usage:".yellow());
        return Ok(());
    }

    let pattern = args[0];
    let path = resolve_path(&ctx.current_path, args[1]);

    match ctx.guestfs.read_file(&path) {
        Ok(contents) => {
            let text = String::from_utf8_lossy(&contents);
            for (i, line) in text.lines().enumerate() {
                if line.contains(pattern) {
                    println!("{}:{}", format!("{}", i + 1).cyan(),
                            line.replace(pattern, &pattern.red().to_string()));
                }
            }
            Ok(())
        }
        Err(e) => {
            eprintln!("{} {}", "Error:".red(), e);
            Ok(())
        }
    }
}

/// Show system information
pub fn cmd_info(ctx: &mut ShellContext, _args: &[&str]) -> Result<()> {
    println!("\n{}", "=== System Information ===".cyan().bold());

    if let Ok(os) = ctx.guestfs.inspect_get_type(&ctx.root) {
        println!("{} {}", "OS Type:".yellow(), os);
    }

    if let Ok(distro) = ctx.guestfs.inspect_get_distro(&ctx.root) {
        println!("{} {}", "Distribution:".yellow(), distro);
    }

    if let Ok(version) = ctx.guestfs.inspect_get_major_version(&ctx.root) {
        println!("{} {}", "Major Version:".yellow(), version);
    }

    if let Ok(hostname) = ctx.guestfs.inspect_get_hostname(&ctx.root) {
        println!("{} {}", "Hostname:".yellow(), hostname);
    }

    if let Ok(arch) = ctx.guestfs.inspect_get_arch(&ctx.root) {
        println!("{} {}", "Architecture:".yellow(), arch);
    }

    println!();
    Ok(())
}

/// AI-powered diagnostics (requires --features ai)
#[cfg(feature = "ai")]
pub fn cmd_ai(ctx: &mut ShellContext, args: &[&str]) -> Result<()> {
    if args.is_empty() {
        eprintln!("{} ai <query>", "Usage:".yellow());
        eprintln!("Example: ai why won't this boot?");
        return Ok(());
    }

    // Check for API key
    if std::env::var("OPENAI_API_KEY").is_err() {
        eprintln!("\n{} {}", "⚠".yellow().bold(), "OPENAI_API_KEY environment variable not set.".yellow());
        eprintln!("\nTo use AI features:");
        eprintln!("  1. Get an API key from https://platform.openai.com/api-keys");
        eprintln!("  2. Set the environment variable:");
        eprintln!("     export OPENAI_API_KEY='your-key-here'");
        eprintln!();
        return Ok(());
    }

    let query = args.join(" ");

    println!("\n{} {}", "🤖".bold(), "Analyzing VM...".cyan());
    println!();

    // Gather diagnostic context based on query
    let context = gather_diagnostic_context(&mut ctx.guestfs, &ctx.root, &query)?;

    println!("{} {}", "→".cyan(), "Consulting AI...".cyan());
    println!();

    // Call OpenAI
    let response = call_openai_simple(&query, &context)?;

    // Display response
    println!("{}", "═".repeat(70).cyan());
    println!("{}", "AI Analysis".yellow().bold());
    println!("{}", "═".repeat(70).cyan());
    println!();
    println!("{}", response);
    println!();
    println!("{}", "═".repeat(70).cyan());
    println!();

    println!("{} Review suggestions carefully before applying", "⚠".yellow().bold());
    println!();

    Ok(())
}

#[cfg(not(feature = "ai"))]
pub fn cmd_ai(_ctx: &mut ShellContext, _args: &[&str]) -> Result<()> {
    eprintln!("\n{} AI features not enabled.", "Error:".red().bold());
    eprintln!("Rebuild with: cargo build --features ai");
    eprintln!();
    Ok(())
}

#[cfg(feature = "ai")]
fn gather_diagnostic_context(guestfs: &mut Guestfs, root: &str, query: &str) -> Result<String> {
    use serde_json::json;

    let query_lower = query.to_lowercase();
    let mut context = String::new();

    context.push_str("=== VM Diagnostic Information ===\n\n");

    // Always include basic system info
    context.push_str("System Information:\n");
    let info = json!({
        "os_type": guestfs.inspect_get_type(root).ok(),
        "distro": guestfs.inspect_get_distro(root).ok(),
        "version": {
            "major": guestfs.inspect_get_major_version(root).ok(),
            "minor": guestfs.inspect_get_minor_version(root).ok(),
        },
        "hostname": guestfs.inspect_get_hostname(root).ok(),
        "architecture": guestfs.inspect_get_arch(root).ok(),
    });
    context.push_str(&serde_json::to_string_pretty(&info).unwrap_or_default());
    context.push('\n');

    // Conditional gathering based on query
    if query_lower.contains("lvm") || query_lower.contains("volume") || query_lower.contains("vg") {
        context.push_str("\nLVM Information:\n");
        if let Ok(lvm) = guestfs.inspect_lvm(root) {
            context.push_str(&serde_json::to_string_pretty(&lvm).unwrap_or_default());
            context.push('\n');
        }
    }

    if query_lower.contains("mount") || query_lower.contains("fstab") || query_lower.contains("filesystem") {
        context.push_str("\nCurrent Mounts:\n");
        if let Ok(mounts) = guestfs.mounts() {
            context.push_str(&mounts.join("\n"));
            context.push('\n');
        }

        context.push_str("\nfstab Configuration:\n");
        if let Ok(fstab) = guestfs.inspect_fstab(root) {
            context.push_str(&serde_json::to_string_pretty(&fstab).unwrap_or_default());
            context.push('\n');
        }
    }

    if query_lower.contains("boot") || query_lower.contains("kernel") || query_lower.contains("grub") {
        context.push_str("\nBoot Configuration:\n");
        if guestfs.is_dir("/boot").unwrap_or(false) {
            context.push_str("Boot directory accessible\n");
        }
    }

    if query_lower.contains("security") || query_lower.contains("selinux") || query_lower.contains("firewall") {
        context.push_str("\nSecurity Status:\n");
        if let Ok(sec) = guestfs.inspect_security(root) {
            context.push_str(&serde_json::to_string_pretty(&sec).unwrap_or_default());
            context.push('\n');
        }
    }

    // Always include block devices
    context.push_str("\nBlock Devices:\n");
    if let Ok(devices) = guestfs.list_devices() {
        for device in devices {
            let size = guestfs.blockdev_getsize64(&device).unwrap_or(0);
            context.push_str(&format!("{}: {} MB\n", device, size / 1024 / 1024));
        }
    }

    Ok(context)
}

#[cfg(feature = "ai")]
fn call_openai_simple(query: &str, context: &str) -> Result<String> {
    use anyhow::Context;

    const SYSTEM_PROMPT: &str = r#"You are an expert Linux system administrator and VM conversion specialist.

Your role is to diagnose VM boot failures, LVM issues, and filesystem problems.

When diagnosing issues:
1. Explain what you found
2. Identify the root cause
3. Suggest specific fixes
4. Provide exact commands when possible

Be concise but thorough. Focus on actionable solutions.

IMPORTANT: Never suggest destructive commands without clear warnings.
Always explain WHAT the command does and WHY it's needed.
"#;

    // Get API key from environment
    let api_key = std::env::var("OPENAI_API_KEY")
        .context("OPENAI_API_KEY environment variable not set")?;

    // Use tokio runtime for async call
    let runtime = tokio::runtime::Runtime::new()?;

    runtime.block_on(async {
        let full_prompt = format!(
            "{}\n\nUser Query: {}\n\n{}\n\nProvide a clear diagnosis and solution:",
            SYSTEM_PROMPT, query, context
        );

        // Create OpenAI client and call completion API using GPT-4o
        let response = openai::Client::<reqwest::Client>::new(&api_key)
            .context("Failed to create OpenAI client")?
            .completions_api()
            .completion_model(openai::GPT_4O)
            .completion_request(&full_prompt)
            .send()
            .await
            .context("Failed to get AI response")?;

        // Extract text from first choice
        match response.choice.first() {
            AssistantContent::Text(text) => Ok(text.text.clone()),
            _ => anyhow::bail!("Unexpected response type from AI"),
        }
    })
}

/// Show mounted filesystems
pub fn cmd_mounts(ctx: &mut ShellContext, _args: &[&str]) -> Result<()> {
    println!("\n{}", "=== Mounted Filesystems ===".cyan().bold());

    if let Ok(mounts) = ctx.guestfs.mounts() {
        for mount in mounts {
            println!("{}", mount.green());
        }
    }

    println!();
    Ok(())
}

/// Show help
pub fn cmd_help(_ctx: &ShellContext, _args: &[&str]) -> Result<()> {
    println!("\n{}", "=== GuestKit Interactive Shell ===".cyan().bold());
    println!("\n{}", "File System Commands:".yellow().bold());
    println!("  {}  - List directory contents", "ls [path]".green());
    println!("  {}  - Show file contents", "cat <file>".green());
    println!("  {}  - Change directory", "cd <path>".green());
    println!("  {}     - Print working directory", "pwd".green());
    println!("  {}  - Find files by name", "find <pattern> [path]".green());
    println!("  {} - Search in file", "grep <pattern> <file>".green());

    println!("\n{}", "System Commands:".yellow().bold());
    println!("  {}    - Show system information", "info".green());
    println!("  {}  - Show mounted filesystems", "mounts".green());
    println!("  {} - Search installed packages", "packages <pattern>".green());
    println!("  {} - List system services", "services [pattern]".green());
    println!("  {}   - List user accounts", "users".green());
    println!("  {} - Show network configuration", "network".green());

    println!("\n{}", "Analysis Commands:".yellow().bold());
    println!("  {} - Show security status", "security".green());
    println!("  {}  - Show system health score", "health".green());
    println!("  {}   - Show security risks", "risks".green());

    println!("\n{}", "Overview & Visualization:".yellow().bold());
    println!("  {} - Beautiful system dashboard", "dashboard, dash".green());
    println!("  {} - Quick system summary", "summary, sum".green());
    println!("  {} - Visualize directory tree", "tree [path] [depth]".green());
    println!("  {}     - Random helpful tip", "tips, tip".green());

    println!("\n{}", "Data Export & Reporting:".yellow().bold());
    println!("  {} - Export data in various formats", "export <type> <format> [file]".green());
    println!("           Types: packages, users, services, system");
    println!("           Formats: json, csv, md, txt");
    println!("           Example: export packages json packages.json");
    println!("  {} - Generate comprehensive snapshot report", "snapshot, snap [file]".green());
    println!("           Creates detailed Markdown report");
    println!("           Example: snapshot system-report.md");
    println!("  {}      - Compare and analyze", "diff <type> <filter>".green());
    println!("           Example: diff package kernel");

    #[cfg(feature = "ai")]
    {
        println!("\n{}", "AI Assistant:".yellow().bold());
        println!("  {}    - Ask AI for help (requires OPENAI_API_KEY)", "ai <query>".green());
        println!("           Example: ai why won't this boot?");
    }

    println!("\n{}", "Intelligence & Discovery:".yellow().bold());
    println!("  {} - Smart recommendations engine", "recommend, rec".green());
    println!("  {} - System profiling & detection", "profile <type>".green());
    println!("           Types: create, quick, detect, show");
    println!("  {} - Automatic system discovery", "discover, disco <type>".green());
    println!("           Types: files, apps, network, all");
    println!("  {}  - Formatted report generator", "report <type>".green());
    println!("           Types: executive, technical, security, compliance");

    println!("\n{}", "Guided Workflows:".yellow().bold());
    println!("  {} - Interactive task wizards", "wizard, wiz <type>".green());
    println!("           Types: security, health, packages, config, export");
    println!("  {}    - System scanners", "scan <type>".green());
    println!("           Types: security, issues, vulns, all");
    println!("  {} - File/directory comparison", "compare, cmp <type>".green());
    println!("           Types: files, dirs");

    println!("\n{}", "Advanced Features:".yellow().bold());
    println!("  {} - Smart search with filters", "search <pattern> [options]".green());
    println!("           Options: --path, --type, --content");
    println!("  {}   - Batch operations", "batch <operation>".green());
    println!("           Operations: cat, find, export");
    println!("  {}   - Watch files/directories", "watch <path>".green());
    println!("  {}     - Pin favorite commands", "pin [command]".green());

    println!("\n{}", "Quick Commands:".yellow().bold());
    println!("  {}   - Quick actions menu", "quick".green());
    println!("  {}   - Command cheat sheet", "cheat".green());
    println!("  {}  - Recently modified files", "recent [path] [limit]".green());
    println!("  {}       - Enhanced history analysis", "h".green());

    println!("\n{}", "Automation & Utilities:".yellow().bold());
    println!("  {}    - Automation presets", "auto <preset>".green());
    println!("           Presets: security-audit, full-analysis, health-check, export-all, documentation");
    println!("  {}    - Interactive command menu", "menu".green());
    println!("  {} - Session timeline visualization", "timeline".green());
    println!("  {}   - Performance benchmarking", "bench <type>".green());
    println!("           Types: files, list, packages, all");
    println!("  {} - Role-based command presets", "presets".green());

    println!("\n{}", "Learning & Guidance:".yellow().bold());
    println!("  {} - Interactive tutorials", "learn <tutorial>".green());
    println!("           Tutorials: basics, navigation, security, export, advanced, automation");
    println!("  {} - Context-aware suggestions", "context".green());
    println!("  {} - Focus on specific aspects", "focus <aspect>".green());
    println!("           Aspects: security, performance, network, storage, users");
    println!("  {} - Operational playbooks", "playbook <name>".green());
    println!("           Playbooks: incident, hardening, audit, forensics, migration, recovery");
    println!("  {} - Deep component inspection", "inspect <component>".green());
    println!("           Components: boot, logging, packages, services, kernel");

    println!("\n{}", "Planning & Strategy:".yellow().bold());
    println!("  {} - Narrative system explanations", "story <topic>".green());
    println!("           Topics: system, security, config, timeline");
    println!("  {} - Interactive advisor Q&A", "advisor <question>".green());
    println!("           Questions: secure, performance, troubleshoot, backup, monitoring, upgrade, compliance, migration");
    println!("  {} - System verification checks", "verify <check>".green());
    println!("           Checks: integrity, security, boot, network, all");
    println!("  {} - Optimization recommendations", "optimize".green());
    println!("  {} - Improvement roadmaps", "roadmap <timeframe>".green());
    println!("           Timeframes: 30-day, 90-day, annual");

    println!("\n{}", "Intelligence & Analytics:".yellow().bold());
    println!("  {} - AI-like intelligent insights", "insights".green());
    println!("  {}  - Comprehensive health diagnostic", "doctor".green());
    println!("  {}   - Goal setting and tracking", "goals <command>".green());
    println!("           Commands: suggest, list, check");
    println!("  {}  - Shell usage analysis", "habits".green());
    println!("  {} - Team collaboration reports", "collaborate <type>".green());
    println!("           Types: handoff, incident, change, status");

    println!("\n{}", "Shell Commands:".yellow().bold());
    println!("  {}    - Show this help", "help".green());
    println!("  {}   - Clear screen", "clear".green());
    println!("  {}   - Show command history", "history".green());
    println!("  {}    - Show shell statistics", "stats".green());
    println!("  {}    - Exit shell", "exit, quit, q".green());

    println!("\n{}", "Aliases & Bookmarks:".yellow().bold());
    println!("  {} - List all aliases", "alias".green());
    println!("  {} - Create an alias", "alias <name> <command>".green());
    println!("  {} - Remove an alias", "unalias <name>".green());
    println!("  {} - List bookmarks", "bookmark".green());
    println!("  {} - Bookmark current path", "bookmark <name>".green());
    println!("  {} - Bookmark specific path", "bookmark <name> <path>".green());
    println!("  {} - Jump to bookmark", "goto <name>".green());

    println!("\n{}", "Default Aliases:".yellow().bold());
    println!("  {} - Same as: ls -l", "ll".cyan());
    println!("  {} - Same as: ls -a", "la".cyan());
    println!("  {} - Same as: cd ..  ", "..".cyan());
    println!("  {}  - Same as: cd /   ", "~".cyan());
    println!("  {}  - Same as: quit  ", "q".cyan());

    println!("\n{}", "Tips:".yellow().bold());
    println!("  • Use {} for command completion", "Tab".cyan());
    println!("  • Use {} for command history", "↑/↓ arrows".cyan());
    println!("  • Paths are relative to current directory");
    println!("  • Commands taking >100ms show execution time");
    println!("  • Prompt shows: {}", "[OS] /current/path>".yellow());
    println!();

    Ok(())
}

/// Manage aliases
pub fn cmd_alias(ctx: &mut ShellContext, args: &[&str]) -> Result<()> {
    if args.is_empty() {
        // List all aliases
        println!("{}", "Current Aliases:".yellow().bold());
        let mut aliases: Vec<_> = ctx.aliases.iter().collect();
        aliases.sort_by_key(|(k, _)| *k);

        for (name, command) in aliases {
            println!("  {} = {}", name.cyan(), command.green());
        }
        println!();
        println!("{}", "Usage: alias <name> <command>".yellow());
        return Ok(());
    }

    if args.len() < 2 {
        println!("{}", "Usage: alias <name> <command>".red());
        println!("{}", "Example: alias ll ls -l".yellow());
        return Ok(());
    }

    let name = args[0].to_string();
    let command = args[1..].join(" ");

    ctx.add_alias(name.clone(), command.clone());
    println!("{} Alias added: {} = {}", "✓".green(), name.cyan(), command.green());

    Ok(())
}

/// Remove an alias
pub fn cmd_unalias(ctx: &mut ShellContext, args: &[&str]) -> Result<()> {
    if args.is_empty() {
        println!("{}", "Usage: unalias <name>".red());
        return Ok(());
    }

    let name = args[0];
    if ctx.aliases.remove(name).is_some() {
        println!("{} Alias removed: {}", "✓".green(), name.cyan());
    } else {
        println!("{} Alias not found: {}", "⚠".yellow(), name);
    }

    Ok(())
}

/// Manage bookmarks
pub fn cmd_bookmark(ctx: &mut ShellContext, args: &[&str]) -> Result<()> {
    if args.is_empty() {
        // List all bookmarks
        println!("{}", "Current Bookmarks:".yellow().bold());
        let mut bookmarks: Vec<_> = ctx.bookmarks.iter().collect();
        bookmarks.sort_by_key(|(k, _)| *k);

        for (name, path) in bookmarks {
            println!("  {} → {}", name.cyan(), path.blue());
        }

        if ctx.bookmarks.is_empty() {
            println!("  {}",  "No bookmarks set".yellow());
        }

        println!();
        println!("{}", "Usage:".yellow());
        println!("  {} - Add bookmark for current path", "bookmark <name>".green());
        println!("  {} - Add bookmark for specific path", "bookmark <name> <path>".green());
        println!("  {} - Jump to bookmark", "goto <name>".green());
        return Ok(());
    }

    let name = args[0].to_string();
    let path = if args.len() > 1 {
        args[1].to_string()
    } else {
        ctx.current_path.clone()
    };

    ctx.add_bookmark(name.clone(), path.clone());
    println!("{} Bookmark added: {} → {}", "✓".green(), name.cyan(), path.blue());

    Ok(())
}

/// Jump to a bookmark
pub fn cmd_goto(ctx: &mut ShellContext, args: &[&str]) -> Result<()> {
    if args.is_empty() {
        println!("{}", "Usage: goto <bookmark>".red());
        println!();
        cmd_bookmark(ctx, &[])?;  // Show available bookmarks
        return Ok(());
    }

    let name = args[0];
    if let Some(path) = ctx.get_bookmark(name) {
        let path = path.clone();  // Clone to avoid borrow conflict

        // Verify path exists
        if ctx.guestfs.is_dir(&path).unwrap_or(false) {
            ctx.current_path = path.clone();
            println!("{} Jumped to: {}", "→".cyan(), path.blue());
        } else {
            println!("{} Bookmark path no longer exists: {}", "⚠".yellow(), path);
        }
    } else {
        println!("{} Bookmark not found: {}", "⚠".yellow(), name);
        println!();
        cmd_bookmark(ctx, &[])?;  // Show available bookmarks
    }

    Ok(())
}

/// Show shell statistics
pub fn cmd_stats(ctx: &ShellContext, _args: &[&str]) -> Result<()> {
    println!("{}", "Shell Statistics:".yellow().bold());
    println!("  OS: {}", ctx.get_os_info().cyan());
    println!("  Current Path: {}", ctx.current_path.blue());
    println!("  Commands Executed: {}", ctx.command_count.to_string().green());

    if let Some(duration) = ctx.last_command_time {
        println!("  Last Command Time: {}", format!("{:.2}ms", duration.as_secs_f64() * 1000.0).cyan());
    }

    println!("  Aliases: {}", ctx.aliases.len().to_string().cyan());
    println!("  Bookmarks: {}", ctx.bookmarks.len().to_string().cyan());

    Ok(())
}

/// Beautiful dashboard with system overview
pub fn cmd_dashboard(ctx: &mut ShellContext, _args: &[&str]) -> Result<()> {
    println!("\n{}", "╔═══════════════════════════════════════════════════════════════════════╗".cyan().bold());
    println!("{}", "║                        SYSTEM DASHBOARD                              ║".cyan().bold());
    println!("{}", "╚═══════════════════════════════════════════════════════════════════════╝".cyan().bold());
    println!();

    // System Information
    println!("{}", "┌─ System Information ─────────────────────────────────────┐".cyan());
    if let Ok(os_type) = ctx.guestfs.inspect_get_type(&ctx.root) {
        println!("  {} {}", "Type:".yellow().bold(), os_type.green());
    }
    if let Ok(distro) = ctx.guestfs.inspect_get_distro(&ctx.root) {
        println!("  {} {}", "Distribution:".yellow().bold(), distro.green());
    }
    if let Ok(version) = ctx.guestfs.inspect_get_product_name(&ctx.root) {
        println!("  {} {}", "Version:".yellow().bold(), version.green());
    }
    if let Ok(arch) = ctx.guestfs.inspect_get_arch(&ctx.root) {
        println!("  {} {}", "Architecture:".yellow().bold(), arch.green());
    }
    if let Ok(hostname) = ctx.guestfs.inspect_get_hostname(&ctx.root) {
        println!("  {} {}", "Hostname:".yellow().bold(), hostname.cyan());
    }
    println!("{}", "└──────────────────────────────────────────────────────────┘".cyan());
    println!();

    // Storage Overview
    println!("{}", "┌─ Storage Overview ───────────────────────────────────────┐".cyan());
    if let Ok(filesystems) = ctx.guestfs.list_filesystems() {
        let fs_count = filesystems.len();
        println!("  {} {}", "Filesystems:".yellow().bold(), fs_count.to_string().green());

        for (device, fstype) in filesystems.iter().take(5) {
            if fstype != "unknown" && !fstype.is_empty() {
                let size_str = if let Ok(size) = ctx.guestfs.blockdev_getsize64(device) {
                    format_bytes(size as u64)
                } else {
                    "unknown".to_string()
                };
                println!("    {} {} ({})", "•".cyan(), device.bright_black(), size_str.yellow());
            }
        }
    }
    println!("{}", "└──────────────────────────────────────────────────────────┘".cyan());
    println!();

    // Quick Stats
    println!("{}", "┌─ Quick Stats ────────────────────────────────────────────┐".cyan());

    if let Ok(pkg_info) = ctx.guestfs.inspect_packages(&ctx.root) {
        println!("  {} {} packages installed", "📦".to_string(), pkg_info.packages.len().to_string().green().bold());
    }

    if let Ok(users) = ctx.guestfs.inspect_users(&ctx.root) {
        let user_count = users.len();
        let root_users = users.iter().filter(|u| u.uid == "0").count();
        println!("  {} {} users ({} root)", "👥".to_string(), user_count.to_string().green().bold(), root_users.to_string().red());
    }

    if let Ok(services) = ctx.guestfs.inspect_systemd_services(&ctx.root) {
        let enabled = services.iter().filter(|s| s.enabled).count();
        println!("  {} {} services ({} enabled)", "⚙".to_string(), services.len().to_string().green().bold(), enabled.to_string().cyan());
    }

    println!("{}", "└──────────────────────────────────────────────────────────┘".cyan());
    println!();

    // Security Status
    if let Ok(sec) = ctx.guestfs.inspect_security(&ctx.root) {
        println!("{}", "┌─ Security Status ────────────────────────────────────────┐".cyan());

        let selinux_icon = if &sec.selinux != "disabled" { "✓" } else { "✗" };
        let selinux_color = if &sec.selinux != "disabled" { sec.selinux.green() } else { sec.selinux.red() };
        println!("  {} SELinux:  {}", selinux_icon, selinux_color);

        let apparmor_icon = if sec.apparmor { "✓" } else { "✗" };
        let apparmor_status = if sec.apparmor { "enabled".green() } else { "disabled".red() };
        println!("  {} AppArmor: {}", apparmor_icon, apparmor_status);

        let auditd_icon = if sec.auditd { "✓" } else { "✗" };
        let auditd_status = if sec.auditd { "enabled".green() } else { "disabled".red() };
        println!("  {} Auditd:   {}", auditd_icon, auditd_status);

        if let Ok(fw) = ctx.guestfs.inspect_firewall(&ctx.root) {
            let fw_icon = if fw.enabled { "✓" } else { "✗" };
            let fw_status = if fw.enabled {
                format!("enabled ({})", fw.firewall_type).green()
            } else {
                format!("disabled ({})", fw.firewall_type).red()
            };
            println!("  {} Firewall: {}", fw_icon, fw_status);
        }

        println!("{}", "└──────────────────────────────────────────────────────────┘".cyan());
    }

    println!("\n{} Use specific commands for detailed information", "💡".to_string().yellow());
    println!();

    Ok(())
}

/// Export data in various formats
pub fn cmd_export(ctx: &mut ShellContext, args: &[&str]) -> Result<()> {
    if args.is_empty() {
        println!("{}", "Usage: export <type> <format> [output_file]".yellow());
        println!();
        println!("{}", "Types:".green().bold());
        println!("  {} - Export package list", "packages".cyan());
        println!("  {} - Export user accounts", "users".cyan());
        println!("  {} - Export services", "services".cyan());
        println!("  {} - Export system info", "system".cyan());
        println!();
        println!("{}", "Formats:".green().bold());
        println!("  {} - JSON format", "json".cyan());
        println!("  {} - CSV format", "csv".cyan());
        println!("  {} - Markdown table", "md".cyan());
        println!("  {} - Plain text", "txt".cyan());
        println!();
        println!("{}", "Examples:".yellow());
        println!("  export packages json packages.json");
        println!("  export users csv users.csv");
        println!("  export system md system.md");
        return Ok(());
    }

    let export_type = args[0];
    let format = if args.len() > 1 { args[1] } else { "json" };
    let output = if args.len() > 2 { Some(args[2]) } else { None };

    println!("{} Exporting {} as {}...", "→".cyan(), export_type.yellow(), format.green());

    match export_type {
        "packages" => export_packages(ctx, format, output)?,
        "users" => export_users(ctx, format, output)?,
        "services" => export_services(ctx, format, output)?,
        "system" => export_system(ctx, format, output)?,
        _ => {
            println!("{} Unknown export type: {}", "Error:".red(), export_type);
            return Ok(());
        }
    }

    println!("{} Export completed!", "✓".green());
    Ok(())
}

/// Show directory tree
pub fn cmd_tree(ctx: &mut ShellContext, args: &[&str]) -> Result<()> {
    let path = if args.is_empty() {
        ctx.current_path.clone()
    } else {
        resolve_path(&ctx.current_path, args[0])
    };

    let max_depth = if args.len() > 1 {
        args[1].parse::<usize>().unwrap_or(3)
    } else {
        3
    };

    println!("\n{} {}", "Tree view of:".yellow().bold(), path.cyan());
    println!();

    print_tree(ctx, &path, "", 0, max_depth)?;
    println!();

    Ok(())
}

/// Quick system summary
pub fn cmd_summary(ctx: &mut ShellContext, _args: &[&str]) -> Result<()> {
    println!("\n{}", "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━".bright_black());

    // One-liner system info
    let os = ctx.guestfs.inspect_get_product_name(&ctx.root).unwrap_or_else(|_| "Unknown".to_string());
    let arch = ctx.guestfs.inspect_get_arch(&ctx.root).unwrap_or_else(|_| "unknown".to_string());
    let hostname = ctx.guestfs.inspect_get_hostname(&ctx.root).unwrap_or_else(|_| "unknown".to_string());

    println!("  {} {} | {} | {}",
        "🖥".to_string(),
        os.green().bold(),
        arch.cyan(),
        hostname.yellow());

    // Quick counts
    let mut quick_stats = Vec::new();

    if let Ok(pkg_info) = ctx.guestfs.inspect_packages(&ctx.root) {
        quick_stats.push(format!("{} pkgs", pkg_info.packages.len()));
    }

    if let Ok(users) = ctx.guestfs.inspect_users(&ctx.root) {
        quick_stats.push(format!("{} users", users.len()));
    }

    if let Ok(services) = ctx.guestfs.inspect_systemd_services(&ctx.root) {
        let enabled = services.iter().filter(|s| s.enabled).count();
        quick_stats.push(format!("{}/{} services", enabled, services.len()));
    }

    if !quick_stats.is_empty() {
        println!("  {}", quick_stats.join(" • ").bright_black());
    }

    println!("{}", "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━".bright_black());
    println!();

    Ok(())
}

/// Show helpful tips
pub fn cmd_tips(_ctx: &ShellContext, _args: &[&str]) -> Result<()> {
    use rand::Rng;

    let tips = vec![
        ("⚡", "Use aliases to speed up common commands", "Try: alias ll 'ls -l'"),
        ("🔖", "Bookmark frequently visited directories", "Try: bookmark config /etc"),
        ("⏱", "Commands >100ms show execution time automatically", ""),
        ("🔍", "Use grep with patterns", "Try: grep 'error' /var/log/syslog"),
        ("📊", "View system overview", "Try: dashboard"),
        ("💾", "Export data for analysis", "Try: export packages json"),
        ("🌳", "Visualize directory structure", "Try: tree /etc 2"),
        ("↑↓", "Navigate command history with arrow keys", ""),
        ("Tab", "Use Tab for command completion", ""),
        ("📈", "Check shell statistics", "Try: stats"),
    ];

    let mut rng = rand::thread_rng();
    let tip = &tips[rng.gen_range(0..tips.len())];

    println!("\n{} {}", "💡 Tip:".yellow().bold(), tip.1.green());
    if !tip.2.is_empty() {
        println!("   {}", tip.2.cyan());
    }
    println!();

    Ok(())
}

/// Generate comprehensive system snapshot report
pub fn cmd_snapshot(ctx: &mut ShellContext, args: &[&str]) -> Result<()> {
    use chrono::Local;

    let timestamp = Local::now().format("%Y-%m-%d %H:%M:%S");
    let output_file = if args.is_empty() {
        format!("snapshot-{}.md", Local::now().format("%Y%m%d-%H%M%S"))
    } else {
        args[0].to_string()
    };

    println!("{} Generating comprehensive system snapshot...", "→".cyan());

    let mut report = String::new();

    // Header
    report.push_str(&format!("# System Snapshot Report\n\n"));
    report.push_str(&format!("**Generated:** {}\n\n", timestamp));
    report.push_str(&format!("---\n\n"));

    // System Information
    report.push_str("## System Information\n\n");
    if let Ok(os_type) = ctx.guestfs.inspect_get_type(&ctx.root) {
        report.push_str(&format!("- **Type:** {}\n", os_type));
    }
    if let Ok(distro) = ctx.guestfs.inspect_get_distro(&ctx.root) {
        report.push_str(&format!("- **Distribution:** {}\n", distro));
    }
    if let Ok(version) = ctx.guestfs.inspect_get_product_name(&ctx.root) {
        report.push_str(&format!("- **Version:** {}\n", version));
    }
    if let Ok(arch) = ctx.guestfs.inspect_get_arch(&ctx.root) {
        report.push_str(&format!("- **Architecture:** {}\n", arch));
    }
    if let Ok(hostname) = ctx.guestfs.inspect_get_hostname(&ctx.root) {
        report.push_str(&format!("- **Hostname:** {}\n", hostname));
    }
    report.push_str("\n");

    // Storage
    report.push_str("## Storage Overview\n\n");
    if let Ok(filesystems) = ctx.guestfs.list_filesystems() {
        report.push_str("| Device | Type | Size |\n");
        report.push_str("|--------|------|------|\n");
        for (device, fstype) in filesystems.iter() {
            if fstype != "unknown" && !fstype.is_empty() {
                let size_str = if let Ok(size) = ctx.guestfs.blockdev_getsize64(device) {
                    format_bytes(size as u64)
                } else {
                    "unknown".to_string()
                };
                report.push_str(&format!("| {} | {} | {} |\n", device, fstype, size_str));
            }
        }
        report.push_str("\n");
    }

    // Packages
    if let Ok(pkg_info) = ctx.guestfs.inspect_packages(&ctx.root) {
        let packages = &pkg_info.packages;
        report.push_str(&format!("## Installed Packages ({})\n\n", packages.len()));
        report.push_str("| Package | Version |\n");
        report.push_str("|---------|----------|\n");
        for pkg in packages.iter().take(50) {
            report.push_str(&format!("| {} | {} |\n", pkg.name, pkg.version));
        }
        if packages.len() > 50 {
            report.push_str(&format!("\n*Showing 50 of {} packages*\n", packages.len()));
        }
        report.push_str("\n");
    }

    // Users
    if let Ok(users) = ctx.guestfs.inspect_users(&ctx.root) {
        report.push_str(&format!("## User Accounts ({})\n\n", users.len()));
        report.push_str("| Username | UID | GID | Home Directory |\n");
        report.push_str("|----------|-----|-----|----------------|\n");
        for user in users {
            let uid_marker = if user.uid == "0" { " ⚠️" } else { "" };
            report.push_str(&format!("| {}{} | {} | {} | {} |\n",
                user.username, uid_marker, user.uid, user.gid, user.home));
        }
        report.push_str("\n");
    }

    // Services
    if let Ok(services) = ctx.guestfs.inspect_systemd_services(&ctx.root) {
        let enabled_count = services.iter().filter(|s| s.enabled).count();
        report.push_str(&format!("## System Services ({} total, {} enabled)\n\n",
            services.len(), enabled_count));
        report.push_str("| Service | Status |\n");
        report.push_str("|---------|--------|\n");
        for svc in services.iter().take(50) {
            let status = if svc.enabled { "✓ Enabled" } else { "✗ Disabled" };
            report.push_str(&format!("| {} | {} |\n", svc.name, status));
        }
        if services.len() > 50 {
            report.push_str(&format!("\n*Showing 50 of {} services*\n", services.len()));
        }
        report.push_str("\n");
    }

    // Security
    if let Ok(sec) = ctx.guestfs.inspect_security(&ctx.root) {
        report.push_str("## Security Configuration\n\n");
        report.push_str("| Feature | Status |\n");
        report.push_str("|---------|--------|\n");

        let selinux_status = if &sec.selinux != "disabled" {
            format!("✓ {}", sec.selinux)
        } else {
            "✗ Disabled".to_string()
        };
        report.push_str(&format!("| SELinux | {} |\n", selinux_status));

        let apparmor = if sec.apparmor { "✓ Enabled" } else { "✗ Disabled" };
        report.push_str(&format!("| AppArmor | {} |\n", apparmor));

        let auditd = if sec.auditd { "✓ Enabled" } else { "✗ Disabled" };
        report.push_str(&format!("| Auditd | {} |\n", auditd));

        let fail2ban = if sec.fail2ban { "✓ Installed" } else { "✗ Not installed" };
        report.push_str(&format!("| fail2ban | {} |\n", fail2ban));

        let aide = if sec.aide { "✓ Installed" } else { "✗ Not installed" };
        report.push_str(&format!("| AIDE | {} |\n", aide));

        if let Ok(fw) = ctx.guestfs.inspect_firewall(&ctx.root) {
            let fw_status = if fw.enabled {
                format!("✓ Enabled ({})", fw.firewall_type)
            } else {
                format!("✗ Disabled ({})", fw.firewall_type)
            };
            report.push_str(&format!("| Firewall | {} |\n", fw_status));
        }
        report.push_str("\n");
    }

    // Network
    if let Ok(interfaces) = ctx.guestfs.inspect_network(&ctx.root) {
        report.push_str(&format!("## Network Configuration ({} interfaces)\n\n", interfaces.len()));
        for iface in interfaces {
            report.push_str(&format!("- {}\n", iface.name));
        }

        if let Ok(dns) = ctx.guestfs.inspect_dns(&ctx.root) {
            if !dns.is_empty() {
                report.push_str("\n**DNS Servers:**\n\n");
                for server in dns {
                    report.push_str(&format!("- {}\n", server));
                }
            }
        }
        report.push_str("\n");
    }

    // Footer
    report.push_str("---\n\n");
    report.push_str("*Generated by GuestKit Interactive Shell*\n");

    // Write to file
    use std::fs;
    fs::write(&output_file, report)?;

    println!("{} Snapshot saved to: {}", "✓".green(), output_file.yellow());
    println!("{} Report includes: system info, storage, packages, users, services, security, network", "→".cyan());

    Ok(())
}

/// Compare two snapshots or system states
pub fn cmd_diff(ctx: &mut ShellContext, args: &[&str]) -> Result<()> {
    if args.len() < 2 {
        println!("{}", "Usage: diff <type> <filter1> [filter2]".yellow());
        println!();
        println!("{}", "Examples:".green().bold());
        println!("  {} - Compare package versions", "diff package kernel".cyan());
        println!("  {} - Compare files", "diff file /etc/fstab".cyan());
        println!("  {} - Show file changes", "diff changes /var/log".cyan());
        return Ok(());
    }

    let diff_type = args[0];

    match diff_type {
        "file" => {
            let file_path = args[1];
            println!("{} Analyzing file: {}", "→".cyan(), file_path.yellow());

            if let Ok(stat) = ctx.guestfs.stat(file_path) {
                println!("\n{}", "File Information:".yellow().bold());
                println!("  Size: {} bytes", stat.size.to_string().green());
                println!("  Mode: {:o}", stat.mode);

                if let Ok(content) = ctx.guestfs.read_file(file_path) {
                    let lines: Vec<&str> = std::str::from_utf8(&content)
                        .unwrap_or("")
                        .lines()
                        .collect();
                    println!("  Lines: {}", lines.len().to_string().green());
                }
            }
        }
        "package" => {
            let pkg_name = args[1];
            println!("{} Searching for package: {}", "→".cyan(), pkg_name.yellow());

            if let Ok(pkg_info) = ctx.guestfs.inspect_packages(&ctx.root) {
                let matches: Vec<_> = pkg_info.packages
                    .iter()
                    .filter(|p| p.name.contains(pkg_name))
                    .collect();

                if matches.is_empty() {
                    println!("{} No matching packages found", "⚠".yellow());
                } else {
                    println!("\n{}", "Matching Packages:".yellow().bold());
                    for pkg in matches {
                        println!("  {} {} - {}", "•".cyan(), pkg.name.green(), pkg.version.to_string().bright_black());
                    }
                }
            }
        }
        _ => {
            println!("{} Unknown diff type: {}", "Error:".red(), diff_type);
        }
    }
    println!();

    Ok(())
}

/// Show recently modified files
pub fn cmd_recent(ctx: &mut ShellContext, args: &[&str]) -> Result<()> {
    let path = if args.is_empty() {
        ctx.current_path.clone()
    } else {
        resolve_path(&ctx.current_path, args[0])
    };

    let limit = if args.len() > 1 {
        args[1].parse::<usize>().unwrap_or(20)
    } else {
        20
    };

    println!("{} Finding recently modified files in: {}", "→".cyan(), path.yellow());
    println!();

    // This is a simplified version - in a real impl, we'd walk the tree and sort by mtime
    if let Ok(entries) = ctx.guestfs.ls(&path) {
        let mut files_with_time = Vec::new();

        for entry in entries.iter().take(limit) {
            let full_path = format!("{}/{}", path.trim_end_matches('/'), entry);
            if let Ok(stat) = ctx.guestfs.stat(&full_path) {
                files_with_time.push((entry.clone(), stat.mtime, stat.size));
            }
        }

        // Sort by modification time (descending)
        files_with_time.sort_by(|a, b| b.1.cmp(&a.1));

        println!("{}", "Recently Modified Files:".yellow().bold());
        println!("{}", "─".repeat(80).cyan());

        for (name, mtime, size) in files_with_time.iter().take(limit) {
            use chrono::{DateTime, Utc};
            let dt = DateTime::<Utc>::from_timestamp(*mtime, 0)
                .unwrap_or_else(|| Utc::now());
            let time_str = dt.format("%Y-%m-%d %H:%M:%S").to_string();

            let size_str = format_bytes(*size as u64);
            println!("  {} {} {} {}",
                time_str.bright_black(),
                name.green(),
                "-".bright_black(),
                size_str.yellow());
        }
        println!();
    }

    Ok(())
}

/// Quick actions menu
pub fn cmd_quick(ctx: &mut ShellContext, args: &[&str]) -> Result<()> {
    if args.is_empty() {
        println!("\n{}", "╔═══════════════════════════════════════════════════╗".cyan().bold());
        println!("{}", "║            Quick Actions Menu                   ║".cyan().bold());
        println!("{}", "╚═══════════════════════════════════════════════════╝".cyan().bold());
        println!();

        println!("{}", "Snapshots & Reports:".yellow().bold());
        println!("  {} - Full system snapshot", "quick snapshot".cyan());
        println!("  {} - Security report", "quick security".cyan());
        println!("  {} - Package inventory", "quick packages".cyan());

        println!("\n{}", "Common Tasks:".yellow().bold());
        println!("  {} - List all users", "quick users".cyan());
        println!("  {} - Show enabled services", "quick services".cyan());
        println!("  {} - Network overview", "quick network".cyan());

        println!("\n{}", "Analysis:".yellow().bold());
        println!("  {} - Show recent files", "quick recent".cyan());
        println!("  {} - Find large files", "quick large".cyan());
        println!("  {} - System health", "quick health".cyan());

        println!();
        return Ok(());
    }

    let action = args[0];

    match action {
        "snapshot" => {
            println!("{} Generating quick snapshot...", "→".cyan());
            cmd_snapshot(ctx, &[])?;
        }
        "security" => {
            println!("{} Security overview:", "→".cyan());
            println!();
            if let Ok(sec) = ctx.guestfs.inspect_security(&ctx.root) {
                let selinux_icon = if &sec.selinux != "disabled" { "✓".green() } else { "✗".red() };
                let apparmor_icon = if sec.apparmor { "✓".green() } else { "✗".red() };
                let auditd_icon = if sec.auditd { "✓".green() } else { "✗".red() };

                println!("  {} SELinux:  {}", selinux_icon, sec.selinux.yellow());
                println!("  {} AppArmor: {}", apparmor_icon, if sec.apparmor { "enabled" } else { "disabled" });
                println!("  {} Auditd:   {}", auditd_icon, if sec.auditd { "enabled" } else { "disabled" });

                if let Ok(fw) = ctx.guestfs.inspect_firewall(&ctx.root) {
                    let fw_icon = if fw.enabled { "✓".green() } else { "✗".red() };
                    println!("  {} Firewall: {} ({})", fw_icon,
                        if fw.enabled { "enabled" } else { "disabled" },
                        fw.firewall_type);
                }
            }
            println!();
        }
        "packages" => {
            println!("{} Package summary:", "→".cyan());
            if let Ok(pkg_info) = ctx.guestfs.inspect_packages(&ctx.root) {
                println!("  Total packages: {}", pkg_info.packages.len().to_string().green().bold());
                println!("  Use {} for details", "'export packages json'".cyan());
            }
            println!();
        }
        "users" => {
            if let Ok(users) = ctx.guestfs.inspect_users(&ctx.root) {
                println!("{} User accounts:", "→".cyan());
                for user in users {
                    let marker = if user.uid == "0" { " ⚠️ " } else { "   " };
                    println!("{}  {} ({})", marker, user.username.green(), user.uid.bright_black());
                }
                println!();
            }
        }
        "services" => {
            if let Ok(services) = ctx.guestfs.inspect_systemd_services(&ctx.root) {
                let enabled: Vec<_> = services.iter().filter(|s| s.enabled).collect();
                println!("{} Enabled services ({}):", "→".cyan(), enabled.len());
                for svc in enabled.iter().take(20) {
                    println!("  {} {}", "✓".green(), svc.name.cyan());
                }
                if enabled.len() > 20 {
                    println!("  ... and {} more", enabled.len() - 20);
                }
                println!();
            }
        }
        "network" => {
            if let Ok(interfaces) = ctx.guestfs.inspect_network(&ctx.root) {
                println!("{} Network interfaces:", "→".cyan());
                for iface in interfaces {
                    println!("  {} {}", "•".cyan(), iface.name.green());
                }

                if let Ok(dns) = ctx.guestfs.inspect_dns(&ctx.root) {
                    if !dns.is_empty() {
                        println!("\n  DNS servers:");
                        for server in dns {
                            println!("    {} {}", "•".cyan(), server.yellow());
                        }
                    }
                }
                println!();
            }
        }
        "recent" => {
            cmd_recent(ctx, &["/etc", "20"])?;
        }
        "large" => {
            println!("{} Finding large files...", "→".cyan());
            println!("{} Use: find . -type f -size +10M", "Hint:".yellow());
            println!();
        }
        "health" => {
            println!("{} Quick health check:", "→".cyan());
            cmd_summary(ctx, &[])?;
        }
        _ => {
            println!("{} Unknown quick action: {}", "Error:".red(), action);
            println!("{} Use 'quick' to see available actions", "Tip:".yellow());
        }
    }

    Ok(())
}

/// Show command cheat sheet
pub fn cmd_cheat(ctx: &ShellContext, _args: &[&str]) -> Result<()> {
    println!("\n{}", "╔═══════════════════════════════════════════════════════════╗".cyan().bold());
    println!("{}", "║                  Command Cheat Sheet                     ║".cyan().bold());
    println!("{}", "╚═══════════════════════════════════════════════════════════╝".cyan().bold());
    println!();

    let cheat = vec![
        ("📂", "File Operations", vec![
            ("ls /etc", "List directory contents"),
            ("cat /etc/fstab", "View file contents"),
            ("tree /etc 2", "Directory tree (2 levels)"),
            ("find . passwd", "Find files by name"),
        ]),
        ("📊", "System Overview", vec![
            ("dashboard", "Beautiful system overview"),
            ("summary", "Quick one-liner status"),
            ("info", "Detailed system info"),
        ]),
        ("💾", "Data Export", vec![
            ("export packages json", "Export to JSON"),
            ("snapshot report.md", "Full system snapshot"),
            ("diff package kernel", "Compare packages"),
        ]),
        ("👥", "User & Security", vec![
            ("users", "List all users"),
            ("security", "Security status"),
            ("services", "System services"),
        ]),
        ("⚡", "Quick Actions", vec![
            ("quick", "Show quick actions menu"),
            ("quick security", "Security overview"),
            ("recent /var/log", "Recent files"),
        ]),
        ("🎯", "Productivity", vec![
            ("alias ll 'ls -l'", "Create alias"),
            ("bookmark cfg /etc", "Bookmark path"),
            ("goto cfg", "Jump to bookmark"),
            ("tips", "Random tip"),
        ]),
    ];

    for (icon, category, commands) in cheat {
        println!("{} {}", icon, category.yellow().bold());
        for (cmd, desc) in commands {
            println!("  {} - {}", cmd.green(), desc.bright_black());
        }
        println!();
    }

    println!("{} Current path: {}", "📍".to_string(), ctx.current_path.cyan());
    println!("{} Type 'help' for complete command list", "💡".to_string().yellow());
    println!();

    Ok(())
}

/// Smart search with filters
pub fn cmd_search(ctx: &mut ShellContext, args: &[&str]) -> Result<()> {
    if args.is_empty() {
        println!("{}", "Usage: search <pattern> [options]".yellow());
        println!();
        println!("{}", "Options:".green().bold());
        println!("  {} - Search in specific path", "--path <path>".cyan());
        println!("  {} - Filter by file type (file/dir)", "--type <type>".cyan());
        println!("  {} - Size filter (e.g., +1M, -100K)", "--size <size>".cyan());
        println!("  {} - Name pattern only (default)", "--name".cyan());
        println!("  {} - Search file contents", "--content".cyan());
        println!();
        println!("{}", "Examples:".yellow());
        println!("  search passwd --path /etc");
        println!("  search *.conf --type file");
        println!("  search error --content --path /var/log");
        return Ok(());
    }

    let pattern = args[0];
    let mut search_path = ctx.current_path.clone();
    let mut search_content = false;
    let mut type_filter = None;

    // Parse options
    let mut i = 1;
    while i < args.len() {
        match args[i] {
            "--path" if i + 1 < args.len() => {
                search_path = args[i + 1].to_string();
                i += 2;
            }
            "--content" => {
                search_content = true;
                i += 1;
            }
            "--type" if i + 1 < args.len() => {
                type_filter = Some(args[i + 1]);
                i += 2;
            }
            _ => i += 1,
        }
    }

    println!("{} Searching for: {} in {}", "→".cyan(), pattern.yellow(), search_path.cyan());
    println!();

    let mut results = Vec::new();

    // Simple recursive search (simplified version)
    if let Ok(entries) = ctx.guestfs.ls(&search_path) {
        for entry in entries {
            let full_path = format!("{}/{}", search_path.trim_end_matches('/'), entry);

            // Name matching
            if entry.to_lowercase().contains(&pattern.to_lowercase()) {
                if let Some(filter) = type_filter {
                    let is_dir = ctx.guestfs.is_dir(&full_path).unwrap_or(false);
                    if (filter == "dir" && !is_dir) || (filter == "file" && is_dir) {
                        continue;
                    }
                }
                results.push((full_path.clone(), entry.clone(), "name".to_string()));
            }

            // Content search for files
            if search_content && !ctx.guestfs.is_dir(&full_path).unwrap_or(true) {
                if let Ok(content) = ctx.guestfs.read_file(&full_path) {
                    if let Ok(text) = std::str::from_utf8(&content) {
                        if text.contains(pattern) {
                            let lines: Vec<&str> = text.lines()
                                .filter(|l| l.contains(pattern))
                                .take(3)
                                .collect();
                            for line in lines {
                                results.push((full_path.clone(), line.to_string(), "content".to_string()));
                            }
                        }
                    }
                }
            }
        }
    }

    if results.is_empty() {
        println!("{} No results found", "⚠".yellow());
    } else {
        println!("{} ({} results)", "Search Results:".yellow().bold(), results.len());
        println!("{}", "─".repeat(80).cyan());

        for (path, content, match_type) in results.iter().take(50) {
            if match_type == "name" {
                println!("  {} {}", "📄".to_string(), path.green());
            } else {
                println!("  {} {} {}", "→".cyan(), path.bright_black(), content.yellow());
            }
        }

        if results.len() > 50 {
            println!("\n{} Showing 50 of {} results", "Note:".yellow(), results.len());
        }
    }
    println!();

    Ok(())
}

/// Watch files/directories for changes (simulation)
pub fn cmd_watch(ctx: &mut ShellContext, args: &[&str]) -> Result<()> {
    if args.is_empty() {
        println!("{}", "Usage: watch <path> [interval]".yellow());
        println!();
        println!("{}", "Examples:".green().bold());
        println!("  watch /var/log 5     - Watch /var/log every 5 seconds");
        println!("  watch /etc/fstab     - Watch single file");
        println!();
        println!("{} This is a snapshot-based watch (not live)", "Note:".yellow());
        return Ok(());
    }

    let watch_path = args[0];
    let full_path = resolve_path(&ctx.current_path, watch_path);

    println!("{} Watching: {}", "→".cyan(), full_path.yellow());
    println!("{} Taking initial snapshot...", "→".cyan());
    println!();

    // Take initial snapshot
    let initial_stat = ctx.guestfs.stat(&full_path)?;
    let initial_size = initial_stat.size;
    let initial_mtime = initial_stat.mtime;

    println!("{}", "Initial State:".yellow().bold());
    println!("  Size: {} bytes", initial_size.to_string().green());
    println!("  Modified: {}", initial_mtime.to_string().bright_black());

    if ctx.guestfs.is_dir(&full_path).unwrap_or(false) {
        if let Ok(entries) = ctx.guestfs.ls(&full_path) {
            println!("  Files: {}", entries.len().to_string().green());
        }
    }

    println!();
    println!("{} Use Ctrl+C to stop watching (in a real implementation)", "Tip:".yellow());
    println!("{} VM filesystems are static snapshots, so changes won't be detected in real-time", "Note:".bright_black());

    Ok(())
}

/// Batch operations on files
pub fn cmd_batch(ctx: &mut ShellContext, args: &[&str]) -> Result<()> {
    if args.is_empty() {
        println!("\n{}", "╔═══════════════════════════════════════════════════╗".cyan().bold());
        println!("{}", "║              Batch Operations                   ║".cyan().bold());
        println!("{}", "╚═══════════════════════════════════════════════════╝".cyan().bold());
        println!();

        println!("{}", "Available Operations:".yellow().bold());
        println!("  {} - List multiple files", "batch cat <pattern>".cyan());
        println!("  {} - Find in multiple locations", "batch find <pattern> <paths...>".cyan());
        println!("  {} - Export multiple types", "batch export <dir>".cyan());
        println!("  {} - Analyze multiple logs", "batch analyze <paths...>".cyan());
        println!();

        println!("{}", "Examples:".green().bold());
        println!("  batch cat /etc/*.conf");
        println!("  batch find passwd /etc /home");
        println!("  batch export /tmp/reports");
        println!();
        return Ok(());
    }

    let operation = args[0];

    match operation {
        "cat" => {
            if args.len() < 2 {
                println!("{} Usage: batch cat <file1> [file2...]", "Error:".red());
                return Ok(());
            }

            println!("{} Reading multiple files...", "→".cyan());
            println!();

            for file in &args[1..] {
                let full_path = resolve_path(&ctx.current_path, file);
                println!("{}", format!("=== {} ===", full_path).yellow().bold());

                match ctx.guestfs.read_file(&full_path) {
                    Ok(content) => {
                        if let Ok(text) = std::str::from_utf8(&content) {
                            let lines: Vec<&str> = text.lines().take(20).collect();
                            for line in lines {
                                println!("{}", line);
                            }
                            if text.lines().count() > 20 {
                                println!("{}", "... (truncated)".bright_black());
                            }
                        }
                    }
                    Err(e) => {
                        println!("{} Failed to read: {}", "Error:".red(), e);
                    }
                }
                println!();
            }
        }
        "export" => {
            let output_dir = if args.len() > 1 { args[1] } else { "." };

            println!("{} Exporting all data types to: {}", "→".cyan(), output_dir.yellow());

            let exports = vec![
                ("packages", "json"),
                ("users", "json"),
                ("services", "json"),
                ("system", "md"),
            ];

            for (export_type, format) in exports {
                let filename = format!("{}/{}.{}", output_dir, export_type, format);
                println!("  {} Exporting {} to {}", "→".cyan(), export_type.green(), filename.bright_black());

                match export_type {
                    "packages" => { let _ = export_packages(ctx, format, Some(&filename)); }
                    "users" => { let _ = export_users(ctx, format, Some(&filename)); }
                    "services" => { let _ = export_services(ctx, format, Some(&filename)); }
                    "system" => { let _ = export_system(ctx, format, Some(&filename)); }
                    _ => {}
                }
            }

            println!();
            println!("{} Batch export complete!", "✓".green());
        }
        "find" => {
            if args.len() < 3 {
                println!("{} Usage: batch find <pattern> <path1> [path2...]", "Error:".red());
                return Ok(());
            }

            let pattern = args[1];
            let paths = &args[2..];

            println!("{} Searching for '{}' in {} locations", "→".cyan(), pattern.yellow(), paths.len());
            println!();

            for path in paths {
                println!("{} Searching in: {}", "→".cyan(), path.yellow());
                if let Ok(entries) = ctx.guestfs.ls(path) {
                    let matches: Vec<_> = entries.iter()
                        .filter(|e| e.to_lowercase().contains(&pattern.to_lowercase()))
                        .collect();

                    if !matches.is_empty() {
                        for entry in matches {
                            let full_path = format!("{}/{}", path.trim_end_matches('/'), entry);
                            println!("  {} {}", "•".cyan(), full_path.green());
                        }
                    }
                }
                println!();
            }
        }
        _ => {
            println!("{} Unknown batch operation: {}", "Error:".red(), operation);
            println!("{} Use 'batch' to see available operations", "Tip:".yellow());
        }
    }

    Ok(())
}

/// Favorites/pinned commands
pub fn cmd_pin(_ctx: &mut ShellContext, args: &[&str]) -> Result<()> {
    // For simplicity, we'll store pins in a static location
    // In a real implementation, this would be persistent

    if args.is_empty() {
        println!("\n{}", "╔═══════════════════════════════════════════════════╗".cyan().bold());
        println!("{}", "║              Pinned Commands                    ║".cyan().bold());
        println!("{}", "╚═══════════════════════════════════════════════════╝".cyan().bold());
        println!();

        println!("{}", "Usage:".yellow().bold());
        println!("  {} - Show all pins", "pin".cyan());
        println!("  {} - Pin a command", "pin <name> <command>".cyan());
        println!("  {} - Run a pinned command", "pin run <name>".cyan());
        println!("  {} - Remove a pin", "pin remove <name>".cyan());
        println!();

        println!("{}", "Examples:".green().bold());
        println!("  pin logs 'cat /var/log/syslog'");
        println!("  pin security 'quick security'");
        println!("  pin run logs");
        println!();

        println!("{}", "Suggested Pins:".yellow().bold());
        println!("  📌 pin errors 'grep ERROR /var/log'");
        println!("  📌 pin users 'quick users'");
        println!("  📌 pin snap 'snapshot'");
        println!();

        return Ok(());
    }

    let action = args[0];

    match action {
        "run" => {
            if args.len() < 2 {
                println!("{} Usage: pin run <name>", "Error:".red());
                return Ok(());
            }
            let pin_name = args[1];
            println!("{} Would execute pinned command: {}", "→".cyan(), pin_name.yellow());
            println!("{} Pin functionality requires persistent storage", "Note:".bright_black());
        }
        "remove" => {
            if args.len() < 2 {
                println!("{} Usage: pin remove <name>", "Error:".red());
                return Ok(());
            }
            let pin_name = args[1];
            println!("{} Would remove pin: {}", "→".cyan(), pin_name.yellow());
        }
        _ => {
            // Assume it's "pin <name> <command>"
            if args.len() < 2 {
                println!("{} Usage: pin <name> <command>", "Error:".red());
                return Ok(());
            }
            let pin_name = args[0];
            let command = args[1..].join(" ");
            println!("{} Would pin command: {} = {}", "→".cyan(), pin_name.yellow(), command.green());
            println!("{} Pin functionality requires persistent storage", "Note:".bright_black());
        }
    }

    Ok(())
}

/// Show command history with analysis
pub fn cmd_history_enhanced(ctx: &ShellContext, _args: &[&str]) -> Result<()> {
    println!("\n{}", "╔═══════════════════════════════════════════════════╗".cyan().bold());
    println!("{}", "║          Command History Analysis               ║".cyan().bold());
    println!("{}", "╚═══════════════════════════════════════════════════╝".cyan().bold());
    println!();

    println!("{}", "Session Statistics:".yellow().bold());
    println!("  Commands executed: {}", ctx.command_count.to_string().green().bold());

    if let Some(duration) = ctx.last_command_time {
        println!("  Last command time: {}", format!("{:.2}ms", duration.as_secs_f64() * 1000.0).cyan());
    }

    println!("  Aliases defined: {}", ctx.aliases.len().to_string().cyan());
    println!("  Bookmarks saved: {}", ctx.bookmarks.len().to_string().cyan());
    println!();

    println!("{}", "Most Useful Commands:".yellow().bold());
    println!("  {} - Quick system overview", "dashboard".green());
    println!("  {} - Export for analysis", "snapshot".green());
    println!("  {} - Fast shortcuts", "quick".green());
    println!("  {} - Search anything", "search".green());
    println!("  {} - Multiple operations", "batch".green());
    println!();

    println!("{} Use 'history' to see full command list", "Tip:".yellow());
    println!("{} Type 'cheat' for command reference", "Tip:".yellow());
    println!();

    Ok(())
}

/// Interactive wizard for common tasks
pub fn cmd_wizard(ctx: &mut ShellContext, args: &[&str]) -> Result<()> {
    if args.is_empty() {
        println!("\n{}", "╔═══════════════════════════════════════════════════════════╗".cyan().bold());
        println!("{}", "║                  Interactive Wizards                     ║".cyan().bold());
        println!("{}", "╚═══════════════════════════════════════════════════════════╝".cyan().bold());
        println!();

        println!("{}", "Available Wizards:".yellow().bold());
        println!("  {} - Security audit wizard", "wizard security".cyan());
        println!("  {} - System health check wizard", "wizard health".cyan());
        println!("  {} - Package analysis wizard", "wizard packages".cyan());
        println!("  {} - Configuration review wizard", "wizard config".cyan());
        println!("  {} - Export/report wizard", "wizard export".cyan());
        println!();

        println!("{}", "What are wizards?".green().bold());
        println!("  Interactive step-by-step guides for complex tasks");
        println!("  Automated checks with detailed explanations");
        println!("  Perfect for learning and thorough analysis");
        println!();

        return Ok(());
    }

    let wizard_type = args[0];

    match wizard_type {
        "security" => {
            println!("\n{}", "🔒 Security Audit Wizard".yellow().bold());
            println!("{}", "═".repeat(60).cyan());
            println!();

            println!("{} Step 1/5: Checking security features...", "→".cyan());
            if let Ok(sec) = ctx.guestfs.inspect_security(&ctx.root) {
                let mut score = 0;
                let mut issues = Vec::new();

                if &sec.selinux != "disabled" {
                    println!("  {} SELinux: {} (enforcing)", "✓".green(), sec.selinux.green());
                    score += 20;
                } else {
                    println!("  {} SELinux: disabled", "✗".red());
                    issues.push("Enable SELinux for mandatory access control");
                }

                if sec.apparmor {
                    println!("  {} AppArmor: enabled", "✓".green());
                    score += 20;
                } else {
                    println!("  {} AppArmor: disabled", "✗".red());
                    issues.push("Enable AppArmor for application confinement");
                }

                if sec.auditd {
                    println!("  {} Auditd: enabled", "✓".green());
                    score += 15;
                } else {
                    println!("  {} Auditd: disabled", "✗".yellow());
                    issues.push("Enable auditd for system auditing");
                }

                if let Ok(fw) = ctx.guestfs.inspect_firewall(&ctx.root) {
                    if fw.enabled {
                        println!("  {} Firewall: enabled ({})", "✓".green(), fw.firewall_type);
                        score += 25;
                    } else {
                        println!("  {} Firewall: disabled", "✗".red());
                        issues.push("Enable firewall for network protection");
                    }
                }

                println!();
                println!("{} Step 2/5: Checking user accounts...", "→".cyan());
                if let Ok(users) = ctx.guestfs.inspect_users(&ctx.root) {
                    let root_users: Vec<_> = users.iter().filter(|u| u.uid == "0").collect();
                    if root_users.len() == 1 {
                        println!("  {} Single root account", "✓".green());
                        score += 10;
                    } else {
                        println!("  {} Multiple root accounts: {}", "✗".red(), root_users.len());
                        issues.push("Multiple root accounts detected - security risk");
                    }
                }

                println!();
                println!("{} Step 3/5: Security score calculation...", "→".cyan());
                let grade = if score >= 80 {
                    "A (Excellent)".green().bold()
                } else if score >= 60 {
                    "B (Good)".cyan().bold()
                } else if score >= 40 {
                    "C (Fair)".yellow().bold()
                } else {
                    "D (Poor)".red().bold()
                };

                println!("  Security Score: {}/100 - Grade: {}", score.to_string().bold(), grade);

                println!();
                println!("{} Step 4/5: Recommendations...", "→".cyan());
                if issues.is_empty() {
                    println!("  {} No critical issues found!", "✓".green());
                } else {
                    for (i, issue) in issues.iter().enumerate() {
                        println!("  {}) {}", i + 1, issue.yellow());
                    }
                }

                println!();
                println!("{} Step 5/5: Next steps...", "→".cyan());
                println!("  • Run {} for detailed security info", "'security'".cyan());
                println!("  • Generate report: {}", "'snapshot security-audit.md'".cyan());
                println!("  • Export data: {}", "'export system json'".cyan());
            }
            println!();
        }
        "health" => {
            println!("\n{}", "🏥 System Health Check Wizard".yellow().bold());
            println!("{}", "═".repeat(60).cyan());
            println!();

            let mut health_score = 100;
            let mut warnings = Vec::new();

            println!("{} Checking system health...", "→".cyan());
            println!();

            // Check 1: Services
            if let Ok(services) = ctx.guestfs.inspect_systemd_services(&ctx.root) {
                let enabled = services.iter().filter(|s| s.enabled).count();
                let ratio = (enabled as f64 / services.len() as f64) * 100.0;

                println!("  {} Services: {}/{} enabled ({:.1}%)",
                    "✓".green(), enabled, services.len(), ratio);

                if ratio < 30.0 {
                    warnings.push("Low service count - system may be minimal");
                    health_score -= 10;
                }
            }

            // Check 2: Packages
            if let Ok(pkg_info) = ctx.guestfs.inspect_packages(&ctx.root) {
                let count = pkg_info.packages.len();
                println!("  {} Packages: {} installed", "✓".green(), count);

                if count < 100 {
                    warnings.push("Very minimal package set - may lack essential tools");
                    health_score -= 5;
                }
            }

            // Check 3: Users
            if let Ok(users) = ctx.guestfs.inspect_users(&ctx.root) {
                println!("  {} Users: {} accounts", "✓".green(), users.len());
            }

            println!();
            let health_grade = if health_score >= 90 {
                "Excellent".green().bold()
            } else if health_score >= 70 {
                "Good".cyan().bold()
            } else {
                "Fair".yellow().bold()
            };

            println!("{} Health Score: {}/100 ({})", "→".cyan(), health_score, health_grade);

            if !warnings.is_empty() {
                println!();
                println!("{}", "Warnings:".yellow().bold());
                for warning in warnings {
                    println!("  {} {}", "⚠".yellow(), warning);
                }
            }

            println!();
            println!("{} Use {} for detailed overview", "Tip:".yellow(), "'dashboard'".cyan());
            println!();
        }
        "packages" => {
            println!("\n{}", "📦 Package Analysis Wizard".yellow().bold());
            println!("{}", "═".repeat(60).cyan());
            println!();

            if let Ok(pkg_info) = ctx.guestfs.inspect_packages(&ctx.root) {
                let packages = &pkg_info.packages;

                println!("{} Analyzing {} packages...", "→".cyan(), packages.len());
                println!();

                // Find interesting packages
                let security_pkgs: Vec<_> = packages.iter()
                    .filter(|p| p.name.contains("security") || p.name.contains("firewall") || p.name.contains("selinux"))
                    .collect();

                let dev_pkgs: Vec<_> = packages.iter()
                    .filter(|p| p.name.contains("dev") || p.name.contains("gcc") || p.name.contains("make"))
                    .collect();

                let server_pkgs: Vec<_> = packages.iter()
                    .filter(|p| p.name.contains("httpd") || p.name.contains("nginx") || p.name.contains("apache"))
                    .collect();

                println!("{}", "Package Categories:".yellow().bold());
                println!("  {} Security: {} packages", "🔒".to_string(), security_pkgs.len());
                println!("  {} Development: {} packages", "⚙".to_string(), dev_pkgs.len());
                println!("  {} Web Servers: {} packages", "🌐".to_string(), server_pkgs.len());

                println!();
                println!("{}", "Recommendations:".green().bold());
                if server_pkgs.is_empty() {
                    println!("  • No web servers detected - workstation/desktop system");
                } else {
                    println!("  • Web server detected - review {} output", "'services'".cyan());
                }

                if dev_pkgs.len() > 50 {
                    println!("  • Heavy development environment detected");
                }

                println!();
                println!("{} Export package list: {}", "Tip:".yellow(), "'export packages json'".cyan());
            }
            println!();
        }
        "config" => {
            println!("\n{}", "⚙ Configuration Review Wizard".yellow().bold());
            println!("{}", "═".repeat(60).cyan());
            println!();

            println!("{} Reviewing critical configuration files...", "→".cyan());
            println!();

            let config_files = vec![
                "/etc/fstab",
                "/etc/hosts",
                "/etc/resolv.conf",
                "/etc/ssh/sshd_config",
            ];

            for config_file in config_files {
                if ctx.guestfs.exists(config_file).unwrap_or(false) {
                    if let Ok(stat) = ctx.guestfs.stat(config_file) {
                        println!("  {} {} ({} bytes)", "✓".green(), config_file.cyan(), stat.size);
                    }
                } else {
                    println!("  {} {} (not found)", "✗".red(), config_file);
                }
            }

            println!();
            println!("{} Use {} to examine files", "Tip:".yellow(), "'cat /etc/fstab'".cyan());
            println!();
        }
        "export" => {
            println!("\n{}", "💾 Export/Report Wizard".yellow().bold());
            println!("{}", "═".repeat(60).cyan());
            println!();

            println!("{} What would you like to export?", "→".cyan());
            println!();
            println!("  1) {} - Complete system snapshot", "Full Report".green());
            println!("  2) {} - All data in JSON format", "All Data (JSON)".green());
            println!("  3) {} - Security configuration only", "Security Report".green());
            println!("  4) {} - Package inventory", "Package List".green());
            println!();
            println!("{}", "Quick commands:".yellow().bold());
            println!("  Full: {}", "snapshot system-report.md".cyan());
            println!("  JSON: {}", "batch export /tmp/data".cyan());
            println!("  Security: {}", "quick security > security.txt".cyan());
            println!("  Packages: {}", "export packages json packages.json".cyan());
            println!();
        }
        _ => {
            println!("{} Unknown wizard: {}", "Error:".red(), wizard_type);
            println!("{} Use 'wizard' to see available wizards", "Tip:".yellow());
        }
    }

    Ok(())
}

/// Comprehensive scanning (security, health, issues)
pub fn cmd_scan(ctx: &mut ShellContext, args: &[&str]) -> Result<()> {
    if args.is_empty() {
        println!("\n{}", "╔═══════════════════════════════════════════════════════════╗".cyan().bold());
        println!("{}", "║                  System Scanner                          ║".cyan().bold());
        println!("{}", "╚═══════════════════════════════════════════════════════════╝".cyan().bold());
        println!();

        println!("{}", "Available Scans:".yellow().bold());
        println!("  {} - Quick security scan", "scan security".cyan());
        println!("  {} - Find common issues", "scan issues".cyan());
        println!("  {} - Scan for vulnerabilities", "scan vulns".cyan());
        println!("  {} - Scan all (comprehensive)", "scan all".cyan());
        println!();

        return Ok(());
    }

    let scan_type = args[0];

    match scan_type {
        "security" => {
            println!("\n{}", "🔍 Security Scan".yellow().bold());
            println!("{}", "─".repeat(60).cyan());
            println!();

            let mut findings = Vec::new();

            // Check 1: Root users
            if let Ok(users) = ctx.guestfs.inspect_users(&ctx.root) {
                let root_count = users.iter().filter(|u| u.uid == "0").count();
                if root_count > 1 {
                    findings.push(("HIGH".red(), format!("{} root accounts found (expected 1)", root_count)));
                }
            }

            // Check 2: Security features
            if let Ok(sec) = ctx.guestfs.inspect_security(&ctx.root) {
                if &sec.selinux == "disabled" {
                    findings.push(("MEDIUM".yellow(), "SELinux is disabled".to_string()));
                }
                if !sec.apparmor {
                    findings.push(("MEDIUM".yellow(), "AppArmor is disabled".to_string()));
                }
                if !sec.auditd {
                    findings.push(("LOW".bright_black(), "Auditd is not enabled".to_string()));
                }
            }

            // Check 3: Firewall
            if let Ok(fw) = ctx.guestfs.inspect_firewall(&ctx.root) {
                if !fw.enabled {
                    findings.push(("HIGH".red(), "Firewall is disabled".to_string()));
                }
            }

            println!("{} ({} findings)", "Security Findings:".yellow().bold(), findings.len());
            if findings.is_empty() {
                println!("  {} No security issues detected!", "✓".green());
            } else {
                for (severity, finding) in findings {
                    println!("  [{}] {}", severity, finding);
                }
            }
            println!();
        }
        "issues" => {
            println!("\n{}", "🔍 Common Issues Scan".yellow().bold());
            println!("{}", "─".repeat(60).cyan());
            println!();

            println!("{} Scanning for common issues...", "→".cyan());
            println!();

            let mut issue_count = 0;

            // Check for empty password users (simplified)
            println!("{}", "Checking user accounts...".yellow());
            if let Ok(users) = ctx.guestfs.inspect_users(&ctx.root) {
                for user in users.iter().take(5) {
                    println!("  {} {} (UID: {})", "•".cyan(), user.username.green(), user.uid.bright_black());
                }
            }

            println!();
            println!("{}", "Checking for common misconfigurations...".yellow());

            // Check fstab
            if ctx.guestfs.exists("/etc/fstab").unwrap_or(false) {
                println!("  {} /etc/fstab exists", "✓".green());
            } else {
                println!("  {} /etc/fstab missing", "✗".red());
                issue_count += 1;
            }

            println!();
            if issue_count == 0 {
                println!("{} No critical issues found", "✓".green());
            } else {
                println!("{} {} issues found", "⚠".yellow(), issue_count);
            }
            println!();
        }
        "vulns" => {
            println!("\n{}", "🔍 Vulnerability Scan".yellow().bold());
            println!("{}", "─".repeat(60).cyan());
            println!();

            println!("{} Checking for known vulnerabilities...", "→".cyan());
            println!();

            if let Ok(pkg_info) = ctx.guestfs.inspect_packages(&ctx.root) {
                println!("  Scanning {} packages...", pkg_info.packages.len());
                println!();
                println!("{} Full vulnerability scanning requires CVE database", "Note:".yellow());
                println!("{} This is a basic package audit", "Note:".yellow());
                println!();

                // Check for very old or suspicious packages
                let kernel_pkgs: Vec<_> = pkg_info.packages.iter()
                    .filter(|p| p.name.contains("kernel"))
                    .collect();

                if !kernel_pkgs.is_empty() {
                    println!("{}", "Kernel packages:".green().bold());
                    for pkg in kernel_pkgs {
                        println!("  {} {}", pkg.name.cyan(), pkg.version.to_string().bright_black());
                    }
                }
            }
            println!();
        }
        "all" => {
            println!("\n{}", "🔍 Comprehensive System Scan".yellow().bold());
            println!("{}", "═".repeat(60).cyan());
            println!();

            println!("{} Running all scans...", "→".cyan());
            println!();

            // Run all scans
            cmd_scan(ctx, &["security"])?;
            cmd_scan(ctx, &["issues"])?;

            println!("{}", "═".repeat(60).cyan());
            println!("{} Scan complete!", "✓".green());
            println!();
        }
        _ => {
            println!("{} Unknown scan type: {}", "Error:".red(), scan_type);
        }
    }

    Ok(())
}

/// Compare two snapshots or states
pub fn cmd_compare(ctx: &mut ShellContext, args: &[&str]) -> Result<()> {
    if args.is_empty() {
        println!("\n{}", "╔═══════════════════════════════════════════════════════════╗".cyan().bold());
        println!("{}", "║                  Comparison Tools                        ║".cyan().bold());
        println!("{}", "╚═══════════════════════════════════════════════════════════╝".cyan().bold());
        println!();

        println!("{}", "Usage:".yellow().bold());
        println!("  {} - Compare two files", "compare files <file1> <file2>".cyan());
        println!("  {} - Compare directories", "compare dirs <dir1> <dir2>".cyan());
        println!("  {} - Compare package lists", "compare packages <snap1> <snap2>".cyan());
        println!();

        println!("{}", "Examples:".green().bold());
        println!("  compare files /etc/fstab /etc/fstab.bak");
        println!("  compare dirs /etc /etc.backup");
        println!();

        return Ok(());
    }

    let compare_type = args[0];

    match compare_type {
        "files" => {
            if args.len() < 3 {
                println!("{} Usage: compare files <file1> <file2>", "Error:".red());
                return Ok(());
            }

            let file1 = args[1];
            let file2 = args[2];

            println!("\n{} Comparing files:", "→".cyan());
            println!("  {} {}", "A:".yellow(), file1.green());
            println!("  {} {}", "B:".yellow(), file2.green());
            println!();

            let stat1 = ctx.guestfs.stat(file1)?;
            let stat2 = ctx.guestfs.stat(file2)?;

            println!("{}", "Size Comparison:".yellow().bold());
            println!("  A: {} bytes", stat1.size.to_string().cyan());
            println!("  B: {} bytes", stat2.size.to_string().cyan());

            if stat1.size == stat2.size {
                println!("  {} Files are same size", "✓".green());
            } else {
                let diff = (stat1.size as i64 - stat2.size as i64).abs();
                println!("  {} Difference: {} bytes", "△".yellow(), diff);
            }

            println!();
            println!("{}", "Modification Time:".yellow().bold());
            println!("  A: {}", stat1.mtime.to_string().cyan());
            println!("  B: {}", stat2.mtime.to_string().cyan());

            if stat1.mtime > stat2.mtime {
                println!("  {} A is newer", "→".cyan());
            } else if stat2.mtime > stat1.mtime {
                println!("  {} B is newer", "→".cyan());
            } else {
                println!("  {} Same modification time", "✓".green());
            }
            println!();
        }
        "dirs" => {
            if args.len() < 3 {
                println!("{} Usage: compare dirs <dir1> <dir2>", "Error:".red());
                return Ok(());
            }

            let dir1 = args[1];
            let dir2 = args[2];

            println!("\n{} Comparing directories:", "→".cyan());
            println!("  {} {}", "A:".yellow(), dir1.green());
            println!("  {} {}", "B:".yellow(), dir2.green());
            println!();

            let entries1 = ctx.guestfs.ls(dir1)?;
            let entries2 = ctx.guestfs.ls(dir2)?;

            println!("{}", "File Count:".yellow().bold());
            println!("  A: {} files", entries1.len().to_string().cyan());
            println!("  B: {} files", entries2.len().to_string().cyan());

            let only_in_a: Vec<_> = entries1.iter()
                .filter(|e| !entries2.contains(e))
                .collect();

            let only_in_b: Vec<_> = entries2.iter()
                .filter(|e| !entries1.contains(e))
                .collect();

            if !only_in_a.is_empty() {
                println!();
                println!("{} ({}):", "Only in A".yellow().bold(), only_in_a.len());
                for entry in only_in_a.iter().take(10) {
                    println!("  {} {}", "-".red(), entry);
                }
                if only_in_a.len() > 10 {
                    println!("  ... and {} more", only_in_a.len() - 10);
                }
            }

            if !only_in_b.is_empty() {
                println!();
                println!("{} ({}):", "Only in B".yellow().bold(), only_in_b.len());
                for entry in only_in_b.iter().take(10) {
                    println!("  {} {}", "+".green(), entry);
                }
                if only_in_b.len() > 10 {
                    println!("  ... and {} more", only_in_b.len() - 10);
                }
            }

            if only_in_a.is_empty() && only_in_b.is_empty() {
                println!();
                println!("{} Directories have identical file lists", "✓".green());
            }
            println!();
        }
        _ => {
            println!("{} Unknown comparison type: {}", "Error:".red(), compare_type);
        }
    }

    Ok(())
}

/// System profiling and fingerprinting
pub fn cmd_profile(ctx: &mut ShellContext, args: &[&str]) -> Result<()> {
    if args.is_empty() {
        println!("\n{}", "╔═══════════════════════════════════════════════════════════╗".cyan().bold());
        println!("{}", "║                  System Profiler                         ║".cyan().bold());
        println!("{}", "╚═══════════════════════════════════════════════════════════╝".cyan().bold());
        println!();

        println!("{}", "Available Profiles:".yellow().bold());
        println!("  {} - Create full system profile", "profile create [name]".cyan());
        println!("  {} - Quick system fingerprint", "profile quick".cyan());
        println!("  {} - Show system characteristics", "profile show".cyan());
        println!("  {} - Detect system purpose", "profile detect".cyan());
        println!();

        return Ok(());
    }

    let profile_type = args[0];

    match profile_type {
        "create" => {
            let profile_name = if args.len() > 1 { args[1] } else { "system-profile" };

            println!("\n{} Creating system profile: {}", "→".cyan(), profile_name.yellow());
            println!();

            let mut profile_data = String::new();
            profile_data.push_str(&format!("# System Profile: {}\n\n", profile_name));

            // Basic info
            if let Ok(os) = ctx.guestfs.inspect_get_product_name(&ctx.root) {
                profile_data.push_str(&format!("**OS:** {}\n", os));
            }
            if let Ok(arch) = ctx.guestfs.inspect_get_arch(&ctx.root) {
                profile_data.push_str(&format!("**Architecture:** {}\n", arch));
            }
            if let Ok(hostname) = ctx.guestfs.inspect_get_hostname(&ctx.root) {
                profile_data.push_str(&format!("**Hostname:** {}\n", hostname));
            }

            profile_data.push_str("\n## Profile Metrics\n\n");

            // Metrics
            if let Ok(pkg_info) = ctx.guestfs.inspect_packages(&ctx.root) {
                profile_data.push_str(&format!("- Packages: {}\n", pkg_info.packages.len()));
            }
            if let Ok(users) = ctx.guestfs.inspect_users(&ctx.root) {
                profile_data.push_str(&format!("- Users: {}\n", users.len()));
            }
            if let Ok(services) = ctx.guestfs.inspect_systemd_services(&ctx.root) {
                let enabled = services.iter().filter(|s| s.enabled).count();
                profile_data.push_str(&format!("- Services: {} ({} enabled)\n", services.len(), enabled));
            }

            let filename = format!("{}.md", profile_name);
            use std::fs;
            fs::write(&filename, profile_data)?;

            println!("{} Profile saved to: {}", "✓".green(), filename.yellow());
            println!();
        }
        "quick" => {
            println!("\n{}", "🔍 Quick System Fingerprint".yellow().bold());
            println!("{}", "─".repeat(60).cyan());
            println!();

            let mut fingerprint = Vec::new();

            if let Ok(os_type) = ctx.guestfs.inspect_get_type(&ctx.root) {
                fingerprint.push(format!("Type: {}", os_type));
            }
            if let Ok(distro) = ctx.guestfs.inspect_get_distro(&ctx.root) {
                fingerprint.push(format!("Distro: {}", distro));
            }
            if let Ok(pkg_info) = ctx.guestfs.inspect_packages(&ctx.root) {
                let count = pkg_info.packages.len();
                let size = if count < 200 { "minimal" } else if count < 500 { "standard" } else { "full" };
                fingerprint.push(format!("Size: {} ({} pkgs)", size, count));
            }

            for item in fingerprint {
                println!("  {} {}", "•".cyan(), item.green());
            }
            println!();
        }
        "detect" => {
            println!("\n{}", "🎯 System Purpose Detection".yellow().bold());
            println!("{}", "─".repeat(60).cyan());
            println!();

            let mut purposes = Vec::new();

            if let Ok(pkg_info) = ctx.guestfs.inspect_packages(&ctx.root) {
                let packages = &pkg_info.packages;

                // Check for web server
                if packages.iter().any(|p| p.name.contains("httpd") || p.name.contains("nginx") || p.name.contains("apache")) {
                    purposes.push(("Web Server", "🌐", "HTTP server software detected"));
                }

                // Check for database
                if packages.iter().any(|p| p.name.contains("mysql") || p.name.contains("postgres") || p.name.contains("mariadb")) {
                    purposes.push(("Database Server", "💾", "Database software detected"));
                }

                // Check for development
                if packages.iter().any(|p| p.name.contains("gcc") || p.name.contains("python-dev") || p.name.contains("build-essential")) {
                    purposes.push(("Development", "⚙", "Development tools detected"));
                }

                // Check for desktop
                if packages.iter().any(|p| p.name.contains("gnome") || p.name.contains("kde") || p.name.contains("xorg")) {
                    purposes.push(("Desktop/Workstation", "🖥", "Desktop environment detected"));
                }

                // Check for container
                if packages.iter().any(|p| p.name.contains("docker") || p.name.contains("podman") || p.name.contains("kubernetes")) {
                    purposes.push(("Container Platform", "📦", "Container runtime detected"));
                }
            }

            if purposes.is_empty() {
                println!("  {} Minimal/Base system", "🔧".to_string());
                println!("  No specific purpose detected - likely a base installation");
            } else {
                println!("{}", "Detected Purposes:".green().bold());
                for (purpose, icon, desc) in purposes {
                    println!("  {} {} - {}", icon.to_string(), purpose.green().bold(), desc.bright_black());
                }
            }
            println!();
        }
        "show" => {
            println!("\n{}", "📋 System Characteristics".yellow().bold());
            println!("{}", "─".repeat(60).cyan());
            println!();

            println!("{}", "System Identity:".green().bold());
            if let Ok(os) = ctx.guestfs.inspect_get_product_name(&ctx.root) {
                println!("  OS: {}", os.cyan());
            }
            if let Ok(arch) = ctx.guestfs.inspect_get_arch(&ctx.root) {
                println!("  Architecture: {}", arch.cyan());
            }

            println!();
            println!("{}", "Security Profile:".green().bold());
            if let Ok(sec) = ctx.guestfs.inspect_security(&ctx.root) {
                let profile = if &sec.selinux != "disabled" && sec.apparmor {
                    "Hardened"
                } else if &sec.selinux != "disabled" || sec.apparmor {
                    "Standard"
                } else {
                    "Basic"
                };
                println!("  Security Level: {}", profile.yellow());
            }

            println!();
        }
        _ => {
            println!("{} Unknown profile command: {}", "Error:".red(), profile_type);
        }
    }

    Ok(())
}

/// Smart recommendations engine
pub fn cmd_recommend(ctx: &mut ShellContext, _args: &[&str]) -> Result<()> {
    println!("\n{}", "╔═══════════════════════════════════════════════════════════╗".cyan().bold());
    println!("{}", "║              Smart Recommendations                       ║".cyan().bold());
    println!("{}", "╚═══════════════════════════════════════════════════════════╝".cyan().bold());
    println!();

    println!("{} Analyzing system and generating recommendations...", "→".cyan());
    println!();

    let mut recommendations = Vec::new();

    // Security recommendations
    if let Ok(sec) = ctx.guestfs.inspect_security(&ctx.root) {
        if &sec.selinux == "disabled" {
            recommendations.push((
                "HIGH",
                "Security",
                "Enable SELinux for enhanced security",
                "wizard security"
            ));
        }

        if !sec.auditd {
            recommendations.push((
                "MEDIUM",
                "Monitoring",
                "Enable auditd for system auditing",
                "scan security"
            ));
        }
    }

    // Firewall recommendation
    if let Ok(fw) = ctx.guestfs.inspect_firewall(&ctx.root) {
        if !fw.enabled {
            recommendations.push((
                "HIGH",
                "Security",
                "Enable firewall for network protection",
                "quick security"
            ));
        }
    }

    // User account recommendations
    if let Ok(users) = ctx.guestfs.inspect_users(&ctx.root) {
        let root_count = users.iter().filter(|u| u.uid == "0").count();
        if root_count > 1 {
            recommendations.push((
                "HIGH",
                "Security",
                "Multiple root accounts detected - review user list",
                "users"
            ));
        }
    }

    // Package recommendations
    if let Ok(pkg_info) = ctx.guestfs.inspect_packages(&ctx.root) {
        if pkg_info.packages.len() < 100 {
            recommendations.push((
                "LOW",
                "System",
                "Very minimal package set - consider if all tools are available",
                "wizard packages"
            ));
        }
    }

    // General recommendations
    recommendations.push((
        "INFO",
        "Analysis",
        "Generate a full system snapshot for documentation",
        "snapshot"
    ));

    recommendations.push((
        "INFO",
        "Export",
        "Export data for external analysis",
        "batch export /tmp/data"
    ));

    if recommendations.is_empty() {
        println!("{} No recommendations - system looks good!", "✓".green());
    } else {
        println!("{} ({} items)", "Recommendations:".yellow().bold(), recommendations.len());
        println!();

        for (priority, category, recommendation, command) in recommendations {
            let priority_colored = match priority {
                "HIGH" => "HIGH".red(),
                "MEDIUM" => "MEDIUM".yellow(),
                "LOW" => "LOW".bright_black(),
                _ => "INFO".cyan(),
            };

            println!("  [{}] {} - {}", priority_colored, category.green().bold(), recommendation);
            println!("      {} {}", "Command:".bright_black(), command.cyan());
            println!();
        }
    }

    println!("{} Run suggested commands to address recommendations", "Tip:".yellow());
    println!();

    Ok(())
}

/// Discover and explore system
pub fn cmd_discover(ctx: &mut ShellContext, args: &[&str]) -> Result<()> {
    if args.is_empty() {
        println!("\n{}", "╔═══════════════════════════════════════════════════════════╗".cyan().bold());
        println!("{}", "║                  System Discovery                        ║".cyan().bold());
        println!("{}", "╚═══════════════════════════════════════════════════════════╝".cyan().bold());
        println!();

        println!("{}", "Discovery Options:".yellow().bold());
        println!("  {} - Discover interesting files", "discover files".cyan());
        println!("  {} - Discover installed applications", "discover apps".cyan());
        println!("  {} - Discover network configuration", "discover network".cyan());
        println!("  {} - Discover all (comprehensive)", "discover all".cyan());
        println!();

        return Ok(());
    }

    let discover_type = args[0];

    match discover_type {
        "files" => {
            println!("\n{}", "📂 Discovering Interesting Files".yellow().bold());
            println!("{}", "─".repeat(60).cyan());
            println!();

            let interesting_paths = vec![
                ("/etc/fstab", "Filesystem table"),
                ("/etc/passwd", "User accounts"),
                ("/etc/shadow", "Password hashes"),
                ("/etc/hosts", "Host name mappings"),
                ("/etc/ssh/sshd_config", "SSH server config"),
                ("/var/log/syslog", "System log"),
                ("/root/.bash_history", "Root command history"),
            ];

            println!("{}", "Critical System Files:".green().bold());
            for (path, description) in interesting_paths {
                if ctx.guestfs.exists(path).unwrap_or(false) {
                    if let Ok(stat) = ctx.guestfs.stat(path) {
                        println!("  {} {} - {} ({} bytes)",
                            "✓".green(),
                            path.cyan(),
                            description.bright_black(),
                            stat.size);
                    }
                } else {
                    println!("  {} {} - {} (not found)", "✗".red(), path, description.bright_black());
                }
            }
            println!();
        }
        "apps" => {
            println!("\n{}", "🚀 Discovering Applications".yellow().bold());
            println!("{}", "─".repeat(60).cyan());
            println!();

            if let Ok(pkg_info) = ctx.guestfs.inspect_packages(&ctx.root) {
                let packages = &pkg_info.packages;

                let categories = vec![
                    ("Web Servers", vec!["httpd", "nginx", "apache"]),
                    ("Databases", vec!["mysql", "postgres", "mariadb", "mongodb"]),
                    ("Programming", vec!["python", "ruby", "nodejs", "java", "golang"]),
                    ("Security Tools", vec!["nmap", "wireshark", "fail2ban", "aide"]),
                    ("System Tools", vec!["systemd", "cron", "rsyslog"]),
                ];

                for (category, keywords) in categories {
                    let found: Vec<_> = packages.iter()
                        .filter(|p| keywords.iter().any(|k| p.name.contains(k)))
                        .collect();

                    if !found.is_empty() {
                        println!("{} ({}):", category.green().bold(), found.len());
                        for pkg in found.iter().take(5) {
                            println!("  {} {} - {}", "•".cyan(), pkg.name.green(), pkg.version.to_string().bright_black());
                        }
                        if found.len() > 5 {
                            println!("  ... and {} more", found.len() - 5);
                        }
                        println!();
                    }
                }
            }
        }
        "network" => {
            println!("\n{}", "🌐 Discovering Network Configuration".yellow().bold());
            println!("{}", "─".repeat(60).cyan());
            println!();

            if let Ok(interfaces) = ctx.guestfs.inspect_network(&ctx.root) {
                println!("{} ({}):", "Network Interfaces".green().bold(), interfaces.len());
                for iface in interfaces {
                    println!("  {} {}", "•".cyan(), iface.name.green());
                }
                println!();
            }

            if let Ok(dns) = ctx.guestfs.inspect_dns(&ctx.root) {
                if !dns.is_empty() {
                    println!("{} ({}):", "DNS Servers".green().bold(), dns.len());
                    for server in dns {
                        println!("  {} {}", "•".cyan(), server.yellow());
                    }
                    println!();
                }
            }

            // Check for common network files
            println!("{}", "Network Configuration Files:".green().bold());
            let net_files = vec!["/etc/hosts", "/etc/resolv.conf", "/etc/hostname"];
            for file in net_files {
                if ctx.guestfs.exists(file).unwrap_or(false) {
                    println!("  {} {}", "✓".green(), file.cyan());
                }
            }
            println!();
        }
        "all" => {
            println!("\n{}", "🔍 Comprehensive System Discovery".yellow().bold());
            println!("{}", "═".repeat(60).cyan());
            println!();

            cmd_discover(ctx, &["files"])?;
            cmd_discover(ctx, &["apps"])?;
            cmd_discover(ctx, &["network"])?;

            println!("{}", "═".repeat(60).cyan());
            println!("{} Discovery complete!", "✓".green());
            println!();
        }
        _ => {
            println!("{} Unknown discovery type: {}", "Error:".red(), discover_type);
        }
    }

    Ok(())
}

/// Generate formatted reports
pub fn cmd_report(ctx: &mut ShellContext, args: &[&str]) -> Result<()> {
    if args.is_empty() {
        println!("\n{}", "╔═══════════════════════════════════════════════════════════╗".cyan().bold());
        println!("{}", "║                  Report Generator                        ║".cyan().bold());
        println!("{}", "╚═══════════════════════════════════════════════════════════╝".cyan().bold());
        println!();

        println!("{}", "Available Reports:".yellow().bold());
        println!("  {} - Executive summary report", "report executive".cyan());
        println!("  {} - Technical detail report", "report technical".cyan());
        println!("  {} - Security audit report", "report security".cyan());
        println!("  {} - Compliance report", "report compliance".cyan());
        println!();

        println!("{}", "Output Options:".green().bold());
        println!("  Add {} to save to file", "--output <file>".cyan());
        println!("  Example: report executive --output summary.md");
        println!();

        return Ok(());
    }

    let report_type = args[0];
    let output_file = if args.len() > 2 && args[1] == "--output" {
        Some(args[2])
    } else {
        None
    };

    let mut report_content = String::new();

    match report_type {
        "executive" => {
            use chrono::Local;
            report_content.push_str("# Executive Summary Report\n\n");
            report_content.push_str(&format!("**Generated:** {}\n\n", Local::now().format("%Y-%m-%d %H:%M:%S")));

            report_content.push_str("## Overview\n\n");

            if let Ok(os) = ctx.guestfs.inspect_get_product_name(&ctx.root) {
                report_content.push_str(&format!("System running **{}**", os));
            }

            if let Ok(hostname) = ctx.guestfs.inspect_get_hostname(&ctx.root) {
                report_content.push_str(&format!(" on host **{}**", hostname));
            }
            report_content.push_str(".\n\n");

            report_content.push_str("## Key Metrics\n\n");

            if let Ok(pkg_info) = ctx.guestfs.inspect_packages(&ctx.root) {
                report_content.push_str(&format!("- **Installed Packages:** {}\n", pkg_info.packages.len()));
            }

            if let Ok(users) = ctx.guestfs.inspect_users(&ctx.root) {
                report_content.push_str(&format!("- **User Accounts:** {}\n", users.len()));
            }

            if let Ok(services) = ctx.guestfs.inspect_systemd_services(&ctx.root) {
                let enabled = services.iter().filter(|s| s.enabled).count();
                report_content.push_str(&format!("- **Active Services:** {}/{}\n", enabled, services.len()));
            }

            report_content.push_str("\n## Recommendations\n\n");
            report_content.push_str("- Review security configuration\n");
            report_content.push_str("- Verify all services are necessary\n");
            report_content.push_str("- Ensure regular updates are applied\n");

            println!("\n{}", "📊 Executive Summary Report".yellow().bold());
            println!("{}", "─".repeat(60).cyan());
            println!();
            println!("{}", report_content);
        }
        "security" => {
            report_content.push_str("# Security Audit Report\n\n");

            report_content.push_str("## Security Features\n\n");

            if let Ok(sec) = ctx.guestfs.inspect_security(&ctx.root) {
                report_content.push_str(&format!("- SELinux: {}\n", sec.selinux));
                report_content.push_str(&format!("- AppArmor: {}\n", if sec.apparmor { "Enabled" } else { "Disabled" }));
                report_content.push_str(&format!("- Auditd: {}\n", if sec.auditd { "Enabled" } else { "Disabled" }));
            }

            report_content.push_str("\n## Firewall Status\n\n");

            if let Ok(fw) = ctx.guestfs.inspect_firewall(&ctx.root) {
                report_content.push_str(&format!("- Status: {}\n", if fw.enabled { "Enabled" } else { "**Disabled**" }));
                report_content.push_str(&format!("- Type: {}\n", fw.firewall_type));
            }

            println!("\n{}", "🔒 Security Audit Report".yellow().bold());
            println!("{}", "─".repeat(60).cyan());
            println!();
            println!("{}", report_content);
        }
        _ => {
            println!("{} Unknown report type: {}", "Error:".red(), report_type);
            return Ok(());
        }
    }

    if let Some(file) = output_file {
        use std::fs;
        fs::write(file, &report_content)?;
        println!("{} Report saved to: {}", "✓".green(), file.yellow());
    }

    println!();
    Ok(())
}

/// Automation and macro system
pub fn cmd_auto(ctx: &mut ShellContext, args: &[&str]) -> Result<()> {
    if args.is_empty() {
        println!("\n{}", "╔═══════════════════════════════════════════════════════════╗".cyan().bold());
        println!("{}", "║                  Automation System                       ║".cyan().bold());
        println!("{}", "╚═══════════════════════════════════════════════════════════╝".cyan().bold());
        println!();

        println!("{}", "Automation Commands:".yellow().bold());
        println!("  {} - Run preset automation", "auto run <preset>".cyan());
        println!("  {} - List available presets", "auto list".cyan());
        println!("  {} - Show preset details", "auto show <preset>".cyan());
        println!();

        println!("{}", "Available Presets:".green().bold());
        println!("  {} - Complete security audit", "security-audit".cyan());
        println!("  {} - Full system analysis", "full-analysis".cyan());
        println!("  {} - Quick health check", "health-check".cyan());
        println!("  {} - Export all data", "export-all".cyan());
        println!("  {} - Documentation package", "documentation".cyan());
        println!();

        println!("{}", "Example:".yellow());
        println!("  auto run security-audit");
        println!();

        return Ok(());
    }

    let auto_command = args[0];

    match auto_command {
        "run" => {
            if args.len() < 2 {
                println!("{} Usage: auto run <preset>", "Error:".red());
                return Ok(());
            }

            let preset = args[1];

            println!("\n{}", "╔═══════════════════════════════════════════════════════════╗".cyan().bold());
            println!("{}", format!("║  Automation: {}                           ║", preset).cyan().bold());
            println!("{}", "╚═══════════════════════════════════════════════════════════╝".cyan().bold());
            println!();

            match preset {
                "security-audit" => {
                    println!("{} Running security audit automation...", "→".cyan());
                    println!();

                    println!("[1/4] {} Running security wizard...", "→".cyan());
                    cmd_wizard(ctx, &["security"])?;

                    println!("[2/4] {} Running security scan...", "→".cyan());
                    cmd_scan(ctx, &["security"])?;

                    println!("[3/4] {} Generating recommendations...", "→".cyan());
                    cmd_recommend(ctx, &[])?;

                    println!("[4/4] {} Creating security report...", "→".cyan());
                    cmd_report(ctx, &["security", "--output", "security-audit.md"])?;

                    println!("{}", "═".repeat(60).cyan());
                    println!("{} Security audit complete!", "✓".green());
                    println!("{} Report saved to: {}", "→".cyan(), "security-audit.md".yellow());
                    println!();
                }
                "full-analysis" => {
                    println!("{} Running full system analysis...", "→".cyan());
                    println!();

                    println!("[1/5] {} System dashboard...", "→".cyan());
                    cmd_dashboard(ctx, &[])?;

                    println!("[2/5] {} System discovery...", "→".cyan());
                    cmd_discover(ctx, &["all"])?;

                    println!("[3/5] {} Health check...", "→".cyan());
                    cmd_wizard(ctx, &["health"])?;

                    println!("[4/5] {} Creating snapshot...", "→".cyan());
                    cmd_snapshot(ctx, &["full-analysis-snapshot"])?;

                    println!("[5/5] {} Generating executive report...", "→".cyan());
                    cmd_report(ctx, &["executive", "--output", "executive-summary.md"])?;

                    println!("{}", "═".repeat(60).cyan());
                    println!("{} Full analysis complete!", "✓".green());
                    println!();
                }
                "health-check" => {
                    println!("{} Running health check...", "→".cyan());
                    println!();

                    cmd_wizard(ctx, &["health"])?;
                    cmd_scan(ctx, &["issues"])?;
                    cmd_summary(ctx, &[])?;

                    println!("{} Health check complete!", "✓".green());
                    println!();
                }
                "export-all" => {
                    println!("{} Exporting all data...", "→".cyan());
                    println!();

                    cmd_batch(ctx, &["export", "/tmp/guestkit-export"])?;

                    println!("{} Export complete! Check /tmp/guestkit-export/", "✓".green());
                    println!();
                }
                "documentation" => {
                    println!("{} Creating documentation package...", "→".cyan());
                    println!();

                    cmd_snapshot(ctx, &["system-documentation"])?;
                    cmd_report(ctx, &["executive", "--output", "executive-summary.md"])?;
                    cmd_report(ctx, &["security", "--output", "security-report.md"])?;
                    cmd_profile(ctx, &["create", "system-profile"])?;

                    println!("{} Documentation package created!", "✓".green());
                    println!("{} Files created:", "→".cyan());
                    println!("  - system-documentation.md");
                    println!("  - executive-summary.md");
                    println!("  - security-report.md");
                    println!("  - system-profile.md");
                    println!();
                }
                _ => {
                    println!("{} Unknown preset: {}", "Error:".red(), preset);
                    println!("{} Use 'auto list' to see available presets", "Tip:".yellow());
                }
            }
        }
        "list" => {
            println!("\n{}", "Available Automation Presets:".yellow().bold());
            println!();

            let presets = vec![
                ("security-audit", "Complete security audit with report", "4 steps"),
                ("full-analysis", "Comprehensive system analysis", "5 steps"),
                ("health-check", "Quick system health check", "3 steps"),
                ("export-all", "Export all data types", "1 step"),
                ("documentation", "Create full documentation package", "4 files"),
            ];

            for (name, description, info) in presets {
                println!("  {} {} - {}", name.cyan().bold(), info.bright_black(), description);
            }
            println!();
        }
        "show" => {
            if args.len() < 2 {
                println!("{} Usage: auto show <preset>", "Error:".red());
                return Ok(());
            }

            let preset = args[1];
            println!("\n{} Preset Details: {}", "→".cyan(), preset.yellow().bold());
            println!();

            match preset {
                "security-audit" => {
                    println!("Steps:");
                    println!("  1. Run security wizard");
                    println!("  2. Run security scan");
                    println!("  3. Generate recommendations");
                    println!("  4. Create security report");
                    println!();
                    println!("Output: security-audit.md");
                }
                _ => {
                    println!("{} Preset not found", "Error:".red());
                }
            }
            println!();
        }
        _ => {
            println!("{} Unknown automation command: {}", "Error:".red(), auto_command);
        }
    }

    Ok(())
}

/// Interactive menu system
pub fn cmd_menu(_ctx: &mut ShellContext, _args: &[&str]) -> Result<()> {
    println!("\n{}", "╔═══════════════════════════════════════════════════════════╗".cyan().bold());
    println!("{}", "║                  Interactive Menu                        ║".cyan().bold());
    println!("{}", "╚═══════════════════════════════════════════════════════════╝".cyan().bold());
    println!();

    println!("{}", "Main Categories:".yellow().bold());
    println!();
    println!("  {} System Overview & Analysis", "1.".cyan().bold());
    println!("  {} Security & Compliance", "2.".cyan().bold());
    println!("  {} Data Export & Reports", "3.".cyan().bold());
    println!("  {} Search & Discovery", "4.".cyan().bold());
    println!("  {} Automation & Workflows", "5.".cyan().bold());
    println!("  {} Help & Documentation", "6.".cyan().bold());
    println!();

    println!("{}", "━".repeat(60).bright_black());
    println!();

    println!("{}", "Quick Actions:".green().bold());
    println!("  {} Quick security check", "S.".yellow());
    println!("  {} System dashboard", "D.".yellow());
    println!("  {} Create snapshot", "N.".yellow());
    println!("  {} Smart recommendations", "R.".yellow());
    println!();

    println!("{}", "Suggestions:".bright_black());
    println!("  • First time? Try: {}", "dashboard".cyan());
    println!("  • Security review? Try: {}", "wizard security".cyan());
    println!("  • Need export? Try: {}", "auto run export-all".cyan());
    println!("  • Want guidance? Try: {}", "wizard".cyan());
    println!();

    Ok(())
}

/// Visual timeline and progress tracking
pub fn cmd_timeline(ctx: &mut ShellContext, _args: &[&str]) -> Result<()> {
    println!("\n{}", "╔═══════════════════════════════════════════════════════════╗".cyan().bold());
    println!("{}", "║                  Session Timeline                        ║".cyan().bold());
    println!("{}", "╚═══════════════════════════════════════════════════════════╝".cyan().bold());
    println!();

    println!("{}", "Current Session:".yellow().bold());
    println!();

    // Visual timeline
    println!("  {} Session started", "┌─".cyan());
    println!("  {} Shell initialized", "├─".cyan());

    if ctx.command_count > 0 {
        println!("  {} {} commands executed", "├─".cyan(), ctx.command_count.to_string().green().bold());
    }

    if !ctx.aliases.is_empty() {
        println!("  {} {} aliases defined", "├─".cyan(), ctx.aliases.len().to_string().green());
    }

    if !ctx.bookmarks.is_empty() {
        println!("  {} {} bookmarks created", "├─".cyan(), ctx.bookmarks.len().to_string().green());
    }

    if let Some(duration) = ctx.last_command_time {
        println!("  {} Last command: {}", "├─".cyan(),
            format!("{:.2}ms", duration.as_secs_f64() * 1000.0).yellow());
    }

    println!("  {} Current state", "└─".cyan());
    println!();

    println!("{}", "Session Statistics:".green().bold());
    println!("  Path: {}", ctx.current_path.cyan());
    println!("  OS: {}", ctx.get_os_info().yellow());
    println!();

    println!("{}", "Suggested Next Steps:".yellow().bold());
    if ctx.command_count < 5 {
        println!("  • Try {} for system overview", "'dashboard'".cyan());
        println!("  • Run {} for guided help", "'wizard'".cyan());
    } else {
        println!("  • Create snapshot: {}", "'snapshot'".cyan());
        println!("  • Get recommendations: {}", "'recommend'".cyan());
    }
    println!();

    Ok(())
}

/// Benchmark and performance testing
pub fn cmd_bench(ctx: &mut ShellContext, args: &[&str]) -> Result<()> {
    if args.is_empty() {
        println!("\n{}", "╔═══════════════════════════════════════════════════════════╗".cyan().bold());
        println!("{}", "║                  Performance Benchmark                   ║".cyan().bold());
        println!("{}", "╚═══════════════════════════════════════════════════════════╝".cyan().bold());
        println!();

        println!("{}", "Benchmark Commands:".yellow().bold());
        println!("  {} - Benchmark file operations", "bench files".cyan());
        println!("  {} - Benchmark directory listing", "bench list".cyan());
        println!("  {} - Benchmark package queries", "bench packages".cyan());
        println!("  {} - Run all benchmarks", "bench all".cyan());
        println!();

        return Ok(());
    }

    let bench_type = args[0];

    match bench_type {
        "files" => {
            println!("\n{}", "📊 File Operations Benchmark".yellow().bold());
            println!("{}", "─".repeat(60).cyan());
            println!();

            let test_file = "/etc/fstab";
            if ctx.guestfs.exists(test_file).unwrap_or(false) {
                let iterations = 10;
                let start = std::time::Instant::now();

                for _ in 0..iterations {
                    let _ = ctx.guestfs.stat(test_file);
                }

                let duration = start.elapsed();
                let avg = duration.as_micros() / iterations;

                println!("  Test: {} stat operations on {}", iterations, test_file.cyan());
                println!("  Total: {:.2}ms", duration.as_secs_f64() * 1000.0);
                println!("  Average: {}μs per operation", avg.to_string().green());
                println!();
            }
        }
        "list" => {
            println!("\n{}", "📊 Directory Listing Benchmark".yellow().bold());
            println!("{}", "─".repeat(60).cyan());
            println!();

            let start = std::time::Instant::now();
            let result = ctx.guestfs.ls("/etc");
            let duration = start.elapsed();

            if let Ok(entries) = result {
                println!("  Listed {} entries in /etc", entries.len().to_string().green());
                println!("  Time: {:.2}ms", duration.as_secs_f64() * 1000.0);
                println!();
            }
        }
        "packages" => {
            println!("\n{}", "📊 Package Query Benchmark".yellow().bold());
            println!("{}", "─".repeat(60).cyan());
            println!();

            let start = std::time::Instant::now();
            let result = ctx.guestfs.inspect_packages(&ctx.root);
            let duration = start.elapsed();

            if let Ok(pkg_info) = result {
                println!("  Queried {} packages", pkg_info.packages.len().to_string().green());
                println!("  Time: {:.2}ms", duration.as_secs_f64() * 1000.0);
                println!();
            }
        }
        "all" => {
            println!("\n{}", "📊 Complete Benchmark Suite".yellow().bold());
            println!("{}", "═".repeat(60).cyan());
            println!();

            cmd_bench(ctx, &["files"])?;
            cmd_bench(ctx, &["list"])?;
            cmd_bench(ctx, &["packages"])?;

            println!("{}", "═".repeat(60).cyan());
            println!("{} Benchmark complete!", "✓".green());
            println!();
        }
        _ => {
            println!("{} Unknown benchmark: {}", "Error:".red(), bench_type);
        }
    }

    Ok(())
}

/// Show system presets and templates
pub fn cmd_presets(_ctx: &ShellContext, _args: &[&str]) -> Result<()> {
    println!("\n{}", "╔═══════════════════════════════════════════════════════════╗".cyan().bold());
    println!("{}", "║                  Command Presets                         ║".cyan().bold());
    println!("{}", "╚═══════════════════════════════════════════════════════════╝".cyan().bold());
    println!();

    println!("{}", "Quick Start Presets:".yellow().bold());
    println!();

    let presets = vec![
        ("First Time User", vec![
            ("Start here", "dashboard"),
            ("Learn commands", "cheat"),
            ("Get a tip", "tips"),
        ]),
        ("Security Analyst", vec![
            ("Security audit", "wizard security"),
            ("Security scan", "scan security"),
            ("Get recommendations", "recommend"),
        ]),
        ("System Administrator", vec![
            ("Full analysis", "auto run full-analysis"),
            ("Health check", "wizard health"),
            ("Create snapshot", "snapshot"),
        ]),
        ("Auditor/Compliance", vec![
            ("Executive report", "report executive --output summary.md"),
            ("Security report", "report security --output security.md"),
            ("Export all data", "batch export /tmp/audit"),
        ]),
        ("Developer/Researcher", vec![
            ("Discover apps", "discover apps"),
            ("Find files", "search <pattern> --path /etc"),
            ("Profile system", "profile detect"),
        ]),
    ];

    for (role, commands) in presets {
        println!("{}", role.green().bold());
        for (description, command) in commands {
            println!("  {} {} {}", "•".cyan(), description.bright_black(), "-".bright_black());
            println!("    {}", command.yellow());
        }
        println!();
    }

    println!("{}", "Workflow Templates:".yellow().bold());
    println!();
    println!("  {} Complete Audit", "1.".cyan());
    println!("     auto run security-audit");
    println!();
    println!("  {} Documentation Package", "2.".cyan());
    println!("     auto run documentation");
    println!();
    println!("  {} Quick Health Check", "3.".cyan());
    println!("     wizard health && recommend");
    println!();

    Ok(())
}

// Helper functions for new commands

fn format_bytes(bytes: u64) -> String {
    const UNITS: &[&str] = &["B", "KB", "MB", "GB", "TB"];
    let mut size = bytes as f64;
    let mut unit_idx = 0;

    while size >= 1024.0 && unit_idx < UNITS.len() - 1 {
        size /= 1024.0;
        unit_idx += 1;
    }

    format!("{:.2} {}", size, UNITS[unit_idx])
}

fn print_tree(ctx: &mut ShellContext, path: &str, prefix: &str, depth: usize, max_depth: usize) -> Result<()> {
    if depth >= max_depth {
        return Ok(());
    }

    let entries = match ctx.guestfs.ls(path) {
        Ok(e) => e,
        Err(_) => return Ok(()),
    };

    for (i, entry) in entries.iter().enumerate() {
        let is_last = i == entries.len() - 1;
        let connector = if is_last { "└── " } else { "├── " };
        let new_prefix = if is_last { "    " } else { "│   " };

        let full_path = format!("{}/{}", path, entry);
        let is_dir = ctx.guestfs.is_dir(&full_path).unwrap_or(false);

        let display_name = if is_dir {
            format!("{}/", entry).cyan().bold()
        } else {
            entry.normal()
        };

        println!("{}{}{}", prefix, connector, display_name);

        if is_dir {
            let new_prefix_full = format!("{}{}", prefix, new_prefix);
            let _ = print_tree(ctx, &full_path, &new_prefix_full, depth + 1, max_depth);
        }
    }

    Ok(())
}

fn export_packages(ctx: &mut ShellContext, format: &str, output: Option<&str>) -> Result<()> {
    let pkg_info = ctx.guestfs.inspect_packages(&ctx.root)?;
    let packages = &pkg_info.packages;

    let content = match format {
        "json" => {
            let mut json_items = Vec::new();
            for pkg in packages {
                json_items.push(format!(
                    r#"  {{"name": "{}", "version": "{}"}}"#,
                    pkg.name, pkg.version
                ));
            }
            format!("[\n{}\n]", json_items.join(",\n"))
        }
        "csv" => {
            let mut lines = vec!["name,version".to_string()];
            for pkg in packages {
                lines.push(format!("{},{}", pkg.name, pkg.version));
            }
            lines.join("\n")
        }
        "md" => {
            let mut lines = vec![
                "| Package | Version |".to_string(),
                "|---------|---------|".to_string(),
            ];
            for pkg in packages {
                lines.push(format!("| {} | {} |", pkg.name, pkg.version));
            }
            lines.join("\n")
        }
        _ => {
            let mut lines = Vec::new();
            for pkg in packages {
                lines.push(format!("{} - {}", pkg.name, pkg.version));
            }
            lines.join("\n")
        }
    };

    if let Some(file) = output {
        use std::fs;
        fs::write(file, content)?;
        println!("{} Written to: {}", "→".cyan(), file.yellow());
    } else {
        println!("{}", content);
    }

    Ok(())
}

fn export_users(ctx: &mut ShellContext, format: &str, output: Option<&str>) -> Result<()> {
    let users = ctx.guestfs.inspect_users(&ctx.root)?;

    let content = match format {
        "json" => {
            let mut json_items = Vec::new();
            for user in users {
                json_items.push(format!(
                    r#"  {{"username": "{}", "uid": "{}", "gid": "{}", "home": "{}"}}"#,
                    user.username, user.uid, user.gid, user.home
                ));
            }
            format!("[\n{}\n]", json_items.join(",\n"))
        }
        "csv" => {
            let mut lines = vec!["username,uid,gid,home".to_string()];
            for user in users {
                lines.push(format!("{},{},{},{}", user.username, user.uid, user.gid, user.home));
            }
            lines.join("\n")
        }
        "md" => {
            let mut lines = vec![
                "| Username | UID | GID | Home |".to_string(),
                "|----------|-----|-----|------|".to_string(),
            ];
            for user in users {
                lines.push(format!("| {} | {} | {} | {} |", user.username, user.uid, user.gid, user.home));
            }
            lines.join("\n")
        }
        _ => {
            let mut lines = Vec::new();
            for user in users {
                lines.push(format!("{} ({}:{}) - {}", user.username, user.uid, user.gid, user.home));
            }
            lines.join("\n")
        }
    };

    if let Some(file) = output {
        use std::fs;
        fs::write(file, content)?;
        println!("{} Written to: {}", "→".cyan(), file.yellow());
    } else {
        println!("{}", content);
    }

    Ok(())
}

fn export_services(ctx: &mut ShellContext, format: &str, output: Option<&str>) -> Result<()> {
    let services = ctx.guestfs.inspect_systemd_services(&ctx.root)?;

    let content = match format {
        "json" => {
            let mut json_items = Vec::new();
            for svc in services {
                json_items.push(format!(
                    r#"  {{"name": "{}", "enabled": {}}}"#,
                    svc.name, svc.enabled
                ));
            }
            format!("[\n{}\n]", json_items.join(",\n"))
        }
        "csv" => {
            let mut lines = vec!["name,enabled".to_string()];
            for svc in services {
                lines.push(format!("{},{}", svc.name, svc.enabled));
            }
            lines.join("\n")
        }
        "md" => {
            let mut lines = vec![
                "| Service | Enabled |".to_string(),
                "|---------|---------|".to_string(),
            ];
            for svc in services {
                lines.push(format!("| {} | {} |", svc.name, svc.enabled));
            }
            lines.join("\n")
        }
        _ => {
            let mut lines = Vec::new();
            for svc in services {
                let status = if svc.enabled { "enabled" } else { "disabled" };
                lines.push(format!("{} - {}", svc.name, status));
            }
            lines.join("\n")
        }
    };

    if let Some(file) = output {
        use std::fs;
        fs::write(file, content)?;
        println!("{} Written to: {}", "→".cyan(), file.yellow());
    } else {
        println!("{}", content);
    }

    Ok(())
}

fn export_system(ctx: &mut ShellContext, format: &str, output: Option<&str>) -> Result<()> {
    let os_type = ctx.guestfs.inspect_get_type(&ctx.root).unwrap_or_else(|_| "unknown".to_string());
    let distro = ctx.guestfs.inspect_get_distro(&ctx.root).unwrap_or_else(|_| "unknown".to_string());
    let version = ctx.guestfs.inspect_get_product_name(&ctx.root).unwrap_or_else(|_| "unknown".to_string());
    let arch = ctx.guestfs.inspect_get_arch(&ctx.root).unwrap_or_else(|_| "unknown".to_string());
    let hostname = ctx.guestfs.inspect_get_hostname(&ctx.root).unwrap_or_else(|_| "unknown".to_string());

    let content = match format {
        "json" => {
            format!(
                r#"{{
  "type": "{}",
  "distribution": "{}",
  "version": "{}",
  "architecture": "{}",
  "hostname": "{}"
}}"#,
                os_type, distro, version, arch, hostname
            )
        }
        "md" => {
            format!(
                "# System Information\n\n\
                | Property | Value |\n\
                |----------|-------|\n\
                | Type | {} |\n\
                | Distribution | {} |\n\
                | Version | {} |\n\
                | Architecture | {} |\n\
                | Hostname | {} |",
                os_type, distro, version, arch, hostname
            )
        }
        _ => {
            format!(
                "System Information:\n\
                  Type: {}\n\
                  Distribution: {}\n\
                  Version: {}\n\
                  Architecture: {}\n\
                  Hostname: {}",
                os_type, distro, version, arch, hostname
            )
        }
    };

    if let Some(file) = output {
        use std::fs;
        fs::write(file, content)?;
        println!("{} Written to: {}", "→".cyan(), file.yellow());
    } else {
        println!("{}", content);
    }

    Ok(())
}

/// Context-aware suggestions based on current location
pub fn cmd_context(ctx: &mut ShellContext, _args: &[&str]) -> Result<()> {
    println!("\n{}", "╔═══════════════════════════════════════════════════════════╗".cyan().bold());
    println!("{}", "║              Context-Aware Suggestions                   ║".cyan().bold());
    println!("{}", "╚═══════════════════════════════════════════════════════════╝".cyan().bold());
    println!();

    println!("{} Current Location: {}", "📍".cyan(), ctx.current_path.yellow().bold());
    println!();

    // Analyze current path and provide context-aware suggestions
    let path = &ctx.current_path;
    let mut suggestions = Vec::new();

    if path.contains("/etc") {
        suggestions.push(("High", "Configuration files detected", "cat /etc/fstab"));
        suggestions.push(("High", "View network configuration", "cat /etc/hosts"));
        suggestions.push(("Medium", "Check installed services", "services"));
        suggestions.push(("Medium", "Review security settings", "security"));
    } else if path.contains("/var/log") {
        suggestions.push(("High", "Search for errors in logs", "grep ERROR ."));
        suggestions.push(("High", "View recent log files", "recent . 10"));
        suggestions.push(("Medium", "Find critical messages", "search critical --content"));
    } else if path.contains("/home") || path.contains("/root") {
        suggestions.push(("High", "List user files", "ls -la"));
        suggestions.push(("Medium", "Find configuration files", "find .* ."));
        suggestions.push(("Low", "Search for SSH keys", "find .ssh ."));
    } else if path.contains("/usr") {
        suggestions.push(("High", "Installed applications", "discover apps"));
        suggestions.push(("Medium", "Package information", "packages"));
    } else if path == "/" {
        suggestions.push(("High", "System overview", "dashboard"));
        suggestions.push(("High", "Quick summary", "summary"));
        suggestions.push(("Medium", "Security analysis", "wizard security"));
        suggestions.push(("Low", "Explore filesystem", "tree / 2"));
    }

    // Add generic suggestions
    if !path.contains("/var/log") {
        suggestions.push(("Info", "Navigate to logs", "cd /var/log"));
    }
    if !path.contains("/etc") {
        suggestions.push(("Info", "Navigate to config", "cd /etc"));
    }

    println!("{}", "Suggested Actions:".yellow().bold());
    println!("{}", "─".repeat(70).cyan());

    for (priority, desc, cmd) in suggestions {
        let priority_colored = match priority {
            "High" => priority.red().bold(),
            "Medium" => priority.yellow().bold(),
            "Low" => priority.green(),
            _ => priority.cyan(),
        };

        println!("  {} {} - {}", priority_colored, desc, cmd.bright_black());
    }

    println!();
    println!("{} Run {} for location-specific help", "💡".yellow(), "context".cyan());
    println!();

    Ok(())
}

/// Interactive tutorial system
pub fn cmd_learn(_ctx: &ShellContext, args: &[&str]) -> Result<()> {
    if args.is_empty() {
        println!("\n{}", "╔═══════════════════════════════════════════════════════════╗".cyan().bold());
        println!("{}", "║                  Learning Center                         ║".cyan().bold());
        println!("{}", "╚═══════════════════════════════════════════════════════════╝".cyan().bold());
        println!();

        println!("{}", "Available Tutorials:".yellow().bold());
        println!("{}", "─".repeat(70).cyan());
        println!();

        let tutorials = vec![
            ("basics", "Getting started with GuestKit", "5 min", "🎓"),
            ("navigation", "Filesystem navigation", "3 min", "🗺"),
            ("security", "Security analysis workflow", "10 min", "🔒"),
            ("export", "Exporting data and reports", "5 min", "💾"),
            ("advanced", "Advanced search and batch operations", "8 min", "⚡"),
            ("automation", "Automation and presets", "7 min", "🤖"),
        ];

        for (name, desc, duration, icon) in tutorials {
            println!("  {} {} - {} {}",
                icon,
                name.green().bold(),
                desc,
                format!("({})", duration).bright_black()
            );
        }

        println!();
        println!("{} learn <tutorial>", "Usage:".yellow());
        println!("{} learn basics", "Example:".cyan());
        println!();
        return Ok(());
    }

    let tutorial = args[0];

    match tutorial {
        "basics" => {
            println!("\n{} {}", "📚".cyan(), "Tutorial: Getting Started with GuestKit".yellow().bold());
            println!("{}", "═".repeat(70).cyan());
            println!();

            println!("{} Introduction", "Step 1:".green().bold());
            println!("GuestKit is a powerful VM inspection shell. You're currently inside");
            println!("a mounted VM filesystem, allowing you to explore it safely.\n");

            println!("{} Basic Navigation", "Step 2:".green().bold());
            println!("  • {} - See where you are", "pwd".cyan());
            println!("  • {} - List files and directories", "ls".cyan());
            println!("  • {} - Change directory", "cd <path>".cyan());
            println!("  • {} - Read file contents", "cat <file>".cyan());
            println!();

            println!("{} Getting Information", "Step 3:".green().bold());
            println!("  • {} - Beautiful system overview", "dashboard".cyan());
            println!("  • {} - Quick one-line summary", "summary".cyan());
            println!("  • {} - System information", "info".cyan());
            println!();

            println!("{} Try it now!", "💡".yellow());
            println!("  Type: {}", "dashboard".green());
            println!();
        }

        "navigation" => {
            println!("\n{} {}", "🗺".cyan(), "Tutorial: Filesystem Navigation".yellow().bold());
            println!("{}", "═".repeat(70).cyan());
            println!();

            println!("{} Understanding Paths", "Lesson 1:".green().bold());
            println!("  • Absolute paths start with /   Example: {}", "/etc/fstab".cyan());
            println!("  • Relative paths are from current location");
            println!("  • {} goes up one directory", "..".cyan());
            println!("  • {} stays in current directory", ".".cyan());
            println!();

            println!("{} Quick Navigation", "Lesson 2:".green().bold());
            println!("  • {} - Create bookmarks for favorite locations", "bookmark".cyan());
            println!("  • {} - Jump to a bookmark", "goto <name>".cyan());
            println!("  • {} - Quick command aliases", "alias".cyan());
            println!();

            println!("{} Visual Tools", "Lesson 3:".green().bold());
            println!("  • {} - Visualize directory structure", "tree".cyan());
            println!("  • {} - Find files by pattern", "find <pattern>".cyan());
            println!("  • {} - Search with filters", "search <pattern>".cyan());
            println!();

            println!("{} Try it!", "💡".yellow());
            println!("  1. {}", "bookmark myspot".green());
            println!("  2. {}", "cd /etc".green());
            println!("  3. {}", "goto myspot".green());
            println!();
        }

        "security" => {
            println!("\n{} {}", "🔒".cyan(), "Tutorial: Security Analysis".yellow().bold());
            println!("{}", "═".repeat(70).cyan());
            println!();

            println!("{} Quick Security Check", "Step 1:".green().bold());
            println!("  Run: {}", "security".cyan());
            println!("  This shows SELinux, AppArmor, Firewall, and audit status\n");

            println!("{} Security Wizard", "Step 2:".green().bold());
            println!("  Run: {}", "wizard security".cyan());
            println!("  Get a security score (A-D grade) with detailed analysis\n");

            println!("{} Vulnerability Scanning", "Step 3:".green().bold());
            println!("  Run: {}", "scan security".cyan());
            println!("  Find security issues with severity ratings\n");

            println!("{} Get Recommendations", "Step 4:".green().bold());
            println!("  Run: {}", "recommend".cyan());
            println!("  Receive prioritized security recommendations\n");

            println!("{} Complete Audit", "Step 5:".green().bold());
            println!("  Run: {}", "auto run security-audit".cyan());
            println!("  Automated full security audit with report generation\n");

            println!("{} Try the wizard now!", "💡".yellow());
            println!("  Type: {}", "wizard security".green());
            println!();
        }

        "export" => {
            println!("\n{} {}", "💾".cyan(), "Tutorial: Exporting Data".yellow().bold());
            println!("{}", "═".repeat(70).cyan());
            println!();

            println!("{} Export Formats", "Step 1:".green().bold());
            println!("  GuestKit supports: JSON, CSV, Markdown, Plain text\n");

            println!("{} Export Examples", "Step 2:".green().bold());
            println!("  • {} - Package list as JSON", "export packages json packages.json".cyan());
            println!("  • {} - Users as CSV", "export users csv users.csv".cyan());
            println!("  • {} - Services as Markdown", "export services md services.md".cyan());
            println!();

            println!("{} Snapshots", "Step 3:".green().bold());
            println!("  • {} - Complete system snapshot", "snapshot report.md".cyan());
            println!("  Creates comprehensive Markdown report with all data\n");

            println!("{} Batch Export", "Step 4:".green().bold());
            println!("  • {} - Export everything", "batch export /tmp/reports".cyan());
            println!("  • {} - All data in one command", "auto run export-all".cyan());
            println!();

            println!("{} Try it!", "💡".yellow());
            println!("  Type: {}", "snapshot my-system.md".green());
            println!();
        }

        "advanced" => {
            println!("\n{} {}", "⚡".cyan(), "Tutorial: Advanced Features".yellow().bold());
            println!("{}", "═".repeat(70).cyan());
            println!();

            println!("{} Smart Search", "Technique 1:".green().bold());
            println!("  • {} - Search by path", "search <pattern> --path /etc".cyan());
            println!("  • {} - Search by type", "search <pattern> --type file".cyan());
            println!("  • {} - Search in content", "search <pattern> --content".cyan());
            println!();

            println!("{} Batch Operations", "Technique 2:".green().bold());
            println!("  • {} - Read multiple files", "batch cat file1 file2 file3".cyan());
            println!("  • {} - Search multiple dirs", "batch find pattern /etc /var".cyan());
            println!();

            println!("{} Pin Favorites", "Technique 3:".green().bold());
            println!("  • {} - Save command", "pin errors 'grep ERROR /var/log'".cyan());
            println!("  • {} - Run pinned command", "pin run errors".cyan());
            println!();

            println!("{} Recent Files", "Technique 4:".green().bold());
            println!("  • {} - Recently modified", "recent /var/log 20".cyan());
            println!();

            println!("{} Try it!", "💡".yellow());
            println!("  Type: {}", "search error --content --path /var/log".green());
            println!();
        }

        "automation" => {
            println!("\n{} {}", "🤖".cyan(), "Tutorial: Automation & Presets".yellow().bold());
            println!("{}", "═".repeat(70).cyan());
            println!();

            println!("{} Automation Presets", "Step 1:".green().bold());
            println!("  • {} - Complete security workflow", "auto run security-audit".cyan());
            println!("  • {} - Full system analysis", "auto run full-analysis".cyan());
            println!("  • {} - Health assessment", "auto run health-check".cyan());
            println!("  • {} - Export everything", "auto run export-all".cyan());
            println!();

            println!("{} Interactive Menu", "Step 2:".green().bold());
            println!("  • {} - Navigate via menu", "menu".cyan());
            println!("  Choose from categorized options\n");

            println!("{} Role-Based Presets", "Step 3:".green().bold());
            println!("  • {} - Get commands for your role", "presets".cyan());
            println!("  Roles: Security Analyst, SysAdmin, Developer, Auditor\n");

            println!("{} Benchmarking", "Step 4:".green().bold());
            println!("  • {} - Performance testing", "bench <type>".cyan());
            println!();

            println!("{} Try a full analysis!", "💡".yellow());
            println!("  Type: {}", "auto run full-analysis".green());
            println!();
        }

        _ => {
            println!("{} Unknown tutorial: {}", "Error:".red(), tutorial);
            println!("{} learn", "Usage:".yellow());
            return Ok(());
        }
    }

    Ok(())
}

/// Focus mode for specific system aspects
pub fn cmd_focus(ctx: &mut ShellContext, args: &[&str]) -> Result<()> {
    if args.is_empty() {
        println!("\n{}", "Usage: focus <aspect>".red());
        println!();
        println!("{}", "Available Focus Areas:".yellow().bold());
        println!("  {} - Security posture and vulnerabilities", "security".green());
        println!("  {} - Performance and resource usage", "performance".green());
        println!("  {} - Network configuration and connectivity", "network".green());
        println!("  {} - Storage and filesystems", "storage".green());
        println!("  {} - User accounts and permissions", "users".green());
        println!();
        return Ok(());
    }

    let aspect = args[0];

    match aspect {
        "security" => {
            println!("\n{} {}", "🔒".cyan(), "Security Focus Mode".yellow().bold());
            println!("{}", "═".repeat(70).cyan());
            println!();

            if let Ok(sec) = ctx.guestfs.inspect_security(&ctx.root) {
                println!("{}", "Security Status:".green().bold());
                println!("  SELinux:  {}", if &sec.selinux != "disabled" { sec.selinux.green() } else { sec.selinux.red() });
                println!("  AppArmor: {}", if sec.apparmor { "enabled".green() } else { "disabled".red() });
                println!("  auditd:   {}", if sec.auditd { "enabled".green() } else { "disabled".red() });
                println!();
            }

            if let Ok(fw) = ctx.guestfs.inspect_firewall(&ctx.root) {
                println!("{}", "Firewall Configuration:".green().bold());
                println!("  Type:   {}", fw.firewall_type.cyan());
                println!("  Status: {}", if fw.enabled { "enabled".green() } else { "disabled".red() });
                println!();
            }

            println!("{}", "Critical Files to Review:".yellow().bold());
            let security_files = vec![
                "/etc/shadow", "/etc/sudoers", "/etc/ssh/sshd_config",
                "/etc/pam.d/", "/etc/security/", "/etc/selinux/config"
            ];
            for file in security_files {
                let exists = ctx.guestfs.exists(file).unwrap_or(false);
                let status = if exists { "✓".green() } else { "✗".red() };
                println!("  {} {}", status, file.cyan());
            }
            println!();

            println!("{} Next Steps:", "💡".yellow());
            println!("  • {}", "wizard security - Get security score".cyan());
            println!("  • {}", "scan security - Find vulnerabilities".cyan());
            println!("  • {}", "recommend - Get security recommendations".cyan());
            println!();
        }

        "performance" => {
            println!("\n{} {}", "⚡".cyan(), "Performance Focus Mode".yellow().bold());
            println!("{}", "═".repeat(70).cyan());
            println!();

            if let Ok(pkg_info) = ctx.guestfs.inspect_packages(&ctx.root) {
                println!("{}", "Package Statistics:".green().bold());
                println!("  Total packages: {}", pkg_info.packages.len().to_string().yellow());
                println!();
            }

            if let Ok(services) = ctx.guestfs.inspect_systemd_services(&ctx.root) {
                let enabled = services.iter().filter(|s| s.enabled).count();
                println!("{}", "Service Statistics:".green().bold());
                println!("  Total services: {}", services.len().to_string().yellow());
                println!("  Enabled: {}", enabled.to_string().green());
                println!("  Disabled: {}", (services.len() - enabled).to_string().bright_black());
                println!();
            }

            println!("{} Benchmarking:", "💡".yellow());
            println!("  • {}", "bench files - Test filesystem operations".cyan());
            println!("  • {}", "bench list - Test directory listing".cyan());
            println!("  • {}", "bench packages - Test package queries".cyan());
            println!();
        }

        "network" => {
            println!("\n{} {}", "🌐".cyan(), "Network Focus Mode".yellow().bold());
            println!("{}", "═".repeat(70).cyan());
            println!();

            if let Ok(interfaces) = ctx.guestfs.inspect_network(&ctx.root) {
                println!("{} ({} total)", "Network Interfaces:".green().bold(), interfaces.len());
                for iface in interfaces {
                    println!("  • {}", iface.name.cyan());
                }
                println!();
            }

            if let Ok(dns) = ctx.guestfs.inspect_dns(&ctx.root) {
                if !dns.is_empty() {
                    println!("{}", "DNS Servers:".green().bold());
                    for server in dns {
                        println!("  • {}", server.yellow());
                    }
                    println!();
                }
            }

            println!("{}", "Network Configuration Files:".yellow().bold());
            let net_files = vec![
                "/etc/hosts", "/etc/resolv.conf", "/etc/hostname",
                "/etc/sysconfig/network", "/etc/network/interfaces"
            ];
            for file in net_files {
                if ctx.guestfs.exists(file).unwrap_or(false) {
                    println!("  {} {}", "✓".green(), file.cyan());
                }
            }
            println!();

            println!("{} Explore further:", "💡".yellow());
            println!("  • {}", "cat /etc/hosts".cyan());
            println!("  • {}", "discover network".cyan());
            println!();
        }

        "storage" => {
            println!("\n{} {}", "💾".cyan(), "Storage Focus Mode".yellow().bold());
            println!("{}", "═".repeat(70).cyan());
            println!();

            if let Ok(devices) = ctx.guestfs.list_devices() {
                println!("{} ({} total)", "Block Devices:".green().bold(), devices.len());
                for device in devices {
                    println!("  • {}", device.cyan());
                }
                println!();
            }

            if let Ok(filesystems) = ctx.guestfs.list_filesystems() {
                println!("{}", "Filesystems:".green().bold());
                for (device, fstype) in filesystems {
                    println!("  {} - {}", device.yellow(), fstype.cyan());
                }
                println!();
            }

            println!("{} Storage Commands:", "💡".yellow());
            println!("  • {}", "mounts - View mounted filesystems".cyan());
            println!("  • {}", "cat /etc/fstab - View mount configuration".cyan());
            println!("  • {}", "tree / 2 - Filesystem overview".cyan());
            println!();
        }

        "users" => {
            println!("\n{} {}", "👥".cyan(), "User Accounts Focus Mode".yellow().bold());
            println!("{}", "═".repeat(70).cyan());
            println!();

            if let Ok(users) = ctx.guestfs.inspect_users(&ctx.root) {
                let root_users: Vec<_> = users.iter().filter(|u| u.uid == "0").collect();
                let system_users: Vec<_> = users.iter().filter(|u| {
                    if let Ok(uid) = u.uid.parse::<u32>() {
                        uid > 0 && uid < 1000
                    } else {
                        false
                    }
                }).collect();
                let regular_users: Vec<_> = users.iter().filter(|u| {
                    if let Ok(uid) = u.uid.parse::<u32>() {
                        uid >= 1000
                    } else {
                        false
                    }
                }).collect();

                println!("{}", "User Statistics:".green().bold());
                println!("  Root accounts:    {} {}", root_users.len().to_string().red().bold(), if root_users.len() > 1 { "(⚠ Multiple root accounts!)".yellow() } else { "".normal() });
                println!("  System accounts:  {}", system_users.len().to_string().cyan());
                println!("  Regular accounts: {}", regular_users.len().to_string().green());
                println!();

                println!("{}", "Regular Users:".yellow().bold());
                for user in regular_users.iter().take(10) {
                    println!("  {} (UID: {}, Home: {})",
                        user.username.green(),
                        user.uid.bright_black(),
                        user.home.cyan()
                    );
                }
                println!();
            }

            println!("{}", "User Configuration Files:".yellow().bold());
            let user_files = vec![
                ("/etc/passwd", "User accounts"),
                ("/etc/shadow", "Password hashes"),
                ("/etc/group", "Group definitions"),
                ("/etc/sudoers", "Sudo privileges"),
            ];
            for (file, desc) in user_files {
                let exists = ctx.guestfs.exists(file).unwrap_or(false);
                let status = if exists { "✓".green() } else { "✗".red() };
                println!("  {} {} - {}", status, file.cyan(), desc.bright_black());
            }
            println!();

            println!("{} Deep dive:", "💡".yellow());
            println!("  • {}", "users - Full user list".cyan());
            println!("  • {}", "cat /etc/passwd".cyan());
            println!();
        }

        _ => {
            println!("{} Unknown focus area: {}", "Error:".red(), aspect);
            println!("{} focus <aspect>", "Usage:".yellow());
            return Ok(());
        }
    }

    Ok(())
}

/// Security and operations playbooks
pub fn cmd_playbook(_ctx: &mut ShellContext, args: &[&str]) -> Result<()> {
    if args.is_empty() {
        println!("\n{}", "╔═══════════════════════════════════════════════════════════╗".cyan().bold());
        println!("{}", "║                    Playbook Library                      ║".cyan().bold());
        println!("{}", "╚═══════════════════════════════════════════════════════════╝".cyan().bold());
        println!();

        println!("{}", "Available Playbooks:".yellow().bold());
        println!("{}", "─".repeat(70).cyan());
        println!();

        let playbooks = vec![
            ("incident", "🚨", "Security incident response", "Advanced"),
            ("hardening", "🔒", "System security hardening", "Intermediate"),
            ("audit", "📋", "Compliance audit checklist", "Intermediate"),
            ("forensics", "🔍", "Digital forensics investigation", "Advanced"),
            ("migration", "📦", "VM migration preparation", "Intermediate"),
            ("recovery", "🔧", "System recovery procedures", "Intermediate"),
        ];

        for (name, icon, desc, level) in playbooks {
            let level_color = match level {
                "Advanced" => level.red(),
                "Intermediate" => level.yellow(),
                _ => level.green(),
            };
            println!("  {} {} - {} {}",
                icon,
                name.green().bold(),
                desc,
                format!("[{}]", level_color).bright_black()
            );
        }

        println!();
        println!("{} playbook <name>", "Usage:".yellow());
        println!("{} playbook incident", "Example:".cyan());
        println!();
        return Ok(());
    }

    let playbook = args[0];

    match playbook {
        "incident" => {
            println!("\n{} {}", "🚨".cyan(), "Security Incident Response Playbook".red().bold());
            println!("{}", "═".repeat(70).cyan());
            println!();

            println!("{} Immediate Actions", "Phase 1:".red().bold());
            println!("  {} Document current time and create snapshot", "1.".yellow());
            println!("     Command: {}", "snapshot incident-$(date +%Y%m%d-%H%M%S).md".cyan());
            println!("  {} Capture system state", "2.".yellow());
            println!("     Command: {}", "dashboard".cyan());
            println!("  {} Check currently logged in users", "3.".yellow());
            println!("     Command: {}", "users".cyan());
            println!();

            println!("{} Investigation", "Phase 2:".yellow().bold());
            println!("  {} Review security configuration", "4.".yellow());
            println!("     Command: {}", "security".cyan());
            println!("  {} Scan for security issues", "5.".yellow());
            println!("     Command: {}", "scan security".cyan());
            println!("  {} Check recent file modifications", "6.".yellow());
            println!("     Command: {}", "recent /etc 50".cyan());
            println!("     Command: {}", "recent /var/log 50".cyan());
            println!("  {} Search for suspicious activity", "7.".yellow());
            println!("     Command: {}", "search failed --content --path /var/log".cyan());
            println!("     Command: {}", "search unauthorized --content --path /var/log".cyan());
            println!();

            println!("{} Analysis", "Phase 3:".green().bold());
            println!("  {} Review network configuration", "8.".yellow());
            println!("     Command: {}", "network".cyan());
            println!("  {} Check running services", "9.".yellow());
            println!("     Command: {}", "services".cyan());
            println!("  {} Analyze installed packages", "10.".yellow());
            println!("     Command: {}", "packages".cyan());
            println!();

            println!("{} Reporting", "Phase 4:".cyan().bold());
            println!("  {} Generate comprehensive report", "11.".yellow());
            println!("     Command: {}", "report security --output incident-report.md".cyan());
            println!("  {} Export all evidence", "12.".yellow());
            println!("     Command: {}", "batch export /tmp/incident-evidence".cyan());
            println!();

            println!("{} This playbook helps investigate security incidents systematically", "Note:".yellow().bold());
            println!();
        }

        "hardening" => {
            println!("\n{} {}", "🔒".cyan(), "System Security Hardening Playbook".yellow().bold());
            println!("{}", "═".repeat(70).cyan());
            println!();

            println!("{} Assessment", "Step 1:".green().bold());
            println!("  • Run security wizard: {}", "wizard security".cyan());
            println!("  • Get recommendations: {}", "recommend".cyan());
            println!();

            println!("{} Security Features", "Step 2:".green().bold());
            println!("  {} Check SELinux status", "•".yellow());
            println!("     Location: {}", "/etc/selinux/config".cyan());
            println!("     Command: {}", "cat /etc/selinux/config".cyan());
            println!("  {} Verify AppArmor profiles", "•".yellow());
            println!("     Command: {}", "cat /etc/apparmor.d/".cyan());
            println!("  {} Review firewall rules", "•".yellow());
            println!("     Command: {}", "security".cyan());
            println!();

            println!("{} User Security", "Step 3:".green().bold());
            println!("  {} Audit user accounts", "•".yellow());
            println!("     Command: {}", "users".cyan());
            println!("  {} Check sudo configuration", "•".yellow());
            println!("     Command: {}", "cat /etc/sudoers".cyan());
            println!("  {} Review SSH configuration", "•".yellow());
            println!("     Command: {}", "cat /etc/ssh/sshd_config".cyan());
            println!();

            println!("{} System Services", "Step 4:".green().bold());
            println!("  {} List all enabled services", "•".yellow());
            println!("     Command: {}", "services".cyan());
            println!("  {} Identify unnecessary services", "•".yellow());
            println!("     Review output and disable unused services");
            println!();

            println!("{} Verification", "Step 5:".green().bold());
            println!("  {} Run security scan", "•".yellow());
            println!("     Command: {}", "scan security".cyan());
            println!("  {} Generate compliance report", "•".yellow());
            println!("     Command: {}", "report compliance".cyan());
            println!();
        }

        "audit" => {
            println!("\n{} {}", "📋".cyan(), "Compliance Audit Checklist".yellow().bold());
            println!("{}", "═".repeat(70).cyan());
            println!();

            println!("{}", "System Information:".green().bold());
            println!("  {} {}", "☐".yellow(), "System overview - dashboard".cyan());
            println!("  {} {}", "☐".yellow(), "OS and version - info".cyan());
            println!("  {} {}", "☐".yellow(), "Installed packages - packages".cyan());
            println!();

            println!("{}", "Security Controls:".green().bold());
            println!("  {} {}", "☐".yellow(), "Security features - security".cyan());
            println!("  {} {}", "☐".yellow(), "Firewall configuration - security".cyan());
            println!("  {} {}", "☐".yellow(), "SELinux/AppArmor status - security".cyan());
            println!("  {} {}", "☐".yellow(), "Security audit - wizard security".cyan());
            println!();

            println!("{}", "Access Controls:".green().bold());
            println!("  {} {}", "☐".yellow(), "User accounts - users".cyan());
            println!("  {} {}", "☐".yellow(), "Sudo privileges - cat /etc/sudoers".cyan());
            println!("  {} {}", "☐".yellow(), "SSH configuration - cat /etc/ssh/sshd_config".cyan());
            println!();

            println!("{}", "Network Security:".green().bold());
            println!("  {} {}", "☐".yellow(), "Network configuration - network".cyan());
            println!("  {} {}", "☐".yellow(), "Open ports and services - services".cyan());
            println!();

            println!("{}", "Logging & Monitoring:".green().bold());
            println!("  {} {}", "☐".yellow(), "Audit daemon status - security".cyan());
            println!("  {} {}", "☐".yellow(), "Log files review - recent /var/log 50".cyan());
            println!();

            println!("{}", "Documentation:".green().bold());
            println!("  {} {}", "☐".yellow(), "Generate snapshot - snapshot audit-report.md".cyan());
            println!("  {} {}", "☐".yellow(), "Compliance report - report compliance".cyan());
            println!();
        }

        "forensics" => {
            println!("\n{} {}", "🔍".cyan(), "Digital Forensics Investigation Playbook".yellow().bold());
            println!("{}", "═".repeat(70).cyan());
            println!();

            println!("{} Preservation", "Phase 1:".red().bold());
            println!("  {} Create complete snapshot immediately", "1.".yellow());
            println!("     {}", "snapshot forensics-$(date +%Y%m%d-%H%M%S).md".cyan());
            println!("  {} Export all data for analysis", "2.".yellow());
            println!("     {}", "auto run export-all".cyan());
            println!();

            println!("{} System Analysis", "Phase 2:".yellow().bold());
            println!("  {} Profile the system", "3.".yellow());
            println!("     {}", "profile detect".cyan());
            println!("  {} Review system configuration", "4.".yellow());
            println!("     {}", "info".cyan());
            println!();

            println!("{} Timeline Analysis", "Phase 3:".green().bold());
            println!("  {} Find recently modified files", "5.".yellow());
            println!("     {}", "recent / 100".cyan());
            println!("  {} Check specific directories", "6.".yellow());
            println!("     {}", "recent /etc 50".cyan());
            println!("     {}", "recent /var 50".cyan());
            println!("     {}", "recent /home 50".cyan());
            println!();

            println!("{} Evidence Collection", "Phase 4:".cyan().bold());
            println!("  {} User activity", "7.".yellow());
            println!("     {}", "users".cyan());
            println!("     {}", "cat /var/log/auth.log".cyan());
            println!("  {} Network connections", "8.".yellow());
            println!("     {}", "network".cyan());
            println!("  {} Installed software", "9.".yellow());
            println!("     {}", "packages".cyan());
            println!("  {} Running services", "10.".yellow());
            println!("     {}", "services".cyan());
            println!();

            println!("{} Content Analysis", "Phase 5:".blue().bold());
            println!("  {} Search for indicators of compromise", "11.".yellow());
            println!("     {}", "search <ioc> --content".cyan());
            println!("  {} Batch file examination", "12.".yellow());
            println!("     {}", "batch cat <files...>".cyan());
            println!();

            println!("{} Reporting", "Phase 6:".magenta().bold());
            println!("  {} Generate technical report", "13.".yellow());
            println!("     {}", "report technical --output forensics-report.md".cyan());
            println!();
        }

        "migration" => {
            println!("\n{} {}", "📦".cyan(), "VM Migration Preparation Playbook".yellow().bold());
            println!("{}", "═".repeat(70).cyan());
            println!();

            println!("{} Discovery", "Step 1:".green().bold());
            println!("  {} System profile", "•".yellow());
            println!("     {}", "profile create".cyan());
            println!("  {} Full analysis", "•".yellow());
            println!("     {}", "auto run full-analysis".cyan());
            println!();

            println!("{} Documentation", "Step 2:".green().bold());
            println!("  {} Create comprehensive snapshot", "•".yellow());
            println!("     {}", "snapshot pre-migration.md".cyan());
            println!("  {} Export configuration data", "•".yellow());
            println!("     {}", "export system json system-config.json".cyan());
            println!();

            println!("{} Configuration Review", "Step 3:".green().bold());
            println!("  {} Network settings", "•".yellow());
            println!("     {}", "network".cyan());
            println!("     {}", "cat /etc/hosts".cyan());
            println!("  {} Storage and mounts", "•".yellow());
            println!("     {}", "mounts".cyan());
            println!("     {}", "cat /etc/fstab".cyan());
            println!("  {} Services", "•".yellow());
            println!("     {}", "services".cyan());
            println!();

            println!("{} Dependencies", "Step 4:".green().bold());
            println!("  {} Installed packages", "•".yellow());
            println!("     {}", "export packages csv packages.csv".cyan());
            println!("  {} User accounts", "•".yellow());
            println!("     {}", "export users csv users.csv".cyan());
            println!();

            println!("{} Final Report", "Step 5:".green().bold());
            println!("  {} Generate executive summary", "•".yellow());
            println!("     {}", "report executive --output migration-plan.md".cyan());
            println!();
        }

        "recovery" => {
            println!("\n{} {}", "🔧".cyan(), "System Recovery Procedures".yellow().bold());
            println!("{}", "═".repeat(70).cyan());
            println!();

            println!("{} Assessment", "Phase 1:".green().bold());
            println!("  {} Check system health", "1.".yellow());
            println!("     {}", "wizard health".cyan());
            println!("  {} Identify issues", "2.".yellow());
            println!("     {}", "scan issues".cyan());
            println!();

            println!("{} Critical Files", "Phase 2:".green().bold());
            println!("  {} Verify boot configuration", "3.".yellow());
            println!("     {}", "cat /etc/fstab".cyan());
            println!("     {}", "cat /boot/grub/grub.cfg".cyan());
            println!("  {} Check network configuration", "4.".yellow());
            println!("     {}", "network".cyan());
            println!();

            println!("{} Services", "Phase 3:".green().bold());
            println!("  {} Review service status", "5.".yellow());
            println!("     {}", "services".cyan());
            println!("  {} Check critical services", "6.".yellow());
            println!("     Look for failed or disabled critical services");
            println!();

            println!("{} Logs", "Phase 4:".green().bold());
            println!("  {} Search for errors", "7.".yellow());
            println!("     {}", "search error --content --path /var/log".cyan());
            println!("     {}", "search fail --content --path /var/log".cyan());
            println!("  {} Recent log activity", "8.".yellow());
            println!("     {}", "recent /var/log 50".cyan());
            println!();

            println!("{} Documentation", "Phase 5:".green().bold());
            println!("  {} Create recovery snapshot", "9.".yellow());
            println!("     {}", "snapshot recovery-assessment.md".cyan());
            println!();
        }

        _ => {
            println!("{} Unknown playbook: {}", "Error:".red(), playbook);
            println!("{} playbook", "Usage:".yellow());
            return Ok(());
        }
    }

    Ok(())
}

/// Deep inspection of specific components
pub fn cmd_inspect(ctx: &mut ShellContext, args: &[&str]) -> Result<()> {
    if args.is_empty() {
        println!("\n{}", "Usage: inspect <component>".red());
        println!();
        println!("{}", "Available Components:".yellow().bold());
        println!("  {} - Boot configuration and kernel", "boot".green());
        println!("  {} - System logging configuration", "logging".green());
        println!("  {} - Package manager and repositories", "packages".green());
        println!("  {} - System services and daemons", "services".green());
        println!("  {} - Kernel modules and drivers", "kernel".green());
        println!();
        return Ok(());
    }

    let component = args[0];

    match component {
        "boot" => {
            println!("\n{} {}", "🚀".cyan(), "Boot Configuration Inspection".yellow().bold());
            println!("{}", "═".repeat(70).cyan());
            println!();

            println!("{}", "Filesystem Table (/etc/fstab):".green().bold());
            if ctx.guestfs.exists("/etc/fstab")? {
                if let Ok(content) = ctx.guestfs.read_file("/etc/fstab") {
                    let lines: Vec<&str> = std::str::from_utf8(&content)
                        .unwrap_or("")
                        .lines()
                        .filter(|line| !line.trim().is_empty() && !line.trim().starts_with('#'))
                        .collect();

                    for line in lines {
                        println!("  {}", line.cyan());
                    }
                }
            } else {
                println!("  {} /etc/fstab not found", "✗".red());
            }
            println!();

            println!("{}", "Bootloader Configuration:".green().bold());
            let grub_files = vec![
                "/boot/grub/grub.cfg",
                "/boot/grub2/grub.cfg",
                "/boot/efi/EFI/*/grub.cfg",
            ];
            for file in grub_files {
                if ctx.guestfs.exists(file).unwrap_or(false) {
                    println!("  {} {}", "✓".green(), file.cyan());
                }
            }
            println!();

            // Kernel information would be displayed here if available
            println!();
        }

        "logging" => {
            println!("\n{} {}", "📝".cyan(), "Logging Configuration Inspection".yellow().bold());
            println!("{}", "═".repeat(70).cyan());
            println!();

            println!("{}", "System Logging:".green().bold());
            let log_configs = vec![
                ("/etc/rsyslog.conf", "rsyslog configuration"),
                ("/etc/syslog-ng/syslog-ng.conf", "syslog-ng configuration"),
                ("/etc/systemd/journald.conf", "systemd journal configuration"),
            ];

            for (file, desc) in log_configs {
                if ctx.guestfs.exists(file).unwrap_or(false) {
                    println!("  {} {} - {}", "✓".green(), file.cyan(), desc.bright_black());
                } else {
                    println!("  {} {} - {}", "✗".bright_black(), file.bright_black(), desc.bright_black());
                }
            }
            println!();

            println!("{}", "Log Directories:".green().bold());
            let log_dirs = vec![
                "/var/log",
                "/var/log/audit",
                "/var/log/journal",
            ];

            for dir in log_dirs {
                if ctx.guestfs.is_dir(dir).unwrap_or(false) {
                    if let Ok(files) = ctx.guestfs.ls(dir) {
                        println!("  {} {} ({} files)", "✓".green(), dir.cyan(), files.len().to_string().yellow());
                    }
                }
            }
            println!();

            println!("{} Commands to explore logs:", "💡".yellow());
            println!("  • {}", "cd /var/log".cyan());
            println!("  • {}", "recent /var/log 20".cyan());
            println!("  • {}", "search error --content --path /var/log".cyan());
            println!();
        }

        "packages" => {
            println!("\n{} {}", "📦".cyan(), "Package Manager Inspection".yellow().bold());
            println!("{}", "═".repeat(70).cyan());
            println!();

            if let Ok(pkg_info) = ctx.guestfs.inspect_packages(&ctx.root) {
                println!("{}", "Package Statistics:".green().bold());
                println!("  Total packages: {}", pkg_info.packages.len().to_string().yellow().bold());
                println!();

                // Categorize packages
                let mut categories: std::collections::HashMap<&str, usize> = std::collections::HashMap::new();
                for pkg in &pkg_info.packages {
                    let name = pkg.name.to_lowercase();
                    if name.contains("lib") {
                        *categories.entry("Libraries").or_insert(0) += 1;
                    } else if name.contains("devel") || name.contains("dev") {
                        *categories.entry("Development").or_insert(0) += 1;
                    } else if name.contains("doc") {
                        *categories.entry("Documentation").or_insert(0) += 1;
                    } else if name.contains("kernel") {
                        *categories.entry("Kernel").or_insert(0) += 1;
                    } else if name.contains("python") || name.contains("perl") || name.contains("ruby") {
                        *categories.entry("Interpreters").or_insert(0) += 1;
                    } else {
                        *categories.entry("Other").or_insert(0) += 1;
                    }
                }

                println!("{}", "Package Categories:".green().bold());
                let mut cat_vec: Vec<_> = categories.iter().collect();
                cat_vec.sort_by_key(|(_, count)| std::cmp::Reverse(**count));
                for (cat, count) in cat_vec {
                    println!("  {:15} {}", cat, count.to_string().cyan());
                }
                println!();
            }

            println!("{}", "Package Manager Configuration:".green().bold());
            let pkg_configs = vec![
                ("/etc/yum.conf", "YUM configuration"),
                ("/etc/dnf/dnf.conf", "DNF configuration"),
                ("/etc/apt/sources.list", "APT sources"),
                ("/etc/zypp/zypp.conf", "Zypper configuration"),
            ];

            for (file, desc) in pkg_configs {
                if ctx.guestfs.exists(file).unwrap_or(false) {
                    println!("  {} {} - {}", "✓".green(), file.cyan(), desc.bright_black());
                }
            }
            println!();

            println!("{} Package commands:", "💡".yellow());
            println!("  • {}", "packages <pattern> - Search packages".cyan());
            println!("  • {}", "export packages json - Export package list".cyan());
            println!();
        }

        "services" => {
            println!("\n{} {}", "⚙".cyan(), "System Services Inspection".yellow().bold());
            println!("{}", "═".repeat(70).cyan());
            println!();

            if let Ok(services) = ctx.guestfs.inspect_systemd_services(&ctx.root) {
                let enabled: Vec<_> = services.iter().filter(|s| s.enabled).collect();
                let disabled: Vec<_> = services.iter().filter(|s| !s.enabled).collect();

                println!("{}", "Service Statistics:".green().bold());
                println!("  Total:    {}", services.len().to_string().yellow());
                println!("  Enabled:  {} {}%",
                    enabled.len().to_string().green(),
                    format!("({:.1})", (enabled.len() as f64 / services.len() as f64) * 100.0).bright_black()
                );
                println!("  Disabled: {}", disabled.len().to_string().bright_black());
                println!();

                println!("{}", "Enabled Services (first 20):".green().bold());
                for svc in enabled.iter().take(20) {
                    println!("  {} {}", "✓".green(), svc.name.cyan());
                }
                if enabled.len() > 20 {
                    println!("  ... and {} more", (enabled.len() - 20).to_string().bright_black());
                }
                println!();

                // Categorize services
                let mut critical = Vec::new();
                let mut network = Vec::new();
                let mut security = Vec::new();

                for svc in &enabled {
                    let name = svc.name.to_lowercase();
                    if name.contains("ssh") || name.contains("systemd") || name.contains("dbus") {
                        critical.push(&svc.name);
                    } else if name.contains("network") || name.contains("firewall") {
                        network.push(&svc.name);
                    } else if name.contains("selinux") || name.contains("audit") {
                        security.push(&svc.name);
                    }
                }

                if !critical.is_empty() {
                    println!("{}", "Critical Services:".red().bold());
                    for svc in critical {
                        println!("  • {}", svc.yellow());
                    }
                    println!();
                }

                if !network.is_empty() {
                    println!("{}", "Network Services:".cyan().bold());
                    for svc in network {
                        println!("  • {}", svc.cyan());
                    }
                    println!();
                }

                if !security.is_empty() {
                    println!("{}", "Security Services:".green().bold());
                    for svc in security {
                        println!("  • {}", svc.green());
                    }
                    println!();
                }
            }

            println!("{} Service commands:", "💡".yellow());
            println!("  • {}", "services - List all services".cyan());
            println!("  • {}", "services <pattern> - Search services".cyan());
            println!();
        }

        "kernel" => {
            println!("\n{} {}", "🔧".cyan(), "Kernel Inspection".yellow().bold());
            println!("{}", "═".repeat(70).cyan());
            println!();

            // Kernel version information would be displayed here if available
            println!();

            println!("{}", "Kernel Modules:".green().bold());
            let mod_dirs = vec![
                "/lib/modules",
                "/usr/lib/modules",
            ];

            for dir in mod_dirs {
                if ctx.guestfs.is_dir(dir).unwrap_or(false) {
                    if let Ok(subdirs) = ctx.guestfs.ls(dir) {
                        println!("  {} {}", "✓".green(), dir.cyan());
                        for subdir in subdirs.iter().take(5) {
                            println!("    • {}", subdir.bright_black());
                        }
                        if subdirs.len() > 5 {
                            println!("    ... and {} more", (subdirs.len() - 5).to_string().bright_black());
                        }
                    }
                }
            }
            println!();

            println!("{}", "Kernel Configuration:".green().bold());
            let kernel_configs = vec![
                "/boot/config-*",
                "/proc/config.gz",
            ];

            for pattern in kernel_configs {
                println!("  {}", pattern.cyan());
            }
            println!();

            println!("{} Explore kernel:", "💡".yellow());
            println!("  • {}", "cd /boot".cyan());
            println!("  • {}", "ls -la /boot".cyan());
            println!("  • {}", "cd /lib/modules".cyan());
            println!();
        }

        _ => {
            println!("{} Unknown component: {}", "Error:".red(), component);
            println!("{} inspect <component>", "Usage:".yellow());
            return Ok(());
        }
    }

    Ok(())
}

/// Generate narrative system explanation
pub fn cmd_story(ctx: &mut ShellContext, args: &[&str]) -> Result<()> {
    if args.is_empty() {
        println!("\n{}", "Usage: story <topic>".red());
        println!();
        println!("{}", "Available Story Topics:".yellow().bold());
        println!("  {} - System origin and purpose story", "system".green());
        println!("  {} - Security posture narrative", "security".green());
        println!("  {} - Configuration journey", "config".green());
        println!("  {} - What happened to this system", "timeline".green());
        println!();
        return Ok(());
    }

    let topic = args[0];

    match topic {
        "system" => {
            println!("\n{} {}", "📖".cyan(), "System Story".yellow().bold());
            println!("{}", "═".repeat(70).cyan());
            println!();

            // Gather information
            let os_type = ctx.guestfs.inspect_get_type(&ctx.root).unwrap_or_else(|_| "unknown".to_string());
            let distro = ctx.guestfs.inspect_get_distro(&ctx.root).unwrap_or_else(|_| "unknown".to_string());
            let arch = ctx.guestfs.inspect_get_arch(&ctx.root).unwrap_or_else(|_| "unknown".to_string());

            println!("{}", "Once upon a time, in a datacenter far away...".italic());
            println!();

            println!("This is a {} system, specifically a {} distribution running on {} architecture.",
                os_type.yellow(), distro.green(), arch.cyan());
            println!();

            if let Ok(pkg_info) = ctx.guestfs.inspect_packages(&ctx.root) {
                let pkg_count = pkg_info.packages.len();
                println!("The system has been carefully assembled with {} packages, each serving its purpose",
                    pkg_count.to_string().yellow());
                println!("in the grand tapestry of this computing environment.");
                println!();

                // Identify character
                let web_packages = pkg_info.packages.iter().filter(|p|
                    p.name.contains("httpd") || p.name.contains("nginx") || p.name.contains("apache")
                ).count();

                let db_packages = pkg_info.packages.iter().filter(|p|
                    p.name.contains("mysql") || p.name.contains("postgres") || p.name.contains("mariadb")
                ).count();

                let dev_packages = pkg_info.packages.iter().filter(|p|
                    p.name.contains("gcc") || p.name.contains("make") || p.name.contains("python-devel")
                ).count();

                if web_packages > 0 {
                    println!("This system bears the marks of a {}, with {} web server packages installed.",
                        "web server".green().bold(), web_packages.to_string().yellow());
                    println!("It has likely served countless HTTP requests, delivering content to users worldwide.");
                }

                if db_packages > 0 {
                    println!("Database packages ({}) suggest this system has been entrusted with {}.",
                        db_packages.to_string().yellow(), "storing precious data".green().bold());
                    println!("Countless queries have been executed within its digital walls.");
                }

                if dev_packages > 0 {
                    println!("Development tools ({}) indicate this is a {}, where code is crafted and compiled.",
                        dev_packages.to_string().yellow(), "builder's workshop".green().bold());
                }

                if web_packages == 0 && db_packages == 0 && dev_packages == 0 {
                    println!("This appears to be a {}, lean and purpose-built for specific tasks.",
                        "minimalist system".green().bold());
                }
                println!();
            }

            if let Ok(users) = ctx.guestfs.inspect_users(&ctx.root) {
                let regular_users: Vec<_> = users.iter().filter(|u| {
                    if let Ok(uid) = u.uid.parse::<u32>() {
                        uid >= 1000
                    } else {
                        false
                    }
                }).collect();

                if !regular_users.is_empty() {
                    println!("{} user accounts have called this system home, each leaving their unique imprint.",
                        regular_users.len().to_string().yellow());
                    println!("Their files and configurations tell tales of work accomplished and challenges overcome.");
                } else {
                    println!("This is a {}, without regular user accounts - a pure service machine.",
                        "sentinel system".green().bold());
                }
                println!();
            }

            println!("{}", "And so our system continues its journey, faithfully executing its duties,".italic());
            println!("{}", "waiting for its next chapter to be written...".italic());
            println!();
        }

        "security" => {
            println!("\n{} {}", "🔒".cyan(), "Security Narrative".yellow().bold());
            println!("{}", "═".repeat(70).cyan());
            println!();

            if let Ok(sec) = ctx.guestfs.inspect_security(&ctx.root) {
                println!("{}", "A Tale of Protection and Defense".green().bold());
                println!();

                // SELinux story
                if &sec.selinux != "disabled" {
                    println!("This system is guarded by the watchful eyes of {}, operating in {} mode.",
                        "SELinux".green().bold(), sec.selinux.yellow());
                    println!("Like a vigilant sentinel, it enforces mandatory access controls,");
                    println!("ensuring that every process stays within its designated boundaries.");
                } else {
                    println!("SELinux, the guardian of mandatory access controls, {} on this system.",
                        "stands silent".red());
                    println!("Its protective embrace has been forgone, for better or worse.");
                }
                println!();

                // Firewall story
                if let Ok(fw) = ctx.guestfs.inspect_firewall(&ctx.root) {
                    if fw.enabled {
                        println!("The {} stands as a mighty barrier, filtering network traffic",
                            fw.firewall_type.green().bold());
                        println!("with rules carefully crafted to protect against the outside world.");
                    } else {
                        println!("The firewall gates {}. This system trusts the network around it,",
                            "stand open".red());
                        println!("or perhaps operates within a protected enclave.");
                    }
                    println!();
                }

                // Audit story
                if sec.auditd {
                    println!("The {} chronicles every significant event,", "audit daemon".green().bold());
                    println!("maintaining detailed logs for forensic analysis and compliance.");
                    println!("Nothing escapes its watchful recording.");
                } else {
                    println!("No audit daemon watches and records. Events pass by {},",
                        "unchronicled".red());
                    println!("leaving no detailed trail for future investigators.");
                }
                println!();

                println!("{}", "Thus the security posture is revealed - a balance between".italic());
                println!("{}", "protection and accessibility, security and convenience.".italic());
                println!();
            }
        }

        "config" => {
            println!("\n{} {}", "⚙".cyan(), "Configuration Journey".yellow().bold());
            println!("{}", "═".repeat(70).cyan());
            println!();

            println!("{}", "The Journey of System Configuration".green().bold());
            println!();

            // Network configuration
            println!("{}", "Chapter 1: Connectivity".yellow());
            if let Ok(interfaces) = ctx.guestfs.inspect_network(&ctx.root) {
                println!("The system was blessed with {} network interfaces, each a gateway to communication.",
                    interfaces.len().to_string().green());
                for iface in interfaces.iter().take(3) {
                    println!("  • {} - a conduit for data flow", iface.name.cyan());
                }
            }
            println!();

            // Storage configuration
            println!("{}", "Chapter 2: Storage".yellow());
            if let Ok(devices) = ctx.guestfs.list_devices() {
                println!("Storage was provisioned across {} devices, the foundation of persistent data.",
                    devices.len().to_string().green());
            }
            if ctx.guestfs.exists("/etc/fstab").unwrap_or(false) {
                println!("The sacred {} defines how these storage realms are mounted,", "/etc/fstab".cyan());
                println!("a map for the system to understand its storage landscape.");
            }
            println!();

            // Services
            println!("{}", "Chapter 3: Services and Daemons".yellow());
            if let Ok(services) = ctx.guestfs.inspect_systemd_services(&ctx.root) {
                let enabled = services.iter().filter(|s| s.enabled).count();
                println!("Of {} services defined, {} were chosen to run at startup,",
                    services.len().to_string().green(),
                    enabled.to_string().yellow());
                println!("each playing its role in the system's daily operations.");
            }
            println!();

            println!("{}", "And thus the system was configured, piece by piece,".italic());
            println!("{}", "each setting a deliberate choice in its creation.".italic());
            println!();
        }

        "timeline" => {
            println!("\n{} {}", "⏰".cyan(), "System Timeline".yellow().bold());
            println!("{}", "═".repeat(70).cyan());
            println!();

            println!("{}", "A Chronicle of Recent Events".green().bold());
            println!();

            println!("{}", "In recent times...".italic());
            println!();

            // Check /etc modifications
            if let Ok(files) = ctx.guestfs.find("/etc") {
                let etc_files: Vec<_> = files.into_iter().take(5).collect();
                println!("Configuration files in /etc have been touched and modified,");
                println!("administrators shaping the system's behavior through careful edits.");
                for file in etc_files {
                    if ctx.guestfs.is_file(&file).unwrap_or(false) {
                        println!("  • {}", file.bright_black());
                    }
                }
            }
            println!();

            // Check logs
            if ctx.guestfs.is_dir("/var/log").unwrap_or(false) {
                println!("The {} directory continues to grow, chronicling system events,",
                    "/var/log".cyan());
                println!("errors encountered, and successes achieved.");
                println!("Each log file a diary entry in the system's ongoing story.");
            }
            println!();

            println!("{}", "The system's journey continues, writing new chapters daily...".italic());
            println!();
        }

        _ => {
            println!("{} Unknown story topic: {}", "Error:".red(), topic);
            println!("{} story <topic>", "Usage:".yellow());
            return Ok(());
        }
    }

    Ok(())
}

/// Interactive advisor system
pub fn cmd_advisor(_ctx: &ShellContext, args: &[&str]) -> Result<()> {
    if args.is_empty() {
        println!("\n{}", "╔═══════════════════════════════════════════════════════════╗".cyan().bold());
        println!("{}", "║                  System Advisor                          ║".cyan().bold());
        println!("{}", "╚═══════════════════════════════════════════════════════════╝".cyan().bold());
        println!();

        println!("{}", "Ask the Advisor:".yellow().bold());
        println!("{}", "─".repeat(70).cyan());
        println!();

        let questions = vec![
            ("secure", "How can I improve security?"),
            ("performance", "How can I optimize performance?"),
            ("troubleshoot", "How do I troubleshoot issues?"),
            ("backup", "What backup strategy should I use?"),
            ("monitoring", "How should I monitor this system?"),
            ("upgrade", "How do I plan for upgrades?"),
            ("compliance", "How do I achieve compliance?"),
            ("migration", "How do I prepare for migration?"),
        ];

        for (cmd, question) in questions {
            println!("  {} {}", cmd.green().bold(), question.bright_black());
        }

        println!();
        println!("{} advisor <question>", "Usage:".yellow());
        println!("{} advisor secure", "Example:".cyan());
        println!();
        return Ok(());
    }

    let question = args[0];

    match question {
        "secure" => {
            println!("\n{} {}", "🛡".cyan(), "Security Improvement Advice".yellow().bold());
            println!("{}", "═".repeat(70).cyan());
            println!();

            println!("{}", "Step 1: Assess Current State".green().bold());
            println!("  Run: {}", "wizard security".cyan());
            println!("  This gives you a security score and identifies gaps.\n");

            println!("{}", "Step 2: Enable Core Security Features".green().bold());
            println!("  • SELinux or AppArmor - Mandatory access control");
            println!("  • Firewall - Network filtering (iptables/firewalld)");
            println!("  • auditd - Security event logging");
            println!("  Check with: {}\n", "security".cyan());

            println!("{}", "Step 3: Harden User Access".green().bold());
            println!("  • Review user accounts: {}", "users".cyan());
            println!("  • Check sudo privileges: {}", "cat /etc/sudoers".cyan());
            println!("  • Strengthen SSH: {}", "cat /etc/ssh/sshd_config".cyan());
            println!("  • Disable unnecessary accounts\n");

            println!("{}", "Step 4: Minimize Attack Surface".green().bold());
            println!("  • Disable unnecessary services: {}", "services".cyan());
            println!("  • Remove unused packages: {}", "packages".cyan());
            println!("  • Close unused network ports\n");

            println!("{}", "Step 5: Implement Monitoring".green().bold());
            println!("  • Enable intrusion detection (fail2ban, AIDE)");
            println!("  • Set up log monitoring");
            println!("  • Configure alerting\n");

            println!("{}", "Step 6: Validate".green().bold());
            println!("  Run: {}", "scan security".cyan());
            println!("  Then: {}", "recommend".cyan());
            println!();

            println!("{} Use {} for a complete security workflow",
                "💡".yellow(), "auto run security-audit".cyan());
            println!();
        }

        "performance" => {
            println!("\n{} {}", "⚡".cyan(), "Performance Optimization Advice".yellow().bold());
            println!("{}", "═".repeat(70).cyan());
            println!();

            println!("{}", "Performance Tuning Strategy:".green().bold());
            println!();

            println!("{}", "1. Benchmark Current Performance".yellow());
            println!("  • Run: {}", "bench all".cyan());
            println!("  • Identify bottlenecks\n");

            println!("{}", "2. Optimize Services".yellow());
            println!("  • Review enabled services: {}", "services".cyan());
            println!("  • Disable unnecessary startup services");
            println!("  • Reduce service footprint\n");

            println!("{}", "3. Storage Optimization".yellow());
            println!("  • Review mount options: {}", "cat /etc/fstab".cyan());
            println!("  • Consider: noatime, barrier=0 (if safe)");
            println!("  • Check filesystem type efficiency\n");

            println!("{}", "4. Reduce Package Overhead".yellow());
            println!("  • Remove unused packages: {}", "packages".cyan());
            println!("  • Fewer packages = smaller footprint\n");

            println!("{}", "5. Network Tuning".yellow());
            println!("  • Review network configuration: {}", "network".cyan());
            println!("  • Optimize TCP/IP stack parameters");
            println!("  • Adjust buffer sizes\n");

            println!("{}", "6. Kernel Parameters".yellow());
            println!("  • Review: {}", "inspect kernel".cyan());
            println!("  • Tune /etc/sysctl.conf");
            println!("  • Load only necessary modules\n");

            println!("{} Start with: {}", "💡".yellow(), "focus performance".cyan());
            println!();
        }

        "troubleshoot" => {
            println!("\n{} {}", "🔧".cyan(), "Troubleshooting Guide".yellow().bold());
            println!("{}", "═".repeat(70).cyan());
            println!();

            println!("{}", "Systematic Troubleshooting Approach:".green().bold());
            println!();

            println!("{}", "Phase 1: Gather Information".yellow());
            println!("  • System overview: {}", "dashboard".cyan());
            println!("  • Check health: {}", "wizard health".cyan());
            println!("  • Review configuration: {}", "info".cyan());
            println!();

            println!("{}", "Phase 2: Identify Issues".yellow());
            println!("  • Scan for problems: {}", "scan issues".cyan());
            println!("  • Search error logs: {}", "search error --content --path /var/log".cyan());
            println!("  • Review recent changes: {}", "recent /etc 50".cyan());
            println!();

            println!("{}", "Phase 3: Isolate the Problem".yellow());
            println!("  • Focus on specific areas: {}", "focus <aspect>".cyan());
            println!("  • Inspect components: {}", "inspect <component>".cyan());
            println!("  • Check dependencies\n");

            println!("{}", "Phase 4: Research Solution".yellow());
            println!("  • Get recommendations: {}", "recommend".cyan());
            println!("  • Check playbooks: {}", "playbook".cyan());
            println!("  • Use context help: {}", "context".cyan());
            println!();

            println!("{}", "Phase 5: Document Findings".yellow());
            println!("  • Create snapshot: {}", "snapshot troubleshooting.md".cyan());
            println!("  • Export evidence: {}", "batch export /tmp/evidence".cyan());
            println!();

            println!("{} For systematic investigation: {}", "💡".yellow(), "playbook forensics".cyan());
            println!();
        }

        "backup" => {
            println!("\n{} {}", "💾".cyan(), "Backup Strategy Advice".yellow().bold());
            println!("{}", "═".repeat(70).cyan());
            println!();

            println!("{}", "Comprehensive Backup Strategy:".green().bold());
            println!();

            println!("{}", "1. Document Current State".yellow());
            println!("  • Create snapshot: {}", "snapshot pre-backup.md".cyan());
            println!("  • Export configurations: {}", "export system json config.json".cyan());
            println!("  • List packages: {}", "export packages csv packages.csv".cyan());
            println!("  • Export users: {}", "export users csv users.csv".cyan());
            println!();

            println!("{}", "2. Identify Critical Data".yellow());
            println!("  • Configuration files in /etc");
            println!("  • User data in /home");
            println!("  • Application data in /var");
            println!("  • Custom scripts and tools\n");

            println!("{}", "3. Backup Key Configurations".yellow());
            println!("  • Network: {}", "cat /etc/hosts /etc/resolv.conf".cyan());
            println!("  • Storage: {}", "cat /etc/fstab".cyan());
            println!("  • Services: {}", "export services md services.md".cyan());
            println!();

            println!("{}", "4. Regular Automation".yellow());
            println!("  • Schedule periodic snapshots");
            println!("  • Automated exports");
            println!("  • Version control for configs\n");

            println!("{}", "5. Test Recovery".yellow());
            println!("  • Verify backup integrity");
            println!("  • Practice restoration");
            println!("  • Document recovery procedures\n");

            println!("{} Quick backup: {}", "💡".yellow(), "auto run export-all".cyan());
            println!();
        }

        "monitoring" => {
            println!("\n{} {}", "📊".cyan(), "Monitoring Strategy".yellow().bold());
            println!("{}", "═".repeat(70).cyan());
            println!();

            println!("{}", "Effective System Monitoring:".green().bold());
            println!();

            println!("{}", "1. Security Monitoring".yellow());
            println!("  • Audit logs: Check auditd status");
            println!("  • Failed logins: Monitor /var/log/auth.log");
            println!("  • File integrity: Use AIDE or similar");
            println!("  • Firewall logs: Review firewall activity\n");

            println!("{}", "2. Performance Monitoring".yellow());
            println!("  • Service health: {}", "services".cyan());
            println!("  • Resource usage: CPU, memory, disk");
            println!("  • Network throughput\n");

            println!("{}", "3. Log Management".yellow());
            println!("  • Centralize logs");
            println!("  • Set retention policies");
            println!("  • Implement log rotation");
            println!("  • Check: {}", "inspect logging".cyan());
            println!();

            println!("{}", "4. Alerting".yellow());
            println!("  • Configure thresholds");
            println!("  • Set up notifications");
            println!("  • Define escalation paths\n");

            println!("{}", "5. Regular Reviews".yellow());
            println!("  • Weekly: {}", "wizard health".cyan());
            println!("  • Monthly: {}", "scan security".cyan());
            println!("  • Quarterly: Full audit\n");

            println!("{} Get current status: {}", "💡".yellow(), "dashboard".cyan());
            println!();
        }

        "upgrade" => {
            println!("\n{} {}", "⬆".cyan(), "Upgrade Planning Advice".yellow().bold());
            println!("{}", "═".repeat(70).cyan());
            println!();

            println!("{}", "Safe Upgrade Strategy:".green().bold());
            println!();

            println!("{}", "Phase 1: Pre-Upgrade Assessment".yellow());
            println!("  • Document current state: {}", "snapshot pre-upgrade.md".cyan());
            println!("  • Check compatibility");
            println!("  • Review release notes");
            println!("  • Export packages: {}", "export packages json".cyan());
            println!();

            println!("{}", "Phase 2: Dependency Analysis".yellow());
            println!("  • Review package dependencies");
            println!("  • Check service dependencies: {}", "services".cyan());
            println!("  • Identify potential conflicts\n");

            println!("{}", "Phase 3: Backup Everything".yellow());
            println!("  • Full system backup");
            println!("  • Configuration exports: {}", "auto run export-all".cyan());
            println!("  • Test backup restoration\n");

            println!("{}", "Phase 4: Test Upgrade".yellow());
            println!("  • Use test environment first");
            println!("  • Validate functionality");
            println!("  • Performance testing: {}", "bench all".cyan());
            println!();

            println!("{}", "Phase 5: Production Upgrade".yellow());
            println!("  • Schedule maintenance window");
            println!("  • Execute upgrade");
            println!("  • Validate: {}", "wizard health".cyan());
            println!();

            println!("{}", "Phase 6: Post-Upgrade".yellow());
            println!("  • Verify services: {}", "services".cyan());
            println!("  • Check security: {}", "security".cyan());
            println!("  • Create snapshot: {}", "snapshot post-upgrade.md".cyan());
            println!();
        }

        "compliance" => {
            println!("\n{} {}", "📋".cyan(), "Compliance Achievement Guide".yellow().bold());
            println!("{}", "═".repeat(70).cyan());
            println!();

            println!("{}", "Path to Compliance:".green().bold());
            println!();

            println!("{}", "1. Understand Requirements".yellow());
            println!("  • Identify applicable standards (PCI-DSS, HIPAA, etc.)");
            println!("  • Document requirements");
            println!("  • Map to controls\n");

            println!("{}", "2. Current State Assessment".yellow());
            println!("  • Run audit checklist: {}", "playbook audit".cyan());
            println!("  • Security assessment: {}", "wizard security".cyan());
            println!("  • Document gaps\n");

            println!("{}", "3. Implement Controls".yellow());
            println!("  • Security hardening: {}", "playbook hardening".cyan());
            println!("  • Access controls: Review {}", "users".cyan());
            println!("  • Audit logging: Enable auditd");
            println!("  • Network security: Configure firewall\n");

            println!("{}", "4. Documentation".yellow());
            println!("  • System documentation: {}", "snapshot compliance-docs.md".cyan());
            println!("  • Configuration records");
            println!("  • Change management logs\n");

            println!("{}", "5. Validation".yellow());
            println!("  • Self-assessment: {}", "scan security".cyan());
            println!("  • Generate reports: {}", "report compliance".cyan());
            println!("  • Third-party audit\n");

            println!("{}", "6. Continuous Compliance".yellow());
            println!("  • Regular reviews");
            println!("  • Automated scanning");
            println!("  • Ongoing documentation\n");

            println!("{} Start here: {}", "💡".yellow(), "playbook audit".cyan());
            println!();
        }

        "migration" => {
            println!("\n{} {}", "🚀".cyan(), "Migration Preparation Guide".yellow().bold());
            println!("{}", "═".repeat(70).cyan());
            println!();

            println!("{}", "Complete Migration Strategy:".green().bold());
            println!();

            println!("{}", "Step 1: Discovery & Documentation".yellow());
            println!("  • Full system analysis: {}", "auto run full-analysis".cyan());
            println!("  • Detect purpose: {}", "profile detect".cyan());
            println!("  • Create baseline: {}", "snapshot pre-migration.md".cyan());
            println!();

            println!("{}", "Step 2: Dependency Mapping".yellow());
            println!("  • Services: {}", "export services csv".cyan());
            println!("  • Packages: {}", "export packages csv".cyan());
            println!("  • Network: {}", "discover network".cyan());
            println!("  • Users: {}", "export users csv".cyan());
            println!();

            println!("{}", "Step 3: Configuration Export".yellow());
            println!("  • Export all data: {}", "auto run export-all".cyan());
            println!("  • Document customizations");
            println!("  • Backup critical files\n");

            println!("{}", "Step 4: Planning".yellow());
            println!("  • Use migration playbook: {}", "playbook migration".cyan());
            println!("  • Define cutover plan");
            println!("  • Identify risks\n");

            println!("{}", "Step 5: Testing".yellow());
            println!("  • Build target environment");
            println!("  • Migrate test data");
            println!("  • Validate functionality\n");

            println!("{}", "Step 6: Execution & Validation".yellow());
            println!("  • Execute migration");
            println!("  • Post-migration verification");
            println!("  • Performance check: {}", "bench all".cyan());
            println!();

            println!("{} Complete workflow: {}", "💡".yellow(), "playbook migration".cyan());
            println!();
        }

        _ => {
            println!("{} Unknown question: {}", "Error:".red(), question);
            println!("{} advisor", "Usage:".yellow());
            return Ok(());
        }
    }

    Ok(())
}

/// System verification and validation
pub fn cmd_verify(ctx: &mut ShellContext, args: &[&str]) -> Result<()> {
    if args.is_empty() {
        println!("\n{}", "Usage: verify <check>".red());
        println!();
        println!("{}", "Available Verifications:".yellow().bold());
        println!("  {} - Verify system integrity", "integrity".green());
        println!("  {} - Verify security configuration", "security".green());
        println!("  {} - Verify boot configuration", "boot".green());
        println!("  {} - Verify network setup", "network".green());
        println!("  {} - Run all verifications", "all".green());
        println!();
        return Ok(());
    }

    let check = args[0];

    match check {
        "integrity" => {
            println!("\n{} {}", "✓".cyan(), "System Integrity Verification".yellow().bold());
            println!("{}", "═".repeat(70).cyan());
            println!();

            let mut passed = 0;
            let mut failed = 0;
            let mut warnings = 0;

            println!("{}", "Critical System Files:".green().bold());
            let critical_files = vec![
                ("/etc/passwd", "User account database", true),
                ("/etc/shadow", "Password hashes", true),
                ("/etc/group", "Group definitions", true),
                ("/etc/fstab", "Filesystem mount table", true),
                ("/etc/hosts", "Host name resolution", false),
                ("/etc/resolv.conf", "DNS configuration", false),
                ("/boot/grub/grub.cfg", "Boot configuration", false),
                ("/boot/grub2/grub.cfg", "Boot configuration (grub2)", false),
            ];

            for (file, desc, critical) in critical_files {
                if ctx.guestfs.exists(file).unwrap_or(false) {
                    println!("  {} {} - {}", "✓".green(), file.cyan(), desc.bright_black());
                    passed += 1;
                } else {
                    if critical {
                        println!("  {} {} - {} {}", "✗".red(), file.cyan(), desc.bright_black(), "[CRITICAL]".red().bold());
                        failed += 1;
                    } else {
                        println!("  {} {} - {} {}", "⚠".yellow(), file.bright_black(), desc.bright_black(), "[OPTIONAL]".yellow());
                        warnings += 1;
                    }
                }
            }
            println!();

            println!("{}", "Results:".green().bold());
            println!("  Passed:   {}", passed.to_string().green());
            if warnings > 0 {
                println!("  Warnings: {}", warnings.to_string().yellow());
            }
            if failed > 0 {
                println!("  Failed:   {}", failed.to_string().red().bold());
            }
            println!();

            if failed == 0 && warnings == 0 {
                println!("{} System integrity: {}", "✓".green().bold(), "EXCELLENT".green().bold());
            } else if failed == 0 {
                println!("{} System integrity: {} ({} warnings)",
                    "✓".green(), "GOOD".green(), warnings.to_string().yellow());
            } else {
                println!("{} System integrity: {} ({} critical failures)",
                    "✗".red().bold(), "POOR".red().bold(), failed.to_string().red());
            }
            println!();
        }

        "security" => {
            println!("\n{} {}", "🔒".cyan(), "Security Configuration Verification".yellow().bold());
            println!("{}", "═".repeat(70).cyan());
            println!();

            if let Ok(sec) = ctx.guestfs.inspect_security(&ctx.root) {
                let mut score = 0;
                let mut max_score = 0;

                println!("{}", "Security Features:".green().bold());

                // SELinux
                max_score += 25;
                if &sec.selinux != "disabled" {
                    println!("  {} SELinux: {} {}", "✓".green(), sec.selinux.green(), "[25 points]".bright_black());
                    score += 25;
                } else {
                    println!("  {} SELinux: {} {}", "✗".red(), "disabled".red(), "[0/25 points]".bright_black());
                }

                // AppArmor
                max_score += 25;
                if sec.apparmor {
                    println!("  {} AppArmor: {} {}", "✓".green(), "enabled".green(), "[25 points]".bright_black());
                    score += 25;
                } else {
                    println!("  {} AppArmor: {} {}", "✗".red(), "disabled".red(), "[0/25 points]".bright_black());
                }

                // Firewall
                max_score += 25;
                if let Ok(fw) = ctx.guestfs.inspect_firewall(&ctx.root) {
                    if fw.enabled {
                        println!("  {} Firewall: {} ({}) {}", "✓".green(), "enabled".green(), fw.firewall_type, "[25 points]".bright_black());
                        score += 25;
                    } else {
                        println!("  {} Firewall: {} {}", "✗".red(), "disabled".red(), "[0/25 points]".bright_black());
                    }
                }

                // Auditd
                max_score += 25;
                if sec.auditd {
                    println!("  {} Auditd: {} {}", "✓".green(), "enabled".green(), "[25 points]".bright_black());
                    score += 25;
                } else {
                    println!("  {} Auditd: {} {}", "✗".red(), "disabled".red(), "[0/25 points]".bright_black());
                }

                println!();
                println!("{}", "Security Score:".green().bold());
                println!("  {}/{} points ({}%)",
                    score.to_string().yellow(),
                    max_score,
                    ((score as f64 / max_score as f64) * 100.0) as i32
                );

                let grade = if score >= 80 {
                    "A (Excellent)".green().bold()
                } else if score >= 60 {
                    "B (Good)".green()
                } else if score >= 40 {
                    "C (Fair)".yellow()
                } else {
                    "D (Poor)".red().bold()
                };

                println!("  Grade: {}", grade);
                println!();

                println!("{} For detailed security analysis: {}", "💡".yellow(), "wizard security".cyan());
                println!();
            }
        }

        "boot" => {
            println!("\n{} {}", "🚀".cyan(), "Boot Configuration Verification".yellow().bold());
            println!("{}", "═".repeat(70).cyan());
            println!();

            let mut issues = Vec::new();

            println!("{}", "Boot Components:".green().bold());

            // Check fstab
            if ctx.guestfs.exists("/etc/fstab")? {
                println!("  {} /etc/fstab present", "✓".green());
            } else {
                println!("  {} /etc/fstab missing", "✗".red());
                issues.push("Missing /etc/fstab");
            }

            // Check grub
            let grub_found = ctx.guestfs.exists("/boot/grub/grub.cfg").unwrap_or(false)
                || ctx.guestfs.exists("/boot/grub2/grub.cfg").unwrap_or(false);

            if grub_found {
                println!("  {} GRUB configuration present", "✓".green());
            } else {
                println!("  {} GRUB configuration not found", "⚠".yellow());
                issues.push("No GRUB configuration found");
            }

            // Check boot directory
            if ctx.guestfs.is_dir("/boot").unwrap_or(false) {
                println!("  {} /boot directory present", "✓".green());
            } else {
                println!("  {} /boot directory missing", "✗".red());
                issues.push("Missing /boot directory");
            }

            println!();

            if issues.is_empty() {
                println!("{} Boot configuration: {}", "✓".green().bold(), "VALID".green().bold());
            } else {
                println!("{} Boot configuration: {} ({} issues)",
                    "⚠".yellow(), "WARNING".yellow(), issues.len());
                println!();
                println!("{}", "Issues:".yellow());
                for issue in issues {
                    println!("  • {}", issue.red());
                }
            }
            println!();

            println!("{} For detailed inspection: {}", "💡".yellow(), "inspect boot".cyan());
            println!();
        }

        "network" => {
            println!("\n{} {}", "🌐".cyan(), "Network Configuration Verification".yellow().bold());
            println!("{}", "═".repeat(70).cyan());
            println!();

            let mut checks = 0;
            let mut passed = 0;

            println!("{}", "Network Configuration:".green().bold());

            // Check interfaces
            checks += 1;
            if let Ok(interfaces) = ctx.guestfs.inspect_network(&ctx.root) {
                if !interfaces.is_empty() {
                    println!("  {} {} network interfaces configured",
                        "✓".green(), interfaces.len().to_string().yellow());
                    passed += 1;
                } else {
                    println!("  {} No network interfaces configured", "⚠".yellow());
                }
            }

            // Check hosts file
            checks += 1;
            if ctx.guestfs.exists("/etc/hosts").unwrap_or(false) {
                println!("  {} /etc/hosts present", "✓".green());
                passed += 1;
            } else {
                println!("  {} /etc/hosts missing", "✗".red());
            }

            // Check DNS
            checks += 1;
            if ctx.guestfs.exists("/etc/resolv.conf").unwrap_or(false) {
                println!("  {} /etc/resolv.conf present", "✓".green());
                passed += 1;
            } else {
                println!("  {} /etc/resolv.conf missing", "⚠".yellow());
            }

            // Check hostname
            checks += 1;
            if let Ok(hostname) = ctx.guestfs.inspect_get_hostname(&ctx.root) {
                println!("  {} Hostname configured: {}", "✓".green(), hostname.yellow());
                passed += 1;
            } else {
                println!("  {} Hostname not configured", "⚠".yellow());
            }

            println!();
            println!("{} {}/{} checks passed", "Results:".green().bold(), passed, checks);
            println!();

            println!("{} For detailed analysis: {}", "💡".yellow(), "focus network".cyan());
            println!();
        }

        "all" => {
            println!("\n{} {}", "🔍".cyan(), "Complete System Verification".yellow().bold());
            println!("{}", "═".repeat(70).cyan());
            println!();

            println!("Running all verification checks...\n");

            println!("{}", "[1/4] Integrity Check".cyan());
            cmd_verify(ctx, &["integrity"])?;

            println!("{}", "[2/4] Security Check".cyan());
            cmd_verify(ctx, &["security"])?;

            println!("{}", "[3/4] Boot Check".cyan());
            cmd_verify(ctx, &["boot"])?;

            println!("{}", "[4/4] Network Check".cyan());
            cmd_verify(ctx, &["network"])?;

            println!("{}", "═".repeat(70).cyan());
            println!("{} Complete system verification finished", "✓".green().bold());
            println!();
        }

        _ => {
            println!("{} Unknown verification: {}", "Error:".red(), check);
            println!("{} verify <check>", "Usage:".yellow());
            return Ok(());
        }
    }

    Ok(())
}

/// Optimization recommendations
pub fn cmd_optimize(_ctx: &ShellContext, _args: &[&str]) -> Result<()> {
    println!("\n{}", "╔═══════════════════════════════════════════════════════════╗".cyan().bold());
    println!("{}", "║              Optimization Recommendations                ║".cyan().bold());
    println!("{}", "╚═══════════════════════════════════════════════════════════╝".cyan().bold());
    println!();

    println!("{}", "System Optimization Guide".yellow().bold());
    println!("{}", "─".repeat(70).cyan());
    println!();

    let categories = vec![
        ("Performance", vec![
            ("Disable unnecessary services", "services -> disable unused", "Medium"),
            ("Remove unused packages", "packages -> uninstall unused", "Low"),
            ("Optimize mount options", "cat /etc/fstab -> add noatime", "Medium"),
            ("Tune kernel parameters", "/etc/sysctl.conf tuning", "High"),
        ]),
        ("Security", vec![
            ("Enable SELinux/AppArmor", "Mandatory access control", "High"),
            ("Configure firewall", "Network filtering", "High"),
            ("Enable audit logging", "auditd configuration", "Medium"),
            ("Harden SSH", "/etc/ssh/sshd_config", "Medium"),
        ]),
        ("Storage", vec![
            ("Clean log files", "Log rotation and cleanup", "Low"),
            ("Remove old kernels", "Keep only recent kernels", "Low"),
            ("Optimize filesystem", "Choice of fs type", "Medium"),
        ]),
        ("Network", vec![
            ("Optimize TCP/IP stack", "sysctl network tuning", "Medium"),
            ("Configure DNS properly", "/etc/resolv.conf", "Low"),
            ("Use connection pooling", "For applications", "Medium"),
        ]),
    ];

    for (category, optimizations) in categories {
        println!("{}", format!("{}:", category).green().bold());
        for (name, action, impact) in optimizations {
            let impact_colored = match impact {
                "High" => impact.red().bold(),
                "Medium" => impact.yellow(),
                _ => impact.green(),
            };
            println!("  {} {} - {} {}",
                "•".cyan(),
                name,
                action.bright_black(),
                format!("[{}]", impact_colored)
            );
        }
        println!();
    }

    println!("{}", "Getting Started:".yellow().bold());
    println!("  • {} - Performance analysis", "focus performance".cyan());
    println!("  • {} - Security improvements", "advisor secure".cyan());
    println!("  • {} - Full system analysis", "auto run full-analysis".cyan());
    println!();

    Ok(())
}

/// Improvement roadmap generator
pub fn cmd_roadmap(_ctx: &mut ShellContext, args: &[&str]) -> Result<()> {
    let timeframe = if args.is_empty() { "30-day" } else { args[0] };

    println!("\n{} {}", "🗺".cyan(), format!("{} Improvement Roadmap", timeframe.to_uppercase()).yellow().bold());
    println!("{}", "═".repeat(70).cyan());
    println!();

    match timeframe {
        "30-day" | "short" => {
            println!("{} (Priority: Quick Wins)", "30-Day Roadmap".green().bold());
            println!();

            println!("{} Week 1: Assessment", "📅".yellow());
            println!("  • Run: {}", "auto run full-analysis".cyan());
            println!("  • Run: {}", "wizard security".cyan());
            println!("  • Run: {}", "wizard health".cyan());
            println!("  • Document baseline: {}", "snapshot baseline.md".cyan());
            println!();

            println!("{} Week 2: Quick Security Fixes", "📅".yellow());
            println!("  • Enable missing security features");
            println!("  • Remove unnecessary user accounts: {}", "users".cyan());
            println!("  • Disable unused services: {}", "services".cyan());
            println!("  • Verify: {}", "verify security".cyan());
            println!();

            println!("{} Week 3: Performance Tuning", "📅".yellow());
            println!("  • Benchmark: {}", "bench all".cyan());
            println!("  • Remove unused packages: {}", "packages".cyan());
            println!("  • Optimize startup services");
            println!("  • Test improvements");
            println!();

            println!("{} Week 4: Documentation & Validation", "📅".yellow());
            println!("  • Create documentation: {}", "auto run documentation".cyan());
            println!("  • Run all verifications: {}", "verify all".cyan());
            println!("  • Generate reports: {}", "report executive".cyan());
            println!("  • Archive baseline for future comparison");
            println!();
        }

        "90-day" | "medium" => {
            println!("{} (Priority: Substantial Improvements)", "90-Day Roadmap".green().bold());
            println!();

            println!("{} Month 1: Foundation", "📅".yellow());
            println!("  • Complete 30-day roadmap");
            println!("  • Establish monitoring");
            println!("  • Implement backup strategy: {}", "advisor backup".cyan());
            println!();

            println!("{} Month 2: Security Hardening", "📅".yellow());
            println!("  • Follow hardening playbook: {}", "playbook hardening".cyan());
            println!("  • Implement intrusion detection");
            println!("  • Configure log centralization");
            println!("  • Security scan: {}", "scan security".cyan());
            println!();

            println!("{} Month 3: Optimization & Compliance", "📅".yellow());
            println!("  • Performance optimization: {}", "advisor performance".cyan());
            println!("  • Compliance assessment: {}", "playbook audit".cyan());
            println!("  • Automated monitoring setup");
            println!("  • Final validation: {}", "verify all".cyan());
            println!();
        }

        "annual" | "long" => {
            println!("{} (Priority: Strategic Transformation)", "Annual Roadmap".green().bold());
            println!();

            println!("{} Q1: Assessment & Planning", "📅".yellow());
            println!("  • Complete current state analysis");
            println!("  • Define target state");
            println!("  • Create detailed project plan");
            println!("  • Stakeholder alignment");
            println!();

            println!("{} Q2: Security & Compliance", "📅".yellow());
            println!("  • Complete security hardening");
            println!("  • Achieve compliance: {}", "advisor compliance".cyan());
            println!("  • Implement monitoring");
            println!("  • Staff training");
            println!();

            println!("{} Q3: Optimization & Automation", "📅".yellow());
            println!("  • Performance optimization");
            println!("  • Automation implementation");
            println!("  • Disaster recovery setup");
            println!("  • Documentation: {}", "auto run documentation".cyan());
            println!();

            println!("{} Q4: Migration & Modernization", "📅".yellow());
            println!("  • Migration planning: {}", "playbook migration".cyan());
            println!("  • Infrastructure modernization");
            println!("  • Continuous improvement process");
            println!("  • Year-end review and reporting");
            println!();
        }

        _ => {
            println!("{} Unknown timeframe: {}", "Error:".red(), timeframe);
            println!("{}", "Available timeframes: 30-day, 90-day, annual".yellow());
            return Ok(());
        }
    }

    println!("{}", "Key Success Metrics:".green().bold());
    println!("  • Security score improvement: Track with {}", "wizard security".cyan());
    println!("  • Health score improvement: Track with {}", "wizard health".cyan());
    println!("  • Performance gains: Measure with {}", "bench all".cyan());
    println!("  • Compliance status: Verify with {}", "verify all".cyan());
    println!();

    println!("{} Start now: {}", "💡".yellow(), "verify all".cyan());
    println!();

    Ok(())
}

/// AI-like intelligent insights
pub fn cmd_insights(ctx: &mut ShellContext, _args: &[&str]) -> Result<()> {
    println!("\n{}", "╔═══════════════════════════════════════════════════════════╗".cyan().bold());
    println!("{}", "║              Intelligent System Insights                ║".cyan().bold());
    println!("{}", "╚═══════════════════════════════════════════════════════════╝".cyan().bold());
    println!();

    println!("{}", "Analyzing system patterns...".yellow());
    println!();

    let mut insights = Vec::new();

    // Analyze packages
    if let Ok(pkg_info) = ctx.guestfs.inspect_packages(&ctx.root) {
        let pkg_count = pkg_info.packages.len();

        if pkg_count > 1500 {
            insights.push((
                "📦",
                "Package Density",
                format!("{} packages detected - This is a feature-rich system", pkg_count),
                "Consider reviewing with 'packages' to identify unused packages".to_string(),
                "Medium"
            ));
        } else if pkg_count < 300 {
            insights.push((
                "📦",
                "Minimal Footprint",
                format!("{} packages - This is a lean, focused system", pkg_count),
                "Minimal attack surface, good for security".to_string(),
                "Info"
            ));
        }

        // Detect development environment
        let dev_packages = pkg_info.packages.iter().filter(|p| {
            p.name.contains("gcc") || p.name.contains("make") ||
            p.name.contains("git") || p.name.contains("devel")
        }).count();

        if dev_packages > 20 {
            insights.push((
                "💻",
                "Development Environment",
                format!("{} development packages found", dev_packages),
                "This appears to be a build/development system - ensure build tools are up to date".to_string(),
                "Info"
            ));
        }

        // Check for multiple web servers
        let web_servers = pkg_info.packages.iter().filter(|p| {
            p.name.contains("httpd") || p.name.contains("nginx") || p.name.contains("apache")
        }).count();

        if web_servers > 1 {
            insights.push((
                "⚠️",
                "Multiple Web Servers",
                format!("{} different web server packages detected", web_servers),
                "Multiple web servers can cause port conflicts - verify only one is enabled".to_string(),
                "Warning"
            ));
        }
    }

    // Analyze security
    if let Ok(sec) = ctx.guestfs.inspect_security(&ctx.root) {
        let mut security_score = 0;
        let mut security_features = Vec::new();

        if &sec.selinux != "disabled" {
            security_score += 1;
            security_features.push("SELinux");
        }
        if sec.apparmor {
            security_score += 1;
            security_features.push("AppArmor");
        }
        if sec.auditd {
            security_score += 1;
            security_features.push("Auditd");
        }

        if let Ok(fw) = ctx.guestfs.inspect_firewall(&ctx.root) {
            if fw.enabled {
                security_score += 1;
                security_features.push("Firewall");
            }
        }

        if security_score >= 3 {
            insights.push((
                "🛡️",
                "Strong Security Posture",
                format!("{} security features active: {}", security_score, security_features.join(", ")),
                "Well-configured security - maintain with regular updates".to_string(),
                "Good"
            ));
        } else if security_score <= 1 {
            insights.push((
                "🚨",
                "Weak Security Posture",
                format!("Only {} security features active", security_score),
                "Critical: Enable SELinux/AppArmor and firewall - run 'advisor secure'".to_string(),
                "Critical"
            ));
        }
    }

    // Analyze users
    if let Ok(users) = ctx.guestfs.inspect_users(&ctx.root) {
        let root_users = users.iter().filter(|u| u.uid == "0").count();
        let regular_users = users.iter().filter(|u| {
            if let Ok(uid) = u.uid.parse::<u32>() {
                uid >= 1000
            } else {
                false
            }
        }).count();

        if root_users > 1 {
            insights.push((
                "⚠️",
                "Multiple Root Accounts",
                format!("{} accounts with UID 0 detected", root_users),
                "Security risk: Review root accounts immediately with 'users'".to_string(),
                "High"
            ));
        }

        if regular_users == 0 {
            insights.push((
                "🤖",
                "Service-Only System",
                "No regular user accounts detected".to_string(),
                "This is a dedicated service system - appropriate for containers/VMs".to_string(),
                "Info"
            ));
        } else if regular_users > 10 {
            insights.push((
                "👥",
                "Multi-User Environment",
                format!("{} regular user accounts", regular_users),
                "Review user access regularly for security - 'users' command".to_string(),
                "Info"
            ));
        }
    }

    // Analyze services
    if let Ok(services) = ctx.guestfs.inspect_systemd_services(&ctx.root) {
        let enabled = services.iter().filter(|s| s.enabled).count();
        let total = services.len();
        let ratio = (enabled as f64 / total as f64) * 100.0;

        if ratio > 70.0 {
            insights.push((
                "⚙️",
                "High Service Density",
                format!("{:.0}% of services enabled ({}/{})", ratio, enabled, total),
                "Many services running - review with 'services' to disable unused ones".to_string(),
                "Medium"
            ));
        } else if ratio < 30.0 {
            insights.push((
                "⚙️",
                "Selective Service Configuration",
                format!("Only {:.0}% of services enabled", ratio),
                "Conservative service configuration - good for security and performance".to_string(),
                "Good"
            ));
        }
    }

    // Display insights
    if insights.is_empty() {
        println!("{}", "No significant patterns detected.".bright_black());
        println!("System appears to be in a standard configuration.");
    } else {
        println!("{} ({} insights)", "Key Insights:".green().bold(), insights.len());
        println!("{}", "─".repeat(70).cyan());
        println!();

        for (icon, title, description, recommendation, priority) in insights {
            let priority_colored = match priority {
                "Critical" => priority.red().bold(),
                "High" => priority.red(),
                "Warning" => priority.yellow().bold(),
                "Medium" => priority.yellow(),
                "Good" => priority.green(),
                _ => priority.cyan(),
            };

            println!("{} {} {}", icon, title.bold(), format!("[{}]", priority_colored));
            println!("  {}", description);
            println!("  {} {}", "→".cyan(), recommendation.bright_black());
            println!();
        }
    }

    println!("{} Next Steps:", "💡".yellow());
    println!("  • {}", "verify all - Comprehensive validation".cyan());
    println!("  • {}", "advisor secure - Security improvements".cyan());
    println!("  • {}", "optimize - Optimization recommendations".cyan());
    println!();

    Ok(())
}

/// Interactive diagnostic doctor
pub fn cmd_doctor(ctx: &mut ShellContext, _args: &[&str]) -> Result<()> {
    println!("\n{}", "╔═══════════════════════════════════════════════════════════╗".cyan().bold());
    println!("{}", "║                  System Doctor                           ║".cyan().bold());
    println!("{}", "╚═══════════════════════════════════════════════════════════╝".cyan().bold());
    println!();

    println!("{}", "Running comprehensive system diagnostic...".yellow());
    println!();

    let mut health_score = 100;
    let mut issues = Vec::new();
    let mut warnings = Vec::new();
    let mut recommendations = Vec::new();

    // Check 1: Critical Files
    println!("{} Checking critical files...", "→".cyan());
    let critical_files = vec![
        ("/etc/passwd", "User database"),
        ("/etc/shadow", "Password hashes"),
        ("/etc/fstab", "Filesystem table"),
    ];

    for (file, desc) in &critical_files {
        if !ctx.guestfs.exists(file).unwrap_or(false) {
            health_score -= 20;
            issues.push(format!("Missing critical file: {} ({})", file, desc));
        }
    }

    // Check 2: Security Configuration
    println!("{} Checking security configuration...", "→".cyan());
    if let Ok(sec) = ctx.guestfs.inspect_security(&ctx.root) {
        if &sec.selinux == "disabled" {
            health_score -= 10;
            warnings.push("SELinux is disabled - mandatory access control not active");
            recommendations.push("Enable SELinux for enhanced security");
        }

        if !sec.apparmor && &sec.selinux == "disabled" {
            health_score -= 10;
            warnings.push("No MAC system active (neither SELinux nor AppArmor)");
        }

        if !sec.auditd {
            health_score -= 5;
            warnings.push("Audit daemon not running - no detailed event logging");
            recommendations.push("Enable auditd for security event tracking");
        }

        if let Ok(fw) = ctx.guestfs.inspect_firewall(&ctx.root) {
            if !fw.enabled {
                health_score -= 15;
                warnings.push("Firewall is disabled - no network filtering");
                recommendations.push("Enable and configure firewall immediately");
            }
        }
    }

    // Check 3: Boot Configuration
    println!("{} Checking boot configuration...", "→".cyan());
    if !ctx.guestfs.exists("/etc/fstab").unwrap_or(false) {
        health_score -= 15;
        issues.push("Missing /etc/fstab - system may not boot properly".to_string());
    }

    let grub_found = ctx.guestfs.exists("/boot/grub/grub.cfg").unwrap_or(false)
        || ctx.guestfs.exists("/boot/grub2/grub.cfg").unwrap_or(false);

    if !grub_found {
        health_score -= 10;
        warnings.push("No GRUB configuration found - boot loader may not be configured");
    }

    // Check 4: User Configuration
    println!("{} Checking user configuration...", "→".cyan());
    if let Ok(users) = ctx.guestfs.inspect_users(&ctx.root) {
        let root_users = users.iter().filter(|u| u.uid == "0").count();
        if root_users > 1 {
            health_score -= 15;
            issues.push(format!("Multiple root accounts detected ({})", root_users));
            recommendations.push("Audit root accounts and remove duplicates");
        }

        let locked_users = users.iter().filter(|u| u.shell.contains("nologin") || u.shell.contains("false")).count();
        if locked_users == users.len() && users.len() > 0 {
            warnings.push("All user accounts appear to be locked");
        }
    }

    // Check 5: Network Configuration
    println!("{} Checking network configuration...", "→".cyan());
    if !ctx.guestfs.exists("/etc/hosts").unwrap_or(false) {
        health_score -= 5;
        warnings.push("Missing /etc/hosts file");
    }

    if !ctx.guestfs.exists("/etc/resolv.conf").unwrap_or(false) {
        health_score -= 5;
        warnings.push("Missing /etc/resolv.conf - DNS may not be configured");
    }

    println!();
    println!("{}", "═".repeat(70).cyan());
    println!();

    // Display Results
    let health_grade = if health_score >= 90 {
        "A (Excellent)".green().bold()
    } else if health_score >= 75 {
        "B (Good)".green()
    } else if health_score >= 60 {
        "C (Fair)".yellow()
    } else if health_score >= 40 {
        "D (Poor)".red()
    } else {
        "F (Critical)".red().bold()
    };

    println!("{} {}/100 - Grade: {}", "Overall Health Score:".green().bold(), health_score, health_grade);
    println!();

    if !issues.is_empty() {
        println!("{} ({} found)", "Critical Issues:".red().bold(), issues.len());
        for issue in &issues {
            println!("  {} {}", "✗".red(), issue);
        }
        println!();
    }

    if !warnings.is_empty() {
        println!("{} ({} found)", "Warnings:".yellow().bold(), warnings.len());
        for warning in &warnings {
            println!("  {} {}", "⚠".yellow(), warning);
        }
        println!();
    }

    if !recommendations.is_empty() {
        println!("{} ({} items)", "Recommended Actions:".cyan().bold(), recommendations.len());
        for (i, rec) in recommendations.iter().enumerate() {
            println!("  {} {}", format!("{}.", i + 1).cyan(), rec);
        }
        println!();
    }

    if issues.is_empty() && warnings.is_empty() {
        println!("{} System is healthy! No critical issues detected.", "✓".green().bold());
        println!();
    }

    println!("{} Detailed Analysis:", "💡".yellow());
    println!("  • {}", "verify all - Run all verification checks".cyan());
    println!("  • {}", "wizard health - Interactive health assessment".cyan());
    println!("  • {}", "scan issues - Scan for specific problems".cyan());
    println!();

    Ok(())
}

/// Goal setting and tracking
pub fn cmd_goals(ctx: &mut ShellContext, args: &[&str]) -> Result<()> {
    if args.is_empty() {
        println!("\n{}", "╔═══════════════════════════════════════════════════════════╗".cyan().bold());
        println!("{}", "║              System Improvement Goals                    ║".cyan().bold());
        println!("{}", "╚═══════════════════════════════════════════════════════════╝".cyan().bold());
        println!();

        println!("{}", "Track your system improvement journey!".yellow());
        println!();

        println!("{}", "Available Commands:".green().bold());
        println!("  {} - Show suggested goals", "goals suggest".cyan());
        println!("  {} - Set a custom goal", "goals set <name>".cyan());
        println!("  {} - List all goals", "goals list".cyan());
        println!("  {} - Check goal status", "goals check <name>".cyan());
        println!();

        println!("{}", "Example Goals:".yellow());
        println!("  • Achieve security score of 80+");
        println!("  • Reduce enabled services by 20%");
        println!("  • Remove 100+ unused packages");
        println!("  • Enable all security features");
        println!("  • Document all configurations");
        println!();

        return Ok(());
    }

    let subcommand = args[0];

    match subcommand {
        "suggest" => {
            println!("\n{} {}", "🎯".cyan(), "Suggested Goals".yellow().bold());
            println!("{}", "═".repeat(70).cyan());
            println!();

            println!("{}", "Based on current system state:".green().bold());
            println!();

            // Security goals
            if let Ok(sec) = ctx.guestfs.inspect_security(&ctx.root) {
                let mut security_goals = 0;

                if &sec.selinux == "disabled" {
                    security_goals += 1;
                    println!("{} {} Enable SELinux", "1.".yellow(), "🔒".cyan());
                    println!("   Target: Activate mandatory access control");
                    println!("   Command: Check /etc/selinux/config");
                    println!();
                }

                if !sec.auditd {
                    security_goals += 1;
                    println!("{} {} Enable Audit Daemon", format!("{}.", security_goals + 1).yellow(), "📝".cyan());
                    println!("   Target: Start security event logging");
                    println!("   Verify: Run 'security' command");
                    println!();
                }

                if let Ok(fw) = ctx.guestfs.inspect_firewall(&ctx.root) {
                    if !fw.enabled {
                        security_goals += 1;
                        println!("{} {} Enable Firewall", format!("{}.", security_goals + 1).yellow(), "🛡️".cyan());
                        println!("   Target: Configure network filtering");
                        println!("   Verify: Run 'verify security'");
                        println!();
                    }
                }
            }

            // Performance goals
            if let Ok(services) = ctx.guestfs.inspect_systemd_services(&ctx.root) {
                let enabled = services.iter().filter(|s| s.enabled).count();
                if enabled > 50 {
                    println!("{} {} Optimize Services", "4.".yellow(), "⚙️".cyan());
                    println!("   Target: Reduce enabled services to <40");
                    println!("   Current: {} enabled", enabled);
                    println!("   Command: 'services' to review");
                    println!();
                }
            }

            // Documentation goals
            println!("{} {} Complete Documentation", "5.".yellow(), "📚".cyan());
            println!("   Target: Generate comprehensive system documentation");
            println!("   Command: 'auto run documentation'");
            println!();

            println!("{} Use {} to track progress", "💡".yellow(), "verify all".cyan());
            println!();
        }

        "list" => {
            println!("\n{} {}", "📋".cyan(), "Goal Tracking".yellow().bold());
            println!("{}", "═".repeat(70).cyan());
            println!();

            println!("{}", "Common Goals:".green().bold());

            let goals = vec![
                ("Security Excellence", "Achieve security score 90+", "verify security"),
                ("Performance Optimization", "Reduce service count by 25%", "services"),
                ("Compliance Ready", "Pass all audit checks", "playbook audit"),
                ("Documentation Complete", "Full system documentation", "auto run documentation"),
                ("Zero Critical Issues", "No critical findings", "doctor"),
            ];

            for (i, (name, target, cmd)) in goals.iter().enumerate() {
                println!("{}. {} {}", i + 1, name.bold(), "🎯".cyan());
                println!("   Target: {}", target);
                println!("   Check: {}", cmd.bright_black());
                println!();
            }

            println!("{} Run commands to check progress towards your goals", "💡".yellow());
            println!();
        }

        "check" => {
            if args.len() < 2 {
                println!("{} Usage: goals check <goal-name>", "Error:".red());
                return Ok(());
            }

            let goal = args[1];
            println!("\n{} {}", "🎯".cyan(), format!("Checking Goal: {}", goal).yellow().bold());
            println!("{}", "═".repeat(70).cyan());
            println!();

            match goal {
                "security" => {
                    println!("Running security verification...");
                    println!();
                    cmd_verify(ctx, &["security"])?;
                }
                "health" => {
                    println!("Running health diagnostic...");
                    println!();
                    cmd_doctor(ctx, &[])?;
                }
                "services" => {
                    if let Ok(services) = ctx.guestfs.inspect_systemd_services(&ctx.root) {
                        let enabled = services.iter().filter(|s| s.enabled).count();
                        println!("{}", "Service Optimization Goal:".green().bold());
                        println!("  Current: {} enabled services", enabled);
                        println!("  Target:  <40 enabled services");

                        if enabled < 40 {
                            println!("  Status:  {} Goal achieved!", "✓".green().bold());
                        } else {
                            println!("  Status:  {} In progress ({} to remove)", "→".yellow(), enabled - 40);
                        }
                        println!();
                    }
                }
                _ => {
                    println!("{} Unknown goal: {}", "Error:".red(), goal);
                    println!("Use {} to see available goals", "goals list".cyan());
                }
            }
        }

        _ => {
            println!("{} Unknown subcommand: {}", "Error:".red(), subcommand);
            println!("{} goals", "Usage:".yellow());
        }
    }

    Ok(())
}

/// Shell usage analysis and habits
pub fn cmd_habits(ctx: &ShellContext, _args: &[&str]) -> Result<()> {
    println!("\n{}", "╔═══════════════════════════════════════════════════════════╗".cyan().bold());
    println!("{}", "║              Shell Usage Analysis                        ║".cyan().bold());
    println!("{}", "╚═══════════════════════════════════════════════════════════╝".cyan().bold());
    println!();

    println!("{}", "Session Statistics:".green().bold());
    println!("{}", "─".repeat(70).cyan());
    println!();

    println!("  Commands executed: {}", ctx.command_count.to_string().yellow());

    if let Some(duration) = ctx.last_command_time {
        println!("  Last command time: {} ms", format!("{:.2}", duration.as_secs_f64() * 1000.0).yellow());
    }

    println!("  Current directory: {}", ctx.current_path.cyan());
    println!("  Active aliases:    {}", ctx.aliases.len().to_string().yellow());
    println!("  Bookmarks saved:   {}", ctx.bookmarks.len().to_string().yellow());
    println!();

    println!("{}", "Usage Patterns:".green().bold());
    println!("{}", "─".repeat(70).cyan());
    println!();

    // Analyze usage patterns
    if ctx.command_count < 5 {
        println!("{} {}", "🌱".cyan(), "Getting Started".bold());
        println!("  You're just beginning your exploration. Try these commands:");
        println!("  • {} - Learn the basics", "learn basics".cyan());
        println!("  • {} - See available commands", "help".cyan());
        println!("  • {} - Get an overview", "dashboard".cyan());
    } else if ctx.command_count < 20 {
        println!("{} {}", "🔍".cyan(), "Active Explorer".bold());
        println!("  You're actively exploring the system. Enhance your workflow:");
        println!("  • {} - Create shortcuts", "alias".cyan());
        println!("  • {} - Save favorite locations", "bookmark".cyan());
        println!("  • {} - Learn advanced features", "learn advanced".cyan());
    } else {
        println!("{} {}", "⭐".cyan(), "Power User".bold());
        println!("  Excellent engagement! Take advantage of advanced features:");
        println!("  • {} - Automate workflows", "auto run <preset>".cyan());
        println!("  • {} - Advanced searches", "search".cyan());
        println!("  • {} - Batch operations", "batch".cyan());
    }
    println!();

    println!("{}", "Efficiency Tips:".yellow().bold());
    println!("{}", "─".repeat(70).cyan());
    println!();

    let tips = vec![
        ("Use Tab completion", "Faster command entry"),
        ("Create aliases", "Shortcut frequently used commands"),
        ("Bookmark paths", "Quick navigation with 'goto'"),
        ("Use 'quick' menu", "Fast access to common actions"),
        ("Try 'context' command", "Get location-specific suggestions"),
    ];

    for (tip, benefit) in tips {
        println!("  {} {} - {}", "💡".yellow(), tip.bold(), benefit.bright_black());
    }
    println!();

    println!("{}", "Recommended Next Steps:".green().bold());
    println!("{}", "─".repeat(70).cyan());
    println!();

    if ctx.bookmarks.is_empty() {
        println!("  {} Create bookmarks for frequently visited paths", "1.".yellow());
        println!("     Command: {}", "bookmark myspot".cyan());
    }

    if ctx.aliases.len() <= 5 {
        println!("  {} Set up custom aliases for your workflow", "2.".yellow());
        println!("     Command: {}", "alias shortname 'full command'".cyan());
    }

    println!("  {} Try automation presets", "3.".yellow());
    println!("     Command: {}", "auto run full-analysis".cyan());
    println!();

    Ok(())
}

/// Team collaboration report generator
pub fn cmd_collaborate(ctx: &mut ShellContext, args: &[&str]) -> Result<()> {
    if args.is_empty() {
        println!("\n{}", "Usage: collaborate <report-type>".red());
        println!();
        println!("{}", "Available Report Types:".yellow().bold());
        println!("  {} - Handoff report for team transition", "handoff".green());
        println!("  {} - Incident report for security team", "incident".green());
        println!("  {} - Change request documentation", "change".green());
        println!("  {} - Weekly status report", "status".green());
        println!();
        return Ok(());
    }

    let report_type = args[0];

    match report_type {
        "handoff" => {
            println!("\n{} {}", "👥".cyan(), "Team Handoff Report".yellow().bold());
            println!("{}", "═".repeat(70).cyan());
            println!();

            println!("{}", "Generating team handoff documentation...".yellow());
            println!();

            println!("{}", "## System Handoff Report".green().bold());
            println!();

            // Current timestamp
            let now = chrono::Local::now();
            println!("**Generated:** {}", now.format("%Y-%m-%d %H:%M:%S"));
            println!("**Inspector:** GuestKit Interactive Shell");
            println!();

            println!("{}", "### System Overview".yellow());
            if let Ok(os_type) = ctx.guestfs.inspect_get_type(&ctx.root) {
                println!("- **OS Type:** {}", os_type);
            }
            if let Ok(distro) = ctx.guestfs.inspect_get_distro(&ctx.root) {
                println!("- **Distribution:** {}", distro);
            }
            if let Ok(arch) = ctx.guestfs.inspect_get_arch(&ctx.root) {
                println!("- **Architecture:** {}", arch);
            }
            println!();

            println!("{}", "### Key Information".yellow());
            if let Ok(pkg_info) = ctx.guestfs.inspect_packages(&ctx.root) {
                println!("- **Total Packages:** {}", pkg_info.packages.len());
            }
            if let Ok(users) = ctx.guestfs.inspect_users(&ctx.root) {
                println!("- **User Accounts:** {}", users.len());
            }
            if let Ok(services) = ctx.guestfs.inspect_systemd_services(&ctx.root) {
                let enabled = services.iter().filter(|s| s.enabled).count();
                println!("- **Services:** {} total, {} enabled", services.len(), enabled);
            }
            println!();

            println!("{}", "### Security Status".yellow());
            if let Ok(sec) = ctx.guestfs.inspect_security(&ctx.root) {
                println!("- **SELinux:** {}", sec.selinux);
                println!("- **AppArmor:** {}", if sec.apparmor { "enabled" } else { "disabled" });
                println!("- **Auditd:** {}", if sec.auditd { "enabled" } else { "disabled" });
            }
            println!();

            println!("{}", "### Recommendations for Incoming Team".yellow());
            println!("1. Run `dashboard` for quick overview");
            println!("2. Run `verify all` to check system health");
            println!("3. Review `security` status");
            println!("4. Check `services` for running daemons");
            println!("5. Use `learn basics` for shell orientation");
            println!();

            println!("{}", "### Critical Files to Review".yellow());
            println!("- /etc/fstab - Filesystem mounts");
            println!("- /etc/hosts - Network configuration");
            println!("- /etc/ssh/sshd_config - SSH settings");
            println!();

            println!("{} Save this report: {}", "💡".yellow(), "snapshot handoff-report.md".cyan());
            println!();
        }

        "incident" => {
            println!("\n{} {}", "🚨".cyan(), "Security Incident Report".yellow().bold());
            println!("{}", "═".repeat(70).cyan());
            println!();

            println!("{}", "## Security Incident Report".green().bold());
            println!();

            let now = chrono::Local::now();
            println!("**Report Date:** {}", now.format("%Y-%m-%d %H:%M:%S"));
            println!("**System:** {}", ctx.root);
            println!("**Reporter:** GuestKit Analysis Tool");
            println!();

            println!("{}", "### Incident Summary".yellow());
            println!("*[To be filled by investigator]*");
            println!();

            println!("{}", "### System State at Time of Incident".yellow());

            println!("\n**Security Configuration:**");
            if let Ok(sec) = ctx.guestfs.inspect_security(&ctx.root) {
                println!("- SELinux: {}", sec.selinux);
                println!("- AppArmor: {}", if sec.apparmor { "Active" } else { "Inactive" });
                println!("- Audit Daemon: {}", if sec.auditd { "Running" } else { "Not running" });
            }

            println!("\n**Active Users:**");
            if let Ok(users) = ctx.guestfs.inspect_users(&ctx.root) {
                let regular = users.iter().filter(|u| {
                    u.uid.parse::<u32>().map(|uid| uid >= 1000).unwrap_or(false)
                }).count();
                println!("- {} regular user accounts", regular);
                println!("- Run 'users' for complete list");
            }

            println!();
            println!("{}", "### Evidence Collection".yellow());
            println!("The following data should be preserved:");
            println!("1. Complete snapshot: `snapshot incident-{}.md`", now.format("%Y%m%d-%H%M%S"));
            println!("2. Security logs: `recent /var/log 100`");
            println!("3. User activity: `users`");
            println!("4. Service status: `services`");
            println!();

            println!("{}", "### Recommended Actions".yellow());
            println!("1. Run `playbook incident` for investigation steps");
            println!("2. Use `search <indicator> --content --path /var/log` for log analysis");
            println!("3. Export evidence: `batch export /tmp/incident-evidence`");
            println!("4. Generate forensics report: `report security`");
            println!();
        }

        "change" => {
            println!("\n{} {}", "📝".cyan(), "Change Request Documentation".yellow().bold());
            println!("{}", "═".repeat(70).cyan());
            println!();

            println!("{}", "## Change Request - System Modification".green().bold());
            println!();

            println!("**Date:** {}", chrono::Local::now().format("%Y-%m-%d"));
            println!("**System:** {}", ctx.root);
            println!("**Prepared by:** GuestKit Shell");
            println!();

            println!("{}", "### Current State Baseline".yellow());
            println!("*Pre-change system snapshot*");
            println!();
            println!("```");
            println!("Command: snapshot pre-change-baseline.md");
            println!("```");
            println!();

            println!("{}", "### Proposed Changes".yellow());
            println!("*[Describe changes to be made]*");
            println!();

            println!("{}", "### Risk Assessment".yellow());
            println!("**Impact Level:** *[Low/Medium/High]*");
            println!("**Affected Components:** *[List components]*");
            println!("**Rollback Plan:** *[Describe rollback procedure]*");
            println!();

            println!("{}", "### Testing Plan".yellow());
            println!("1. Pre-change verification: `verify all`");
            println!("2. Implement changes");
            println!("3. Post-change verification: `verify all`");
            println!("4. Performance check: `bench all`");
            println!("5. Health assessment: `doctor`");
            println!();

            println!("{}", "### Approval".yellow());
            println!("**Requested by:** ___________");
            println!("**Approved by:** ___________");
            println!("**Date:** ___________");
            println!();
        }

        "status" => {
            println!("\n{} {}", "📊".cyan(), "Weekly Status Report".yellow().bold());
            println!("{}", "═".repeat(70).cyan());
            println!();

            println!("{}", "## Weekly System Status".green().bold());
            println!();

            let now = chrono::Local::now();
            println!("**Report Period:** Week of {}", now.format("%Y-%m-%d"));
            println!();

            println!("{}", "### System Health".yellow());
            println!("Run `doctor` for comprehensive health check");
            println!();

            println!("{}", "### Security Status".yellow());
            if let Ok(sec) = ctx.guestfs.inspect_security(&ctx.root) {
                let features = vec![
                    ("SELinux", &sec.selinux != "disabled"),
                    ("AppArmor", sec.apparmor),
                    ("Auditd", sec.auditd),
                ];

                let active = features.iter().filter(|(_, enabled)| *enabled).count();
                println!("**Security Features Active:** {}/3", active);

                for (name, enabled) in features {
                    let status = if enabled { "✓" } else { "✗" };
                    println!("  {} {}", status, name);
                }
            }
            println!();

            println!("{}", "### Activity Summary".yellow());
            println!("- Shell sessions: {}", ctx.command_count);
            println!("- Commands executed: {}", ctx.command_count);
            println!("- Bookmarks created: {}", ctx.bookmarks.len());
            println!();

            println!("{}", "### Recommendations".yellow());
            println!("1. Run monthly security audit: `auto run security-audit`");
            println!("2. Update system documentation: `auto run documentation`");
            println!("3. Review service status: `services`");
            println!();

            println!("{}", "### Next Week Goals".yellow());
            println!("Use `goals suggest` to set improvement targets");
            println!();
        }

        _ => {
            println!("{} Unknown report type: {}", "Error:".red(), report_type);
            println!("{} collaborate <report-type>", "Usage:".yellow());
            return Ok(());
        }
    }

    Ok(())
}

/// Helper: Resolve relative path
fn resolve_path(current: &str, path: &str) -> String {
    if path.starts_with('/') {
        path.to_string()
    } else if path == ".." {
        let parts: Vec<&str> = current.trim_end_matches('/').split('/').collect();
        if parts.len() > 1 {
            parts[..parts.len() - 1].join("/")
        } else {
            "/".to_string()
        }
    } else if path == "." {
        current.to_string()
    } else {
        format!("{}/{}", current.trim_end_matches('/'), path)
    }
}
