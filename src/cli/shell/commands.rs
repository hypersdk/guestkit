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
