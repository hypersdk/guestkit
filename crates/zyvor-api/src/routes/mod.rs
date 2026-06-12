// SPDX-License-Identifier: Apache-2.0

mod agent;
mod config;
mod copilot;
mod health;
mod jobs;
pub(crate) mod kubevirt;
mod storage;
mod system;
mod vmtools;
mod vms;

use axum::routing::{get, post, put};
use axum::Router;
use crate::state::AppState;

pub fn api_router() -> Router<AppState> {
    Router::new()
        .route("/api/v1/health", get(health::health))
        .route("/api/v1/config", get(config::get_config))
        .route("/api/v1/system/status", get(system::get_system_status))
        .route("/api/v1/storage/roots", get(storage::list_storage_roots))
        .route("/api/v1/storage/browse", get(storage::browse_storage))
        .route("/api/v1/vms/import-from-storage", post(storage::import_from_storage))
        .route("/api/v1/vmtools/bundle", get(vmtools::get_bundle))
        .route("/api/v1/vmtools/coverage", get(vmtools::get_coverage))
        .route("/api/v1/vmtools/policy", get(vmtools::get_policy).put(vmtools::put_policy))
        .route(
            "/api/v1/vmtools/policy/reconcile",
            post(vmtools::reconcile_policy),
        )
        .route(
            "/api/v1/kubevirt/vms/:namespace/:name/vmtools",
            get(vmtools::get_vm_vmtools),
        )
        .route(
            "/api/v1/kubevirt/vms/:namespace/:name/vmtools/install",
            post(vmtools::install_vm_vmtools),
        )
        .route(
            "/api/v1/kubevirt/vms/:namespace/:name/vmtools/diagnostics",
            post(vmtools::run_vm_diagnostics),
        )
        .route(
            "/api/v1/kubevirt/vms/:namespace/:name/vmtools/quiesce",
            post(crate::kubevirt_vmtools_ops::quiesce_vm_handler),
        )
        .route(
            "/api/v1/kubevirt/vms/:namespace/:name/vmtools/unquiesce",
            post(crate::kubevirt_vmtools_ops::unquiesce_vm_handler),
        )
        .route(
            "/api/v1/kubevirt/vms/:namespace/:name/vmtools/reboot",
            post(crate::kubevirt_vmtools_ops::reboot_vm_handler),
        )
        .route(
            "/api/v1/kubevirt/vms/:namespace/:name/vmtools/shutdown",
            post(crate::kubevirt_vmtools_ops::shutdown_vm_handler),
        )
        .route(
            "/api/v1/kubevirt/vms/:namespace/:name/vmtools/exec",
            post(crate::kubevirt_vmtools_ops::exec_vm_handler),
        )
        .route("/api/v1/vms/import", post(vms::import_vm))
        .route("/api/v1/vms/import-from-url", post(vms::import_from_url))
        .route("/api/v1/vms/import-from-s3", post(vms::import_from_s3))
        .route("/api/v1/vms/import-from-nfs", post(vms::import_from_nfs))
        .route("/api/v1/vms/compare", post(vms::compare_vms))
        .route("/api/v1/vms/cleanup-shadows", post(vms::cleanup_shadow_vms))
        .route("/api/v1/vms", get(vms::list_vms))
        .route("/api/v1/vms/:id", axum::routing::delete(vms::delete_vm))
        .route("/api/v1/vms/:id/jobs", get(vms::list_vm_jobs))
        .route("/api/v1/vms/:id/inspect", post(vms::inspect_vm))
        .route("/api/v1/vms/:id/doctor", post(vms::doctor_vm))
        .route("/api/v1/vms/:id/migration-plan", post(vms::migration_plan_vm))
        .route("/api/v1/vms/:id/repair-plan", post(vms::repair_plan_vm))
        .route("/api/v1/vms/:id/convert", post(vms::convert_vm))
        .route("/api/v1/vms/:id/readiness-report", post(vms::readiness_report))
        .route("/api/v1/vms/:id/provision", post(vms::provision_vm))
        .route("/api/v1/jobs/:id", get(jobs::get_job))
        .route("/api/v1/vms/:id/copilot/ask", post(copilot::ask_copilot))
        .route("/api/v1/vms/:id/copilot/briefing", get(copilot::get_vm_briefing))
        .route("/api/v1/vms/:id/copilot/launch-advice", post(copilot::launch_advice))
        .route("/api/v1/vms/:id/copilot/explain-check", post(copilot::explain_check))
        .route("/api/v1/vms/compare/copilot", post(copilot::compare_copilot))
        .route("/api/v1/copilot/fleet-overview", post(copilot::fleet_overview))
        .route("/api/v1/vms/:id/agent/ping", post(agent::ping_agent))
        .route("/api/v1/vms/:id/agent/evidence", post(agent::agent_evidence))
        .route("/api/v1/vms/:id/agent/doctor", post(agent::agent_doctor))
        .route("/api/v1/vms/:id/agent/rpc", post(agent::agent_rpc))
        .route("/api/v1/vms/:id/agent/fix", post(agent::agent_fix))
        .route("/api/v1/kubevirt/namespaces", get(kubevirt::list_kubevirt_namespaces))
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
        .route(
            "/api/v1/kubevirt/vms/:namespace/:name/restart",
            put(crate::kubevirt_lifecycle::restart_vm_handler),
        )
        .route(
            "/api/v1/kubevirt/vms/:namespace/:name/start",
            put(crate::kubevirt_lifecycle::start_vm_handler),
        )
        .route(
            "/api/v1/kubevirt/vms/:namespace/:name/stop",
            put(crate::kubevirt_lifecycle::stop_vm_handler),
        )
        .route(
            "/api/v1/kubevirt/vms/:namespace/:name/inspect",
            post(crate::kubevirt_inspect::post_inspect_vm),
        )
        .route(
            "/api/v1/kubevirt/vms/:namespace/:name/doctor",
            post(crate::kubevirt_inspect::post_doctor_vm),
        )
        .route(
            "/api/v1/kubevirt/vms/:namespace/:name/export-disk",
            post(crate::kubevirt_export::export_vm_disk),
        )
        .route(
            "/api/v1/kubevirt/vms/:namespace/:name/copilot/briefing",
            post(crate::kubevirt_copilot::cluster_briefing),
        )
        .route(
            "/api/v1/kubevirt/vms/:namespace/:name/copilot/ask",
            post(crate::kubevirt_copilot::cluster_ask),
        )
        .route(
            "/api/v1/kubevirt/apply",
            post(crate::kubevirt_apply::apply_yaml_handler),
        )
}
