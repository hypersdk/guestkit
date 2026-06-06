// SPDX-License-Identifier: Apache-2.0

use axum::extract::{Multipart, Path, Query, State};
use axum::Json;
use guestkit::export::{generate_kubevirt_manifests, manifests_to_yaml, DiskMetadata};
use serde::Deserialize;
use std::path::PathBuf;
use uuid::Uuid;
use crate::error::{ApiError, ApiResult};
use crate::jobs::{build_job, enqueue_job};
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
    let mut filename = String::from("uploaded-image");
    let mut bytes: Vec<u8> = Vec::new();

    while let Some(field) = multipart
        .next_field()
        .await
        .map_err(|e| ApiError::bad_request(e.to_string()))?
    {
        if field.name() == Some("file") {
            if let Some(name) = field.file_name() {
                filename = name.to_string();
            }
            bytes = field
                .bytes()
                .await
                .map_err(|e| ApiError::bad_request(e.to_string()))?
                .to_vec();
        }
    }

    if bytes.is_empty() {
        return Err(ApiError::bad_request("file field is required"));
    }

    let id = Uuid::new_v4();
    let format = PathBuf::from(&filename)
        .extension()
        .and_then(|e| e.to_str())
        .unwrap_or("qcow2")
        .to_lowercase();
    let object_key = format!("{id}.{format}");
    let disk_path = state.config.storage_path.join(&object_key);
    tokio::fs::write(&disk_path, &bytes)
        .await
        .map_err(|e| ApiError::internal(e.to_string()))?;

    sqlx::query(
        r#"INSERT INTO vm_images (id, tenant, name, object_key, format, size_bytes, status)
           VALUES ($1, $2, $3, $4, $5, $6, $7)"#,
    )
    .bind(id)
    .bind("default")
    .bind(&filename)
    .bind(&object_key)
    .bind(&format)
    .bind(bytes.len() as i64)
    .bind("imported")
    .execute(&state.pool)
    .await
    .map_err(|e| ApiError::internal(e.to_string()))?;

    Ok(Json(ApiResponse::ok(VmImportResponse {
        id,
        name: filename,
        format,
        size_bytes: bytes.len() as i64,
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

async fn load_vm(state: &AppState, id: Uuid) -> ApiResult<VmImage> {
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
    let job_id = Uuid::new_v4();
    let image_path = state.config.storage_path.join(&vm.object_key);
    if let Some(obj) = data.as_object_mut() {
        obj.insert(
            "image".into(),
            serde_json::json!({
                "path": image_path.display().to_string(),
                "format": vm.format,
            }),
        );
    }

    let job = build_job(job_id, operation, payload_type, data)
        .map_err(|e| ApiError::internal(e.to_string()))?;

    let mut redis = state.redis.clone();
    enqueue_job(&mut redis, &job)
        .await
        .map_err(|e| ApiError::internal(e.to_string()))?;

    sqlx::query(
        "INSERT INTO jobs (id, vm_id, operation, status) VALUES ($1, $2, $3, $4)",
    )
    .bind(job_id)
    .bind(vm.id)
    .bind(operation)
    .bind("pending")
    .execute(&state.pool)
    .await
    .map_err(|e| ApiError::internal(e.to_string()))?;

    Ok(JobEnqueueResponse {
        job_id,
        operation: operation.to_string(),
        status: "pending".to_string(),
    })
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

    if query.apply {
        tracing::warn!("apply=true requested but cluster apply is not configured in MVP");
    }

    Ok(Json(ApiResponse::ok(ProvisionResponse {
        vm_id: id,
        yaml,
        applied: false,
    })))
}
