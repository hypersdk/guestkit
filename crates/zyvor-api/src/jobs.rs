// SPDX-License-Identifier: Apache-2.0

use anyhow::{Context, Result};
use chrono::Utc;
use guestkit_job_spec::{builder::JobBuilder, JobDocument};
use redis::aio::ConnectionManager;
use redis::AsyncCommands;
use uuid::Uuid;

use crate::error::{ApiError, ApiResult};
use crate::models::JobEnqueueResponse;
use crate::state::AppState;

pub const JOBS_STREAM: &str = "zyvor:jobs";

pub async fn enqueue_job(redis: &mut ConnectionManager, job: &JobDocument) -> Result<()> {
    let job_json = serde_json::to_string(job).context("serialize job")?;

    let _: String = redis::cmd("XADD")
        .arg(JOBS_STREAM)
        .arg("*")
        .arg("job")
        .arg(job_json)
        .query_async(redis)
        .await
        .context("XADD job")?;

    let status_key = format!("zyvor:job-status:{}", job.job_id);
    let status = serde_json::json!({
        "job_id": job.job_id,
        "status": "pending",
        "updated_at": Utc::now().to_rfc3339(),
    });
    redis
        .set_ex::<_, _, ()>(&status_key, status.to_string(), 86400)
        .await
        .context("set job status")?;
    Ok(())
}

pub async fn get_job_status(redis: &mut ConnectionManager, job_id: &str) -> Option<serde_json::Value> {
    let key = format!("zyvor:job-status:{job_id}");
    let raw: Option<String> = redis.get(&key).await.ok()?;
    raw.and_then(|s| serde_json::from_str(&s).ok())
}

pub async fn get_job_result(redis: &mut ConnectionManager, job_id: &str) -> Option<serde_json::Value> {
    let key = format!("zyvor:results:{job_id}");
    let raw: Option<String> = redis.get(&key).await.ok()?;
    raw.and_then(|s| serde_json::from_str(&s).ok())
}

pub fn build_job(
    job_id: Uuid,
    operation: &str,
    payload_type: &str,
    data: serde_json::Value,
) -> Result<JobDocument> {
    JobBuilder::new()
        .job_id(job_id.to_string())
        .operation(operation)
        .payload(payload_type, data)
        .timeout_seconds(7200)
        .build()
        .map_err(|e| anyhow::anyhow!("{e}"))
}

pub async fn submit_disk_path_job(
    state: &AppState,
    vm_id: Uuid,
    disk_path: &std::path::Path,
    format: &str,
    operation: &str,
    payload_type: &str,
    mut data: serde_json::Value,
) -> ApiResult<JobEnqueueResponse> {
    let job_id = Uuid::new_v4();
    if let Some(obj) = data.as_object_mut() {
        obj.insert(
            "image".into(),
            serde_json::json!({
                "path": disk_path.display().to_string(),
                "format": format,
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
    .bind(vm_id)
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
