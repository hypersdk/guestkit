// SPDX-License-Identifier: LGPL-3.0-or-later
//! Tab completion for interactive shell

use rustyline::completion::{Completer, Pair};
use rustyline::Context;
use rustyline::Result;

#[allow(dead_code)]
pub struct ShellCompleter {
    commands: Vec<String>,
}

#[allow(dead_code)]
impl ShellCompleter {
    pub fn new() -> Self {
        Self {
            commands: vec![
                "ls".to_string(),
                "cat".to_string(),
                "cd".to_string(),
                "pwd".to_string(),
                "find".to_string(),
                "grep".to_string(),
                "info".to_string(),
                "mounts".to_string(),
                "packages".to_string(),
                "services".to_string(),
                "users".to_string(),
                "network".to_string(),
                "security".to_string(),
                "health".to_string(),
                "risks".to_string(),
                "help".to_string(),
                "clear".to_string(),
                "history".to_string(),
                "exit".to_string(),
                "quit".to_string(),
            ],
        }
    }
}

impl Completer for ShellCompleter {
    type Candidate = Pair;

    fn complete(
        &self,
        line: &str,
        pos: usize,
        _ctx: &Context<'_>,
    ) -> Result<(usize, Vec<Pair>)> {
        let mut candidates = Vec::new();

        // Get the word being completed
        let start = line[..pos].rfind(' ').map(|i| i + 1).unwrap_or(0);
        let word = &line[start..pos];

        // Complete commands
        for cmd in &self.commands {
            if cmd.starts_with(word) {
                candidates.push(Pair {
                    display: cmd.clone(),
                    replacement: cmd.clone(),
                });
            }
        }

        Ok((start, candidates))
    }
}

impl Default for ShellCompleter {
    fn default() -> Self {
        Self::new()
    }
}
