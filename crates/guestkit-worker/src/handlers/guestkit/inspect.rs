//! Guestkit inspect handler - VM disk inspection

use async_trait::async_trait;
use guestkit_job_spec::Payload;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use crate::error::{WorkerError, WorkerResult};
use crate::handler::{OperationHandler, HandlerContext, HandlerResult};

/// Inspect operation payload
#[derive(Debug, Clone, Deserialize, Serialize)]
struct InspectPayload {
    image: ImageSpec,
    #[serde(default)]
    options: InspectOptions,
    #[serde(skip_serializing_if = "Option::is_none")]
    output: Option<OutputSpec>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
struct ImageSpec {
    path: String,
    format: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    checksum: Option<String>,
    #[serde(default = "default_true")]
    read_only: bool,
}

fn default_true() -> bool {
    true
}

#[derive(Debug, Clone, Deserialize, Serialize, Default)]
struct InspectOptions {
    #[serde(default)]
    deep_scan: bool,
    #[serde(default = "default_true")]
    include_packages: bool,
    #[serde(default = "default_true")]
    include_services: bool,
    #[serde(default = "default_true")]
    include_users: bool,
    #[serde(default = "default_true")]
    include_network: bool,
    #[serde(default = "default_true")]
    include_security: bool,
    #[serde(default = "default_true")]
    include_storage: bool,
    #[serde(default)]
    include_databases: bool,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
struct OutputSpec {
    format: String,
    destination: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    compression: Option<String>,
}

/// Guestkit inspect handler
pub struct InspectHandler {
    /// Temporary directory for operations
    temp_dir: PathBuf,
}

impl InspectHandler {
    /// Create a new inspect handler
    pub fn new() -> Self {
        Self {
            temp_dir: std::env::temp_dir().join("guestkit-inspect"),
        }
    }

    /// Verify image checksum if provided
    async fn verify_checksum(&self, path: &str, expected: &str) -> WorkerResult<bool> {
        // TODO: Implement checksum verification
        // For now, just log and return true
        log::debug!("Checksum verification for {}: {} (not implemented)", path, expected);
        Ok(true)
    }

    /// Perform VM disk inspection
    async fn inspect_vm(
        &self,
        context: &HandlerContext,
        payload: &InspectPayload,
    ) -> WorkerResult<serde_json::Value> {
        context.report_progress("validation", Some(5), "Validating image").await?;

        // Verify image exists
        let image_path = std::path::Path::new(&payload.image.path);
        if !image_path.exists() {
            return Err(WorkerError::ExecutionError(
                format!("Image not found: {}", payload.image.path)
            ));
        }

        // Verify checksum if provided
        if let Some(ref checksum) = payload.image.checksum {
            if !self.verify_checksum(&payload.image.path, checksum).await? {
                return Err(WorkerError::ExecutionError(
                    "Image checksum verification failed".to_string()
                ));
            }
        }

        context.report_progress("inspection", Some(20), "Starting VM inspection").await?;

        // Perform real inspection using guestkit library
        let inspection_result = self.real_inspection(&payload).await?;

        context.report_progress("analysis", Some(80), "Analyzing results").await?;

        // Generate output
        let output_path = if let Some(ref output) = payload.output {
            context.report_progress("export", Some(90), "Writing output file").await?;
            self.write_output(&inspection_result, output).await?
        } else {
            // Write to temp directory
            let temp_file = context.work_dir.join(format!("{}-result.json", context.job_id));
            tokio::fs::write(
                &temp_file,
                serde_json::to_string_pretty(&inspection_result)?
            ).await?;
            temp_file.to_string_lossy().to_string()
        };

        context.report_progress("complete", Some(100), "Inspection complete").await?;

        Ok(serde_json::json!({
            "status": "success",
            "output_file": output_path,
            "summary": {
                "image": payload.image.path,
                "format": payload.image.format,
                "inspection_time": chrono::Utc::now().to_rfc3339(),
            }
        }))
    }

    /// Real inspection using guestkit library
    async fn real_inspection(&self, payload: &InspectPayload) -> WorkerResult<serde_json::Value> {
        // Run blocking guestkit operations in a separate thread
        let payload_clone = payload.clone();

        tokio::task::spawn_blocking(move || -> WorkerResult<serde_json::Value> {
            use guestkit::Guestfs;

            // Create guestfs handle
            let mut g = Guestfs::new()
                .map_err(|e| WorkerError::ExecutionError(format!("Failed to create Guestfs handle: {}", e)))?;

            // Add drive in read-only mode
            g.add_drive_ro(&payload_clone.image.path)
                .map_err(|e| WorkerError::ExecutionError(format!("Failed to add drive: {}", e)))?;

            // Launch the VM
            g.launch()
                .map_err(|e| WorkerError::ExecutionError(format!("Failed to launch: {}", e)))?;

            // Inspect the OS
            let inspected_oses = g.inspect()
                .map_err(|e| WorkerError::ExecutionError(format!("Failed to inspect OS: {}", e)))?;

            if inspected_oses.is_empty() {
                return Err(WorkerError::ExecutionError("No operating system found in image".to_string()));
            }

            // Use the first OS found
            let os_info = &inspected_oses[0];

            // Build result structure
            let mut result = serde_json::json!({
                "version": "1.0",
                "image": {
                    "path": payload_clone.image.path,
                    "format": payload_clone.image.format,
                },
                "operating_system": {
                    "type": os_info.os_type,
                    "distribution": os_info.distro,
                    "product_name": os_info.product_name,
                    "version": format!("{}.{}", os_info.major_version, os_info.minor_version),
                    "major_version": os_info.major_version,
                    "minor_version": os_info.minor_version,
                    "hostname": os_info.hostname,
                    "arch": os_info.arch,
                    "package_format": os_info.package_format,
                },
                "mountpoints": os_info.mountpoints,
            });

            // Mount the root filesystem
            g.mount_ro(&os_info.root, "/")
                .map_err(|e| WorkerError::ExecutionError(format!("Failed to mount root: {}", e)))?;

            // Collect packages if requested
            if payload_clone.options.include_packages {
                let packages = match os_info.package_format.as_str() {
                    "deb" => g.dpkg_list().ok(),
                    "rpm" => g.rpm_list().ok(),
                    _ => None,
                };

                if let Some(pkg_list) = packages {
                    result["packages"] = serde_json::json!({
                        "count": pkg_list.len(),
                        "manager": os_info.package_format,
                        "packages": pkg_list,
                    });
                }
            }

            // Collect services if requested
            if payload_clone.options.include_services {
                if let Ok(services) = g.list_enabled_services() {
                    result["services"] = serde_json::json!({
                        "count": services.len(),
                        "enabled_services": services,
                    });
                }
            }

            // Collect network interfaces if requested
            if payload_clone.options.include_network {
                if let Ok(interfaces) = g.list_network_interfaces() {
                    result["network"] = serde_json::json!({
                        "interfaces": interfaces,
                    });
                }

                // Get hostname
                if let Ok(hostname) = g.get_hostname() {
                    if let Some(net) = result.get_mut("network") {
                        net["hostname"] = serde_json::Value::String(hostname);
                    }
                }
            }

            // Collect security information if requested
            if payload_clone.options.include_security {
                let mut security = serde_json::json!({});

                // SELinux status
                if let Ok(selinux_status) = g.getcon() {
                    security["selinux"] = serde_json::json!({
                        "status": selinux_status,
                        "enabled": selinux_status != "disabled",
                    });
                }

                // Check for AppArmor
                if g.exists("/sys/kernel/security/apparmor").unwrap_or(false) {
                    security["apparmor"] = serde_json::json!({
                        "enabled": true,
                    });
                } else {
                    security["apparmor"] = serde_json::json!({
                        "enabled": false,
                    });
                }

                result["security"] = security;
            }

            // Unmount and cleanup
            let _ = g.umount_all();
            let _ = g.shutdown();

            Ok(result)
        })
        .await
        .map_err(|e| WorkerError::ExecutionError(format!("Task join error: {}", e)))?
    }

    /// Write output to specified destination
    async fn write_output(
        &self,
        data: &serde_json::Value,
        output: &OutputSpec,
    ) -> WorkerResult<String> {
        let content = match output.format.as_str() {
            "json" => serde_json::to_string_pretty(data)?,
            "yaml" => serde_yaml::to_string(data)
                .map_err(|e| WorkerError::ExecutionError(format!("YAML serialization failed: {}", e)))?,
            _ => {
                return Err(WorkerError::ExecutionError(
                    format!("Unsupported output format: {}", output.format)
                ));
            }
        };

        // Ensure parent directory exists
        let output_path = std::path::Path::new(&output.destination);
        if let Some(parent) = output_path.parent() {
            tokio::fs::create_dir_all(parent).await?;
        }

        tokio::fs::write(&output.destination, content).await?;

        Ok(output.destination.clone())
    }
}

impl Default for InspectHandler {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl OperationHandler for InspectHandler {
    fn name(&self) -> &str {
        "guestkit-inspect"
    }

    fn operations(&self) -> Vec<String> {
        vec!["guestkit.inspect".to_string()]
    }

    async fn validate(&self, payload: &Payload) -> WorkerResult<()> {
        // Parse and validate payload
        let inspect_payload: InspectPayload = serde_json::from_value(payload.data.clone())
            .map_err(|e| WorkerError::ExecutionError(
                format!("Invalid inspect payload: {}", e)
            ))?;

        // Validate image path
        if inspect_payload.image.path.is_empty() {
            return Err(WorkerError::ExecutionError(
                "Image path cannot be empty".to_string()
            ));
        }

        // Validate format
        let supported_formats = ["qcow2", "vmdk", "vdi", "vhdx", "raw", "img"];
        if !supported_formats.contains(&inspect_payload.image.format.as_str()) {
            return Err(WorkerError::ExecutionError(
                format!("Unsupported image format: {}", inspect_payload.image.format)
            ));
        }

        Ok(())
    }

    async fn execute(
        &self,
        context: HandlerContext,
        payload: Payload,
    ) -> WorkerResult<HandlerResult> {
        log::info!("Starting VM inspection for job {}", context.job_id);

        // Parse payload
        let inspect_payload: InspectPayload = serde_json::from_value(payload.data)
            .map_err(|e| WorkerError::ExecutionError(
                format!("Failed to parse inspect payload: {}", e)
            ))?;

        // Perform inspection
        let result_data = self.inspect_vm(&context, &inspect_payload).await?;

        // Extract output path
        let output_file = result_data
            .get("output_file")
            .and_then(|v| v.as_str())
            .map(|s| s.to_string());

        Ok(HandlerResult::new()
            .with_output(output_file.unwrap_or_default())
            .with_data(result_data))
    }

    async fn cleanup(&self, context: &HandlerContext) -> WorkerResult<()> {
        // Clean up any temporary files
        log::debug!("Cleanup for job {}", context.job_id);
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::progress::ProgressTracker;
    use std::sync::Arc;
    use tempfile::TempDir;

    #[tokio::test]
    async fn test_inspect_handler_validation() {
        let handler = InspectHandler::new();

        // Valid payload
        let valid_payload = Payload {
            payload_type: "guestkit.inspect.v1".to_string(),
            data: serde_json::json!({
                "image": {
                    "path": "/vms/test.qcow2",
                    "format": "qcow2"
                }
            }),
        };

        assert!(handler.validate(&valid_payload).await.is_ok());

        // Invalid format
        let invalid_payload = Payload {
            payload_type: "guestkit.inspect.v1".to_string(),
            data: serde_json::json!({
                "image": {
                    "path": "/vms/test.invalid",
                    "format": "invalid"
                }
            }),
        };

        assert!(handler.validate(&invalid_payload).await.is_err());
    }

    #[tokio::test]
    async fn test_inspect_handler_operations() {
        let handler = InspectHandler::new();
        assert_eq!(handler.operations(), vec!["guestkit.inspect"]);
        assert_eq!(handler.name(), "guestkit-inspect");
    }
}
