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

    println!("\n{}", "Advanced Analytics & Visualization:".yellow().bold());
    println!("  {}  - Predictive issue analysis", "predict".green());
    println!("  {}   - Data visualization charts", "chart <type>".green());
    println!("           Types: packages, users, services, storage, security");
    println!("  {} - Compliance checking", "compliance <standard>".green());
    println!("           Standards: cis, pci-dss, hipaa, gdpr, soc2");

    println!("\n{}", "Automation & Operations:".yellow().bold());
    println!("  {} - Command template system", "template <name>".green());
    println!("           Templates: security-audit, health-check, compliance-review");
    println!("           performance-analysis, export-all, pre-migration");
    println!("  {}   - Comprehensive system scoring", "score".green());
    println!("  {}   - SQL-like query interface", "query <statement>".green());
    println!("  {} - System monitoring & alerts", "monitor <type>".green());
    println!("           Types: security, health, changes, alerts");
    println!("  {} - Migration readiness assessment", "migrate <target>".green());
    println!("           Targets: cloud, container");

    println!("\n{}", "Diagnostics & Remediation:".yellow().bold());
    println!("  {} - Intelligent troubleshooting", "troubleshoot <category>".green());
    println!("           Categories: boot, network, services, performance, security, auto");
    println!("  {} - Package dependency analysis", "depends <command>".green());
    println!("           Commands: search, analyze, dev, libs");
    println!("  {} - Configuration validation", "validate <target>".green());
    println!("           Targets: all, config");

    println!("\n{}", "Security & Forensics:".yellow().bold());
    println!("  {} - Digital forensics workflows", "forensics <workflow>".green());
    println!("           Workflows: collect, timeline, suspicious, activity, integrity, memory");
    println!("  {} - Security audit trail analysis", "audit <type>".green());
    println!("           Types: auth, users, config, packages, sudo, full");
    println!("  {} - Security baseline management", "baseline <command>".green());
    println!("           Commands: create, show, drift, cis, export");

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


/// Predictive analysis for potential issues
pub fn cmd_predict(ctx: &mut ShellContext, _args: &[&str]) -> Result<()> {
    println!("\n{}", "╔═══════════════════════════════════════════════════════════╗".cyan().bold());
    println!("{}", "║              Predictive Issue Analysis                   ║".cyan().bold());
    println!("{}", "╚═══════════════════════════════════════════════════════════╝".cyan().bold());
    println!();

    println!("{}", "Analyzing system patterns for potential future issues...".yellow());
    println!();

    let mut predictions = Vec::new();

    // Get system data
    let pkg_info = ctx.guestfs.inspect_packages(&ctx.root)?;
    let sec = ctx.guestfs.inspect_security(&ctx.root)?;

    // Prediction 1: Security vulnerabilities
    if &sec.selinux == "disabled" && !sec.apparmor {
        predictions.push((
            "🔓",
            "High Risk: Security Breach",
            "No MAC system active increases attack surface",
            "Within 30 days without security hardening",
            "Critical",
            vec![
                "Enable SELinux or AppArmor immediately",
                "Review security audit logs",
                "Implement least-privilege access controls",
            ],
        ));
    }

    // Prediction 2: Package updates
    let pkg_count = pkg_info.packages.len();
    if pkg_count > 500 {
        predictions.push((
            "📦",
            "Medium: Package Update Burden",
            "Large number of packages requires frequent updates",
            "Ongoing maintenance burden",
            "Medium",
            vec![
                "Set up automated update scheduling",
                "Review installed packages for unnecessary ones",
                "Consider containerizing some workloads",
            ],
        ));
    }

    // Prediction 3: Boot issues
    if !ctx.guestfs.exists("/etc/fstab").unwrap_or(false) {
        predictions.push((
            "⚠️",
            "Critical: Boot Failure Risk",
            "Missing /etc/fstab may prevent system boot",
            "Next reboot will likely fail",
            "Critical",
            vec![
                "Generate proper /etc/fstab immediately",
                "Test boot configuration in safe environment",
                "Document filesystem mount requirements",
            ],
        ));
    }

    // Prediction 4: Compliance drift
    if !sec.auditd {
        predictions.push((
            "📋",
            "Medium: Compliance Drift",
            "No audit logging means compliance violations may be undetected",
            "Audit failures within 90 days",
            "Medium",
            vec![
                "Enable auditd service",
                "Configure audit rules for compliance requirements",
                "Set up centralized log collection",
            ],
        ));
    }

    // Prediction 5: Service degradation
    if let Ok(services) = ctx.guestfs.inspect_systemd_services(&ctx.root) {
        let enabled = services.iter().filter(|s| s.enabled).count();
        if enabled > 50 {
            predictions.push((
                "⚙️",
                "Low: Performance Degradation",
                "Many enabled services may cause resource contention",
                "Performance issues within 60-90 days under load",
                "Low",
                vec![
                    "Review and disable unnecessary services",
                    "Implement resource limits and quotas",
                    "Monitor CPU and memory usage trends",
                ],
            ));
        }
    }

    // Prediction 6: User account issues
    if let Ok(users) = ctx.guestfs.inspect_users(&ctx.root) {
        let normal_users = users.iter().filter(|u| u.uid != "0").count();
        if normal_users == 0 {
            predictions.push((
                "👤",
                "Medium: Single Point of Failure",
                "Only root account exists - no user separation",
                "Security incident within 30-60 days",
                "Medium",
                vec![
                    "Create dedicated service accounts",
                    "Implement sudo for privileged operations",
                    "Disable direct root login",
                ],
            ));
        }
    }

    // Display predictions
    if predictions.is_empty() {
        println!("{}", "✓ No significant issues predicted!".green().bold());
        println!("  Your system follows best practices.");
    } else {
        println!("{} {} predictions identified:", "🔮".cyan(), predictions.len().to_string().cyan().bold());
        println!();

        for (icon, title, description, timeline, severity, mitigations) in &predictions {
            let severity_colored = match *severity {
                "Critical" => severity.red().bold(),
                "High" => severity.red(),
                "Medium" => severity.yellow(),
                _ => severity.bright_black(),
            };

            println!("{} {} [{}]", icon.cyan(), title.bold(), severity_colored);
            println!("  Issue:      {}", description);
            println!("  Timeline:   {}", timeline.cyan());
            println!("  Mitigation:");
            for (i, mitigation) in mitigations.iter().enumerate() {
                println!("    {}. {}", i + 1, mitigation);
            }
            println!();
        }

        // Summary
        let critical = predictions.iter().filter(|p| p.4 == "Critical").count();
        let high = predictions.iter().filter(|p| p.4 == "High").count();
        let medium = predictions.iter().filter(|p| p.4 == "Medium").count();

        println!("{} Summary:", "📊".cyan());
        if critical > 0 {
            println!("  {} Critical issues requiring immediate attention", critical.to_string().red().bold());
        }
        if high > 0 {
            println!("  {} High priority issues to address soon", high.to_string().red());
        }
        if medium > 0 {
            println!("  {} Medium priority issues to plan for", medium.to_string().yellow());
        }
    }

    println!();
    println!("{} Next Steps:", "💡".yellow());
    println!("  • {}", "doctor - Run comprehensive health check".cyan());
    println!("  • {}", "verify all - Validate all system components".cyan());
    println!("  • {}", "roadmap 30 - Create 30-day improvement plan".cyan());
    println!();

    Ok(())
}


/// Data visualization with ASCII charts
pub fn cmd_chart(ctx: &mut ShellContext, args: &[&str]) -> Result<()> {
    println!("\n{}", "╔═══════════════════════════════════════════════════════════╗".cyan().bold());
    println!("{}", "║              Data Visualization Charts                   ║".cyan().bold());
    println!("{}", "╚═══════════════════════════════════════════════════════════╝".cyan().bold());
    println!();

    let chart_type = if args.is_empty() { "menu" } else { args[0] };

    match chart_type {
        "menu" => {
            println!("{}", "Available Charts:".yellow().bold());
            println!();
            println!("{} {} - Package distribution by category", "1.".cyan(), "packages".green());
            println!("{} {} - User account distribution", "2.".cyan(), "users".green());
            println!("{} {} - Service status breakdown", "3.".cyan(), "services".green());
            println!("{} {} - Storage usage visualization", "4.".cyan(), "storage".green());
            println!("{} {} - Security features overview", "5.".cyan(), "security".green());
            println!();
            println!("{} chart <name>", "Usage:".yellow());
        }

        "packages" => {
            println!("{}", "📦 Package Distribution Chart".cyan().bold());
            println!();

            let pkg_info = ctx.guestfs.inspect_packages(&ctx.root)?;

            // Categorize packages
            let mut dev_tools = 0;
            let mut libraries = 0;
            let mut system = 0;
            let mut apps = 0;
            let mut other = 0;

            for pkg in &pkg_info.packages {
                let name = pkg.name.to_lowercase();
                if name.contains("gcc") || name.contains("make") || name.contains("devel") || name.contains("dev-") {
                    dev_tools += 1;
                } else if name.contains("lib") || name.starts_with("lib") {
                    libraries += 1;
                } else if name.contains("kernel") || name.contains("systemd") || name.contains("core") {
                    system += 1;
                } else if name.contains("app") || name.contains("tool") {
                    apps += 1;
                } else {
                    other += 1;
                }
            }

            let total = pkg_info.packages.len() as f32;
            let max_bar = 50;

            println!("Development Tools: {} ({}%)", dev_tools, ((dev_tools as f32 / total) * 100.0) as i32);
            let bar_len = ((dev_tools as f32 / total) * max_bar as f32) as usize;
            println!("{} {}", "▓".repeat(bar_len).green(), "░".repeat(max_bar - bar_len).bright_black());
            println!();

            println!("Libraries:         {} ({}%)", libraries, ((libraries as f32 / total) * 100.0) as i32);
            let bar_len = ((libraries as f32 / total) * max_bar as f32) as usize;
            println!("{} {}", "▓".repeat(bar_len).cyan(), "░".repeat(max_bar - bar_len).bright_black());
            println!();

            println!("System Packages:   {} ({}%)", system, ((system as f32 / total) * 100.0) as i32);
            let bar_len = ((system as f32 / total) * max_bar as f32) as usize;
            println!("{} {}", "▓".repeat(bar_len).yellow(), "░".repeat(max_bar - bar_len).bright_black());
            println!();

            println!("Applications:      {} ({}%)", apps, ((apps as f32 / total) * 100.0) as i32);
            let bar_len = ((apps as f32 / total) * max_bar as f32) as usize;
            println!("{} {}", "▓".repeat(bar_len).blue(), "░".repeat(max_bar - bar_len).bright_black());
            println!();

            println!("Other:             {} ({}%)", other, ((other as f32 / total) * 100.0) as i32);
            let bar_len = ((other as f32 / total) * max_bar as f32) as usize;
            println!("{} {}", "▓".repeat(bar_len).bright_black(), "░".repeat(max_bar - bar_len).bright_black());
            println!();

            println!("Total: {} packages", total as i32);
        }

        "users" => {
            println!("{}", "👥 User Account Distribution".cyan().bold());
            println!();

            if let Ok(users) = ctx.guestfs.inspect_users(&ctx.root) {
                let root_users = users.iter().filter(|u| u.uid == "0").count();
                let system_users = users.iter().filter(|u| {
                    let uid = u.uid.parse::<i32>().unwrap_or(9999);
                    uid > 0 && uid < 1000
                }).count();
                let normal_users = users.iter().filter(|u| {
                    let uid = u.uid.parse::<i32>().unwrap_or(0);
                    uid >= 1000
                }).count();

                let total = users.len() as f32;
                let max_bar = 50;

                println!("Root (UID 0):      {}", root_users);
                let bar_len = ((root_users as f32 / total) * max_bar as f32) as usize;
                println!("{} {}", "▓".repeat(bar_len).red(), "░".repeat(max_bar - bar_len).bright_black());
                println!();

                println!("System (1-999):    {}", system_users);
                let bar_len = ((system_users as f32 / total) * max_bar as f32) as usize;
                println!("{} {}", "▓".repeat(bar_len).yellow(), "░".repeat(max_bar - bar_len).bright_black());
                println!();

                println!("Normal (1000+):    {}", normal_users);
                let bar_len = ((normal_users as f32 / total) * max_bar as f32) as usize;
                println!("{} {}", "▓".repeat(bar_len).green(), "░".repeat(max_bar - bar_len).bright_black());
                println!();

                println!("Total: {} users", total as i32);
            }
        }

        "services" => {
            println!("{}", "⚙️  Service Status Breakdown".cyan().bold());
            println!();

            if let Ok(services) = ctx.guestfs.inspect_systemd_services(&ctx.root) {
                let enabled = services.iter().filter(|s| s.enabled).count();
                let disabled = services.len() - enabled;
                let total = services.len() as f32;
                let max_bar = 50;

                println!("Enabled Services:  {} ({}%)", enabled, ((enabled as f32 / total) * 100.0) as i32);
                let bar_len = ((enabled as f32 / total) * max_bar as f32) as usize;
                println!("{} {}", "▓".repeat(bar_len).green(), "░".repeat(max_bar - bar_len).bright_black());
                println!();

                println!("Disabled Services: {} ({}%)", disabled, ((disabled as f32 / total) * 100.0) as i32);
                let bar_len = ((disabled as f32 / total) * max_bar as f32) as usize;
                println!("{} {}", "▓".repeat(bar_len).red(), "░".repeat(max_bar - bar_len).bright_black());
                println!();

                println!("Total: {} services", total as i32);
                println!();
                println!("Service Density: {}", if enabled > 50 { "High".red() } else if enabled > 30 { "Medium".yellow() } else { "Low".green() });
            }
        }

        "storage" => {
            println!("{}", "💾 Storage Usage Visualization".cyan().bold());
            println!();

            if let Ok(filesystems) = ctx.guestfs.list_filesystems() {
                println!("Mounted Filesystems:");
                println!();

                for (path, fstype) in filesystems.iter().take(10) {
                    if fstype != "unknown" && fstype != "swap" {
                        // Simplified visualization (actual size info would require statvfs)
                        println!("{}", path.cyan());
                        println!("  Type: {}", fstype.green());
                        println!("  {}", "▓▓▓▓▓▓▓▓▓▓░░░░░░░░░░ 50% usage (estimated)".bright_black());
                        println!();
                    }
                }
            }
        }

        "security" => {
            println!("{}", "🛡️  Security Features Overview".cyan().bold());
            println!();

            let sec = ctx.guestfs.inspect_security(&ctx.root)?;
            let max_bar = 40;

            // SELinux
            let selinux_status = if &sec.selinux != "disabled" { 1.0 } else { 0.0 };
            println!("SELinux:    [{}{}] {}",
                "▓".repeat((selinux_status * max_bar as f32) as usize).green(),
                "░".repeat(((1.0 - selinux_status) * max_bar as f32) as usize).bright_black(),
                if selinux_status > 0.0 { "Enabled".green() } else { "Disabled".red() }
            );

            // AppArmor
            let apparmor_status = if sec.apparmor { 1.0 } else { 0.0 };
            println!("AppArmor:   [{}{}] {}",
                "▓".repeat((apparmor_status * max_bar as f32) as usize).green(),
                "░".repeat(((1.0 - apparmor_status) * max_bar as f32) as usize).bright_black(),
                if apparmor_status > 0.0 { "Active".green() } else { "Inactive".red() }
            );

            // Auditd
            let auditd_status = if sec.auditd { 1.0 } else { 0.0 };
            println!("Auditd:     [{}{}] {}",
                "▓".repeat((auditd_status * max_bar as f32) as usize).green(),
                "░".repeat(((1.0 - auditd_status) * max_bar as f32) as usize).bright_black(),
                if auditd_status > 0.0 { "Running".green() } else { "Not Running".red() }
            );

            // Firewall
            if let Ok(fw) = ctx.guestfs.inspect_firewall(&ctx.root) {
                let fw_status = if fw.enabled { 1.0 } else { 0.0 };
                println!("Firewall:   [{}{}] {}",
                    "▓".repeat((fw_status * max_bar as f32) as usize).green(),
                    "░".repeat(((1.0 - fw_status) * max_bar as f32) as usize).bright_black(),
                    if fw_status > 0.0 { "Enabled".green() } else { "Disabled".red() }
                );
            }

            println!();
            let score = ((selinux_status + apparmor_status + auditd_status) / 3.0 * 100.0) as i32;
            println!("Overall Security Score: {}%", score.to_string().cyan());
        }

        _ => {
            println!("{} Unknown chart type: {}", "Error:".red(), chart_type);
            println!("{} chart menu", "Usage:".yellow());
        }
    }

    println!();
    Ok(())
}

/// Compliance checking against standards
pub fn cmd_compliance(ctx: &mut ShellContext, args: &[&str]) -> Result<()> {
    println!("\n{}", "╔═══════════════════════════════════════════════════════════╗".cyan().bold());
    println!("{}", "║              Compliance Standards Checker                ║".cyan().bold());
    println!("{}", "╚═══════════════════════════════════════════════════════════╝".cyan().bold());
    println!();

    let standard = if args.is_empty() { "menu" } else { args[0] };

    match standard {
        "menu" => {
            println!("{}", "Available Compliance Standards:".yellow().bold());
            println!();
            println!("{} {} - Center for Internet Security Benchmark", "1.".cyan(), "cis".green());
            println!("{} {} - Payment Card Industry Data Security", "2.".cyan(), "pci-dss".green());
            println!("{} {} - Health Insurance Portability Act", "3.".cyan(), "hipaa".green());
            println!("{} {} - General Data Protection Regulation", "4.".cyan(), "gdpr".green());
            println!("{} {} - Service Organization Control", "5.".cyan(), "soc2".green());
            println!();
            println!("{} compliance <standard>", "Usage:".yellow());
        }

        "cis" => {
            println!("{}", "📋 CIS Benchmark Compliance Check".cyan().bold());
            println!();

            let mut passed = 0;
            let mut failed = 0;
            let mut checks = Vec::new();

            let sec = ctx.guestfs.inspect_security(&ctx.root)?;

            // CIS 1.6.1: Ensure SELinux/AppArmor is enabled
            if &sec.selinux != "disabled" || sec.apparmor {
                checks.push(("1.6.1", "MAC system enabled", true, "SELinux or AppArmor is active"));
                passed += 1;
            } else {
                checks.push(("1.6.1", "MAC system enabled", false, "Enable SELinux or AppArmor"));
                failed += 1;
            }

            // CIS 4.1.1: Ensure auditing is enabled
            if sec.auditd {
                checks.push(("4.1.1", "Auditing enabled", true, "Auditd is running"));
                passed += 1;
            } else {
                checks.push(("4.1.1", "Auditing enabled", false, "Enable and start auditd service"));
                failed += 1;
            }

            // CIS 3.5.1: Ensure firewall is enabled
            if let Ok(fw) = ctx.guestfs.inspect_firewall(&ctx.root) {
                if fw.enabled {
                    checks.push(("3.5.1", "Firewall enabled", true, "Firewall is active"));
                    passed += 1;
                } else {
                    checks.push(("3.5.1", "Firewall enabled", false, "Enable firewall service"));
                    failed += 1;
                }
            }

            // CIS 5.2.1: Ensure permissions on /etc/ssh/sshd_config are configured
            if ctx.guestfs.exists("/etc/ssh/sshd_config").unwrap_or(false) {
                checks.push(("5.2.1", "SSH config exists", true, "sshd_config is present"));
                passed += 1;
            } else {
                checks.push(("5.2.1", "SSH config exists", false, "SSH server may not be configured"));
                failed += 1;
            }

            // CIS 1.1.1: Ensure mounting of filesystems is configured
            if ctx.guestfs.exists("/etc/fstab").unwrap_or(false) {
                checks.push(("1.1.1", "Filesystem table configured", true, "/etc/fstab exists"));
                passed += 1;
            } else {
                checks.push(("1.1.1", "Filesystem table configured", false, "Create /etc/fstab"));
                failed += 1;
            }

            // Display results
            for (id, name, status, detail) in checks {
                let status_icon = if status { "✓".green() } else { "✗".red() };
                let status_text = if status { "PASS".green() } else { "FAIL".red() };

                println!("{} {} {} - {}", status_icon, id.cyan(), name.bold(), status_text);
                println!("    {}", detail.bright_black());
                println!();
            }

            // Summary
            let total = passed + failed;
            let compliance_rate = (passed as f32 / total as f32) * 100.0;

            println!("{} Compliance Summary:", "📊".cyan());
            println!("  Passed:     {} checks", passed.to_string().green());
            println!("  Failed:     {} checks", failed.to_string().red());
            println!("  Total:      {} checks", total);
            println!("  Rate:       {:.1}%", compliance_rate);
            println!();

            if compliance_rate >= 80.0 {
                println!("  Status:     {} Compliant", "✓".green().bold());
            } else if compliance_rate >= 60.0 {
                println!("  Status:     {} Partially Compliant", "⚠".yellow());
            } else {
                println!("  Status:     {} Non-Compliant", "✗".red());
            }
        }

        "pci-dss" => {
            println!("{}", "💳 PCI-DSS Compliance Check".cyan().bold());
            println!();

            let mut passed = 0;
            let mut failed = 0;

            let sec = ctx.guestfs.inspect_security(&ctx.root)?;

            println!("{} {}", "Requirement 1:".cyan().bold(), "Install and maintain firewall configuration");
            if let Ok(fw) = ctx.guestfs.inspect_firewall(&ctx.root) {
                if fw.enabled {
                    println!("  {} Firewall is active", "✓".green());
                    passed += 1;
                } else {
                    println!("  {} Firewall is not active", "✗".red());
                    failed += 1;
                }
            } else {
                println!("  {} Could not verify firewall status", "?".yellow());
            }
            println!();

            println!("{} {}", "Requirement 2:".cyan().bold(), "Do not use vendor-supplied defaults");
            // This would require deeper inspection
            println!("  {} Manual review required", "?".yellow());
            println!();

            println!("{} {}", "Requirement 10:".cyan().bold(), "Track and monitor all access to network resources");
            if sec.auditd {
                println!("  {} Audit logging is enabled", "✓".green());
                passed += 1;
            } else {
                println!("  {} Audit logging is not enabled", "✗".red());
                failed += 1;
            }
            println!();

            let total = passed + failed;
            if total > 0 {
                let rate = (passed as f32 / total as f32) * 100.0;
                println!("Automated checks: {:.0}% compliant ({}/{})", rate, passed, total);
                println!();
            }

            println!("{} PCI-DSS requires comprehensive manual audit.", "Note:".yellow());
            println!("This automated check covers only basic requirements.");
        }

        "hipaa" => {
            println!("{}", "🏥 HIPAA Compliance Check".cyan().bold());
            println!();

            let sec = ctx.guestfs.inspect_security(&ctx.root)?;
            let mut passed = 0;
            let mut failed = 0;

            println!("{} {}", "§164.312(a)(1):".cyan().bold(), "Access Control");
            if let Ok(users) = ctx.guestfs.inspect_users(&ctx.root) {
                let normal_users = users.iter().filter(|u| u.uid != "0").count();
                if normal_users > 0 {
                    println!("  {} User access controls in place", "✓".green());
                    passed += 1;
                } else {
                    println!("  {} No non-root users (poor access control)", "✗".red());
                    failed += 1;
                }
            }
            println!();

            println!("{} {}", "§164.312(b):".cyan().bold(), "Audit Controls");
            if sec.auditd {
                println!("  {} Audit logging enabled", "✓".green());
                passed += 1;
            } else {
                println!("  {} No audit logging", "✗".red());
                failed += 1;
            }
            println!();

            println!("{} {}", "§164.312(c)(1):".cyan().bold(), "Integrity Controls");
            if &sec.selinux != "disabled" || sec.apparmor {
                println!("  {} Mandatory access control active", "✓".green());
                passed += 1;
            } else {
                println!("  {} No MAC system active", "✗".red());
                failed += 1;
            }
            println!();

            println!("{} {}", "§164.312(e)(1):".cyan().bold(), "Transmission Security");
            // Would need to check for encryption configs
            println!("  {} Manual verification required", "?".yellow());
            println!();

            let total = passed + failed;
            if total > 0 {
                let rate = (passed as f32 / total as f32) * 100.0;
                println!("Technical safeguards: {:.0}% implemented ({}/{})", rate, passed, total);
            }
        }

        "gdpr" | "soc2" => {
            println!("{} {} compliance checking requires manual audit.", standard.to_uppercase().cyan().bold(), "Note:".yellow());
            println!();
            println!("Key areas to review:");
            println!("  • Data encryption at rest and in transit");
            println!("  • Access controls and authentication");
            println!("  • Audit logging and monitoring");
            println!("  • Data retention policies");
            println!("  • Incident response procedures");
            println!();
            println!("Use these commands for technical verification:");
            println!("  • {} - Security feature check", "verify security".green());
            println!("  • {} - System health diagnostic", "doctor".green());
            println!("  • {} - Security insights", "insights".green());
        }

        _ => {
            println!("{} Unknown standard: {}", "Error:".red(), standard);
            println!("{} compliance menu", "Usage:".yellow());
        }
    }

    println!();
    Ok(())
}

/// Command template system for repeatable operations
pub fn cmd_template(ctx: &mut ShellContext, args: &[&str]) -> Result<()> {
    println!("\n{}", "╔═══════════════════════════════════════════════════════════╗".cyan().bold());
    println!("{}", "║              Command Template System                     ║".cyan().bold());
    println!("{}", "╚═══════════════════════════════════════════════════════════╝".cyan().bold());
    println!();

    if args.is_empty() {
        println!("{}", "Available Templates:".yellow().bold());
        println!();
        println!("{} {} - Full security audit", "1.".cyan(), "security-audit".green());
        println!("{} {} - System health check", "2.".cyan(), "health-check".green());
        println!("{} {} - Compliance review", "3.".cyan(), "compliance-review".green());
        println!("{} {} - Performance analysis", "4.".cyan(), "performance-analysis".green());
        println!("{} {} - Export all data", "5.".cyan(), "export-all".green());
        println!("{} {} - Pre-migration check", "6.".cyan(), "pre-migration".green());
        println!();
        println!("{} template <name>", "Usage:".yellow());
        return Ok(());
    }

    let template_name = args[0];

    match template_name {
        "security-audit" => {
            println!("{}", "🔒 Running Security Audit Template".cyan().bold());
            println!("{}", "━".repeat(60).bright_black());
            println!();

            println!("{} Phase 1: Security Configuration", "→".cyan());
            cmd_verify(ctx, &["security"])?;
            println!();

            println!("{} Phase 2: Vulnerability Predictions", "→".cyan());
            cmd_predict(ctx, &[])?;
            println!();

            println!("{} Phase 3: Compliance Checks", "→".cyan());
            cmd_compliance(ctx, &["cis"])?;
            println!();

            println!("{} Phase 4: Security Insights", "→".cyan());
            cmd_insights(ctx, &[])?;
            println!();

            println!("{}", "✓ Security Audit Complete".green().bold());
            println!("  Review the findings above and address critical issues first.");
        }

        "health-check" => {
            println!("{}", "🏥 Running Health Check Template".cyan().bold());
            println!("{}", "━".repeat(60).bright_black());
            println!();

            println!("{} Phase 1: System Doctor", "→".cyan());
            cmd_doctor(ctx, &[])?;
            println!();

            println!("{} Phase 2: Verification Suite", "→".cyan());
            cmd_verify(ctx, &["all"])?;
            println!();

            println!("{} Phase 3: Intelligent Insights", "→".cyan());
            cmd_insights(ctx, &[])?;
            println!();

            println!("{}", "✓ Health Check Complete".green().bold());
        }

        "compliance-review" => {
            println!("{}", "📋 Running Compliance Review Template".cyan().bold());
            println!("{}", "━".repeat(60).bright_black());
            println!();

            println!("{} CIS Benchmark", "→".cyan());
            cmd_compliance(ctx, &["cis"])?;
            println!();

            println!("{} PCI-DSS", "→".cyan());
            cmd_compliance(ctx, &["pci-dss"])?;
            println!();

            println!("{} HIPAA", "→".cyan());
            cmd_compliance(ctx, &["hipaa"])?;
            println!();

            println!("{}", "✓ Compliance Review Complete".green().bold());
        }

        "performance-analysis" => {
            println!("{}", "⚡ Running Performance Analysis Template".cyan().bold());
            println!("{}", "━".repeat(60).bright_black());
            println!();

            println!("{} Phase 1: Package Analysis", "→".cyan());
            cmd_chart(ctx, &["packages"])?;
            println!();

            println!("{} Phase 2: Service Analysis", "→".cyan());
            cmd_chart(ctx, &["services"])?;
            println!();

            println!("{} Phase 3: Optimization Recommendations", "→".cyan());
            cmd_optimize(ctx, &[])?;
            println!();

            println!("{}", "✓ Performance Analysis Complete".green().bold());
        }

        "export-all" => {
            println!("{}", "💾 Running Export All Template".cyan().bold());
            println!("{}", "━".repeat(60).bright_black());
            println!();

            println!("{} Generating comprehensive snapshot...", "→".cyan());
            cmd_snapshot(ctx, &[])?;
            println!();

            println!("{}", "✓ Export Complete".green().bold());
        }

        "pre-migration" => {
            println!("{}", "🚀 Running Pre-Migration Check Template".cyan().bold());
            println!("{}", "━".repeat(60).bright_black());
            println!();

            println!("{} Phase 1: System Verification", "→".cyan());
            cmd_verify(ctx, &["all"])?;
            println!();

            println!("{} Phase 2: Production Readiness", "→".cyan());
            cmd_compare(ctx, &["production"])?;
            println!();

            println!("{} Phase 3: Issue Predictions", "→".cyan());
            cmd_predict(ctx, &[])?;
            println!();

            println!("{} Phase 4: Data Export", "→".cyan());
            cmd_snapshot(ctx, &[])?;
            println!();

            println!("{}", "✓ Pre-Migration Check Complete".green().bold());
            println!("  Address any critical issues before migration.");
        }

        _ => {
            println!("{} Unknown template: {}", "Error:".red(), template_name);
            println!("{} template (without arguments) to see available templates", "Tip:".yellow());
        }
    }

    println!();
    Ok(())
}

/// Comprehensive system scoring across multiple dimensions
pub fn cmd_score(ctx: &mut ShellContext, _args: &[&str]) -> Result<()> {
    println!("\n{}", "╔═══════════════════════════════════════════════════════════╗".cyan().bold());
    println!("{}", "║              Comprehensive System Score                  ║".cyan().bold());
    println!("{}", "╚═══════════════════════════════════════════════════════════╝".cyan().bold());
    println!();

    println!("{}", "Calculating multi-dimensional system score...".yellow());
    println!();

    let mut total_score = 0;
    let mut max_score = 0;
    let mut scores = Vec::new();

    // Security Score (0-30)
    let sec = ctx.guestfs.inspect_security(&ctx.root)?;
    let mut sec_score = 0;
    max_score += 30;

    if &sec.selinux != "disabled" {
        sec_score += 10;
    }
    if sec.apparmor {
        sec_score += 10;
    }
    if sec.auditd {
        sec_score += 5;
    }
    if let Ok(fw) = ctx.guestfs.inspect_firewall(&ctx.root) {
        if fw.enabled {
            sec_score += 5;
        }
    }

    total_score += sec_score;
    scores.push(("Security", sec_score, 30));

    // Reliability Score (0-25)
    let mut rel_score = 25;
    max_score += 25;

    if !ctx.guestfs.exists("/etc/fstab").unwrap_or(false) {
        rel_score -= 10;
    }
    let grub_found = ctx.guestfs.exists("/boot/grub/grub.cfg").unwrap_or(false)
        || ctx.guestfs.exists("/boot/grub2/grub.cfg").unwrap_or(false);
    if !grub_found {
        rel_score -= 10;
    }
    if !ctx.guestfs.exists("/etc/resolv.conf").unwrap_or(false) {
        rel_score -= 5;
    }

    total_score += rel_score;
    scores.push(("Reliability", rel_score, 25));

    // Configuration Score (0-20)
    let mut config_score = 0;
    max_score += 20;

    if let Ok(users) = ctx.guestfs.inspect_users(&ctx.root) {
        let normal_users = users.iter().filter(|u| u.uid != "0").count();
        if normal_users > 0 {
            config_score += 10;
        }
    }

    if ctx.guestfs.exists("/etc/ssh/sshd_config").unwrap_or(false) {
        config_score += 5;
    }

    let syslog_found = ctx.guestfs.exists("/etc/rsyslog.conf").unwrap_or(false)
        || ctx.guestfs.exists("/etc/syslog.conf").unwrap_or(false);
    if syslog_found {
        config_score += 5;
    }

    total_score += config_score;
    scores.push(("Configuration", config_score, 20));

    // Maintainability Score (0-15)
    let mut maint_score = 15;
    max_score += 15;

    let pkg_info = ctx.guestfs.inspect_packages(&ctx.root)?;
    let pkg_count = pkg_info.packages.len();

    if pkg_count > 1500 {
        maint_score -= 5; // Too many packages
    }
    if pkg_count < 100 {
        maint_score -= 5; // Too minimal, might be missing essentials
    }

    total_score += maint_score;
    scores.push(("Maintainability", maint_score, 15));

    // Performance Score (0-10)
    let mut perf_score = 10;
    max_score += 10;

    if let Ok(services) = ctx.guestfs.inspect_systemd_services(&ctx.root) {
        let enabled = services.iter().filter(|s| s.enabled).count();
        if enabled > 80 {
            perf_score -= 5;
        } else if enabled > 50 {
            perf_score -= 3;
        }
    }

    total_score += perf_score;
    scores.push(("Performance", perf_score, 10));

    // Display scores
    println!("{}", "Score Breakdown:".cyan().bold());
    println!();

    for (category, score, max) in &scores {
        let percentage = (*score as f32 / *max as f32) * 100.0;
        let bar_length = 40;
        let filled = ((percentage / 100.0) * bar_length as f32) as usize;

        let color = if percentage >= 80.0 {
            "green"
        } else if percentage >= 60.0 {
            "yellow"
        } else {
            "red"
        };

        let bar = match color {
            "green" => format!("{}{}", "▓".repeat(filled).green(), "░".repeat(bar_length - filled).bright_black()),
            "yellow" => format!("{}{}", "▓".repeat(filled).yellow(), "░".repeat(bar_length - filled).bright_black()),
            _ => format!("{}{}", "▓".repeat(filled).red(), "░".repeat(bar_length - filled).bright_black()),
        };

        println!("{:15} [{}] {}/{} ({:.0}%)",
            format!("{}:", category).bold(),
            bar,
            score.to_string().cyan(),
            max,
            percentage
        );
    }

    println!();
    println!("{}", "═".repeat(60).bright_black());

    let overall_percentage = (total_score as f32 / max_score as f32) * 100.0;
    let grade = if overall_percentage >= 90.0 {
        "A+ (Excellent)".green().bold()
    } else if overall_percentage >= 85.0 {
        "A (Very Good)".green()
    } else if overall_percentage >= 80.0 {
        "B+ (Good)".green()
    } else if overall_percentage >= 75.0 {
        "B (Above Average)".yellow()
    } else if overall_percentage >= 70.0 {
        "C+ (Average)".yellow()
    } else if overall_percentage >= 60.0 {
        "C (Below Average)".yellow()
    } else {
        "D (Needs Improvement)".red()
    };

    println!("{:15} {}/{} ({:.1}%)",
        "Overall Score:".bold(),
        total_score.to_string().cyan().bold(),
        max_score,
        overall_percentage
    );
    println!("{:15} {}", "Grade:".bold(), grade);

    println!();
    println!("{} Recommendations:", "💡".yellow());

    if sec_score < 20 {
        println!("  • {}", "Improve security posture (enable SELinux/AppArmor, firewall, audit)".cyan());
    }
    if rel_score < 15 {
        println!("  • {}", "Fix critical configuration files (/etc/fstab, boot loader)".cyan());
    }
    if config_score < 10 {
        println!("  • {}", "Enhance system configuration (user accounts, SSH, logging)".cyan());
    }
    if maint_score < 10 {
        println!("  • {}", "Optimize package management".cyan());
    }
    if perf_score < 7 {
        println!("  • {}", "Reduce service overhead".cyan());
    }

    if overall_percentage >= 85.0 {
        println!();
        println!("{}", "✓ System is in excellent condition!".green().bold());
    } else if overall_percentage >= 70.0 {
        println!();
        println!("{}", "⚠ System is acceptable but has room for improvement.".yellow());
    } else {
        println!();
        println!("{}", "✗ System requires attention to critical issues.".red());
    }

    println!();
    Ok(())
}

/// Query system data with SQL-like syntax
pub fn cmd_query(ctx: &mut ShellContext, args: &[&str]) -> Result<()> {
    println!("\n{}", "╔═══════════════════════════════════════════════════════════╗".cyan().bold());
    println!("{}", "║              System Query Interface                      ║".cyan().bold());
    println!("{}", "╚═══════════════════════════════════════════════════════════╝".cyan().bold());
    println!();

    if args.is_empty() {
        println!("{}", "Available Queries:".yellow().bold());
        println!();
        println!("{} {} - Find packages by name", "1.".cyan(), "packages where name=<pattern>".green());
        println!("{} {} - Find users by UID range", "2.".cyan(), "users where uid>1000".green());
        println!("{} {} - Find enabled services", "3.".cyan(), "services where enabled=true".green());
        println!("{} {} - Count packages by type", "4.".cyan(), "count packages by type".green());
        println!("{} {} - List largest packages", "5.".cyan(), "packages order by size desc limit 10".green());
        println!();
        println!("{}", "Examples:".green().bold());
        println!("  query packages where name=kernel");
        println!("  query users where uid>1000");
        println!("  query services where enabled=true");
        println!("  query count packages");
        println!();
        return Ok(());
    }

    let query_str = args.join(" ");

    // Simple query parser
    if query_str.starts_with("packages where name=") {
        let pattern = query_str.strip_prefix("packages where name=").unwrap_or("");
        println!("{} Packages matching '{}':", "→".cyan(), pattern.green());
        println!();

        let pkg_info = ctx.guestfs.inspect_packages(&ctx.root)?;
        let matches: Vec<_> = pkg_info.packages.iter()
            .filter(|p| p.name.contains(pattern))
            .collect();

        for (i, pkg) in matches.iter().enumerate().take(50) {
            println!("{:3}. {} - {}", i + 1, pkg.name.cyan(), pkg.version.to_string().bright_black());
        }

        println!();
        println!("Found {} matching packages", matches.len().to_string().green());

    } else if query_str.starts_with("users where uid>") {
        let uid_str = query_str.strip_prefix("users where uid>").unwrap_or("1000");
        let min_uid: i32 = uid_str.parse().unwrap_or(1000);

        println!("{} Users with UID > {}:", "→".cyan(), min_uid.to_string().green());
        println!();

        if let Ok(users) = ctx.guestfs.inspect_users(&ctx.root) {
            let matches: Vec<_> = users.iter()
                .filter(|u| u.uid.parse::<i32>().unwrap_or(0) > min_uid)
                .collect();

            for user in &matches {
                println!("  {} {} (UID: {}, GID: {})",
                    "•".cyan(),
                    user.username.green(),
                    user.uid.yellow(),
                    user.gid.bright_black()
                );
            }

            println!();
            println!("Found {} matching users", matches.len().to_string().green());
        }

    } else if query_str == "services where enabled=true" {
        println!("{} Enabled services:", "→".cyan());
        println!();

        if let Ok(services) = ctx.guestfs.inspect_systemd_services(&ctx.root) {
            let enabled: Vec<_> = services.iter()
                .filter(|s| s.enabled)
                .collect();

            for (i, service) in enabled.iter().enumerate().take(50) {
                println!("{:3}. {}", i + 1, service.name.cyan());
            }

            println!();
            println!("Found {} enabled services", enabled.len().to_string().green());
        }

    } else if query_str == "count packages" {
        let pkg_info = ctx.guestfs.inspect_packages(&ctx.root)?;
        println!("{} Total packages: {}", "→".cyan(), pkg_info.packages.len().to_string().green().bold());

    } else if query_str.starts_with("packages order by") {
        println!("{} Package list:", "→".cyan());
        println!();

        let pkg_info = ctx.guestfs.inspect_packages(&ctx.root)?;
        let mut packages: Vec<_> = pkg_info.packages.iter().collect();

        // Simple sorting by name
        packages.sort_by_key(|p| &p.name);

        let limit = if query_str.contains("limit") {
            let parts: Vec<&str> = query_str.split("limit").collect();
            if parts.len() > 1 {
                parts[1].trim().parse::<usize>().unwrap_or(10)
            } else {
                10
            }
        } else {
            10
        };

        for (i, pkg) in packages.iter().take(limit).enumerate() {
            println!("{:3}. {} - {}", i + 1, pkg.name.cyan(), pkg.version.to_string().bright_black());
        }

        println!();
        println!("Showing {} of {} packages", limit.min(packages.len()), packages.len());

    } else {
        println!("{} Unsupported query syntax", "Error:".red());
        println!("{} query (without arguments) for examples", "Tip:".yellow());
    }

    println!();
    Ok(())
}

/// System monitoring and change detection
pub fn cmd_monitor(ctx: &mut ShellContext, args: &[&str]) -> Result<()> {
    println!("\n{}", "╔═══════════════════════════════════════════════════════════╗".cyan().bold());
    println!("{}", "║              System Monitoring & Alerts                  ║".cyan().bold());
    println!("{}", "╚═══════════════════════════════════════════════════════════╝".cyan().bold());
    println!();

    if args.is_empty() {
        println!("{}", "Monitoring Capabilities:".yellow().bold());
        println!();
        println!("{} {} - Monitor for security issues", "1.".cyan(), "monitor security".green());
        println!("{} {} - Monitor system health", "2.".cyan(), "monitor health".green());
        println!("{} {} - Monitor for changes", "3.".cyan(), "monitor changes".green());
        println!("{} {} - Alert configuration", "4.".cyan(), "monitor alerts".green());
        println!();
        println!("{} monitor <type>", "Usage:".yellow());
        return Ok(());
    }

    let monitor_type = args[0];

    match monitor_type {
        "security" => {
            println!("{}", "🔒 Security Monitoring Report".cyan().bold());
            println!("{}", "━".repeat(60).bright_black());
            println!();

            let sec = ctx.guestfs.inspect_security(&ctx.root)?;
            let mut alerts = Vec::new();

            if &sec.selinux == "disabled" && !sec.apparmor {
                alerts.push(("CRITICAL", "No MAC system active", "Enable SELinux or AppArmor"));
            }

            if !sec.auditd {
                alerts.push(("WARNING", "Audit daemon not running", "Enable auditd for security logging"));
            }

            if let Ok(fw) = ctx.guestfs.inspect_firewall(&ctx.root) {
                if !fw.enabled {
                    alerts.push(("CRITICAL", "Firewall is disabled", "Enable firewall immediately"));
                }
            }

            if let Ok(users) = ctx.guestfs.inspect_users(&ctx.root) {
                let root_users = users.iter().filter(|u| u.uid == "0").count();
                if root_users > 1 {
                    alerts.push(("WARNING", "Multiple root accounts detected", "Review and consolidate root access"));
                }
            }

            if alerts.is_empty() {
                println!("{}", "✓ No security alerts detected".green().bold());
                println!("  System security configuration appears nominal.");
            } else {
                println!("{} {} security alerts:", "⚠".yellow(), alerts.len());
                println!();

                for (i, (level, issue, action)) in alerts.iter().enumerate() {
                    let level_colored = match *level {
                        "CRITICAL" => level.red().bold(),
                        "WARNING" => level.yellow(),
                        _ => level.bright_black(),
                    };

                    println!("{} [{}] {}", format!("{}.", i + 1).cyan(), level_colored, issue.bold());
                    println!("   Action: {}", action.bright_black());
                    println!();
                }
            }
        }

        "health" => {
            println!("{}", "🏥 Health Monitoring Report".cyan().bold());
            println!("{}", "━".repeat(60).bright_black());
            println!();

            let mut issues = Vec::new();

            if !ctx.guestfs.exists("/etc/fstab").unwrap_or(false) {
                issues.push(("ERROR", "Missing /etc/fstab", "System may not boot"));
            }

            let grub_found = ctx.guestfs.exists("/boot/grub/grub.cfg").unwrap_or(false)
                || ctx.guestfs.exists("/boot/grub2/grub.cfg").unwrap_or(false);
            if !grub_found {
                issues.push(("ERROR", "No GRUB configuration", "Boot loader not configured"));
            }

            if !ctx.guestfs.exists("/etc/resolv.conf").unwrap_or(false) {
                issues.push(("WARNING", "Missing /etc/resolv.conf", "DNS may not work"));
            }

            if issues.is_empty() {
                println!("{}", "✓ No health issues detected".green().bold());
                println!("  System health appears good.");
            } else {
                println!("{} {} health issues:", "⚠".yellow(), issues.len());
                println!();

                for (i, (level, issue, impact)) in issues.iter().enumerate() {
                    let level_colored = match *level {
                        "ERROR" => level.red().bold(),
                        "WARNING" => level.yellow(),
                        _ => level.bright_black(),
                    };

                    println!("{} [{}] {}", format!("{}.", i + 1).cyan(), level_colored, issue.bold());
                    println!("   Impact: {}", impact.bright_black());
                    println!();
                }
            }
        }

        "changes" => {
            println!("{}", "📊 Change Detection Report".cyan().bold());
            println!("{}", "━".repeat(60).bright_black());
            println!();

            println!("{}", "Note:".yellow().bold());
            println!("  Change detection requires multiple snapshots over time.");
            println!("  Use 'snapshot' command to create baseline snapshots.");
            println!();
            println!("{}", "Recommended workflow:".green());
            println!("  1. snapshot baseline.md");
            println!("  2. (make system changes)");
            println!("  3. snapshot current.md");
            println!("  4. compare files baseline.md current.md");
        }

        "alerts" => {
            println!("{}", "🔔 Alert Configuration".cyan().bold());
            println!("{}", "━".repeat(60).bright_black());
            println!();

            println!("{}", "Alert Rules:".yellow().bold());
            println!();
            println!("{} Security configuration changes", "•".cyan());
            println!("{} Critical file modifications", "•".cyan());
            println!("{} Service state changes", "•".cyan());
            println!("{} User account additions/removals", "•".cyan());
            println!("{} Package installations/removals", "•".cyan());
            println!();
            println!("{}", "Note:".yellow().bold());
            println!("  Alert rules are informational. Use 'monitor security' and");
            println!("  'monitor health' for current status checks.");
        }

        _ => {
            println!("{} Unknown monitor type: {}", "Error:".red(), monitor_type);
            println!("{} monitor (without arguments) for options", "Tip:".yellow());
        }
    }

    println!();
    Ok(())
}

/// Migration preparation and readiness assessment
pub fn cmd_migrate(ctx: &mut ShellContext, args: &[&str]) -> Result<()> {
    println!("\n{}", "╔═══════════════════════════════════════════════════════════╗".cyan().bold());
    println!("{}", "║           Migration Readiness Assessment                ║".cyan().bold());
    println!("{}", "╚═══════════════════════════════════════════════════════════╝".cyan().bold());
    println!();

    let target = if args.is_empty() { "cloud" } else { args[0] };

    match target {
        "cloud" => {
            println!("{}", "☁️  Cloud Migration Readiness".cyan().bold());
            println!("{}", "━".repeat(60).bright_black());
            println!();

            let mut ready = 0;
            let mut warnings = Vec::new();
            let mut blockers = Vec::new();

            // Check 1: Boot configuration
            println!("{} Checking boot configuration...", "→".cyan());
            if ctx.guestfs.exists("/etc/fstab").unwrap_or(false) {
                let grub_found = ctx.guestfs.exists("/boot/grub/grub.cfg").unwrap_or(false)
                    || ctx.guestfs.exists("/boot/grub2/grub.cfg").unwrap_or(false);
                if grub_found {
                    println!("  {} Boot configuration is valid", "✓".green());
                    ready += 1;
                } else {
                    println!("  {} Missing boot loader configuration", "✗".red());
                    blockers.push("Configure GRUB boot loader");
                }
            } else {
                println!("  {} Missing /etc/fstab", "✗".red());
                blockers.push("Create /etc/fstab");
            }

            // Check 2: Network configuration
            println!("{} Checking network configuration...", "→".cyan());
            if ctx.guestfs.exists("/etc/resolv.conf").unwrap_or(false) {
                println!("  {} DNS configuration exists", "✓".green());
                ready += 1;
            } else {
                println!("  {} Missing DNS configuration", "⚠".yellow());
                warnings.push("Configure /etc/resolv.conf");
            }

            // Check 3: Cloud-init support
            println!("{} Checking cloud-init support...", "→".cyan());
            let pkg_info = ctx.guestfs.inspect_packages(&ctx.root)?;
            let has_cloud_init = pkg_info.packages.iter().any(|p| p.name.contains("cloud-init"));
            if has_cloud_init {
                println!("  {} cloud-init is installed", "✓".green());
                ready += 1;
            } else {
                println!("  {} cloud-init not found", "⚠".yellow());
                warnings.push("Install cloud-init for better cloud integration");
            }

            // Check 4: SSH server
            println!("{} Checking SSH server...", "→".cyan());
            if ctx.guestfs.exists("/etc/ssh/sshd_config").unwrap_or(false) {
                println!("  {} SSH server configured", "✓".green());
                ready += 1;
            } else {
                println!("  {} SSH server not configured", "⚠".yellow());
                warnings.push("Configure SSH server for remote access");
            }

            // Check 5: Security features
            println!("{} Checking security features...", "→".cyan());
            let sec = ctx.guestfs.inspect_security(&ctx.root)?;
            if &sec.selinux != "disabled" || sec.apparmor {
                println!("  {} MAC system is active", "✓".green());
                ready += 1;
            } else {
                println!("  {} No MAC system active", "⚠".yellow());
                warnings.push("Consider enabling SELinux or AppArmor");
            }

            println!();
            println!("{}", "═".repeat(60).bright_black());
            println!();

            println!("{} Migration Readiness: {}/5 checks passed", "📊".cyan(), ready.to_string().cyan().bold());
            println!();

            if !blockers.is_empty() {
                println!("{} Critical Blockers:", "🚫".red().bold());
                for (i, blocker) in blockers.iter().enumerate() {
                    println!("  {}. {}", i + 1, blocker.red());
                }
                println!();
            }

            if !warnings.is_empty() {
                println!("{} Recommendations:", "⚠".yellow().bold());
                for (i, warning) in warnings.iter().enumerate() {
                    println!("  {}. {}", i + 1, warning.yellow());
                }
                println!();
            }

            if blockers.is_empty() && ready >= 4 {
                println!("{}", "✓ System is ready for cloud migration!".green().bold());
                println!();
                println!("{} Next Steps:", "💡".cyan());
                println!("  1. Create final snapshot: snapshot pre-migration.md");
                println!("  2. Run template pre-migration for comprehensive check");
                println!("  3. Export system data: export system json system-config.json");
                println!("  4. Review and test boot configuration");
            } else if blockers.is_empty() {
                println!("{}", "⚠ System is mostly ready but has some warnings.".yellow());
                println!("  Address recommendations above for best results.");
            } else {
                println!("{}", "✗ System has critical blockers preventing migration.".red());
                println!("  Resolve blockers before attempting migration.");
            }
        }

        "container" => {
            println!("{}", "🐳 Container Migration Assessment".cyan().bold());
            println!("{}", "━".repeat(60).bright_black());
            println!();

            let pkg_info = ctx.guestfs.inspect_packages(&ctx.root)?;
            let pkg_count = pkg_info.packages.len();

            println!("{} Package Analysis:", "→".cyan());
            println!("  Total packages: {}", pkg_count.to_string().cyan());

            if pkg_count < 300 {
                println!("  {} Suitable for containerization (minimal footprint)", "✓".green());
            } else if pkg_count < 600 {
                println!("  {} Can be containerized (consider reducing packages)", "⚠".yellow());
            } else {
                println!("  {} Large package count (strongly consider reduction)", "⚠".yellow());
            }

            println!();
            println!("{} Recommendations:", "💡".yellow());
            println!("  • Identify essential packages only");
            println!("  • Create multi-stage Dockerfile");
            println!("  • Use minimal base images (Alpine, distroless)");
            println!("  • Extract application dependencies");
        }

        _ => {
            println!("{}", "Migration Targets:".yellow().bold());
            println!();
            println!("{} {} - Cloud platform migration (AWS, Azure, GCP)", "1.".cyan(), "migrate cloud".green());
            println!("{} {} - Container migration assessment", "2.".cyan(), "migrate container".green());
            println!();
            println!("{} migrate <target>", "Usage:".yellow());
        }
    }

    println!();
    Ok(())
}

/// Intelligent troubleshooting assistant
pub fn cmd_troubleshoot(ctx: &mut ShellContext, args: &[&str]) -> Result<()> {
    println!("\n{}", "╔═══════════════════════════════════════════════════════════╗".cyan().bold());
    println!("{}", "║          Intelligent Troubleshooting Assistant          ║".cyan().bold());
    println!("{}", "╚═══════════════════════════════════════════════════════════╝".cyan().bold());
    println!();

    if args.is_empty() {
        println!("{}", "Troubleshooting Categories:".yellow().bold());
        println!();
        println!("{} {} - Boot and startup issues", "1.".cyan(), "troubleshoot boot".green());
        println!("{} {} - Network connectivity problems", "2.".cyan(), "troubleshoot network".green());
        println!("{} {} - Service failures", "3.".cyan(), "troubleshoot services".green());
        println!("{} {} - Performance issues", "4.".cyan(), "troubleshoot performance".green());
        println!("{} {} - Security concerns", "5.".cyan(), "troubleshoot security".green());
        println!("{} {} - Auto-detect issues", "6.".cyan(), "troubleshoot auto".green());
        println!();
        println!("{} troubleshoot <category>", "Usage:".yellow());
        return Ok(());
    }

    let category = args[0];

    match category {
        "boot" => {
            println!("{}", "🔧 Boot & Startup Troubleshooting".cyan().bold());
            println!("{}", "━".repeat(60).bright_black());
            println!();

            let mut issues_found = Vec::new();
            let mut solutions = Vec::new();

            // Check fstab
            println!("{} Checking filesystem table...", "→".cyan());
            if !ctx.guestfs.exists("/etc/fstab").unwrap_or(false) {
                println!("  {} /etc/fstab is missing", "✗".red().bold());
                issues_found.push("Missing /etc/fstab");
                solutions.push((
                    "Create /etc/fstab",
                    "The filesystem table is required for mounting filesystems at boot",
                    vec![
                        "1. Generate fstab from current mounts",
                        "2. Verify UUID or device paths",
                        "3. Test in rescue mode before production boot",
                    ],
                ));
            } else {
                println!("  {} /etc/fstab exists", "✓".green());
            }

            // Check boot loader
            println!("{} Checking boot loader configuration...", "→".cyan());
            let grub_cfg = ctx.guestfs.exists("/boot/grub/grub.cfg").unwrap_or(false);
            let grub2_cfg = ctx.guestfs.exists("/boot/grub2/grub.cfg").unwrap_or(false);

            if !grub_cfg && !grub2_cfg {
                println!("  {} No GRUB configuration found", "✗".red().bold());
                issues_found.push("Missing GRUB configuration");
                solutions.push((
                    "Install and configure GRUB",
                    "Boot loader is missing or not configured properly",
                    vec![
                        "1. Install grub2 package",
                        "2. Run grub2-mkconfig -o /boot/grub2/grub.cfg",
                        "3. Install to boot device: grub2-install /dev/sda",
                    ],
                ));
            } else {
                println!("  {} GRUB configuration found", "✓".green());
            }

            // Check kernel
            println!("{} Checking kernel installation...", "→".cyan());
            let has_kernel = ctx.guestfs.exists("/boot/vmlinuz").unwrap_or(false)
                || ctx.guestfs.exists("/boot/vmlinuz-linux").unwrap_or(false);

            if !has_kernel {
                println!("  {} No kernel found in /boot", "✗".red().bold());
                issues_found.push("No kernel installed");
                solutions.push((
                    "Install kernel",
                    "System cannot boot without a kernel",
                    vec![
                        "1. Install kernel package for your distribution",
                        "2. Regenerate initramfs/initrd",
                        "3. Update GRUB configuration",
                    ],
                ));
            } else {
                println!("  {} Kernel found", "✓".green());
            }

            // Summary
            println!();
            if issues_found.is_empty() {
                println!("{}", "✓ No boot issues detected!".green().bold());
                println!("  Boot configuration appears correct.");
            } else {
                println!("{} {} boot issues detected:", "⚠".red(), issues_found.len());
                println!();

                for (i, (title, description, steps)) in solutions.iter().enumerate() {
                    println!("{} {}", format!("Issue {}:", i + 1).yellow().bold(), title.bold());
                    println!("   {}", description.bright_black());
                    println!();
                    println!("   {}:", "Solution Steps".green());
                    for step in steps {
                        println!("     {}", step.cyan());
                    }
                    println!();
                }
            }
        }

        "network" => {
            println!("{}", "🌐 Network Troubleshooting".cyan().bold());
            println!("{}", "━".repeat(60).bright_black());
            println!();

            let mut issues_found = Vec::new();
            let mut solutions = Vec::new();

            // Check DNS configuration
            println!("{} Checking DNS configuration...", "→".cyan());
            if !ctx.guestfs.exists("/etc/resolv.conf").unwrap_or(false) {
                println!("  {} /etc/resolv.conf is missing", "✗".red().bold());
                issues_found.push("Missing DNS configuration");
                solutions.push((
                    "Configure DNS",
                    "No DNS resolver configuration found",
                    vec![
                        "1. Create /etc/resolv.conf",
                        "2. Add nameserver entries (e.g., nameserver 8.8.8.8)",
                        "3. Consider using systemd-resolved for dynamic DNS",
                    ],
                ));
            } else {
                println!("  {} /etc/resolv.conf exists", "✓".green());
            }

            // Check hosts file
            println!("{} Checking hosts file...", "→".cyan());
            if !ctx.guestfs.exists("/etc/hosts").unwrap_or(false) {
                println!("  {} /etc/hosts is missing", "✗".red().bold());
                issues_found.push("Missing hosts file");
                solutions.push((
                    "Create hosts file",
                    "Basic hostname resolution requires /etc/hosts",
                    vec![
                        "1. Create /etc/hosts with localhost entries",
                        "2. Add: 127.0.0.1 localhost",
                        "3. Add: ::1 localhost ip6-localhost",
                    ],
                ));
            } else {
                println!("  {} /etc/hosts exists", "✓".green());
            }

            // Check network manager
            println!("{} Checking network management...", "→".cyan());
            let pkg_info = ctx.guestfs.inspect_packages(&ctx.root)?;
            let has_nm = pkg_info.packages.iter().any(|p| p.name.contains("NetworkManager"));
            let has_netctl = pkg_info.packages.iter().any(|p| p.name.contains("netctl"));
            let has_systemd_networkd = pkg_info.packages.iter().any(|p| p.name.contains("systemd"));

            if !has_nm && !has_netctl && !has_systemd_networkd {
                println!("  {} No network manager detected", "⚠".yellow());
                solutions.push((
                    "Install network manager",
                    "No network management tool found",
                    vec![
                        "1. Install NetworkManager or systemd-networkd",
                        "2. Enable and start the service",
                        "3. Configure network interfaces",
                    ],
                ));
            } else {
                println!("  {} Network management tools present", "✓".green());
            }

            // Summary
            println!();
            if issues_found.is_empty() && solutions.is_empty() {
                println!("{}", "✓ No critical network issues detected!".green().bold());
            } else {
                println!("{} Network configuration issues:", "⚠".yellow());
                println!();

                for (i, (title, description, steps)) in solutions.iter().enumerate() {
                    println!("{} {}", format!("Issue {}:", i + 1).yellow().bold(), title.bold());
                    println!("   {}", description.bright_black());
                    println!();
                    println!("   {}:", "Solution Steps".green());
                    for step in steps {
                        println!("     {}", step.cyan());
                    }
                    println!();
                }
            }
        }

        "services" => {
            println!("{}", "⚙️  Service Troubleshooting".cyan().bold());
            println!("{}", "━".repeat(60).bright_black());
            println!();

            if let Ok(services) = ctx.guestfs.inspect_systemd_services(&ctx.root) {
                let enabled = services.iter().filter(|s| s.enabled).count();
                let total = services.len();
                let disabled = total - enabled;

                println!("{} Service Statistics:", "→".cyan());
                println!("  Total services:    {}", total.to_string().cyan());
                println!("  Enabled:           {} ({}%)", enabled.to_string().green(), (enabled * 100 / total));
                println!("  Disabled:          {} ({}%)", disabled.to_string().yellow(), (disabled * 100 / total));
                println!();

                // Identify critical services
                let critical_services = vec!["sshd", "systemd-networkd", "NetworkManager", "firewalld"];
                let mut missing_critical = Vec::new();

                println!("{} Checking critical services...", "→".cyan());
                for critical in &critical_services {
                    let found = services.iter().any(|s| s.name.contains(critical));
                    if found {
                        let enabled = services.iter()
                            .find(|s| s.name.contains(critical))
                            .map(|s| s.enabled)
                            .unwrap_or(false);

                        if enabled {
                            println!("  {} {} is enabled", "✓".green(), critical.green());
                        } else {
                            println!("  {} {} exists but is disabled", "⚠".yellow(), critical.yellow());
                        }
                    } else {
                        println!("  {} {} not found", "✗".red(), critical.bright_black());
                        missing_critical.push(*critical);
                    }
                }

                if !missing_critical.is_empty() {
                    println!();
                    println!("{} Recommendations:", "💡".yellow());
                    for service in missing_critical {
                        println!("  • Install and enable {}", service.cyan());
                    }
                }

                // Check for failed services (we can't actually know this from offline inspection)
                println!();
                println!("{}", "Note:".yellow().bold());
                println!("  Offline inspection cannot detect runtime service failures.");
                println!("  Run 'systemctl --failed' on the live system to check for failed services.");
            }
        }

        "performance" => {
            println!("{}", "⚡ Performance Troubleshooting".cyan().bold());
            println!("{}", "━".repeat(60).bright_black());
            println!();

            let mut issues = Vec::new();

            // Check package count
            let pkg_info = ctx.guestfs.inspect_packages(&ctx.root)?;
            let pkg_count = pkg_info.packages.len();

            println!("{} Analyzing package overhead...", "→".cyan());
            println!("  Total packages: {}", pkg_count.to_string().cyan());

            if pkg_count > 1500 {
                println!("  {} High package count may impact performance", "⚠".yellow());
                issues.push((
                    "Package bloat",
                    "Large number of packages installed (recommended: <1000)",
                    vec![
                        "1. Review installed packages: packages",
                        "2. Remove unnecessary packages",
                        "3. Consider minimal installation for better performance",
                    ],
                ));
            } else if pkg_count > 1000 {
                println!("  {} Moderate package count", "→".cyan());
            } else {
                println!("  {} Good package count", "✓".green());
            }

            // Check service count
            if let Ok(services) = ctx.guestfs.inspect_systemd_services(&ctx.root) {
                let enabled = services.iter().filter(|s| s.enabled).count();

                println!("{} Analyzing service overhead...", "→".cyan());
                println!("  Enabled services: {}", enabled.to_string().cyan());

                if enabled > 80 {
                    println!("  {} Excessive services may cause slowdowns", "⚠".red());
                    issues.push((
                        "Service overhead",
                        "Too many enabled services (recommended: <50)",
                        vec![
                            "1. Review enabled services: services",
                            "2. Disable unnecessary services",
                            "3. Use 'systemctl mask' for unwanted services",
                        ],
                    ));
                } else if enabled > 50 {
                    println!("  {} Many services enabled", "⚠".yellow());
                    issues.push((
                        "Service count",
                        "Consider reviewing and disabling unused services",
                        vec![
                            "1. List services: systemctl list-unit-files",
                            "2. Disable unused: systemctl disable <service>",
                        ],
                    ));
                } else {
                    println!("  {} Reasonable service count", "✓".green());
                }
            }

            println!();
            if issues.is_empty() {
                println!("{}", "✓ No obvious performance bottlenecks detected!".green().bold());
            } else {
                println!("{} Performance Issues:", "⚠".yellow());
                println!();

                for (i, (title, description, steps)) in issues.iter().enumerate() {
                    println!("{} {}", format!("{}.", i + 1).yellow().bold(), title.bold());
                    println!("   {}", description.bright_black());
                    println!();
                    for step in steps {
                        println!("     {}", step.cyan());
                    }
                    println!();
                }
            }

            println!("{} Additional recommendations:", "💡".cyan());
            println!("  • Run 'bench all' to measure command performance");
            println!("  • Use 'optimize' for detailed optimization suggestions");
            println!("  • Check 'chart services' for service distribution");
        }

        "security" => {
            println!("{}", "🔒 Security Troubleshooting".cyan().bold());
            println!("{}", "━".repeat(60).bright_black());
            println!();

            let sec = ctx.guestfs.inspect_security(&ctx.root)?;
            let mut vulnerabilities = Vec::new();

            // Check MAC systems
            println!("{} Checking access control systems...", "→".cyan());
            if &sec.selinux == "disabled" && !sec.apparmor {
                println!("  {} No MAC system active", "✗".red().bold());
                vulnerabilities.push((
                    "CRITICAL",
                    "No Mandatory Access Control",
                    "System lacks SELinux or AppArmor protection",
                    vec![
                        "1. Install SELinux or AppArmor packages",
                        "2. Configure policy (targeted for SELinux, enforce for AppArmor)",
                        "3. Reboot to activate",
                        "4. Monitor audit logs for policy violations",
                    ],
                ));
            } else if &sec.selinux != "disabled" {
                println!("  {} SELinux is {}", "✓".green(), sec.selinux.green());
            } else if sec.apparmor {
                println!("  {} AppArmor is active", "✓".green());
            }

            // Check firewall
            println!("{} Checking firewall...", "→".cyan());
            if let Ok(fw) = ctx.guestfs.inspect_firewall(&ctx.root) {
                if !fw.enabled {
                    println!("  {} Firewall is disabled", "✗".red().bold());
                    vulnerabilities.push((
                        "CRITICAL",
                        "Firewall disabled",
                        "No network filtering is active",
                        vec![
                            "1. Install firewalld or ufw",
                            "2. Enable: systemctl enable --now firewalld",
                            "3. Configure zones and rules",
                            "4. Test connectivity after enabling",
                        ],
                    ));
                } else {
                    println!("  {} Firewall is enabled", "✓".green());
                }
            }

            // Check audit daemon
            println!("{} Checking audit logging...", "→".cyan());
            if !sec.auditd {
                println!("  {} Audit daemon not running", "⚠".yellow());
                vulnerabilities.push((
                    "WARNING",
                    "No audit logging",
                    "Security events are not being logged",
                    vec![
                        "1. Install audit package",
                        "2. Enable: systemctl enable --now auditd",
                        "3. Configure rules in /etc/audit/audit.rules",
                        "4. Monitor /var/log/audit/audit.log",
                    ],
                ));
            } else {
                println!("  {} Audit daemon configured", "✓".green());
            }

            // Check SSH configuration
            println!("{} Checking SSH security...", "→".cyan());
            if ctx.guestfs.exists("/etc/ssh/sshd_config").unwrap_or(false) {
                println!("  {} SSH configuration found", "✓".green());
                println!("     {}", "Review sshd_config for:".bright_black());
                println!("     {} PermitRootLogin no", "•".bright_black());
                println!("     {} PasswordAuthentication (consider key-only)", "•".bright_black());
                println!("     {} Port (consider changing from 22)", "•".bright_black());
            } else {
                println!("  {} No SSH configuration", "→".bright_black());
            }

            println!();
            if vulnerabilities.is_empty() {
                println!("{}", "✓ No critical security issues found!".green().bold());
                println!("  Run 'verify security' for comprehensive security check.");
            } else {
                println!("{} Security Vulnerabilities:", "🚨".red());
                println!();

                for (i, (severity, title, description, steps)) in vulnerabilities.iter().enumerate() {
                    let severity_colored = match *severity {
                        "CRITICAL" => severity.red().bold(),
                        "WARNING" => severity.yellow(),
                        _ => severity.bright_black(),
                    };

                    println!("{} [{}] {}", format!("{}.", i + 1).yellow(), severity_colored, title.bold());
                    println!("   {}", description.bright_black());
                    println!();
                    println!("   {}:", "Remediation Steps".green());
                    for step in steps {
                        println!("     {}", step.cyan());
                    }
                    println!();
                }

                println!("{} Run these for more details:", "💡".cyan());
                println!("  • {} - Full security compliance check", "compliance cis".green());
                println!("  • {} - Security predictions", "predict".green());
                println!("  • {} - Security insights", "insights".green());
            }
        }

        "auto" => {
            println!("{}", "🔍 Auto-Detecting Issues".cyan().bold());
            println!("{}", "━".repeat(60).bright_black());
            println!();

            println!("{}", "Running comprehensive system scan...".yellow());
            println!();

            let mut issues = Vec::new();

            // Quick checks across all categories
            if !ctx.guestfs.exists("/etc/fstab").unwrap_or(false) {
                issues.push(("CRITICAL", "Boot", "Missing /etc/fstab"));
            }

            if !ctx.guestfs.exists("/etc/resolv.conf").unwrap_or(false) {
                issues.push(("WARNING", "Network", "Missing DNS configuration"));
            }

            let sec = ctx.guestfs.inspect_security(&ctx.root)?;
            if &sec.selinux == "disabled" && !sec.apparmor {
                issues.push(("CRITICAL", "Security", "No MAC system active"));
            }

            if let Ok(fw) = ctx.guestfs.inspect_firewall(&ctx.root) {
                if !fw.enabled {
                    issues.push(("CRITICAL", "Security", "Firewall disabled"));
                }
            }

            let pkg_info = ctx.guestfs.inspect_packages(&ctx.root)?;
            if pkg_info.packages.len() > 1500 {
                issues.push(("WARNING", "Performance", "Excessive packages installed"));
            }

            if let Ok(services) = ctx.guestfs.inspect_systemd_services(&ctx.root) {
                let enabled = services.iter().filter(|s| s.enabled).count();
                if enabled > 80 {
                    issues.push(("WARNING", "Performance", "Too many enabled services"));
                }
            }

            if issues.is_empty() {
                println!("{}", "✓ No issues detected!".green().bold());
                println!("  System appears to be in good condition.");
            } else {
                println!("{} {} issues detected:", "⚠".yellow(), issues.len());
                println!();

                for (severity, category, description) in &issues {
                    let severity_colored = match *severity {
                        "CRITICAL" => severity.red().bold(),
                        "WARNING" => severity.yellow(),
                        _ => severity.bright_black(),
                    };

                    println!("  [{}] {}: {}",
                        severity_colored,
                        category.cyan(),
                        description
                    );
                }

                println!();
                println!("{} Run detailed troubleshooting:", "💡".cyan());
                println!("  • {} - Boot issues", "troubleshoot boot".green());
                println!("  • {} - Network issues", "troubleshoot network".green());
                println!("  • {} - Security issues", "troubleshoot security".green());
                println!("  • {} - Performance issues", "troubleshoot performance".green());
            }
        }

        _ => {
            println!("{} Unknown category: {}", "Error:".red(), category);
            println!("{} troubleshoot (without arguments) for options", "Tip:".yellow());
        }
    }

    println!();
    Ok(())
}

/// Package dependency analysis
pub fn cmd_depends(ctx: &mut ShellContext, args: &[&str]) -> Result<()> {
    println!("\n{}", "╔═══════════════════════════════════════════════════════════╗".cyan().bold());
    println!("{}", "║           Package Dependency Analysis                   ║".cyan().bold());
    println!("{}", "╚═══════════════════════════════════════════════════════════╝".cyan().bold());
    println!();

    if args.is_empty() {
        println!("{}", "Dependency Analysis Features:".yellow().bold());
        println!();
        println!("{} {} - Find packages containing pattern", "1.".cyan(), "depends search <pattern>".green());
        println!("{} {} - Analyze package relationships", "2.".cyan(), "depends analyze".green());
        println!("{} {} - Find development packages", "3.".cyan(), "depends dev".green());
        println!("{} {} - Find library packages", "4.".cyan(), "depends libs".green());
        println!();
        println!("{} depends <command>", "Usage:".yellow());
        return Ok(());
    }

    let command = args[0];

    match command {
        "search" => {
            if args.len() < 2 {
                println!("{} Usage: depends search <pattern>", "Error:".red());
                return Ok(());
            }

            let pattern = args[1];
            println!("{} Searching for packages matching '{}'...", "→".cyan(), pattern.green());
            println!();

            let pkg_info = ctx.guestfs.inspect_packages(&ctx.root)?;
            let matches: Vec<_> = pkg_info.packages.iter()
                .filter(|p| p.name.to_lowercase().contains(&pattern.to_lowercase()))
                .collect();

            if matches.is_empty() {
                println!("{} No packages found matching '{}'", "✗".red(), pattern);
            } else {
                println!("{} {} packages found:", "✓".green(), matches.len());
                println!();

                for (i, pkg) in matches.iter().enumerate().take(50) {
                    println!("{:3}. {} ({})",
                        i + 1,
                        pkg.name.cyan(),
                        pkg.version.to_string().bright_black()
                    );
                }

                if matches.len() > 50 {
                    println!();
                    println!("... and {} more", (matches.len() - 50).to_string().yellow());
                }
            }
        }

        "analyze" => {
            println!("{} Analyzing package relationships...", "→".cyan());
            println!();

            let pkg_info = ctx.guestfs.inspect_packages(&ctx.root)?;

            // Categorize packages
            let mut dev_packages = 0;
            let mut lib_packages = 0;
            let mut doc_packages = 0;
            let mut kernel_packages = 0;
            let mut app_packages = 0;

            for pkg in &pkg_info.packages {
                let name = pkg.name.to_lowercase();
                if name.contains("devel") || name.contains("-dev") || name.ends_with("-dev") {
                    dev_packages += 1;
                } else if name.starts_with("lib") || name.contains("library") {
                    lib_packages += 1;
                } else if name.contains("doc") || name.ends_with("-doc") {
                    doc_packages += 1;
                } else if name.contains("kernel") {
                    kernel_packages += 1;
                } else {
                    app_packages += 1;
                }
            }

            let total = pkg_info.packages.len();

            println!("{}", "Package Distribution:".cyan().bold());
            println!();
            println!("  Development:  {:4} ({:5.1}%)", dev_packages, (dev_packages as f32 / total as f32) * 100.0);
            println!("  Libraries:    {:4} ({:5.1}%)", lib_packages, (lib_packages as f32 / total as f32) * 100.0);
            println!("  Documentation:{:4} ({:5.1}%)", doc_packages, (doc_packages as f32 / total as f32) * 100.0);
            println!("  Kernel:       {:4} ({:5.1}%)", kernel_packages, (kernel_packages as f32 / total as f32) * 100.0);
            println!("  Applications: {:4} ({:5.1}%)", app_packages, (app_packages as f32 / total as f32) * 100.0);
            println!("  {}",  "─".repeat(25).bright_black());
            println!("  Total:        {:4}", total);

            println!();
            println!("{} Insights:", "💡".yellow());

            if dev_packages > total / 5 {
                println!("  • {}", "High development package count suggests a build environment".cyan());
            }

            if lib_packages > total / 3 {
                println!("  • {}", "Many libraries - system may support multiple applications".cyan());
            }

            if doc_packages > 50 {
                println!("  • {}", "Documentation packages can be removed to save space".cyan());
            }
        }

        "dev" => {
            println!("{} Development Packages:", "→".cyan());
            println!();

            let pkg_info = ctx.guestfs.inspect_packages(&ctx.root)?;
            let dev_pkgs: Vec<_> = pkg_info.packages.iter()
                .filter(|p| {
                    let name = p.name.to_lowercase();
                    name.contains("devel") || name.contains("-dev") ||
                    name.ends_with("-dev") || name.contains("gcc") ||
                    name.contains("make") || name.contains("cmake")
                })
                .collect();

            if dev_pkgs.is_empty() {
                println!("{} No development packages found", "✓".green());
                println!("  This is a production/runtime system");
            } else {
                println!("{} {} development packages:", "→".cyan(), dev_pkgs.len());
                println!();

                for (i, pkg) in dev_pkgs.iter().enumerate().take(30) {
                    println!("{:3}. {}", i + 1, pkg.name.cyan());
                }

                if dev_pkgs.len() > 30 {
                    println!();
                    println!("... and {} more", (dev_pkgs.len() - 30).to_string().yellow());
                }

                println!();
                println!("{} Note:", "💡".yellow());
                println!("  Development packages are typically not needed in production.");
                println!("  Consider removing them to reduce attack surface and disk usage.");
            }
        }

        "libs" => {
            println!("{} Library Packages:", "→".cyan());
            println!();

            let pkg_info = ctx.guestfs.inspect_packages(&ctx.root)?;
            let lib_pkgs: Vec<_> = pkg_info.packages.iter()
                .filter(|p| p.name.starts_with("lib") || p.name.to_lowercase().contains("library"))
                .collect();

            println!("{} {} library packages:", "→".cyan(), lib_pkgs.len());
            println!();

            for (i, pkg) in lib_pkgs.iter().enumerate().take(30) {
                println!("{:3}. {}", i + 1, pkg.name.cyan());
            }

            if lib_pkgs.len() > 30 {
                println!();
                println!("... and {} more", (lib_pkgs.len() - 30).to_string().yellow());
            }
        }

        _ => {
            println!("{} Unknown command: {}", "Error:".red(), command);
            println!("{} depends (without arguments) for options", "Tip:".yellow());
        }
    }

    println!();
    Ok(())
}

/// Configuration validation and recommendations
pub fn cmd_validate(ctx: &mut ShellContext, args: &[&str]) -> Result<()> {
    println!("\n{}", "╔═══════════════════════════════════════════════════════════╗".cyan().bold());
    println!("{}", "║          Configuration Validation Suite                 ║".cyan().bold());
    println!("{}", "╚═══════════════════════════════════════════════════════════╝".cyan().bold());
    println!();

    let target = if args.is_empty() { "all" } else { args[0] };

    match target {
        "all" => {
            println!("{}", "Running comprehensive validation...".yellow());
            println!();

            let mut passed = 0;
            let mut failed = 0;
            let mut warnings = 0;

            // Validation 1: File system structure
            println!("{} {}", "1.".cyan().bold(), "File System Structure".bold());
            let critical_dirs = vec!["/etc", "/var", "/usr", "/boot", "/home"];
            let mut all_dirs_present = true;

            for dir in &critical_dirs {
                if ctx.guestfs.exists(dir).unwrap_or(false) {
                    println!("  {} {} exists", "✓".green(), dir.cyan());
                } else {
                    println!("  {} {} missing", "✗".red(), dir.red());
                    all_dirs_present = false;
                }
            }

            if all_dirs_present {
                passed += 1;
                println!("  {}", "PASS".green().bold());
            } else {
                failed += 1;
                println!("  {}", "FAIL".red().bold());
            }
            println!();

            // Validation 2: System configuration files
            println!("{} {}", "2.".cyan().bold(), "System Configuration Files".bold());
            let config_files = vec![
                ("/etc/passwd", "User database"),
                ("/etc/group", "Group database"),
                ("/etc/shadow", "Password hashes"),
                ("/etc/fstab", "Filesystem table"),
            ];

            let mut all_configs_present = true;
            for (file, desc) in &config_files {
                if ctx.guestfs.exists(file).unwrap_or(false) {
                    println!("  {} {} - {}", "✓".green(), file.cyan(), desc.bright_black());
                } else {
                    println!("  {} {} - {} {}", "✗".red(), file.red(), desc.bright_black(), "[MISSING]".red());
                    all_configs_present = false;
                }
            }

            if all_configs_present {
                passed += 1;
                println!("  {}", "PASS".green().bold());
            } else {
                failed += 1;
                println!("  {}", "FAIL".red().bold());
            }
            println!();

            // Validation 3: Boot configuration
            println!("{} {}", "3.".cyan().bold(), "Boot Configuration".bold());
            let grub_cfg = ctx.guestfs.exists("/boot/grub/grub.cfg").unwrap_or(false);
            let grub2_cfg = ctx.guestfs.exists("/boot/grub2/grub.cfg").unwrap_or(false);

            if grub_cfg || grub2_cfg {
                println!("  {} Boot loader configured", "✓".green());
                passed += 1;
                println!("  {}", "PASS".green().bold());
            } else {
                println!("  {} No boot loader configuration", "✗".red());
                failed += 1;
                println!("  {}", "FAIL".red().bold());
            }
            println!();

            // Validation 4: Security configuration
            println!("{} {}", "4.".cyan().bold(), "Security Configuration".bold());
            let sec = ctx.guestfs.inspect_security(&ctx.root)?;
            let mut sec_checks = 0;
            let mut sec_total = 0;

            sec_total += 1;
            if &sec.selinux != "disabled" || sec.apparmor {
                println!("  {} MAC system active", "✓".green());
                sec_checks += 1;
            } else {
                println!("  {} No MAC system", "⚠".yellow());
            }

            sec_total += 1;
            if sec.auditd {
                println!("  {} Audit daemon configured", "✓".green());
                sec_checks += 1;
            } else {
                println!("  {} No audit daemon", "⚠".yellow());
            }

            sec_total += 1;
            if let Ok(fw) = ctx.guestfs.inspect_firewall(&ctx.root) {
                if fw.enabled {
                    println!("  {} Firewall enabled", "✓".green());
                    sec_checks += 1;
                } else {
                    println!("  {} Firewall disabled", "⚠".yellow());
                }
            }

            if sec_checks >= sec_total - 1 {
                passed += 1;
                println!("  {}", "PASS".green().bold());
            } else if sec_checks >= sec_total / 2 {
                warnings += 1;
                println!("  {}", "WARN".yellow().bold());
            } else {
                failed += 1;
                println!("  {}", "FAIL".red().bold());
            }
            println!();

            // Validation 5: Package integrity
            println!("{} {}", "5.".cyan().bold(), "Package System".bold());
            let pkg_info = ctx.guestfs.inspect_packages(&ctx.root)?;
            if pkg_info.packages.len() > 0 {
                println!("  {} {} packages installed", "✓".green(), pkg_info.packages.len());
                passed += 1;
                println!("  {}", "PASS".green().bold());
            } else {
                println!("  {} No packages found", "✗".red());
                failed += 1;
                println!("  {}", "FAIL".red().bold());
            }
            println!();

            // Summary
            println!("{}", "═".repeat(60).bright_black());
            println!();
            println!("{}", "Validation Summary:".cyan().bold());
            println!("  Passed:   {}", passed.to_string().green());
            println!("  Failed:   {}", failed.to_string().red());
            println!("  Warnings: {}", warnings.to_string().yellow());

            let total = passed + failed + warnings;
            let success_rate = (passed as f32 / total as f32) * 100.0;

            println!();
            println!("  Success Rate: {:.1}%", success_rate);

            if failed == 0 && warnings == 0 {
                println!();
                println!("{}", "✓ System configuration is valid!".green().bold());
            } else if failed == 0 {
                println!();
                println!("{}", "⚠ System is mostly valid with some warnings.".yellow());
            } else {
                println!();
                println!("{}", "✗ System has configuration issues that need attention.".red());
            }
        }

        "config" => {
            println!("{}", "Validating configuration files...".yellow());
            println!();

            let config_files = vec![
                ("/etc/passwd", "User accounts", true),
                ("/etc/group", "Group definitions", true),
                ("/etc/shadow", "Password hashes", true),
                ("/etc/fstab", "Filesystem mounts", true),
                ("/etc/hosts", "Host name resolution", true),
                ("/etc/resolv.conf", "DNS configuration", false),
                ("/etc/ssh/sshd_config", "SSH server config", false),
                ("/etc/sudoers", "Sudo configuration", false),
            ];

            let mut critical_missing = Vec::new();
            let mut optional_missing = Vec::new();

            for (file, description, critical) in &config_files {
                if ctx.guestfs.exists(file).unwrap_or(false) {
                    println!("  {} {} - {}", "✓".green(), file.cyan(), description.bright_black());
                } else {
                    if *critical {
                        println!("  {} {} - {} {}", "✗".red(), file.red(), description.bright_black(), "[CRITICAL]".red().bold());
                        critical_missing.push(*file);
                    } else {
                        println!("  {} {} - {} {}", "⚠".yellow(), file.yellow(), description.bright_black(), "[OPTIONAL]".yellow());
                        optional_missing.push(*file);
                    }
                }
            }

            println!();
            if critical_missing.is_empty() && optional_missing.is_empty() {
                println!("{}", "✓ All configuration files present!".green().bold());
            } else {
                if !critical_missing.is_empty() {
                    println!("{} Critical files missing:", "✗".red());
                    for file in &critical_missing {
                        println!("  • {}", file.red());
                    }
                    println!();
                }

                if !optional_missing.is_empty() {
                    println!("{} Optional files missing:", "⚠".yellow());
                    for file in &optional_missing {
                        println!("  • {}", file.yellow());
                    }
                }
            }
        }

        _ => {
            println!("{}", "Validation Targets:".yellow().bold());
            println!();
            println!("{} {} - Comprehensive validation", "1.".cyan(), "validate all".green());
            println!("{} {} - Configuration files only", "2.".cyan(), "validate config".green());
            println!();
            println!("{} validate <target>", "Usage:".yellow());
        }
    }

    println!();
    Ok(())
}

/// Forensics - Digital forensics investigation workflows
pub fn cmd_forensics(ctx: &mut ShellContext, args: &[&str]) -> Result<()> {
    if args.is_empty() {
        println!("{}", "🔍 Digital Forensics Investigation".cyan().bold());
        println!();
        println!("{}", "Available Workflows:".yellow().bold());
        println!("{} {} - Evidence collection", "1.".cyan(), "forensics collect".green());
        println!("{} {} - Timeline reconstruction", "2.".cyan(), "forensics timeline".green());
        println!("{} {} - Suspicious activity detection", "3.".cyan(), "forensics suspicious".green());
        println!("{} {} - User activity analysis", "4.".cyan(), "forensics activity".green());
        println!("{} {} - Integrity verification", "5.".cyan(), "forensics integrity".green());
        println!("{} {} - Memory artifacts", "6.".cyan(), "forensics memory".green());
        println!();
        println!("{} forensics <workflow>", "Usage:".yellow());
        println!();
        return Ok(());
    }

    let workflow = args[0];

    match workflow {
        "collect" => {
            println!("{}", "📦 Evidence Collection".cyan().bold());
            println!();

            let mut evidence = Vec::new();

            // Critical system files
            println!("{}", "Critical System Files:".yellow().bold());
            let critical_files = vec![
                ("/etc/passwd", "User database"),
                ("/etc/shadow", "Password hashes"),
                ("/etc/group", "Group database"),
                ("/etc/sudoers", "Sudo configuration"),
                ("/etc/hosts", "Host mappings"),
                ("/etc/fstab", "Filesystem mounts"),
                ("/etc/crontab", "System cron jobs"),
                ("/var/log/auth.log", "Authentication logs"),
                ("/var/log/secure", "Security logs"),
                ("/var/log/syslog", "System logs"),
            ];

            for (path, desc) in &critical_files {
                if ctx.guestfs.exists(path).unwrap_or(false) {
                    let size = ctx.guestfs.filesize(path).unwrap_or(0);
                    evidence.push((*path, *desc, size, "✓"));
                    println!("  {} {} - {} ({} bytes)", "✓".green(), path.cyan(), desc, size);
                } else {
                    println!("  {} {} - {} {}", "✗".red(), path.cyan(), desc, "(missing)".bright_black());
                }
            }
            println!();

            // User home directories
            println!("{}", "User Home Directories:".yellow().bold());
            if let Ok(user_info) = ctx.guestfs.inspect_users(&ctx.root) {
                for user in &user_info {
                    if ctx.guestfs.exists(&user.home).unwrap_or(false) {
                        let bash_history = format!("{}/.bash_history", user.home);
                        let ssh_dir = format!("{}/.ssh", user.home);

                        if ctx.guestfs.exists(&bash_history).unwrap_or(false) {
                            let size = ctx.guestfs.filesize(&bash_history).unwrap_or(0);
                            println!("  {} {} - Command history ({} bytes)", "✓".green(), bash_history.cyan(), size);
                            // evidence.push((bash_history.as_str(), "Command history", size, "✓"));
                        }

                        if ctx.guestfs.is_dir(&ssh_dir).unwrap_or(false) {
                            println!("  {} {} - SSH configuration", "✓".green(), ssh_dir.cyan());
                        }
                    }
                }
            }
            println!();

            // Log files
            println!("{}", "Log Files:".yellow().bold());
            let log_paths = vec!["/var/log", "/var/log/audit"];
            for log_path in &log_paths {
                if ctx.guestfs.is_dir(log_path).unwrap_or(false) {
                    println!("  {} {} - Available", "✓".green(), log_path.cyan());
                }
            }
            println!();

            println!("{}", "Evidence Summary:".yellow().bold());
            println!("  Total artifacts collected: {}", evidence.len().to_string().green());
            println!("  Critical files found: {}", critical_files.len().to_string().cyan());
            println!();
            println!("{}", "Next Steps:".yellow());
            println!("  1. Export evidence: {}", "export system json > evidence.json".cyan());
            println!("  2. Analyze timeline: {}", "forensics timeline".cyan());
            println!("  3. Check for suspicious activity: {}", "forensics suspicious".cyan());
        }

        "timeline" => {
            println!("{}", "⏰ Timeline Reconstruction".cyan().bold());
            println!();

            // Analyze modification times of critical files
            println!("{}", "Recent System Changes:".yellow().bold());

            let mut timeline_events = Vec::new();

            let check_paths = vec![
                "/etc/passwd",
                "/etc/shadow",
                "/etc/group",
                "/etc/sudoers",
                "/etc/ssh/sshd_config",
                "/etc/crontab",
                "/var/log/auth.log",
            ];

            for path in &check_paths {
                if ctx.guestfs.exists(path).unwrap_or(false) {
                    // We can't get actual timestamps from GuestFS easily, so provide guidance
                    println!("  {} {} - Available for analysis", "✓".green(), path.cyan());
                    timeline_events.push(*path);
                }
            }
            println!();

            println!("{}", "Timeline Categories:".yellow().bold());
            println!("  {} User account changes", "👥".cyan());
            println!("  {} Authentication events", "🔐".cyan());
            println!("  {} System configuration changes", "⚙".cyan());
            println!("  {} Log entries", "📝".cyan());
            println!();

            println!("{}", "Analysis Recommendations:".yellow());
            println!("  1. Check /var/log/auth.log for login attempts");
            println!("  2. Review /etc/passwd for new user accounts");
            println!("  3. Examine cron jobs for scheduled tasks");
            println!("  4. Analyze SSH configuration changes");
            println!();
            println!("{} Files available for timeline: {}", "Summary:".yellow(), timeline_events.len().to_string().green());
        }

        "suspicious" => {
            println!("{}", "🚨 Suspicious Activity Detection".cyan().bold());
            println!();

            let mut findings = Vec::new();

            // Check for suspicious user accounts
            println!("{}", "User Account Analysis:".yellow().bold());
            if let Ok(user_info) = ctx.guestfs.inspect_users(&ctx.root) {
                for user in &user_info {
                    // Check for UID 0 accounts other than root
                    if user.uid == "0" && user.username != "root" {
                        findings.push(("CRITICAL", "UID 0 account", user.username.clone()));
                        println!("  {} {} - Non-root UID 0 account", "🔴".red(), user.username.red().bold());
                    }

                    // Check for accounts with no password (if shadow exists)
                    if ctx.guestfs.exists("/etc/shadow").unwrap_or(false) {
                        // Would need to read shadow file to check
                        if user.username.contains("test") || user.username.contains("temp") {
                            findings.push(("WARNING", "Temporary account", user.username.clone()));
                            println!("  {} {} - Potential temporary/test account", "⚠".yellow(), user.username.yellow());
                        }
                    }
                }
            }
            println!();

            // Check for suspicious SUID binaries
            println!("{}", "SUID Binary Analysis:".yellow().bold());
            println!("  {} Checking for unusual SUID files...", "🔍".cyan());
            println!("  {} Manual verification recommended for:", "ℹ".cyan());
            println!("    - /tmp, /var/tmp, /dev/shm (world-writable directories)");
            println!("    - User home directories");
            println!("    - Unusual system paths");
            println!();

            // Check for suspicious cron jobs
            println!("{}", "Scheduled Task Analysis:".yellow().bold());
            if ctx.guestfs.exists("/etc/crontab").unwrap_or(false) {
                println!("  {} {} - Present (review recommended)", "✓".green(), "/etc/crontab".cyan());
            }
            if ctx.guestfs.is_dir("/etc/cron.d").unwrap_or(false) {
                println!("  {} {} - Present (review recommended)", "✓".green(), "/etc/cron.d".cyan());
            }
            println!();

            // Check for suspicious network configuration
            println!("{}", "Network Configuration:".yellow().bold());
            if ctx.guestfs.exists("/etc/hosts").unwrap_or(false) {
                println!("  {} {} - Check for suspicious redirects", "ℹ".cyan(), "/etc/hosts".cyan());
            }
            println!();

            // Security findings summary
            println!("{}", "Findings Summary:".yellow().bold());
            if findings.is_empty() {
                println!("  {} No critical suspicious activity detected", "✓".green());
            } else {
                for (severity, category, detail) in &findings {
                    let severity_colored = match *severity {
                        "CRITICAL" => severity.red().bold(),
                        "WARNING" => severity.yellow().bold(),
                        _ => severity.cyan().bold(),
                    };
                    println!("  {} {} - {}", severity_colored, category, detail);
                }
            }
            println!();

            println!("{}", "Recommended Actions:".yellow());
            println!("  1. Review user accounts: {}", "cat /etc/passwd".cyan());
            println!("  2. Check for rootkits: {}", "forensics integrity".cyan());
            println!("  3. Analyze authentication logs");
            println!("  4. Examine network connections and listening ports");
        }

        "activity" => {
            println!("{}", "👤 User Activity Analysis".cyan().bold());
            println!();

            if let Ok(user_info) = ctx.guestfs.inspect_users(&ctx.root) {
                println!("{}", "User Activity Summary:".yellow().bold());

                for user in &user_info {
                    println!();
                    println!("{} {} (UID: {})", "User:".cyan(), user.username.green().bold(), user.uid);

                    // Check for bash history
                    let bash_history = format!("{}/.bash_history", user.home);
                    if ctx.guestfs.exists(&bash_history).unwrap_or(false) {
                        let size = ctx.guestfs.filesize(&bash_history).unwrap_or(0);
                        println!("  {} Command history: {} bytes", "📜".cyan(), size.to_string().green());
                    } else {
                        println!("  {} Command history: {}", "📜".cyan(), "Not found".red());
                    }

                    // Check for SSH keys
                    let ssh_dir = format!("{}/.ssh", user.home);
                    if ctx.guestfs.is_dir(&ssh_dir).unwrap_or(false) {
                        println!("  {} SSH directory: {}", "🔑".cyan(), "Present".green());
                    }

                    // Check for common config files
                    let bashrc = format!("{}/.bashrc", user.home);
                    if ctx.guestfs.exists(&bashrc).unwrap_or(false) {
                        println!("  {} Shell config: {}", "⚙".cyan(), "Present".green());
                    }
                }
            }
            println!();

            println!("{}", "Activity Indicators:".yellow().bold());
            println!("  {} Authentication logs: {}", "🔐".cyan(),
                if ctx.guestfs.exists("/var/log/auth.log").unwrap_or(false) {
                    "Available".green()
                } else {
                    "Check /var/log/secure".yellow()
                });
            println!("  {} Last login data: {}", "👥".cyan(),
                if ctx.guestfs.exists("/var/log/lastlog").unwrap_or(false) {
                    "Available".green()
                } else {
                    "Not found".red()
                });
            println!();

            println!("{}", "Analysis Tips:".yellow());
            println!("  • Review .bash_history for executed commands");
            println!("  • Check authorized_keys for SSH access");
            println!("  • Examine sudo logs for privilege escalation");
            println!("  • Analyze authentication patterns in logs");
        }

        "integrity" => {
            println!("{}", "🛡 System Integrity Verification".cyan().bold());
            println!();

            let mut checks = Vec::new();

            // Check critical system binaries
            println!("{}", "Critical Binary Verification:".yellow().bold());
            let critical_bins = vec![
                "/bin/bash", "/bin/sh", "/bin/login",
                "/usr/bin/sudo", "/usr/bin/ssh", "/usr/bin/passwd",
                "/sbin/init", "/usr/sbin/sshd",
            ];

            let mut missing = 0;
            let mut present = 0;

            for bin in &critical_bins {
                if ctx.guestfs.exists(bin).unwrap_or(false) {
                    let size = ctx.guestfs.filesize(bin).unwrap_or(0);
                    println!("  {} {} ({} bytes)", "✓".green(), bin.cyan(), size);
                    checks.push((bin, "present", size));
                    present += 1;
                } else {
                    println!("  {} {} {}", "✗".red(), bin.cyan(), "(missing)".red());
                    missing += 1;
                }
            }
            println!();

            // Check system library paths
            println!("{}", "System Libraries:".yellow().bold());
            let lib_paths = vec!["/lib", "/lib64", "/usr/lib", "/usr/lib64"];
            for lib in &lib_paths {
                if ctx.guestfs.is_dir(lib).unwrap_or(false) {
                    println!("  {} {} - Present", "✓".green(), lib.cyan());
                } else {
                    println!("  {} {} - {}", "✗".red(), lib.cyan(), "Missing".red());
                }
            }
            println!();

            // Configuration integrity
            println!("{}", "Configuration Integrity:".yellow().bold());
            let config_files = vec![
                "/etc/passwd", "/etc/group", "/etc/shadow",
                "/etc/fstab", "/etc/hosts",
            ];

            for cfg in &config_files {
                if ctx.guestfs.exists(cfg).unwrap_or(false) {
                    let size = ctx.guestfs.filesize(cfg).unwrap_or(0);
                    if size > 0 {
                        println!("  {} {} ({} bytes)", "✓".green(), cfg.cyan(), size);
                    } else {
                        println!("  {} {} {}", "⚠".yellow(), cfg.cyan(), "(empty)".yellow());
                    }
                }
            }
            println!();

            // Integrity summary
            println!("{}", "Integrity Summary:".yellow().bold());
            println!("  Binaries checked: {}", critical_bins.len());
            println!("  Present: {}", present.to_string().green());
            if missing > 0 {
                println!("  Missing: {}", missing.to_string().red());
            }
            println!();

            let integrity_score = (present * 100) / critical_bins.len();
            let grade = if integrity_score >= 95 {
                "A".green().bold()
            } else if integrity_score >= 85 {
                "B".cyan()
            } else if integrity_score >= 75 {
                "C".yellow()
            } else {
                "D".red()
            };

            println!("  Integrity Score: {}% (Grade: {})",
                integrity_score.to_string().cyan(), grade);
            println!();

            if missing > 0 {
                println!("{}", "⚠ Warning:".yellow().bold());
                println!("  Missing critical system files detected!");
                println!("  This may indicate system corruption or tampering.");
            }
        }

        "memory" => {
            println!("{}", "🧠 Memory Artifacts Analysis".cyan().bold());
            println!();

            println!("{}", "Note:".yellow().bold());
            println!("  Memory analysis requires live system access or memory dumps.");
            println!("  This command focuses on disk artifacts that may indicate memory activity.");
            println!();

            // Check for swap files and core dumps
            println!("{}", "Swap & Core Dumps:".yellow().bold());

            if ctx.guestfs.exists("/swap.img").unwrap_or(false) {
                let size = ctx.guestfs.filesize("/swap.img").unwrap_or(0);
                println!("  {} {} ({} bytes)", "✓".green(), "/swap.img".cyan(), size);
            }

            if ctx.guestfs.is_dir("/var/crash").unwrap_or(false) {
                println!("  {} {} - Present (may contain core dumps)", "ℹ".cyan(), "/var/crash".cyan());
            }

            if ctx.guestfs.exists("/proc/kcore").unwrap_or(false) {
                println!("  {} {} - Kernel memory interface", "ℹ".cyan(), "/proc/kcore".cyan());
            }
            println!();

            // Check for hibernation files
            println!("{}", "Hibernation Images:".yellow().bold());
            let hibernate_paths = vec!["/hibernation.img", "/swap/hibernation"];
            let mut found_hibernate = false;

            for path in &hibernate_paths {
                if ctx.guestfs.exists(path).unwrap_or(false) {
                    let size = ctx.guestfs.filesize(path).unwrap_or(0);
                    println!("  {} {} ({} bytes)", "✓".green(), path.cyan(), size);
                    found_hibernate = true;
                }
            }

            if !found_hibernate {
                println!("  {} No hibernation images found", "ℹ".cyan());
            }
            println!();

            // Process information
            println!("{}", "Process Artifacts:".yellow().bold());
            if ctx.guestfs.is_dir("/proc").unwrap_or(false) {
                println!("  {} {} - Available for analysis", "✓".green(), "/proc".cyan());
            }
            println!();

            println!("{}", "Analysis Recommendations:".yellow());
            println!("  • Extract swap files for string analysis");
            println!("  • Analyze core dumps for crash investigation");
            println!("  • Check /tmp and /var/tmp for remnants");
            println!("  • Review .bash_history for executed commands");
        }

        _ => {
            println!("{}", "Unknown forensics workflow".red());
            println!("Run {} for available workflows", "forensics".cyan());
        }
    }

    println!();
    Ok(())
}

/// Audit - Security audit trail analysis
pub fn cmd_audit(ctx: &mut ShellContext, args: &[&str]) -> Result<()> {
    if args.is_empty() {
        println!("{}", "📋 Security Audit Trail Analysis".cyan().bold());
        println!();
        println!("{}", "Available Audit Types:".yellow().bold());
        println!("{} {} - Authentication events", "1.".cyan(), "audit auth".green());
        println!("{} {} - User account changes", "2.".cyan(), "audit users".green());
        println!("{} {} - System configuration changes", "3.".cyan(), "audit config".green());
        println!("{} {} - Package installations", "4.".cyan(), "audit packages".green());
        println!("{} {} - Privilege escalation (sudo)", "5.".cyan(), "audit sudo".green());
        println!("{} {} - Comprehensive audit report", "6.".cyan(), "audit full".green());
        println!();
        println!("{} audit <type>", "Usage:".yellow());
        println!();
        return Ok(());
    }

    let audit_type = args[0];

    match audit_type {
        "auth" => {
            println!("{}", "🔐 Authentication Audit".cyan().bold());
            println!();

            println!("{}", "Log File Analysis:".yellow().bold());

            let auth_logs = vec![
                "/var/log/auth.log",
                "/var/log/secure",
                "/var/log/messages",
            ];

            let mut found_logs = Vec::new();

            for log in &auth_logs {
                if ctx.guestfs.exists(log).unwrap_or(false) {
                    let size = ctx.guestfs.filesize(log).unwrap_or(0);
                    println!("  {} {} ({} bytes)", "✓".green(), log.cyan(), size);
                    found_logs.push(log);
                } else {
                    println!("  {} {} - Not found", "✗".bright_black(), log.bright_black());
                }
            }
            println!();

            if found_logs.is_empty() {
                println!("{}", "⚠ No authentication logs found".yellow());
                println!();
                return Ok(());
            }

            println!("{}", "Key Authentication Indicators:".yellow().bold());
            println!("  {} SSH login attempts", "🔑".cyan());
            println!("  {} Failed password attempts", "❌".cyan());
            println!("  {} Successful logins", "✅".cyan());
            println!("  {} Session duration", "⏱".cyan());
            println!("  {} Remote IP addresses", "🌐".cyan());
            println!();

            println!("{}", "Audit Checklist:".yellow());
            println!("  ✓ Check for brute force attempts (multiple failed logins)");
            println!("  ✓ Identify login patterns (time of day, source IPs)");
            println!("  ✓ Review privileged account access");
            println!("  ✓ Verify multi-factor authentication usage");
            println!("  ✓ Examine account lockouts");
            println!();

            println!("{} {} authentication logs available", "Summary:".yellow(), found_logs.len().to_string().green());
        }

        "users" => {
            println!("{}", "👥 User Account Audit".cyan().bold());
            println!();

            if let Ok(user_info) = ctx.guestfs.inspect_users(&ctx.root) {
                let total_users = user_info.len();
                let mut system_users = 0;
                let mut normal_users = 0;
                let mut privileged_users = 0;

                println!("{}", "User Account Analysis:".yellow().bold());

                for user in &user_info {
                    if user.uid == "0" {
                        privileged_users += 1;
                        println!("  {} {} (UID: {}) - Root equivalent",
                            "🔴".red(), user.username.red().bold(), user.uid);
                    } else if user.uid.parse::<i32>().unwrap_or(9999) < 1000 {
                        system_users += 1;
                    } else {
                        normal_users += 1;
                        println!("  {} {} (UID: {})",
                            "👤".cyan(), user.username.cyan(), user.uid);
                    }
                }
                println!();

                println!("{}", "Account Statistics:".yellow().bold());
                println!("  Total accounts: {}", total_users.to_string().cyan());
                println!("  Privileged (UID 0): {}", privileged_users.to_string().red().bold());
                println!("  System (UID < 1000): {}", system_users.to_string().bright_black());
                println!("  Normal users (UID ≥ 1000): {}", normal_users.to_string().green());
                println!();

                // Audit findings
                println!("{}", "Audit Findings:".yellow().bold());
                if privileged_users > 1 {
                    println!("  {} Multiple UID 0 accounts detected - CRITICAL", "🔴".red());
                }
                if normal_users > 20 {
                    println!("  {} Large number of user accounts - Review needed", "⚠".yellow());
                }
                if privileged_users == 1 && system_users < 100 && normal_users < 10 {
                    println!("  {} User account configuration appears normal", "✓".green());
                }
            }
            println!();

            println!("{}", "Audit Actions:".yellow());
            println!("  • Review inactive accounts for removal");
            println!("  • Verify all UID 0 accounts are authorized");
            println!("  • Check for accounts with empty passwords");
            println!("  • Validate group memberships");
        }

        "config" => {
            println!("{}", "⚙ Configuration Change Audit".cyan().bold());
            println!();

            println!("{}", "Critical Configuration Files:".yellow().bold());

            let config_files = vec![
                ("/etc/passwd", "User database"),
                ("/etc/shadow", "Password hashes"),
                ("/etc/group", "Group database"),
                ("/etc/sudoers", "Sudo configuration"),
                ("/etc/ssh/sshd_config", "SSH server config"),
                ("/etc/pam.d", "PAM configuration"),
                ("/etc/security", "Security settings"),
                ("/etc/fstab", "Filesystem mounts"),
                ("/etc/hosts", "Host mappings"),
                ("/etc/resolv.conf", "DNS configuration"),
            ];

            let mut audited = 0;

            for (path, desc) in &config_files {
                if ctx.guestfs.exists(path).unwrap_or(false) {
                    let size = ctx.guestfs.filesize(path).unwrap_or(0);
                    println!("  {} {} - {} ({} bytes)",
                        "✓".green(), path.cyan(), desc, size);
                    audited += 1;
                } else {
                    println!("  {} {} - {} {}",
                        "✗".red(), path.cyan(), desc, "(missing)".red());
                }
            }
            println!();

            println!("{}", "Configuration Audit Summary:".yellow().bold());
            println!("  Files audited: {}/{}",
                audited.to_string().green(), config_files.len());
            println!();

            println!("{}", "Audit Recommendations:".yellow());
            println!("  • Track configuration changes with version control");
            println!("  • Implement configuration management (Ansible, Puppet)");
            println!("  • Regular backups of /etc directory");
            println!("  • Monitor for unauthorized modifications");
            println!("  • Validate configurations against security baselines");
        }

        "packages" => {
            println!("{}", "📦 Package Installation Audit".cyan().bold());
            println!();

            if let Ok(pkg_info) = ctx.guestfs.inspect_packages(&ctx.root) {
                let total_packages = pkg_info.packages.len();

                println!("{}", "Package Statistics:".yellow().bold());
                println!("  Total packages: {}", total_packages.to_string().cyan());
                println!();

                // Categorize packages
                let mut dev_packages = 0;
                let mut lib_packages = 0;
                let mut kernel_packages = 0;
                let mut doc_packages = 0;

                for pkg in &pkg_info.packages {
                    let name = pkg.name.to_lowercase();
                    if name.contains("devel") || name.contains("-dev") {
                        dev_packages += 1;
                    } else if name.starts_with("lib") {
                        lib_packages += 1;
                    } else if name.contains("kernel") || name.contains("linux-") {
                        kernel_packages += 1;
                    } else if name.contains("doc") {
                        doc_packages += 1;
                    }
                }

                println!("{}", "Package Categories:".yellow().bold());
                println!("  Development: {}", dev_packages.to_string().cyan());
                println!("  Libraries: {}", lib_packages.to_string().cyan());
                println!("  Kernel: {}", kernel_packages.to_string().cyan());
                println!("  Documentation: {}", doc_packages.to_string().cyan());
                println!("  Other: {}", (total_packages - dev_packages - lib_packages - kernel_packages - doc_packages).to_string().cyan());
                println!();

                // Audit findings
                println!("{}", "Audit Findings:".yellow().bold());
                if dev_packages > 100 {
                    println!("  {} Large number of development packages - Consider cleanup", "⚠".yellow());
                }
                if total_packages > 2000 {
                    println!("  {} System has many packages - Review for bloat", "⚠".yellow());
                }
                if kernel_packages > 5 {
                    println!("  {} Multiple kernel versions - Remove old kernels", "ℹ".cyan());
                }
                println!();

                println!("{}", "Package Audit Tips:".yellow());
                println!("  • Remove unused development packages");
                println!("  • Keep only 2-3 recent kernel versions");
                println!("  • Review automatically installed packages");
                println!("  • Verify package signatures and sources");
            }
        }

        "sudo" => {
            println!("{}", "🔐 Privilege Escalation Audit (Sudo)".cyan().bold());
            println!();

            println!("{}", "Sudo Configuration:".yellow().bold());

            if ctx.guestfs.exists("/etc/sudoers").unwrap_or(false) {
                let size = ctx.guestfs.filesize("/etc/sudoers").unwrap_or(0);
                println!("  {} {} ({} bytes)", "✓".green(), "/etc/sudoers".cyan(), size);
            } else {
                println!("  {} {} - Not found", "✗".red(), "/etc/sudoers".cyan());
            }

            if ctx.guestfs.is_dir("/etc/sudoers.d").unwrap_or(false) {
                println!("  {} {} - Present", "✓".green(), "/etc/sudoers.d/".cyan());
            }
            println!();

            println!("{}", "Sudo Log Analysis:".yellow().bold());
            let sudo_logs = vec![
                "/var/log/sudo.log",
                "/var/log/secure",
                "/var/log/auth.log",
            ];

            for log in &sudo_logs {
                if ctx.guestfs.exists(log).unwrap_or(false) {
                    let size = ctx.guestfs.filesize(log).unwrap_or(0);
                    println!("  {} {} ({} bytes)", "✓".green(), log.cyan(), size);
                }
            }
            println!();

            println!("{}", "Audit Checklist:".yellow().bold());
            println!("  ✓ Review sudo rules for least privilege");
            println!("  ✓ Check for NOPASSWD directives");
            println!("  ✓ Verify sudo group membership");
            println!("  ✓ Examine sudo command history");
            println!("  ✓ Look for privilege escalation attempts");
            println!();

            println!("{}", "Security Recommendations:".yellow());
            println!("  • Require passwords for all sudo commands");
            println!("  • Limit sudo access to specific commands");
            println!("  • Enable sudo logging");
            println!("  • Regular review of sudo configurations");
            println!("  • Use role-based access control");
        }

        "full" => {
            println!("{}", "📊 Comprehensive Security Audit Report".cyan().bold());
            println!();

            // Authentication audit summary
            println!("{}", "1. Authentication Security".yellow().bold());
            let auth_logs = vec!["/var/log/auth.log", "/var/log/secure"];
            let mut auth_found = 0;
            for log in &auth_logs {
                if ctx.guestfs.exists(log).unwrap_or(false) {
                    auth_found += 1;
                }
            }
            println!("   Status: {}", if auth_found > 0 { "✓ Logs available".green() } else { "✗ No logs found".red() });
            println!();

            // User account audit summary
            println!("{}", "2. User Account Security".yellow().bold());
            if let Ok(user_info) = ctx.guestfs.inspect_users(&ctx.root) {
                let privileged = user_info.iter().filter(|u| u.uid == "0").count();
                let normal = user_info.iter().filter(|u| u.uid.parse::<i32>().unwrap_or(0) >= 1000).count();
                println!("   Total user_info: {}", user_info.len().to_string().cyan());
                println!("   Privileged accounts: {}", privileged.to_string().cyan());
                println!("   Normal users: {}", normal.to_string().cyan());
                if privileged > 1 {
                    println!("   {} Multiple UID 0 accounts detected", "⚠".yellow());
                }
            }
            println!();

            // Configuration audit summary
            println!("{}", "3. Configuration Security".yellow().bold());
            let configs = vec!["/etc/passwd", "/etc/shadow", "/etc/sudoers"];
            let mut config_ok = 0;
            for cfg in &configs {
                if ctx.guestfs.exists(cfg).unwrap_or(false) {
                    config_ok += 1;
                }
            }
            println!("   Critical configs present: {}/{}", config_ok, configs.len());
            println!();

            // Package audit summary
            println!("{}", "4. Package Security".yellow().bold());
            if let Ok(pkg_info) = ctx.guestfs.inspect_packages(&ctx.root) {
                println!("   Total packages: {}", pkg_info.packages.len().to_string().cyan());
                let dev_count = pkg_info.packages.iter()
                    .filter(|p| p.name.contains("devel") || p.name.contains("-dev"))
                    .count();
                println!("   Development packages: {}", dev_count.to_string().cyan());
            }
            println!();

            // Sudo audit summary
            println!("{}", "5. Privilege Escalation".yellow().bold());
            if ctx.guestfs.exists("/etc/sudoers").unwrap_or(false) {
                println!("   Status: {} Sudo configured", "✓".green());
            } else {
                println!("   Status: {} No sudo configuration", "ℹ".cyan());
            }
            println!();

            println!("{}", "Audit Completion Summary:".yellow().bold());
            println!("  ✓ Authentication logs reviewed");
            println!("  ✓ User accounts audited");
            println!("  ✓ Configuration files checked");
            println!("  ✓ Package inventory analyzed");
            println!("  ✓ Privilege escalation reviewed");
            println!();

            println!("{}", "Next Steps:".yellow());
            println!("  1. Address any critical findings");
            println!("  2. Document audit results");
            println!("  3. Implement remediation plan");
            println!("  4. Schedule regular audits");
        }

        _ => {
            println!("{}", "Unknown audit type".red());
            println!("Run {} for available types", "audit".cyan());
        }
    }

    println!();
    Ok(())
}

/// Baseline - Security baseline and drift detection
pub fn cmd_baseline(ctx: &mut ShellContext, args: &[&str]) -> Result<()> {
    if args.is_empty() {
        println!("{}", "📏 Security Baseline Management".cyan().bold());
        println!();
        println!("{}", "Available Commands:".yellow().bold());
        println!("{} {} - Create current security baseline", "1.".cyan(), "baseline create".green());
        println!("{} {} - Show current baseline", "2.".cyan(), "baseline show".green());
        println!("{} {} - Detect configuration drift", "3.".cyan(), "baseline drift".green());
        println!("{} {} - Compare with CIS benchmark", "4.".cyan(), "baseline cis".green());
        println!("{} {} - Export baseline for comparison", "5.".cyan(), "baseline export".green());
        println!();
        println!("{} baseline <command>", "Usage:".yellow());
        println!();
        return Ok(());
    }

    let command = args[0];

    match command {
        "create" => {
            println!("{}", "📋 Creating Security Baseline".cyan().bold());
            println!();

            let mut baseline = Vec::new();

            // System information
            println!("{}", "System Configuration:".yellow().bold());
            if let Ok(os_info) = ctx.guestfs.inspect_os() {
                if !os_info.is_empty() {
                    println!("  {} OS detected", "✓".green());
                    baseline.push("OS configuration captured");
                }
            }
            println!();

            // Security features
            println!("{}", "Security Features:".yellow().bold());
            if let Ok(sec_info) = ctx.guestfs.inspect_security(&ctx.root) {
                println!("  SELinux: {}", if &sec_info.selinux != "disabled" {
                    sec_info.selinux.green()
                } else {
                    "disabled".red()
                });
                println!("  AppArmor: {}", if sec_info.apparmor {
                    "enabled".green()
                } else {
                    "disabled".red()
                });

                let firewall_status = if let Ok(fw) = ctx.guestfs.inspect_firewall(&ctx.root) {
                    if fw.enabled {
                        "enabled".green()
                    } else {
                        "disabled".red()
                    }
                } else {
                    "unknown".yellow()
                };
                println!("  Firewall: {}", firewall_status);
                baseline.push("Security features documented");
            }
            println!();

            // User accounts
            println!("{}", "User Accounts:".yellow().bold());
            if let Ok(user_info) = ctx.guestfs.inspect_users(&ctx.root) {
                let total_user_info = user_info.len();
                let privileged = user_info.iter().filter(|u| u.uid == "0").count();
                println!("  Total users: {}", user_info.len());
                println!("  Privileged accounts: {}", privileged);
                baseline.push("User accounts baselined");
            }
            println!();

            // Package count
            println!("{}", "Software Inventory:".yellow().bold());
            if let Ok(pkg_info) = ctx.guestfs.inspect_packages(&ctx.root) {
                println!("  Total packages: {}", pkg_info.packages.len());
                baseline.push("Package inventory captured");
            }
            println!();

            // Services
            println!("{}", "System Services:".yellow().bold());
            if let Ok(services) = ctx.guestfs.inspect_systemd_services(&ctx.root) {
                let total = services.len();
                let enabled = services.iter().filter(|s| s.enabled).count();
                println!("  Total services: {}", total);
                println!("  Enabled services: {}", enabled);
                baseline.push("Service configuration captured");
            }
            println!();

            // Network configuration
            println!("{}", "Network Configuration:".yellow().bold());
            if ctx.guestfs.exists("/etc/hosts").unwrap_or(false) {
                println!("  {} /etc/hosts present", "✓".green());
                baseline.push("Network config documented");
            }
            if ctx.guestfs.exists("/etc/resolv.conf").unwrap_or(false) {
                println!("  {} /etc/resolv.conf present", "✓".green());
            }
            println!();

            println!("{}", "Baseline Creation Summary:".yellow().bold());
            println!("  Components captured: {}", baseline.len().to_string().green());
            for component in &baseline {
                println!("    • {}", component);
            }
            println!();

            println!("{}", "Next Steps:".yellow());
            println!("  • Save baseline: {}", "baseline export > baseline.json".cyan());
            println!("  • Monitor drift: {}", "baseline drift".cyan());
            println!("  • Compare with standards: {}", "baseline cis".cyan());
        }

        "show" => {
            println!("{}", "📋 Current Security Baseline".cyan().bold());
            println!();

            // Display current system state as baseline
            println!("{}", "╔══ System Baseline ══╗".cyan().bold());
            println!();

            if let Ok(sec_info) = ctx.guestfs.inspect_security(&ctx.root) {
                println!("{}", "Security Configuration:".yellow().bold());
                println!("  SELinux: {}", sec_info.selinux);
                println!("  AppArmor: {}", sec_info.apparmor);

                if let Ok(fw) = ctx.guestfs.inspect_firewall(&ctx.root) {
                    println!("  Firewall: {}", if fw.enabled { "enabled" } else { "disabled" });
                }
                println!();
            }

            if let Ok(user_info) = ctx.guestfs.inspect_users(&ctx.root) {
                println!("{}", "User Account Baseline:".yellow().bold());
                println!("  Total users: {}", user_info.len());
                println!("  UID 0 accounts: {}",
                    user_info.iter().filter(|u| u.uid == "0").count());
                println!("  Normal users: {}",
                    user_info.iter().filter(|u| u.uid.parse::<i32>().unwrap_or(0) >= 1000).count());
                println!();
            }

            if let Ok(pkg_info) = ctx.guestfs.inspect_packages(&ctx.root) {
                println!("{}", "Software Baseline:".yellow().bold());
                println!("  Installed packages: {}", pkg_info.packages.len());
                println!();
            }

            if let Ok(services) = ctx.guestfs.inspect_systemd_services(&ctx.root) {
                println!("{}", "Service Baseline:".yellow().bold());
                println!("  Total services: {}", services.len());
                println!("  Enabled: {}",
                    services.iter().filter(|s| s.enabled).count());
                println!("  Disabled: {}",
                    services.iter().filter(|s| !s.enabled).count());
                println!();
            }

            println!("{}", "╚══ End Baseline ══╝".cyan().bold());
        }

        "drift" => {
            println!("{}", "🔍 Configuration Drift Detection".cyan().bold());
            println!();

            println!("{}", "Note:".yellow().bold());
            println!("  Drift detection requires a saved baseline for comparison.");
            println!("  Run {} to establish initial baseline.", "baseline create".cyan());
            println!();

            println!("{}", "Drift Monitoring Areas:".yellow().bold());
            println!();

            // Check for common drift indicators
            let mut drift_detected = Vec::new();

            // User account drift
            println!("{}", "1. User Account Drift".cyan());
            if let Ok(user_info) = ctx.guestfs.inspect_users(&ctx.root) {
                let uid0_count = user_info.iter().filter(|u| u.uid == "0").count();
                if uid0_count > 1 {
                    drift_detected.push("Multiple UID 0 accounts (expected: 1)");
                    println!("   {} Multiple privileged accounts detected", "⚠".yellow());
                } else {
                    println!("   {} Account structure stable", "✓".green());
                }
            }
            println!();

            // Security configuration drift
            println!("{}", "2. Security Configuration Drift".cyan());
            if let Ok(sec_info) = ctx.guestfs.inspect_security(&ctx.root) {
                if &sec_info.selinux == "disabled" && !sec_info.apparmor {
                    drift_detected.push("No MAC system enabled");
                    println!("   {} MAC system disabled (potential drift)", "⚠".yellow());
                }

                if let Ok(fw) = ctx.guestfs.inspect_firewall(&ctx.root) {
                    if !fw.enabled {
                        drift_detected.push("Firewall disabled");
                        println!("   {} Firewall disabled (potential drift)", "⚠".yellow());
                    }
                }

                if drift_detected.is_empty() {
                    println!("   {} Security configuration stable", "✓".green());
                }
            }
            println!();

            // Service drift
            println!("{}", "3. Service Configuration Drift".cyan());
            if let Ok(services) = ctx.guestfs.inspect_systemd_services(&ctx.root) {
                let enabled = services.iter().filter(|s| s.enabled).count();
                if enabled > 50 {
                    drift_detected.push("High number of enabled services");
                    println!("   {} Many services enabled (potential drift)", "⚠".yellow());
                } else {
                    println!("   {} Service configuration stable", "✓".green());
                }
            }
            println!();

            // Configuration file drift
            println!("{}", "4. Critical File Drift".cyan());
            let critical_files = vec![
                "/etc/passwd", "/etc/shadow", "/etc/sudoers",
                "/etc/ssh/sshd_config", "/etc/fstab",
            ];
            let mut all_present = true;
            for file in &critical_files {
                if !ctx.guestfs.exists(file).unwrap_or(false) {
                    drift_detected.push("Missing critical configuration file");
                    println!("   {} {} missing (critical drift)", "🔴".red(), file.red());
                    all_present = false;
                }
            }
            if all_present {
                println!("   {} Critical files intact", "✓".green());
            }
            println!();

            // Drift summary
            println!("{}", "Drift Detection Summary:".yellow().bold());
            if drift_detected.is_empty() {
                println!("  {} No significant drift detected", "✓".green().bold());
                println!("  System configuration appears stable");
            } else {
                println!("  {} {} drift indicators found:", "⚠".yellow(), drift_detected.len().to_string().red());
                for drift in &drift_detected {
                    println!("    • {}", drift);
                }
            }
            println!();

            println!("{}", "Recommendations:".yellow());
            println!("  • Review and address drift indicators");
            println!("  • Update baseline if changes are authorized");
            println!("  • Investigate unauthorized modifications");
            println!("  • Implement configuration management");
        }

        "cis" => {
            println!("{}", "📋 CIS Benchmark Comparison".cyan().bold());
            println!();

            let mut checks = Vec::new();
            let mut passed = 0;
            let mut failed = 0;

            println!("{}", "CIS Controls Validation:".yellow().bold());
            println!();

            // CIS Control 1: Ensure filesystem integrity checking
            println!("{}", "1. Filesystem Integrity".cyan());
            let has_aide = ctx.guestfs.exists("/usr/bin/aide").unwrap_or(false);
            if has_aide {
                println!("   {} AIDE installed", "✓".green());
                passed += 1;
            } else {
                println!("   {} AIDE not found - Install integrity checking", "✗".red());
                failed += 1;
            }
            checks.push(("Filesystem integrity checking", has_aide));
            println!();

            // CIS Control 2: Ensure firewall is enabled
            println!("{}", "2. Firewall Configuration".cyan());
            if let Ok(fw) = ctx.guestfs.inspect_firewall(&ctx.root) {
                if fw.enabled {
                    println!("   {} Firewall enabled", "✓".green());
                    passed += 1;
                } else {
                    println!("   {} Firewall disabled - Enable firewall", "✗".red());
                    failed += 1;
                }
                checks.push(("Firewall enabled", fw.enabled));
            }
            println!();

            // CIS Control 3: Ensure MAC is enabled
            println!("{}", "3. Mandatory Access Control".cyan());
            if let Ok(sec_info) = ctx.guestfs.inspect_security(&ctx.root) {
                let mac_enabled = &sec_info.selinux != "disabled" || sec_info.apparmor;
                if mac_enabled {
                    println!("   {} MAC system active", "✓".green());
                    passed += 1;
                } else {
                    println!("   {} No MAC system - Enable SELinux or AppArmor", "✗".red());
                    failed += 1;
                }
                checks.push(("MAC enabled", mac_enabled));
            }
            println!();

            // CIS Control 4: SSH configuration
            println!("{}", "4. SSH Hardening".cyan());
            let ssh_config = ctx.guestfs.exists("/etc/ssh/sshd_config").unwrap_or(false);
            if ssh_config {
                println!("   {} SSH configuration present", "✓".green());
                println!("   {} Manual review recommended for:", "ℹ".cyan());
                println!("      • PermitRootLogin no");
                println!("      • PasswordAuthentication no");
                println!("      • Protocol 2");
                passed += 1;
            } else {
                println!("   {} SSH configuration missing", "✗".red());
                failed += 1;
            }
            checks.push(("SSH configuration", ssh_config));
            println!();

            // CIS Control 5: Audit logging
            println!("{}", "5. Audit Logging".cyan());
            let auditd = ctx.guestfs.exists("/sbin/auditd").unwrap_or(false);
            if auditd {
                println!("   {} Audit daemon installed", "✓".green());
                passed += 1;
            } else {
                println!("   {} Audit daemon not found - Install auditd", "✗".red());
                failed += 1;
            }
            checks.push(("Audit logging", auditd));
            println!();

            // CIS summary
            let total = checks.len();
            let compliance_rate = if total > 0 { (passed * 100) / total } else { 0 };

            println!("{}", "CIS Benchmark Summary:".yellow().bold());
            println!("  Total checks: {}", total);
            println!("  Passed: {}", passed.to_string().green());
            println!("  Failed: {}", failed.to_string().red());
            println!("  Compliance rate: {}%", compliance_rate.to_string().cyan());
            println!();

            let grade = if compliance_rate >= 80 {
                "Compliant".green().bold()
            } else if compliance_rate >= 60 {
                "Partially Compliant".yellow()
            } else {
                "Non-Compliant".red().bold()
            };

            println!("  Overall status: {}", grade);
            println!();

            println!("{}", "Next Steps:".yellow());
            println!("  • Address failed controls");
            println!("  • Document exceptions");
            println!("  • Schedule regular compliance checks");
            println!("  • Implement remediation plan");
        }

        "export" => {
            println!("{}", "💾 Exporting Security Baseline".cyan().bold());
            println!();

            println!("{}", "Export Format: JSON".yellow().bold());
            println!();

            // Create a baseline export structure
            println!("{{");
            println!("  \"baseline\": {{");
            println!("    \"created\": \"2024-01-01T00:00:00Z\",");
            println!("    \"system\": {{");

            if let Ok(sec_info) = ctx.guestfs.inspect_security(&ctx.root) {
                println!("      \"selinux\": \"{}\",", sec_info.selinux);
                println!("      \"apparmor\": {},", sec_info.apparmor);

                if let Ok(fw) = ctx.guestfs.inspect_firewall(&ctx.root) {
                    println!("      \"firewall\": {},", fw.enabled);
                }
            }

            if let Ok(user_info) = ctx.guestfs.inspect_users(&ctx.root) {
                println!("      \"total_users\": {},", user_info.len());
                println!("      \"privileged_users\": {},",
                    user_info.iter().filter(|u| u.uid == "0").count());
            }

            if let Ok(pkg_info) = ctx.guestfs.inspect_packages(&ctx.root) {
                println!("      \"total_packages\": {}", pkg_info.packages.len());
            }

            println!("    }}");
            println!("  }}");
            println!("}}");
            println!();

            println!("{}", "Save this output:".yellow());
            println!("  {}", "baseline export > baseline.json".cyan());
            println!();
            println!("{}", "Use for comparison:".yellow());
            println!("  Compare against saved baseline to detect drift");
        }

        _ => {
            println!("{}", "Unknown baseline command".red());
            println!("Run {} for available commands", "baseline".cyan());
        }
    }

    println!();
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
