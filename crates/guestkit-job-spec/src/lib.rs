//! VM Operations Job Protocol - Type definitions and validation
//!
//! This crate provides the type definitions for the VM Operations Job Protocol v1.
//! It supports serialization/deserialization and validation of job specifications.

pub mod error;
pub mod types;
pub mod validation;
pub mod builder;

// Re-export main types
pub use error::{JobError, JobResult};
pub use types::{
    Job, JobDocument, JobMetadata, ExecutionPolicy, Constraints,
    Routing, Observability, Audit, Payload, WorkerCapabilities,
    JobResult as JobResultType, ProgressEvent, JobStatus,
    ExecutionSummary, JobOutputs, JobExecutionError, ExecutionMetrics,
};
pub use validation::JobValidator;
pub use builder::JobBuilder;

/// Protocol version
pub const PROTOCOL_VERSION: &str = "1.0";

/// Operation namespaces
pub mod operations {
    /// Guestkit operations
    pub const GUESTKIT_INSPECT: &str = "guestkit.inspect";
    pub const GUESTKIT_PROFILE: &str = "guestkit.profile";
    pub const GUESTKIT_FIX: &str = "guestkit.fix";
    pub const GUESTKIT_CONVERT: &str = "guestkit.convert";
    pub const GUESTKIT_COMPARE: &str = "guestkit.compare";

    /// hyper2kvm operations (future)
    pub const HYPER2KVM_CONVERT: &str = "hyper2kvm.convert";
    pub const HYPER2KVM_VALIDATE: &str = "hyper2kvm.validate";

    /// System operations
    pub const SYSTEM_HEALTH_CHECK: &str = "system.health-check";
    pub const SYSTEM_CAPABILITY_PROBE: &str = "system.capability-probe";
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_protocol_version() {
        assert_eq!(PROTOCOL_VERSION, "1.0");
    }
}
