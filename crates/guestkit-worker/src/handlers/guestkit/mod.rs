//! Guestkit operation handlers
//!
//! These handlers integrate with the guestkit core library to perform
//! actual VM operations.

pub mod inspect;
pub mod profile;
pub mod agent;

pub use inspect::InspectHandler;
pub use profile::ProfileHandler;
pub use agent::{AgentEvidenceHandler, AgentFixHandler};
