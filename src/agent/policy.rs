// SPDX-License-Identifier: Apache-2.0
//! Agent remediation policy (allowlist).

use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use std::fs;
use std::path::Path;

const DEFAULT_POLICY_PATH: &str = "/etc/zyvor/agent-policy.yaml";

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct AgentPolicy {
    #[serde(default)]
    pub actions: PolicyActions,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct PolicyActions {
    #[serde(default)]
    pub restart_unit: RestartUnitPolicy,
    #[serde(default)]
    pub run_shell_command: ShellPolicy,
    #[serde(default)]
    pub reboot_vm: ApprovalPolicy,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct RestartUnitPolicy {
    #[serde(default)]
    pub enabled: bool,
    #[serde(default)]
    pub allowed_units: Vec<String>,
    #[serde(default)]
    pub require_approval: bool,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ShellPolicy {
    #[serde(default)]
    pub enabled: bool,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ApprovalPolicy {
    #[serde(default)]
    pub require_approval: bool,
}

impl AgentPolicy {
    pub fn load() -> Self {
        let path = std::env::var("ZYVOR_AGENT_POLICY")
            .unwrap_or_else(|_| DEFAULT_POLICY_PATH.to_string());
        if Path::new(&path).exists() {
            fs::read_to_string(&path)
                .ok()
                .and_then(|content| serde_yaml::from_str(&content).ok())
                .unwrap_or_default()
        } else {
            Self::default_permissive_dev()
        }
    }

    pub fn default_permissive_dev() -> Self {
        Self {
            actions: PolicyActions {
                restart_unit: RestartUnitPolicy {
                    enabled: true,
                    allowed_units: vec![],
                    require_approval: false,
                },
                run_shell_command: ShellPolicy { enabled: false },
                reboot_vm: ApprovalPolicy {
                    require_approval: true,
                },
            },
        }
    }

    pub fn can_restart_unit(&self, unit: &str) -> bool {
        if !self.actions.restart_unit.enabled {
            return false;
        }
        if self.actions.restart_unit.allowed_units.is_empty() {
            return true;
        }
        let allowed: HashSet<&str> = self
            .actions
            .restart_unit
            .allowed_units
            .iter()
            .map(String::as_str)
            .collect();
        allowed.contains(unit)
    }

    pub fn shell_enabled(&self) -> bool {
        self.actions.run_shell_command.enabled
    }
}
