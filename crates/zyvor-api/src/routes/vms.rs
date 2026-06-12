// SPDX-License-Identifier: Apache-2.0

use axum::extract::{Multipart, Path, Query, State};
use axum::Json;
use guestkit::export::{generate_kubevirt_manifests, manifests_to_yaml, DiskMetadata};
use serde::Deserialize;
use std::path::PathBuf;
use uuid::Uuid;
use crate::error::{ApiError, ApiResult};
use crate::jobs::{build_job, enqueue_job, submit_disk_path_job};
use crate::models::{
    ApiResponse, JobEnqueueResponse, JobRecord, ProvisionResponse, VmImage, VmImportResponse,
};
use crate::state::AppState;

#[derive(Debug, Deserialize)]
pub struct DoctorQuery {
    #[serde(default = "default_target")]
    pub target: String,
    #[serde(default)]
    pub explain: bool,
}

fn default_target() -> String {
    "kubevirt".to_string()
}

#[derive(Debug, Deserialize)]
pub struct ProvisionQuery {
    #[serde(default)]
    pub apply: bool,
}

pub async fn import_vm(
    State(state): State<AppState>,
    mut multipart: Multipart,
) -> ApiResult<Json<ApiResponse<VmImportResponse>>> {
    use tokio::io::AsyncWriteExt;

    let mut filename = String::from("uploaded-image");
    let mut size_bytes: u64 = 0;
    let mut wrote_file = false;
    let id = Uuid::new_v4();
    let mut disk_path = state.config.storage_path.join("pending");

    while let Some(mut field) = multipart
        .next_field()
        .await
        .map_err(|e| ApiError::bad_request(e.to_string()))?
    {
        if field.name() == Some("file") {
            if let Some(name) = field.file_name() {
                filename = name.to_string();
            }
            let format = PathBuf::from(&filename)
                .extension()
                .and_then(|e| e.to_str())
                .unwrap_or("qcow2")
                .to_lowercase();
            let object_key = format!("{id}.{format}");
            disk_path = state.config.storage_path.join(&object_key);
            let mut out = tokio::fs::File::create(&disk_path)
                .await
                .map_err(|e| ApiError::internal(e.to_string()))?;

            while let Some(chunk) = field
                .chunk()
                .await
                .map_err(|e| ApiError::bad_request(e.to_string()))?
            {
                out.write_all(&chunk)
                    .await
                    .map_err(|e| ApiError::internal(e.to_string()))?;
                size_bytes += chunk.len() as u64;
            }
            out.flush()
                .await
                .map_err(|e| ApiError::internal(e.to_string()))?;
            wrote_file = true;
        }
    }

    if !wrote_file || size_bytes == 0 {
        let _ = tokio::fs::remove_file(&disk_path).await;
        return Err(ApiError::bad_request("file field is required"));
    }

    let format = PathBuf::from(&filename)
        .extension()
        .and_then(|e| e.to_str())
        .unwrap_or("qcow2")
        .to_lowercase();
    let object_key = format!("{id}.{format}");

    sqlx::query(
        r#"INSERT INTO vm_images (id, tenant, name, object_key, format, size_bytes, status)
           VALUES ($1, $2, $3, $4, $5, $6, $7)"#,
    )
    .bind(id)
    .bind("default")
    .bind(&filename)
    .bind(&object_key)
    .bind(&format)
    .bind(size_bytes as i64)
    .bind("imported")
    .execute(&state.pool)
    .await
    .map_err(|e| ApiError::internal(e.to_string()))?;

    Ok(Json(ApiResponse::ok(VmImportResponse {
        id,
        name: filename,
        format,
        size_bytes: size_bytes as i64,
        path: disk_path.display().to_string(),
    })))
}

pub async fn list_vms(
    State(state): State<AppState>,
) -> ApiResult<Json<ApiResponse<Vec<VmImage>>>> {
    let rows = sqlx::query_as::<_, VmImage>(
        "SELECT id, tenant, name, object_key, format, size_bytes, checksum, status, created_at FROM vm_images ORDER BY created_at DESC",
    )
    .fetch_all(&state.pool)
    .await
    .map_err(|e| ApiError::internal(e.to_string()))?;
    Ok(Json(ApiResponse::ok(rows)))
}

pub(crate) async fn load_vm(state: &AppState, id: Uuid) -> ApiResult<VmImage> {
    sqlx::query_as::<_, VmImage>(
        "SELECT id, tenant, name, object_key, format, size_bytes, checksum, status, created_at FROM vm_images WHERE id = $1",
    )
    .bind(id)
    .fetch_optional(&state.pool)
    .await
    .map_err(|e| ApiError::internal(e.to_string()))?
    .ok_or_else(|| ApiError::not_found(format!("VM {id} not found")))
}

async fn submit_vm_job(
    state: &AppState,
    vm: &VmImage,
    operation: &str,
    payload_type: &str,
    mut data: serde_json::Value,
) -> ApiResult<JobEnqueueResponse> {
    let image_path = state.config.storage_path.join(&vm.object_key);
    submit_disk_path_job(
        state,
        vm.id,
        &image_path,
        &vm.format,
        operation,
        payload_type,
        data,
    )
    .await
}

pub async fn inspect_vm(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> ApiResult<Json<ApiResponse<JobEnqueueResponse>>> {
    let vm = load_vm(&state, id).await?;
    let resp = submit_vm_job(
        &state,
        &vm,
        "guestkit.inspect",
        "guestkit.inspect.v1",
        serde_json::json!({
            "options": {
                "include_packages": true,
                "include_services": true,
                "include_security": true
            }
        }),
    )
    .await?;
    Ok(Json(ApiResponse::ok(resp)))
}

pub async fn doctor_vm(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
    Query(query): Query<DoctorQuery>,
) -> ApiResult<Json<ApiResponse<JobEnqueueResponse>>> {
    let vm = load_vm(&state, id).await?;
    let resp = submit_vm_job(
        &state,
        &vm,
        "guestkit.doctor",
        "guestkit.doctor.v1",
        serde_json::json!({
            "target": query.target,
            "explain": query.explain,
        }),
    )
    .await?;
    Ok(Json(ApiResponse::ok(resp)))
}

pub async fn migration_plan_vm(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
    Query(query): Query<DoctorQuery>,
) -> ApiResult<Json<ApiResponse<JobEnqueueResponse>>> {
    let vm = load_vm(&state, id).await?;
    let resp = submit_vm_job(
        &state,
        &vm,
        "guestkit.migrate-plan",
        "guestkit.migrate-plan.v1",
        serde_json::json!({
            "target": query.target,
            "explain": query.explain,
            "export_fix_plan": true,
        }),
    )
    .await?;
    Ok(Json(ApiResponse::ok(resp)))
}

pub async fn repair_plan_vm(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> ApiResult<Json<ApiResponse<JobEnqueueResponse>>> {
    let vm = load_vm(&state, id).await?;
    let resp = submit_vm_job(
        &state,
        &vm,
        "guestkit.repair",
        "guestkit.repair.v1",
        serde_json::json!({
            "fix": "boot",
            "dry_run": true,
        }),
    )
    .await?;
    Ok(Json(ApiResponse::ok(resp)))
}

pub async fn provision_vm(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
    Query(query): Query<ProvisionQuery>,
) -> ApiResult<Json<ApiResponse<ProvisionResponse>>> {
    let vm = load_vm(&state, id).await?;
    let image_path = state.config.storage_path.join(&vm.object_key);

    let plan_result = guestkit::assurance::run_migrate_plan(
        &image_path,
        "kubevirt",
        &guestkit::assurance::MigratePlanOptions {
            explain: false,
            verbose: false,
            export_fix_plan: true,
            inject_agent: false,
            ..Default::default()
        },
    )
    .map_err(|e| ApiError::internal(e.to_string()))?;

    let mut disk = DiskMetadata::from_image_path(
        &image_path,
        &state.config.default_namespace,
        &state.config.storage_class,
    );
    disk.import_url = Some(format!(
        "{}/{}",
        state.config.storage_public_url.trim_end_matches('/'),
        vm.object_key
    ));

    let manifests = generate_kubevirt_manifests(&plan_result, &disk)
        .map_err(|e| ApiError::internal(e.to_string()))?;
    let yaml = manifests_to_yaml(&manifests).map_err(|e| ApiError::internal(e.to_string()))?;

    let mut applied = false;
    let mut resources = None;
    let mut apply_errors = None;
    if query.apply {
        let client = state.kube.clone().ok_or_else(|| {
            ApiError::bad_request("provision apply=true requires in-cluster Kubernetes access")
        })?;
        let result = crate::kubevirt_apply::apply_kubevirt_manifests(&client, &yaml).await?;
        applied = result.applied;
        resources = Some(result.resources);
        if !result.errors.is_empty() {
            apply_errors = Some(result.errors);
        }
    }

    Ok(Json(ApiResponse::ok(ProvisionResponse {
        vm_id: id,
        yaml,
        applied,
        resources,
        apply_errors,
    })))
}
