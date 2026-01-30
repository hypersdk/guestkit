//! REST API server for job submission and management

pub mod handlers;
pub mod server;
pub mod types;

pub use server::{ApiServer, ApiServerConfig};
pub use types::{ApiError, ApiResponse, JobSubmitRequest, JobStatusResponse};
