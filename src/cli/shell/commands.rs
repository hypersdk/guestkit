// SPDX-License-Identifier: LGPL-3.0-or-later
//! Command implementations for interactive shell

use anyhow::Result;
use guestctl::Guestfs;
use colored::Colorize;

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
}

impl ShellContext {
    pub fn new(guestfs: Guestfs, root: String) -> Self {
        Self {
            guestfs,
            root,
            current_path: "/".to_string(),
        }
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
            eprintln!("{} {}", "Error:".red(), e);
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

    #[cfg(feature = "ai")]
    {
        println!("\n{}", "AI Assistant:".yellow().bold());
        println!("  {}    - Ask AI for help (requires OPENAI_API_KEY)", "ai <query>".green());
        println!("           Example: ai why won't this boot?");
    }

    println!("\n{}", "Shell Commands:".yellow().bold());
    println!("  {}    - Show this help", "help".green());
    println!("  {}   - Clear screen", "clear".green());
    println!("  {}   - Show command history", "history".green());
    println!("  {}    - Exit shell", "exit, quit, q".green());

    println!("\n{}", "Tips:".yellow().bold());
    println!("  • Use {} for command completion", "Tab".cyan());
    println!("  • Use {} for command history", "↑/↓ arrows".cyan());
    println!("  • Paths are relative to current directory");
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
