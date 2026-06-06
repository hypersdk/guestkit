// SPDX-License-Identifier: Apache-2.0

use axum::extract::{Path, State};
use axum::Json;
use guestkit::assurance::{answer_copilot_question, CopilotInsight, MigrationBriefing};
use serde::Deserialize;
use uuid::Uuid;
use crate::error::{ApiError, ApiResult};
use crate::jobs::get_job_result;
use crate::models::ApiResponse;
use crate::state::AppState;

#[derive(Debug, Deserialize)]
pub struct CopilotAskRequest {
    pub question: String,
    #[serde(default)]
    pub job_id: Option<Uuid>,
}

#[derive(Debug, serde::Serialize)]
pub struct CopilotAskResponse {
    pub insight: CopilotInsight,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub briefing: Option<MigrationBriefing>,
}

fn extract_briefing(job_result: &serde_json::Value) -> Option<MigrationBriefing> {
    if let Some(inner) = job_result.get("data") {
        if let Some(copilot) = inner.get("copilot") {
            return serde_json::from_value(copilot.clone()).ok();
        }
    }
    job_result
        .get("copilot")
        .and_then(|v| serde_json::from_value(v.clone()).ok())
}

pub async fn ask_copilot(
    State(state): State<AppState>,
    Path(_id): Path<Uuid>,
    Json(body): Json<CopilotAskRequest>,
) -> ApiResult<Json<ApiResponse<CopilotAskResponse>>> {
    if body.question.trim().is_empty() {
        return Err(ApiError::bad_request("question is required"));
    }

    let briefing = if let Some(job_id) = body.job_id {
        let mut redis = state.redis.clone();
        let result = get_job_result(&mut redis, &job_id.to_string())
            .await
            .ok_or_else(|| ApiError::not_found(format!("No result for job {job_id}")))?;
        extract_briefing(&result).ok_or_else(|| {
            ApiError::bad_request(
                "Job result has no copilot briefing — run doctor with explain=true first",
            )
        })?
    } else {
        return Err(ApiError::bad_request(
            "job_id is required — run doctor or migration-plan with explain=true first",
        ));
    };

    let insight = answer_copilot_question(&body.question, &briefing);

    Ok(Json(ApiResponse::ok(CopilotAskResponse {
        insight,
        briefing: Some(briefing),
    })))
}
