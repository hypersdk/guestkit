// SPDX-License-Identifier: LGPL-3.0-or-later
//! Worker handlers for live guest agent RPC via host proxy.

use async_trait::async_trait;
use guestkit_job_spec::Payload;
use serde::{Deserialize, Serialize};
use crate::error::{WorkerError, WorkerResult};
use crate::handler::{HandlerContext, HandlerResult, OperationHandler};

#[derive(Debug, Clone, Deserialize, Serialize)]
struct AgentProxyPayload {
    /// HTTP base URL of guestkit agent-proxy (e.g. http://127.0.0.1:8765)
    proxy_url: String,
    #[serde(default = "default_target")]
    target: String,
}

fn default_target() -> String {
    "kvm".to_string()
}

async fn http_get_json(url: &str) -> WorkerResult<serde_json::Value> {
    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(120))
        .build()
        .map_err(|e| WorkerError::ExecutionError(e.to_string()))?;
    let resp = client
        .get(url)
        .send()
        .await
        .map_err(|e| WorkerError::ExecutionError(format!("agent proxy request failed: {e}")))?;
    if !resp.status().is_success() {
        return Err(WorkerError::ExecutionError(format!(
            "agent proxy returned {}",
            resp.status()
        )));
    }
    resp.json()
        .await
        .map_err(|e| WorkerError::ExecutionError(format!("parse agent response: {e}")))
}

pub struct AgentEvidenceHandler;

#[async_trait]
impl OperationHandler for AgentEvidenceHandler {
    fn name(&self) -> &str {
        "guestkit-agent-evidence"
    }

    fn operations(&self) -> Vec<String> {
        vec![guestkit_job_spec::operations::GUESTKIT_AGENT_EVIDENCE.to_string()]
    }

    async fn validate(&self, payload: &Payload) -> WorkerResult<()> {
        let p: AgentProxyPayload = serde_json::from_value(payload.data.clone())
            .map_err(|e| WorkerError::ExecutionError(e.to_string()))?;
        if p.proxy_url.is_empty() {
            return Err(WorkerError::ExecutionError(
                "proxy_url is required".into(),
            ));
        }
        Ok(())
    }

    async fn execute(
        &self,
        context: HandlerContext,
        payload: Payload,
    ) -> WorkerResult<HandlerResult> {
        let p: AgentProxyPayload = serde_json::from_value(payload.data)
            .map_err(|e| WorkerError::ExecutionError(e.to_string()))?;
        context
            .report_progress("agent", Some(10), "Fetching live evidence")
            .await?;
        let url = format!("{}/evidence", p.proxy_url.trim_end_matches('/'));
        let data = http_get_json(&url).await?;
        context.report_progress("complete", Some(100), "Done").await?;
        Ok(HandlerResult::new().with_data(data))
    }
}

pub struct AgentFixHandler;

#[async_trait]
impl OperationHandler for AgentFixHandler {
    fn name(&self) -> &str {
        "guestkit-agent-fix"
    }

    fn operations(&self) -> Vec<String> {
        vec![guestkit_job_spec::operations::GUESTKIT_AGENT_FIX.to_string()]
    }

    async fn validate(&self, payload: &Payload) -> WorkerResult<()> {
        let _: AgentProxyPayload = serde_json::from_value(payload.data.clone())
            .map_err(|e| WorkerError::ExecutionError(e.to_string()))?;
        Ok(())
    }

    async fn execute(
        &self,
        context: HandlerContext,
        payload: Payload,
    ) -> WorkerResult<HandlerResult> {
        let plan = payload.data.get("plan").cloned().unwrap_or_default();
        let p: AgentProxyPayload = serde_json::from_value(payload.data)
            .map_err(|e| WorkerError::ExecutionError(e.to_string()))?;
        context
            .report_progress("agent", Some(10), "Applying fix plan via agent")
            .await?;
        let url = format!("{}/fix-plan", p.proxy_url.trim_end_matches('/'));
        let client = reqwest::Client::new();
        let resp = client
            .post(&url)
            .json(&plan)
            .send()
            .await
            .map_err(|e| WorkerError::ExecutionError(e.to_string()))?;
        let data: serde_json::Value = resp
            .json()
            .await
            .map_err(|e| WorkerError::ExecutionError(e.to_string()))?;
        context.report_progress("complete", Some(100), "Done").await?;
        Ok(HandlerResult::new().with_data(data))
    }
}
