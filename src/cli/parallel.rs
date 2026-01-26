// SPDX-License-Identifier: LGPL-3.0-or-later
//! Parallel processing for batch VM inspection operations
//!
//! This module provides parallel batch inspection capabilities using rayon,
//! enabling efficient processing of multiple VM disks concurrently.
//!
//! # Performance
//!
//! - Sequential: Process one disk at a time
//! - Parallel: Process N disks concurrently (where N = number of CPU cores)
//! - Expected speedup: ~4x on 4-core systems, ~8x on 8-core systems
//!
//! # Examples
//!
//! ```no_run
//! use guestctl::cli::parallel::{ParallelInspector, InspectionConfig};
//!
//! let disks = vec!["vm1.qcow2", "vm2.qcow2", "vm3.qcow2"];
//! let config = InspectionConfig::default();
//!
//! let results = ParallelInspector::new(config)
//!     .inspect_batch(&disks)?;
//!
//! for result in results {
//!     println!("Inspected: {:?}", result);
//! }
//! ```

use crate::core::{Error, Result};
use rayon::prelude::*;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

/// Configuration for parallel inspection operations
#[derive(Debug, Clone)]
pub struct InspectionConfig {
    /// Maximum number of parallel workers (0 = use all CPU cores)
    pub max_workers: usize,

    /// Enable caching for inspection results
    pub enable_cache: bool,

    /// Maximum time to wait for each inspection (seconds)
    pub timeout_secs: u64,

    /// Continue processing on error (don't fail fast)
    pub continue_on_error: bool,

    /// Enable verbose progress reporting
    pub verbose: bool,
}

impl Default for InspectionConfig {
    fn default() -> Self {
        Self {
            max_workers: 0, // Use all cores
            enable_cache: true,
            timeout_secs: 300, // 5 minutes per disk
            continue_on_error: true,
            verbose: false,
        }
    }
}

/// Result of a single inspection operation
#[derive(Debug, Clone)]
pub struct InspectionResult {
    /// Path to the inspected disk
    pub disk_path: PathBuf,

    /// Whether inspection was successful
    pub success: bool,

    /// Error message if inspection failed
    pub error: Option<String>,

    /// Time taken for inspection
    pub duration: Duration,

    /// OS type detected (if successful)
    pub os_type: Option<String>,

    /// OS product name (if successful)
    pub product_name: Option<String>,

    /// Whether result came from cache
    pub from_cache: bool,
}

impl InspectionResult {
    /// Create a successful result
    pub fn success(
        disk_path: PathBuf,
        duration: Duration,
        os_type: String,
        product_name: String,
        from_cache: bool,
    ) -> Self {
        Self {
            disk_path,
            success: true,
            error: None,
            duration,
            os_type: Some(os_type),
            product_name: Some(product_name),
            from_cache,
        }
    }

    /// Create a failed result
    pub fn failure(disk_path: PathBuf, duration: Duration, error: String) -> Self {
        Self {
            disk_path,
            success: false,
            error: Some(error),
            duration,
            os_type: None,
            product_name: None,
            from_cache: false,
        }
    }
}

/// Parallel batch inspector for VM disks
pub struct ParallelInspector {
    config: InspectionConfig,
}

impl ParallelInspector {
    /// Create a new parallel inspector with the given configuration
    pub fn new(config: InspectionConfig) -> Self {
        // Configure rayon thread pool
        if config.max_workers > 0 {
            rayon::ThreadPoolBuilder::new()
                .num_threads(config.max_workers)
                .build_global()
                .ok(); // Ignore error if already configured
        }

        Self { config }
    }

    /// Create a new inspector with default configuration
    pub fn default() -> Self {
        Self::new(InspectionConfig::default())
    }

    /// Inspect multiple disks in parallel
    ///
    /// This method processes all disks concurrently using rayon's parallel
    /// iterator. Results are returned in the same order as the input disks.
    ///
    /// # Arguments
    ///
    /// * `disk_paths` - Slice of disk paths to inspect
    ///
    /// # Returns
    ///
    /// Vector of inspection results, one per disk, in the same order as input.
    ///
    /// # Errors
    ///
    /// Returns error only if the entire operation fails. Individual disk
    /// inspection failures are captured in InspectionResult.
    pub fn inspect_batch<P: AsRef<Path> + Send + Sync>(
        &self,
        disk_paths: &[P],
    ) -> Result<Vec<InspectionResult>> {
        let start = Instant::now();

        if self.config.verbose {
            println!("🚀 Starting parallel inspection of {} disks", disk_paths.len());
            println!("👷 Workers: {}", self.num_workers());
        }

        // Convert paths to PathBuf for parallel processing
        let paths: Vec<PathBuf> = disk_paths
            .iter()
            .map(|p| p.as_ref().to_path_buf())
            .collect();

        // Process disks in parallel
        let results: Vec<InspectionResult> = paths
            .par_iter()
            .map(|path| self.inspect_single(path))
            .collect();

        let total_duration = start.elapsed();

        if self.config.verbose {
            self.print_summary(&results, total_duration);
        }

        Ok(results)
    }

    /// Inspect a single disk (called by parallel workers)
    fn inspect_single(&self, disk_path: &Path) -> InspectionResult {
        let start = Instant::now();

        if self.config.verbose {
            println!("🔍 Inspecting: {}", disk_path.display());
        }

        // TODO: Replace with actual guestfs inspection
        // For now, simulate inspection with a placeholder
        match self.simulate_inspection(disk_path) {
            Ok((os_type, product_name, from_cache)) => {
                let duration = start.elapsed();
                InspectionResult::success(
                    disk_path.to_path_buf(),
                    duration,
                    os_type,
                    product_name,
                    from_cache,
                )
            }
            Err(e) => {
                let duration = start.elapsed();
                InspectionResult::failure(
                    disk_path.to_path_buf(),
                    duration,
                    e.to_string(),
                )
            }
        }
    }

    /// Simulate inspection (placeholder for actual guestfs integration)
    fn simulate_inspection(&self, disk_path: &Path) -> Result<(String, String, bool)> {
        // Validate disk exists
        if !disk_path.exists() {
            return Err(Error::NotFound(format!(
                "Disk not found: {}",
                disk_path.display()
            )));
        }

        // TODO: Check cache first if enabled
        // TODO: Launch guestfs and inspect
        // TODO: Save to cache if enabled

        // Placeholder return
        Ok((
            "linux".to_string(),
            "Ubuntu 22.04 LTS".to_string(),
            false,
        ))
    }

    /// Get the number of worker threads
    pub fn num_workers(&self) -> usize {
        if self.config.max_workers > 0 {
            self.config.max_workers
        } else {
            rayon::current_num_threads()
        }
    }

    /// Print summary of batch inspection results
    fn print_summary(&self, results: &[InspectionResult], total_duration: Duration) {
        let successful = results.iter().filter(|r| r.success).count();
        let failed = results.iter().filter(|r| !r.success).count();
        let from_cache = results.iter().filter(|r| r.from_cache).count();

        println!("\n📊 Batch Inspection Summary");
        println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
        println!("Total disks:      {}", results.len());
        println!("✅ Successful:    {}", successful);
        println!("❌ Failed:        {}", failed);
        println!("💾 From cache:    {}", from_cache);
        println!("⏱️  Total time:    {:?}", total_duration);
        println!("⚡ Avg per disk:  {:?}", total_duration / results.len() as u32);
        println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━\n");
    }
}

/// Batch inspection with progress tracking
pub struct ProgressiveInspector {
    config: InspectionConfig,
    progress: Arc<Mutex<usize>>,
}

impl ProgressiveInspector {
    /// Create a new progressive inspector
    pub fn new(config: InspectionConfig) -> Self {
        Self {
            config,
            progress: Arc::new(Mutex::new(0)),
        }
    }

    /// Inspect batch with progress updates
    pub fn inspect_batch_with_progress<P, F>(
        &self,
        disk_paths: &[P],
        mut progress_callback: F,
    ) -> Result<Vec<InspectionResult>>
    where
        P: AsRef<Path> + Send + Sync,
        F: FnMut(usize, usize) + Send + Sync,
    {
        let total = disk_paths.len();
        let paths: Vec<PathBuf> = disk_paths
            .iter()
            .map(|p| p.as_ref().to_path_buf())
            .collect();

        let progress = Arc::clone(&self.progress);
        let callback = Arc::new(Mutex::new(progress_callback));

        let results: Vec<InspectionResult> = paths
            .par_iter()
            .map(|path| {
                // Perform inspection
                let inspector = ParallelInspector::new(self.config.clone());
                let result = inspector.inspect_single(path);

                // Update progress
                let mut count = progress.lock().unwrap();
                *count += 1;
                let current = *count;
                drop(count);

                // Call progress callback
                let mut cb = callback.lock().unwrap();
                cb(current, total);
                drop(cb);

                result
            })
            .collect();

        Ok(results)
    }
}

/// Helper function for simple batch inspection
pub fn inspect_batch<P: AsRef<Path> + Send + Sync>(
    disk_paths: &[P],
) -> Result<Vec<InspectionResult>> {
    ParallelInspector::default().inspect_batch(disk_paths)
}

/// Helper function for batch inspection with custom workers
pub fn inspect_batch_with_workers<P: AsRef<Path> + Send + Sync>(
    disk_paths: &[P],
    num_workers: usize,
) -> Result<Vec<InspectionResult>> {
    let config = InspectionConfig {
        max_workers: num_workers,
        ..Default::default()
    };
    ParallelInspector::new(config).inspect_batch(disk_paths)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    #[test]
    fn test_inspection_config_default() {
        let config = InspectionConfig::default();
        assert_eq!(config.max_workers, 0);
        assert!(config.enable_cache);
        assert_eq!(config.timeout_secs, 300);
        assert!(config.continue_on_error);
        assert!(!config.verbose);
    }

    #[test]
    fn test_inspection_result_success() {
        let result = InspectionResult::success(
            PathBuf::from("test.qcow2"),
            Duration::from_secs(5),
            "linux".to_string(),
            "Ubuntu 22.04".to_string(),
            false,
        );

        assert!(result.success);
        assert!(result.error.is_none());
        assert_eq!(result.os_type, Some("linux".to_string()));
        assert!(!result.from_cache);
    }

    #[test]
    fn test_inspection_result_failure() {
        let result = InspectionResult::failure(
            PathBuf::from("test.qcow2"),
            Duration::from_secs(1),
            "Disk not found".to_string(),
        );

        assert!(!result.success);
        assert!(result.error.is_some());
        assert!(result.os_type.is_none());
    }

    #[test]
    fn test_parallel_inspector_creation() {
        let config = InspectionConfig::default();
        let inspector = ParallelInspector::new(config);
        assert!(inspector.num_workers() > 0);
    }

    #[test]
    fn test_parallel_inspector_workers() {
        let config = InspectionConfig {
            max_workers: 4,
            ..Default::default()
        };
        let inspector = ParallelInspector::new(config);
        assert_eq!(inspector.num_workers(), 4);
    }

    #[test]
    fn test_inspect_batch_nonexistent() {
        let inspector = ParallelInspector::default();
        let disks = vec!["/nonexistent/disk1.qcow2", "/nonexistent/disk2.qcow2"];

        let results = inspector.inspect_batch(&disks).unwrap();
        assert_eq!(results.len(), 2);
        assert!(!results[0].success);
        assert!(!results[1].success);
    }

    #[test]
    fn test_inspect_batch_with_temp_files() {
        let temp_dir = TempDir::new().unwrap();
        let disk1 = temp_dir.path().join("disk1.img");
        let disk2 = temp_dir.path().join("disk2.img");

        fs::write(&disk1, b"fake disk data").unwrap();
        fs::write(&disk2, b"fake disk data").unwrap();

        let inspector = ParallelInspector::default();
        let disks = vec![&disk1, &disk2];

        let results = inspector.inspect_batch(&disks).unwrap();
        assert_eq!(results.len(), 2);
        // With placeholder implementation, both should succeed
        assert!(results[0].success);
        assert!(results[1].success);
    }

    #[test]
    fn test_helper_inspect_batch() {
        let temp_dir = TempDir::new().unwrap();
        let disk = temp_dir.path().join("disk.img");
        fs::write(&disk, b"fake").unwrap();

        let results = inspect_batch(&[&disk]).unwrap();
        assert_eq!(results.len(), 1);
    }

    #[test]
    fn test_helper_inspect_batch_with_workers() {
        let temp_dir = TempDir::new().unwrap();
        let disk = temp_dir.path().join("disk.img");
        fs::write(&disk, b"fake").unwrap();

        let results = inspect_batch_with_workers(&[&disk], 2).unwrap();
        assert_eq!(results.len(), 1);
    }
}
