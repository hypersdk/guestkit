// SPDX-License-Identifier: Apache-2.0
//! Phase 2 — multi-step agent loop over evidence snapshot tools.

use crate::ai::prompts::{self, tool_loop_instructions};
use crate::ai::providers::{self, ProviderConfig};
use crate::ai::tools::SnapshotTools;
use crate::assurance::boot_target_from_str;
use crate::boot::{analyze_bootability, BootabilityReport};
use crate::evidence::snapshot::EvidenceSnapshot;
use anyhow::{anyhow, Result};
use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentConfig {
    pub max_steps: usize,
    pub boot_target: String,
    pub provider: ProviderConfig,
}

impl Default for AgentConfig {
    fn default() -> Self {
        Self {
            max_steps: 5,
            boot_target: "generic".into(),
            provider: ProviderConfig::default(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentResult {
    pub answer: String,
    pub steps: usize,
    pub tool_calls: Vec<ToolCallRecord>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolCallRecord {
    pub tool: String,
    pub args: Value,
    pub result_preview: String,
}

/// Run the Guest Intelligence Agent on a pre-collected evidence snapshot.
pub async fn run_agent_on_evidence(
    evidence: &EvidenceSnapshot,
    query: &str,
    config: &AgentConfig,
) -> Result<AgentResult> {
    let boot_target = boot_target_from_str(&config.boot_target);
    let boot = analyze_bootability(evidence, boot_target);
    run_agent_on_evidence_with_boot(evidence, Some(&boot), query, config).await
}

pub async fn run_agent_on_evidence_with_boot(
    evidence: &EvidenceSnapshot,
    boot: Option<&BootabilityReport>,
    query: &str,
    config: &AgentConfig,
) -> Result<AgentResult> {
    let tools = SnapshotTools::new(evidence, boot);
    let provider = if config.provider.api_key.is_some()
        || config.provider.provider == providers::Provider::Ollama
    {
        config.provider.clone()
    } else {
        ProviderConfig::from_env()?
    };

    let mut transcript = format!(
        "Evidence OS: {} {}\nQuery: {}\n\nInitial context:\n{}\n",
        evidence.os.distribution,
        evidence.os.version,
        query,
        serde_json::to_string_pretty(&tools.call("get_semantic_summary", &Value::Null)?)?
    );

    let system = format!(
        "{}\n\n{}",
        prompts::system_prompt(),
        tool_loop_instructions()
    );

    let mut tool_calls = Vec::new();
    let mut answer = String::new();

    for step in 0..config.max_steps {
        let user = if step == 0 {
            format!("{transcript}\nAnswer the query or request a tool.")
        } else {
            format!("{transcript}\nContinue — answer or call another tool.")
        };

        let response = providers::completion(&provider, &system, &user).await?;
        if let Some(call) = parse_tool_call(&response) {
            let result = tools.call(&call.tool, &call.args)?;
            let preview = serde_json::to_string(&result)?
                .chars()
                .take(2000)
                .collect::<String>();
            tool_calls.push(ToolCallRecord {
                tool: call.tool.clone(),
                args: call.args.clone(),
                result_preview: preview.clone(),
            });
            transcript.push_str(&format!(
                "\nTool `{}` returned:\n{preview}\n",
                call.tool
            ));
            continue;
        }
        answer = response;
        return Ok(AgentResult {
            answer,
            steps: step + 1,
            tool_calls,
        });
    }

    if answer.is_empty() {
        answer = providers::completion(
            &provider,
            &system,
            &format!("{transcript}\nProvide final answer now without tools."),
        )
        .await?;
    }

    Ok(AgentResult {
        answer,
        steps: config.max_steps,
        tool_calls,
    })
}

#[derive(Debug)]
struct ParsedToolCall {
    tool: String,
    args: Value,
}

fn parse_tool_call(response: &str) -> Option<ParsedToolCall> {
    for line in response.lines() {
        let line = line.trim();
        if !line.starts_with('{') {
            continue;
        }
        if let Ok(v) = serde_json::from_str::<Value>(line) {
            if let (Some(tool), args) = (v.get("tool").and_then(|t| t.as_str()), v.get("args")) {
                return Some(ParsedToolCall {
                    tool: tool.to_string(),
                    args: args.cloned().unwrap_or(Value::Object(Default::default())),
                });
            }
        }
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_tool_json_line() {
        let call = parse_tool_call(r#"Checking… {"tool":"list_systemd_units","args":{"type":"timer"}}"#);
        assert!(call.is_some());
        assert_eq!(call.unwrap().tool, "list_systemd_units");
    }
}
