// SPDX-License-Identifier: Apache-2.0

mod agent;
mod copilot;
mod health;
mod jobs;
pub(crate) mod kubevirt;
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
        .route("/api/v1/vms/:id/agent/ping", post(agent::ping_agent))
        .route("/api/v1/vms/:id/agent/evidence", post(agent::agent_evidence))
        .route("/api/v1/vms/:id/agent/doctor", post(agent::agent_doctor))
        .route("/api/v1/vms/:id/agent/rpc", post(agent::agent_rpc))
        .route("/api/v1/vms/:id/agent/fix", post(agent::agent_fix))
        .route("/api/v1/kubevirt/vms", get(kubevirt::list_kubevirt_vms))
        .route(
            "/api/v1/kubevirt/vms/:namespace/:name/guest-agent",
            get(kubevirt::get_guest_agent_info),
        )
        .route(
            "/api/v1/kubevirt/vms/:namespace/:name/guest-agent/install",
            post(kubevirt::install_guest_agent),
        )
        .route(
            "/api/v1/kubevirt/vms/:namespace/:name/boot-inspect",
            get(crate::kubevirt_boot_inspect::get_boot_inspect)
                .post(crate::kubevirt_boot_inspect::post_boot_inspect_vm),
        )
        .route(
            "/api/v1/kubevirt/vms/:namespace/:name/inspect/boot",
            get(crate::kubevirt_boot_inspect::get_boot_inspect)
                .post(crate::kubevirt_boot_inspect::post_boot_inspect_vm),
        )
        .route(
            "/api/v1/kubevirt/boot-inspect",
            post(crate::kubevirt_boot_inspect::post_boot_inspect),
        )
}
