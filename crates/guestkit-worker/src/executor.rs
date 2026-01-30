//! Job executor - orchestrates job execution using handlers

use guestkit_job_spec::{JobDocument, JobValidator};
use chrono::Utc;
use std::sync::Arc;
use std::time::Duration;
use crate::error::{WorkerError, WorkerResult};
use crate::handler::{HandlerRegistry, HandlerContext};
use crate::progress::ProgressTracker;
use crate::result::ResultWriter;
use crate::state::{JobState, JobStateMachine};
use dashmap::DashMap;

/// Job executor
pub struct JobExecutor {
    /// Worker ID
    worker_id: String,

    /// Handler registry
    registry: Arc<HandlerRegistry>,

    /// Result writer
    result_writer: Arc<ResultWriter>,

    /// Working directory
    work_dir: std::path::PathBuf,

    /// Idempotency cache (key -> result path)
    idempotency_cache: Arc<DashMap<String, String>>,
}

impl JobExecutor {
    /// Create a new executor
    pub fn new(
        worker_id: impl Into<String>,
        registry: Arc<HandlerRegistry>,
        result_writer: Arc<ResultWriter>,
        work_dir: impl Into<std::path::PathBuf>,
    ) -> Self {
        Self {
            worker_id: worker_id.into(),
            registry,
            result_writer,
            work_dir: work_dir.into(),
            idempotency_cache: Arc::new(DashMap::new()),
        }
    }

    /// Execute a job
    pub async fn execute(&self, job: JobDocument) -> WorkerResult<()> {
        let job_id = job.job_id.clone();
        let started_at = Utc::now();

        log::info!("Starting execution of job {}", job_id);

        // State machine
        let mut state = JobStateMachine::new();

        // Check idempotency
        if let Some(ref exec) = job.execution {
            if let Some(ref key) = exec.idempotency_key {
                if let Some(result_path) = self.idempotency_cache.get(key) {
                    log::info!(
                        "Job {} already completed with idempotency key {}: {}",
                        job_id,
                        key,
                        result_path.value()
                    );
                    return Ok(());
                }
            }
        }

        // Validate job
        state.transition(JobState::Queued)?;
        if let Err(e) = self.validate_job(&job).await {
            log::error!("Job {} validation failed: {}", job_id, e);
            self.result_writer
                .write_failure(
                    &job_id,
                    &self.worker_id,
                    started_at,
                    1,
                    "VALIDATION_ERROR",
                    e.to_string(),
                    Some("validation".to_string()),
                    false,
                )
                .await?;
            return Err(e);
        }

        // Assign and run
        state.transition(JobState::Assigned)?;
        state.transition(JobState::Running)?;

        // Setup timeout
        let timeout = job.execution.as_ref()
            .map(|e| Duration::from_secs(e.timeout_seconds))
            .unwrap_or(Duration::from_secs(3600));

        // Execute with timeout
        let result = tokio::time::timeout(
            timeout,
            self.execute_with_handler(job.clone())
        ).await;

        match result {
            Ok(Ok(handler_result)) => {
                // Success
                state.transition(JobState::Completed)?;

                log::info!("Job {} completed successfully", job_id);

                let result_path = self.result_writer
                    .write_success(
                        &job_id,
                        &self.worker_id,
                        started_at,
                        job.execution.as_ref().map(|e| e.attempt).unwrap_or(1),
                        job.execution.as_ref().and_then(|e| e.idempotency_key.clone()),
                        handler_result.output_file,
                        handler_result.artifacts,
                    )
                    .await?;

                // Cache idempotency result
                if let Some(ref exec) = job.execution {
                    if let Some(ref key) = exec.idempotency_key {
                        self.idempotency_cache.insert(key.clone(), result_path);
                    }
                }

                Ok(())
            }
            Ok(Err(e)) => {
                // Execution error
                state.transition(JobState::Failed)?;

                log::error!("Job {} failed: {}", job_id, e);

                self.result_writer
                    .write_failure(
                        &job_id,
                        &self.worker_id,
                        started_at,
                        job.execution.as_ref().map(|e| e.attempt).unwrap_or(1),
                        "EXECUTION_ERROR",
                        e.to_string(),
                        Some("execution".to_string()),
                        true,
                    )
                    .await?;

                Err(e)
            }
            Err(_) => {
                // Timeout
                state.transition(JobState::Timeout)?;

                log::error!("Job {} timed out after {:?}", job_id, timeout);

                self.result_writer
                    .write_failure(
                        &job_id,
                        &self.worker_id,
                        started_at,
                        job.execution.as_ref().map(|e| e.attempt).unwrap_or(1),
                        "TIMEOUT",
                        format!("Job exceeded timeout of {:?}", timeout),
                        Some("execution".to_string()),
                        true,
                    )
                    .await?;

                Err(WorkerError::Timeout {
                    seconds: timeout.as_secs(),
                })
            }
        }
    }

    /// Validate job before execution
    async fn validate_job(&self, job: &JobDocument) -> WorkerResult<()> {
        // Validate protocol
        JobValidator::validate(job)?;

        // Check if operation is supported
        if !self.registry.supports(&job.operation) {
            return Err(WorkerError::HandlerNotFound(job.operation.clone()));
        }

        // Get handler and validate payload
        if let Some(handler) = self.registry.get(&job.operation) {
            handler.validate(&job.payload).await?;
        }

        Ok(())
    }

    /// Execute job with handler
    async fn execute_with_handler(
        &self,
        job: JobDocument,
    ) -> WorkerResult<crate::handler::HandlerResult> {
        let handler = self.registry
            .get(&job.operation)
            .ok_or_else(|| WorkerError::HandlerNotFound(job.operation.clone()))?;

        // Create progress tracker
        let (progress, mut rx) = ProgressTracker::new(&job.job_id);

        // Spawn progress logger
        let job_id = job.job_id.clone();
        tokio::spawn(async move {
            while let Some(event) = rx.recv().await {
                log::info!(
                    "[{}] {} - {} ({}%)",
                    job_id,
                    event.phase,
                    event.message,
                    event.progress_percent.unwrap_or(0)
                );
            }
        });

        // Create handler context
        let context = HandlerContext::new(
            job.job_id.clone(),
            self.worker_id.clone(),
            Arc::new(progress),
            self.work_dir.clone(),
        );

        // Execute handler
        let result = handler.execute(context.clone(), job.payload).await;

        // Cleanup (always run, even on failure)
        if let Err(e) = handler.cleanup(&context).await {
            log::warn!("Cleanup failed for job {}: {}", job.job_id, e);
        }

        result
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::handler::{OperationHandler, HandlerResult};
    use async_trait::async_trait;
    use guestkit_job_spec::{Payload, builder::JobBuilder};
    use tempfile::TempDir;

    struct TestHandler;

    #[async_trait]
    impl OperationHandler for TestHandler {
        fn name(&self) -> &str {
            "test-handler"
        }

        fn operations(&self) -> Vec<String> {
            vec!["test.operation".to_string()]
        }

        async fn execute(
            &self,
            context: HandlerContext,
            _payload: Payload,
        ) -> WorkerResult<HandlerResult> {
            context.report_progress("testing", Some(50), "Running test").await?;
            Ok(HandlerResult::new().with_output("/tmp/result.json"))
        }
    }

    #[tokio::test]
    async fn test_executor() {
        let temp_dir = TempDir::new().unwrap();

        let mut registry = HandlerRegistry::new();
        registry.register(Arc::new(TestHandler));

        let result_writer = Arc::new(ResultWriter::new(temp_dir.path()));

        let executor = JobExecutor::new(
            "worker-test",
            Arc::new(registry),
            result_writer,
            temp_dir.path(),
        );

        let job = JobBuilder::new()
            .job_id("test-job-123")
            .operation("test.operation")
            .payload("test.operation.v1", serde_json::json!({}))
            .build()
            .unwrap();

        let result = executor.execute(job).await;
        assert!(result.is_ok());
    }
}
