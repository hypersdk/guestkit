// SPDX-License-Identifier: Apache-2.0

mod copilot;
mod health;
mod jobs;
mod vms;

use axum::routing::{get, post};
use axum::Router;
use crate::state::AppState;

pub fn api_router() -> Router<AppState> {
    Router::new()
        .route("/api/v1/health", get(health::health))
        .route("/api/v1/vms/import", post(vms::import_vm))
        .route("/api/v1/vms", get(vms::list_vms))
        .route("/api/v1/vms/:id/inspect", post(vms::inspect_vm))
        .route("/api/v1/vms/:id/doctor", post(vms::doctor_vm))
        .route("/api/v1/vms/:id/migration-plan", post(vms::migration_plan_vm))
        .route("/api/v1/vms/:id/repair-plan", post(vms::repair_plan_vm))
        .route("/api/v1/vms/:id/provision", post(vms::provision_vm))
        .route("/api/v1/jobs/:id", get(jobs::get_job))
        .route("/api/v1/vms/:id/copilot/ask", post(copilot::ask_copilot))
}
