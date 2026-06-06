//! Guestkit operation handlers
//!
//! These handlers integrate with the guestkit core library to perform
//! actual VM operations.

pub mod agent;
pub mod doctor;
pub mod inspect;
pub mod migrate_plan;
pub mod profile;
pub mod repair;

pub use agent::{AgentEvidenceHandler, AgentFixHandler};
pub use doctor::DoctorHandler;
pub use inspect::InspectHandler;
pub use migrate_plan::MigratePlanHandler;
pub use profile::ProfileHandler;
pub use repair::RepairHandler;
