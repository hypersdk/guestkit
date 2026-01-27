// SPDX-License-Identifier: LGPL-3.0-or-later
//! Plan application - executes fix plans with safety checks

use super::types::*;
use anyhow::{Result, Context};
use std::path::Path;

/// Applies fix plans to VM disks
pub struct PlanApplicator {
    vm_path: String,
    dry_run: bool,
}

impl PlanApplicator {
    /// Create a new plan applicator
    pub fn new(vm_path: String, dry_run: bool) -> Self {
        Self { vm_path, dry_run }
    }

    /// Apply a fix plan
    pub fn apply(&self, plan: &FixPlan) -> Result<ApplyResult> {
        if self.dry_run {
            return Ok(ApplyResult {
                success: true,
                operations_applied: 0,
                operations_failed: 0,
                operations_skipped: plan.operations.len(),
                message: "Dry run completed - no changes made".to_string(),
            });
        }

        // TODO: Implement actual application logic
        // This requires integration with guestfs operations

        Ok(ApplyResult {
            success: false,
            operations_applied: 0,
            operations_failed: 0,
            operations_skipped: plan.operations.len(),
            message: "Plan application not yet implemented".to_string(),
        })
    }

    /// Validate a plan before applying
    pub fn validate(&self, plan: &FixPlan) -> Result<ValidationResult> {
        let mut errors = Vec::new();
        let mut warnings = Vec::new();

        // Check VM exists
        if !Path::new(&self.vm_path).exists() {
            errors.push(format!("VM disk not found: {}", self.vm_path));
        }

        // Check for circular dependencies
        if self.has_circular_dependencies(plan) {
            errors.push("Plan contains circular dependencies".to_string());
        }

        // Check for missing dependencies
        for op in &plan.operations {
            for dep_id in &op.depends_on {
                if !plan.operations.iter().any(|o| &o.id == dep_id) {
                    errors.push(format!(
                        "Operation {} depends on non-existent operation {}",
                        op.id, dep_id
                    ));
                }
            }
        }

        // Warn about non-reversible operations
        let non_reversible: Vec<&str> = plan.operations.iter()
            .filter(|op| !op.reversible)
            .map(|op| op.id.as_str())
            .collect();

        if !non_reversible.is_empty() {
            warnings.push(format!(
                "Non-reversible operations: {}",
                non_reversible.join(", ")
            ));
        }

        Ok(ValidationResult {
            valid: errors.is_empty(),
            errors,
            warnings,
        })
    }

    /// Check for circular dependencies
    fn has_circular_dependencies(&self, _plan: &FixPlan) -> bool {
        // TODO: Implement proper cycle detection
        false
    }

    /// Create backup before applying plan
    fn create_backup(&self) -> Result<String> {
        // TODO: Implement backup creation
        Ok("/backup/vm-state".to_string())
    }

    /// Rollback to a previous state
    pub fn rollback(&self, _backup_path: &str) -> Result<()> {
        // TODO: Implement rollback
        Err(anyhow::anyhow!("Rollback not yet implemented"))
    }
}

/// Result of applying a plan
#[derive(Debug)]
pub struct ApplyResult {
    pub success: bool,
    pub operations_applied: usize,
    pub operations_failed: usize,
    pub operations_skipped: usize,
    pub message: String,
}

/// Result of validating a plan
#[derive(Debug)]
pub struct ValidationResult {
    pub valid: bool,
    pub errors: Vec<String>,
    pub warnings: Vec<String>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_applicator_creation() {
        let applicator = PlanApplicator::new("test.qcow2".to_string(), true);
        assert_eq!(applicator.vm_path, "test.qcow2");
        assert!(applicator.dry_run);
    }

    #[test]
    fn test_dry_run() {
        let applicator = PlanApplicator::new("test.qcow2".to_string(), true);
        let plan = FixPlan::new("test.qcow2".to_string(), "security".to_string());
        let result = applicator.apply(&plan).unwrap();
        assert!(result.success);
        assert_eq!(result.operations_applied, 0);
    }
}
