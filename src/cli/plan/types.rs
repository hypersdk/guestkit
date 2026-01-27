// SPDX-License-Identifier: LGPL-3.0-or-later
//! Plan type definitions

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Complete fix plan containing all operations and metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FixPlan {
    /// Plan format version
    pub version: String,

    /// VM disk path
    pub vm: String,

    /// When the plan was generated
    pub generated: DateTime<Utc>,

    /// Profile that generated this plan
    pub profile: String,

    /// Overall risk level
    pub overall_risk: String,

    /// Estimated duration
    pub estimated_duration: String,

    /// Plan metadata
    pub metadata: PlanMetadata,

    /// List of operations to perform
    pub operations: Vec<Operation>,

    /// Actions to run after all operations complete
    pub post_apply: Vec<PostApplyAction>,
}

/// Metadata about the plan
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlanMetadata {
    /// Who/what generated the plan
    pub author: String,

    /// Whether human review is required
    pub review_required: bool,

    /// Whether all operations are reversible
    pub reversible: bool,

    /// Optional description
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,

    /// Optional tags
    #[serde(default)]
    pub tags: Vec<String>,
}

/// A single fix operation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Operation {
    /// Unique operation ID
    pub id: String,

    /// Operation type
    #[serde(flatten)]
    pub op_type: OperationType,

    /// Priority level
    pub priority: Priority,

    /// Human-readable description
    pub description: String,

    /// Risk level of this operation
    pub risk: String,

    /// Whether this operation can be reversed
    pub reversible: bool,

    /// IDs of operations this depends on
    #[serde(default)]
    pub depends_on: Vec<String>,

    /// Optional validation to run after operation
    #[serde(skip_serializing_if = "Option::is_none")]
    pub validation: Option<ValidationCheck>,

    /// Optional undo information
    #[serde(skip_serializing_if = "Option::is_none")]
    pub undo: Option<UndoInfo>,
}

/// Types of operations that can be performed
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum OperationType {
    /// Edit a file
    FileEdit(FileEdit),

    /// Install packages
    PackageInstall(PackageInstall),

    /// Service operation (enable, start, restart)
    ServiceOperation(ServiceOperation),

    /// SELinux mode change
    SelinuxMode(SELinuxMode),

    /// Windows registry edit
    RegistryEdit(RegistryEdit),

    /// Execute a command
    CommandExec(CommandExec),

    /// Copy a file
    FileCopy(FileCopy),

    /// Create a directory
    DirectoryCreate(DirectoryCreate),

    /// Set file permissions
    FilePermissions(FilePermissions),
}

/// File editing operation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileEdit {
    /// Path to file
    pub file: String,

    /// Whether to create backup
    #[serde(default = "default_true")]
    pub backup: bool,

    /// Changes to make
    pub changes: Vec<FileChange>,
}

/// A single file change
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileChange {
    /// Line number (1-indexed)
    pub line: usize,

    /// Content before change
    pub before: String,

    /// Content after change
    pub after: String,

    /// Optional context lines for display
    #[serde(skip_serializing_if = "Option::is_none")]
    pub context: Option<String>,
}

/// Package installation operation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PackageInstall {
    /// Packages to install
    pub packages: Vec<String>,

    /// Estimated total size
    #[serde(skip_serializing_if = "Option::is_none")]
    pub estimated_size: Option<String>,
}

/// Service operation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServiceOperation {
    /// Service name
    pub service: String,

    /// Desired state (enabled/disabled)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub state: Option<String>,

    /// Whether to start the service
    #[serde(default)]
    pub start: bool,

    /// Whether to restart the service
    #[serde(default)]
    pub restart: bool,
}

/// SELinux mode change
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SELinuxMode {
    /// Config file path
    pub file: String,

    /// Current mode
    pub current: String,

    /// Target mode
    pub target: String,

    /// Optional warning message
    #[serde(skip_serializing_if = "Option::is_none")]
    pub warning: Option<String>,
}

/// Windows registry edit
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RegistryEdit {
    /// Registry key path
    pub key: String,

    /// Value name
    pub value: String,

    /// Current data
    pub current_data: serde_json::Value,

    /// New data
    pub new_data: serde_json::Value,

    /// Data type (DWORD, String, etc.)
    pub data_type: String,
}

/// Command execution
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CommandExec {
    /// Command to execute
    pub command: String,

    /// Expected exit code
    #[serde(default)]
    pub expected_exit: i32,

    /// Optional timeout in seconds
    #[serde(skip_serializing_if = "Option::is_none")]
    pub timeout: Option<u64>,
}

/// File copy operation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileCopy {
    /// Source path
    pub source: String,

    /// Destination path
    pub destination: String,

    /// Whether to create backup of destination
    #[serde(default = "default_true")]
    pub backup: bool,
}

/// Directory creation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DirectoryCreate {
    /// Path to create
    pub path: String,

    /// Permissions (octal string like "0755")
    #[serde(skip_serializing_if = "Option::is_none")]
    pub mode: Option<String>,
}

/// File permissions change
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FilePermissions {
    /// Path to file/directory
    pub path: String,

    /// New mode (octal string like "0644")
    pub mode: String,

    /// Optional owner
    #[serde(skip_serializing_if = "Option::is_none")]
    pub owner: Option<String>,

    /// Optional group
    #[serde(skip_serializing_if = "Option::is_none")]
    pub group: Option<String>,
}

/// Validation check to run after operation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidationCheck {
    /// Command to run
    pub command: String,

    /// Expected exit code
    #[serde(default)]
    pub expected_exit: i32,

    /// Optional expected output
    #[serde(skip_serializing_if = "Option::is_none")]
    pub expected_output: Option<String>,
}

/// Information for undoing an operation
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum UndoInfo {
    /// File changes to restore
    FileChanges(Vec<FileChange>),

    /// Command to run for undo
    Command { command: String },

    /// Generic undo data
    Data(HashMap<String, serde_json::Value>),
}

/// Priority levels for operations
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Priority {
    Critical,
    High,
    Medium,
    Low,
    Info,
}

impl Priority {
    pub fn as_str(&self) -> &'static str {
        match self {
            Priority::Critical => "critical",
            Priority::High => "high",
            Priority::Medium => "medium",
            Priority::Low => "low",
            Priority::Info => "info",
        }
    }

    pub fn emoji(&self) -> &'static str {
        match self {
            Priority::Critical => "🔴",
            Priority::High => "🟠",
            Priority::Medium => "🟡",
            Priority::Low => "🟢",
            Priority::Info => "ℹ️",
        }
    }
}

/// Actions to perform after all operations complete
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum PostApplyAction {
    /// Restart services
    ServiceRestart {
        services: Vec<String>,
    },

    /// Run validation
    Validation {
        command: String,
        #[serde(skip_serializing_if = "Option::is_none")]
        expected_output: Option<String>,
    },

    /// Display message
    Message {
        message: String,
    },

    /// Reboot required
    RebootRequired {
        reason: String,
    },
}

impl FixPlan {
    /// Create a new empty plan
    pub fn new(vm: String, profile: String) -> Self {
        Self {
            version: "1.0".to_string(),
            vm,
            generated: Utc::now(),
            profile,
            overall_risk: "unknown".to_string(),
            estimated_duration: "unknown".to_string(),
            metadata: PlanMetadata {
                author: "guestkit-profiles".to_string(),
                review_required: true,
                reversible: true,
                description: None,
                tags: Vec::new(),
            },
            operations: Vec::new(),
            post_apply: Vec::new(),
        }
    }

    /// Add an operation to the plan
    pub fn add_operation(&mut self, operation: Operation) {
        self.operations.push(operation);
    }

    /// Get operations sorted by priority
    pub fn operations_by_priority(&self) -> Vec<&Operation> {
        let mut ops: Vec<&Operation> = self.operations.iter().collect();
        ops.sort_by_key(|op| op.priority);
        ops
    }

    /// Get count by priority
    pub fn count_by_priority(&self, priority: Priority) -> usize {
        self.operations.iter().filter(|op| op.priority == priority).count()
    }

    /// Check if plan has any critical operations
    pub fn has_critical(&self) -> bool {
        self.operations.iter().any(|op| op.priority == Priority::Critical)
    }
}

fn default_true() -> bool {
    true
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_plan_creation() {
        let plan = FixPlan::new("test.qcow2".to_string(), "security".to_string());
        assert_eq!(plan.version, "1.0");
        assert_eq!(plan.vm, "test.qcow2");
        assert_eq!(plan.profile, "security");
    }

    #[test]
    fn test_priority_ordering() {
        assert!(Priority::Critical < Priority::High);
        assert!(Priority::High < Priority::Medium);
        assert!(Priority::Medium < Priority::Low);
    }
}
