// SPDX-License-Identifier: LGPL-3.0-or-later
//! Binary cache implementation using bincode for fast serialization

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;
use std::time::{SystemTime, UNIX_EPOCH};

/// Cached inspection result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CachedInspection {
    /// Timestamp when cached (Unix epoch)
    pub timestamp: u64,
    /// SHA256 hash of disk image
    pub disk_hash: String,
    /// Operating system information
    pub os_info: OsInfoCache,
    /// Filesystem information
    pub filesystems: Vec<FilesystemCache>,
    /// User accounts
    pub users: Vec<UserCache>,
    /// Installed packages (limited to first 1000)
    pub packages: Vec<PackageCache>,
    /// Network configuration
    pub network: Option<NetworkCache>,
    /// System configuration
    pub system_config: Option<SystemConfigCache>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OsInfoCache {
    pub os_type: String,
    pub distribution: String,
    pub product_name: String,
    pub version_major: i32,
    pub version_minor: i32,
    pub architecture: String,
    pub hostname: String,
    pub package_format: String,
    pub init_system: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FilesystemCache {
    pub device: String,
    pub fs_type: String,
    pub size: i64,
    pub uuid: String,
    pub label: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserCache {
    pub username: String,
    pub uid: String,
    pub gid: String,
    pub home_dir: String,
    pub shell: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PackageCache {
    pub name: String,
    pub version: String,
    pub arch: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetworkCache {
    pub interfaces: Vec<NetworkInterfaceCache>,
    pub dns_servers: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetworkInterfaceCache {
    pub name: String,
    pub ip_address: String,
    pub mac_address: String,
    pub status: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SystemConfigCache {
    pub timezone: String,
    pub locale: String,
    pub selinux: String,
    pub cloud_init: bool,
}

/// Binary cache manager using bincode for fast serialization
pub struct BinaryCache {
    cache_dir: PathBuf,
}

impl BinaryCache {
    /// Create new binary cache manager
    pub fn new() -> Result<Self> {
        let cache_dir = dirs::cache_dir()
            .context("Could not find cache directory")?
            .join("guestctl")
            .join("binary");

        fs::create_dir_all(&cache_dir)
            .context("Failed to create cache directory")?;

        Ok(Self { cache_dir })
    }

    /// Get cache file path for a given key
    fn cache_path(&self, key: &str) -> PathBuf {
        self.cache_dir.join(format!("{}.bin", key))
    }

    /// Save inspection result to binary cache
    ///
    /// Uses bincode for efficient binary serialization - 5-10x faster than JSON
    /// and produces 50-70% smaller files.
    pub fn save(&self, key: &str, data: &CachedInspection) -> Result<()> {
        let path = self.cache_path(key);

        // Serialize to binary format
        let encoded = bincode::serialize(data)
            .context("Failed to serialize cache data")?;

        // Write atomically using temp file + rename
        let temp_path = path.with_extension("tmp");
        fs::write(&temp_path, encoded)
            .context("Failed to write cache file")?;

        fs::rename(&temp_path, &path)
            .context("Failed to rename cache file")?;

        log::debug!("Saved cache to {:?} ({} bytes)", path, data.size());

        Ok(())
    }

    /// Load inspection result from binary cache
    pub fn load(&self, key: &str) -> Result<CachedInspection> {
        let path = self.cache_path(key);

        let bytes = fs::read(&path)
            .context("Failed to read cache file")?;

        let data: CachedInspection = bincode::deserialize(&bytes)
            .context("Failed to deserialize cache data")?;

        log::debug!("Loaded cache from {:?} ({} bytes)", path, bytes.len());

        Ok(data)
    }

    /// Check if cache exists for given key
    pub fn exists(&self, key: &str) -> bool {
        self.cache_path(key).exists()
    }

    /// Check if cache is valid (not older than max_age_seconds)
    pub fn is_valid(&self, key: &str, max_age_seconds: u64) -> Result<bool> {
        if !self.exists(key) {
            return Ok(false);
        }

        let data = self.load(key)?;
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)?
            .as_secs();

        let age = now.saturating_sub(data.timestamp);
        Ok(age < max_age_seconds)
    }

    /// Delete cache entry
    pub fn delete(&self, key: &str) -> Result<()> {
        let path = self.cache_path(key);
        if path.exists() {
            fs::remove_file(&path)
                .context("Failed to delete cache file")?;
        }
        Ok(())
    }

    /// Clear all cache entries
    pub fn clear_all(&self) -> Result<usize> {
        let mut count = 0;

        for entry in fs::read_dir(&self.cache_dir)? {
            let entry = entry?;
            let path = entry.path();

            if path.extension().and_then(|s| s.to_str()) == Some("bin") {
                fs::remove_file(&path)?;
                count += 1;
            }
        }

        log::info!("Cleared {} cache entries", count);
        Ok(count)
    }

    /// Get cache statistics
    pub fn stats(&self) -> Result<CacheStats> {
        let mut total_entries = 0;
        let mut total_size = 0u64;
        let mut oldest_timestamp = u64::MAX;
        let mut newest_timestamp = 0u64;

        for entry in fs::read_dir(&self.cache_dir)? {
            let entry = entry?;
            let path = entry.path();

            if path.extension().and_then(|s| s.to_str()) == Some("bin") {
                total_entries += 1;

                // Get file size
                if let Ok(metadata) = fs::metadata(&path) {
                    total_size += metadata.len();
                }

                // Get timestamp from cached data
                if let Ok(bytes) = fs::read(&path) {
                    if let Ok(data) = bincode::deserialize::<CachedInspection>(&bytes) {
                        oldest_timestamp = oldest_timestamp.min(data.timestamp);
                        newest_timestamp = newest_timestamp.max(data.timestamp);
                    }
                }
            }
        }

        Ok(CacheStats {
            total_entries,
            total_size_bytes: total_size,
            oldest_entry: if oldest_timestamp != u64::MAX {
                Some(oldest_timestamp)
            } else {
                None
            },
            newest_entry: if newest_timestamp != 0 {
                Some(newest_timestamp)
            } else {
                None
            },
        })
    }

    /// Clear cache entries older than specified seconds
    pub fn clear_older_than(&self, max_age_seconds: u64) -> Result<usize> {
        let mut count = 0;
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)?
            .as_secs();

        for entry in fs::read_dir(&self.cache_dir)? {
            let entry = entry?;
            let path = entry.path();

            if path.extension().and_then(|s| s.to_str()) == Some("bin") {
                if let Ok(bytes) = fs::read(&path) {
                    if let Ok(data) = bincode::deserialize::<CachedInspection>(&bytes) {
                        let age = now.saturating_sub(data.timestamp);
                        if age > max_age_seconds {
                            fs::remove_file(&path)?;
                            count += 1;
                        }
                    }
                }
            }
        }

        log::info!("Cleared {} cache entries older than {} seconds", count, max_age_seconds);
        Ok(count)
    }
}

impl Default for BinaryCache {
    fn default() -> Self {
        Self::new().expect("Failed to create binary cache")
    }
}

/// Cache statistics
#[derive(Debug, Clone)]
pub struct CacheStats {
    /// Total number of cache entries
    pub total_entries: usize,
    /// Total size in bytes
    pub total_size_bytes: u64,
    /// Oldest cache entry timestamp
    pub oldest_entry: Option<u64>,
    /// Newest cache entry timestamp
    pub newest_entry: Option<u64>,
}

impl CacheStats {
    /// Get total size in human-readable format
    pub fn total_size_human(&self) -> String {
        let size = self.total_size_bytes as f64;
        if size < 1024.0 {
            format!("{} B", size)
        } else if size < 1024.0 * 1024.0 {
            format!("{:.2} KB", size / 1024.0)
        } else if size < 1024.0 * 1024.0 * 1024.0 {
            format!("{:.2} MB", size / (1024.0 * 1024.0))
        } else {
            format!("{:.2} GB", size / (1024.0 * 1024.0 * 1024.0))
        }
    }
}

impl CachedInspection {
    /// Get approximate size in bytes
    fn size(&self) -> usize {
        // Approximate calculation
        let mut size = 0;
        size += std::mem::size_of::<u64>(); // timestamp
        size += self.disk_hash.len();
        size += std::mem::size_of::<OsInfoCache>();
        size += self.filesystems.len() * std::mem::size_of::<FilesystemCache>();
        size += self.users.len() * std::mem::size_of::<UserCache>();
        size += self.packages.len() * std::mem::size_of::<PackageCache>();
        size
    }

    /// Create new cached inspection with current timestamp
    pub fn new(disk_hash: String) -> Self {
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();

        Self {
            timestamp,
            disk_hash,
            os_info: OsInfoCache::default(),
            filesystems: Vec::new(),
            users: Vec::new(),
            packages: Vec::new(),
            network: None,
            system_config: None,
        }
    }
}

impl Default for OsInfoCache {
    fn default() -> Self {
        Self {
            os_type: String::new(),
            distribution: String::new(),
            product_name: String::new(),
            version_major: 0,
            version_minor: 0,
            architecture: String::new(),
            hostname: String::new(),
            package_format: String::new(),
            init_system: String::new(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_binary_cache_roundtrip() {
        let cache = BinaryCache::new().unwrap();

        let data = CachedInspection::new("abc123".to_string());

        cache.save("test-key", &data).unwrap();
        let loaded = cache.load("test-key").unwrap();

        assert_eq!(data.disk_hash, loaded.disk_hash);
        assert_eq!(data.timestamp, loaded.timestamp);
    }

    #[test]
    fn test_cache_exists() {
        let cache = BinaryCache::new().unwrap();

        assert!(!cache.exists("nonexistent"));

        let data = CachedInspection::new("test".to_string());
        cache.save("exists-test", &data).unwrap();

        assert!(cache.exists("exists-test"));
    }

    #[test]
    fn test_cache_stats() {
        let cache = BinaryCache::new().unwrap();
        cache.clear_all().unwrap();

        let data = CachedInspection::new("test".to_string());
        cache.save("stats-test", &data).unwrap();

        let stats = cache.stats().unwrap();
        assert_eq!(stats.total_entries, 1);
        assert!(stats.total_size_bytes > 0);
    }

    #[test]
    fn test_clear_older_than() {
        let cache = BinaryCache::new().unwrap();

        let mut old_data = CachedInspection::new("old".to_string());
        old_data.timestamp = 0; // Very old

        let new_data = CachedInspection::new("new".to_string());

        cache.save("old", &old_data).unwrap();
        cache.save("new", &new_data).unwrap();

        // Clear entries older than 1 day
        let cleared = cache.clear_older_than(86400).unwrap();
        assert_eq!(cleared, 1); // Only old entry cleared

        assert!(!cache.exists("old"));
        assert!(cache.exists("new"));
    }
}
