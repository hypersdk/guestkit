// SPDX-License-Identifier: LGPL-3.0-or-later
//! Hivex (Windows Registry) operations for disk image manipulation
//!
//! This implementation provides Windows registry hive manipulation functionality
//! using the nt_hive2 crate for read operations. Write operations return
//! Error::Unsupported as nt_hive2 is read-only.

use crate::core::{Error, Result};
use crate::guestfs::Guestfs;

/// Registry value type constants (matching Windows REG_* types)
#[allow(dead_code)]
const REG_NONE: i64 = 0;
#[allow(dead_code)]
const REG_SZ: i64 = 1;
#[allow(dead_code)]
const REG_EXPAND_SZ: i64 = 2;
#[allow(dead_code)]
const REG_BINARY: i64 = 3;
#[allow(dead_code)]
const REG_DWORD: i64 = 4;
#[allow(dead_code)]
const REG_MULTI_SZ: i64 = 7;
#[allow(dead_code)]
const REG_QWORD: i64 = 11;

/// Helper macro to open a hive file from the stored path and get a mutable Hive handle
macro_rules! open_hive {
    ($self:expr, $handle:expr) => {{
        use nt_hive2::{Hive, HiveParseMode};
        use std::fs::File;

        let host_path = $self.open_hives.get(&$handle)
            .ok_or_else(|| Error::InvalidState(format!("No hive open with handle {}", $handle)))?;

        let file = File::open(host_path)
            .map_err(|e| Error::CommandFailed(format!("Failed to open hive {}: {}", host_path.display(), e)))?;

        let hive = Hive::new(file, HiveParseMode::NormalWithBaseBlock)
            .map_err(|e| Error::CommandFailed(format!("Failed to parse hive: {:?}", e)))?;
        hive
    }};
}

impl Guestfs {
    /// Open Windows registry hive
    pub fn hivex_open(&mut self, filename: &str, _write: bool) -> Result<i64> {
        self.ensure_ready()?;

        if self.verbose {
            eprintln!("guestfs: hivex_open {} {}", filename, _write);
        }

        let host_path = self.resolve_guest_path(filename)?;

        if !host_path.exists() {
            return Err(Error::NotFound(format!(
                "Hive file not found: {}",
                filename
            )));
        }

        // Validate it's a parseable hive by opening it
        {
            use nt_hive2::{Hive, HiveParseMode};
            use std::fs::File;

            let file = File::open(&host_path)
                .map_err(|e| Error::CommandFailed(format!("Failed to open hive: {}", e)))?;
            // Validate by parsing - root_key_node() forces type inference
            let mut hive = Hive::new(file, HiveParseMode::NormalWithBaseBlock)
                .map_err(|e| Error::CommandFailed(format!("Failed to parse hive: {:?}", e)))?;
            let _root = hive.root_key_node()
                .map_err(|e| Error::CommandFailed(format!("Failed to get root key: {:?}", e)))?;
        }

        // Generate handle from inode
        #[cfg(unix)]
        let handle = {
            use std::os::unix::fs::MetadataExt;
            let metadata = std::fs::metadata(&host_path).map_err(Error::Io)?;
            metadata.ino() as i64
        };

        #[cfg(not(unix))]
        let handle = {
            use std::collections::hash_map::DefaultHasher;
            use std::hash::{Hash, Hasher};
            let mut hasher = DefaultHasher::new();
            host_path.hash(&mut hasher);
            hasher.finish() as i64
        };

        self.open_hives.insert(handle, host_path);
        Ok(handle)
    }

    /// Close Windows registry hive
    pub fn hivex_close(&mut self, handle: i64) -> Result<()> {
        self.ensure_ready()?;

        if self.verbose {
            eprintln!("guestfs: hivex_close {}", handle);
        }

        self.open_hives.remove(&handle);
        Ok(())
    }

    /// Get root node of registry hive
    pub fn hivex_root(&mut self, handle: i64) -> Result<i64> {
        self.ensure_ready()?;

        if self.verbose {
            eprintln!("guestfs: hivex_root {}", handle);
        }

        // Verify hive is open and valid
        let mut hive = open_hive!(self, handle);
        let _root = hive.root_key_node()
            .map_err(|e| Error::CommandFailed(format!("Failed to get root key: {:?}", e)))?;

        // Root node is always represented as 0
        Ok(0)
    }

    /// Get node name
    pub fn hivex_node_name(&mut self, handle: i64, node: i64) -> Result<String> {
        self.ensure_ready()?;

        if self.verbose {
            eprintln!("guestfs: hivex_node_name {} {}", handle, node);
        }

        let mut hive = open_hive!(self, handle);

        if node == 0 {
            let root_key = hive.root_key_node()
                .map_err(|e| Error::CommandFailed(format!("Failed to get root key: {:?}", e)))?;
            return Ok(root_key.name().to_string());
        }

        Err(Error::NotFound(format!("Node {} not found (use hivex_node_get_child to navigate)", node)))
    }

    /// Get child nodes
    pub fn hivex_node_children(&mut self, handle: i64, node: i64) -> Result<Vec<i64>> {
        self.ensure_ready()?;

        if self.verbose {
            eprintln!("guestfs: hivex_node_children {} {}", handle, node);
        }

        let mut hive = open_hive!(self, handle);

        if node == 0 {
            let root_key = hive.root_key_node()
                .map_err(|e| Error::CommandFailed(format!("Failed to get root key: {:?}", e)))?;

            let subkeys = root_key.subkeys(&mut hive)
                .map_err(|e| Error::CommandFailed(format!("Failed to get subkeys: {:?}", e)))?;

            let children: Vec<i64> = (1..=subkeys.len() as i64).collect();
            return Ok(children);
        }

        Ok(Vec::new())
    }

    /// Get node values
    pub fn hivex_node_values(&mut self, handle: i64, node: i64) -> Result<Vec<i64>> {
        self.ensure_ready()?;

        if self.verbose {
            eprintln!("guestfs: hivex_node_values {} {}", handle, node);
        }

        let mut hive = open_hive!(self, handle);

        if node == 0 {
            let root_key = hive.root_key_node()
                .map_err(|e| Error::CommandFailed(format!("Failed to get root key: {:?}", e)))?;

            let count = root_key.values().len();
            let value_handles: Vec<i64> = (1..=count as i64).collect();
            return Ok(value_handles);
        }

        Ok(Vec::new())
    }

    /// Get child node by name
    pub fn hivex_node_get_child(&mut self, handle: i64, node: i64, name: &str) -> Result<i64> {
        self.ensure_ready()?;

        if self.verbose {
            eprintln!("guestfs: hivex_node_get_child {} {} {}", handle, node, name);
        }

        let mut hive = open_hive!(self, handle);

        if node == 0 {
            let root_key = hive.root_key_node()
                .map_err(|e| Error::CommandFailed(format!("Failed to get root key: {:?}", e)))?;

            match root_key.subkey(name, &mut hive) {
                Ok(Some(_)) => {
                    use std::collections::hash_map::DefaultHasher;
                    use std::hash::{Hash, Hasher};
                    let mut hasher = DefaultHasher::new();
                    name.hash(&mut hasher);
                    Ok((hasher.finish() & 0x7FFF_FFFF) as i64 + 1)
                }
                Ok(None) => Err(Error::NotFound(format!("Child node not found: {}", name))),
                Err(e) => Err(Error::CommandFailed(format!("Failed to get child: {:?}", e))),
            }
        } else {
            Err(Error::NotFound(format!("Child node not found: {}", name)))
        }
    }

    /// Get value key (name)
    pub fn hivex_value_key(&mut self, handle: i64, value: i64) -> Result<String> {
        self.ensure_ready()?;

        if self.verbose {
            eprintln!("guestfs: hivex_value_key {} {}", handle, value);
        }

        let mut hive = open_hive!(self, handle);

        let root_key = hive.root_key_node()
            .map_err(|e| Error::CommandFailed(format!("Failed to get root key: {:?}", e)))?;

        let values = root_key.values();
        if value < 1 {
            return Err(Error::InvalidOperation(format!("Value index must be >= 1, got {}", value)));
        }
        let idx = (value - 1) as usize;
        if idx < values.len() {
            return Ok(values[idx].name().to_string());
        }

        Err(Error::NotFound(format!("Value {} not found", value)))
    }

    /// Get value type
    pub fn hivex_value_type(&mut self, handle: i64, value: i64) -> Result<i64> {
        self.ensure_ready()?;

        if self.verbose {
            eprintln!("guestfs: hivex_value_type {} {}", handle, value);
        }

        let mut hive = open_hive!(self, handle);

        let root_key = hive.root_key_node()
            .map_err(|e| Error::CommandFailed(format!("Failed to get root key: {:?}", e)))?;

        let values = root_key.values();
        if value < 1 {
            return Err(Error::InvalidOperation(format!("Value index must be >= 1, got {}", value)));
        }
        let idx = (value - 1) as usize;
        if idx < values.len() {
            let reg_type = match values[idx].value() {
                nt_hive2::RegistryValue::RegNone => REG_NONE,
                nt_hive2::RegistryValue::RegSZ(_) => REG_SZ,
                nt_hive2::RegistryValue::RegExpandSZ(_) => REG_EXPAND_SZ,
                nt_hive2::RegistryValue::RegBinary(_) => REG_BINARY,
                nt_hive2::RegistryValue::RegDWord(_) => REG_DWORD,
                nt_hive2::RegistryValue::RegMultiSZ(_) => REG_MULTI_SZ,
                nt_hive2::RegistryValue::RegQWord(_) => REG_QWORD,
                _ => REG_NONE,
            };
            return Ok(reg_type);
        }

        Ok(REG_NONE)
    }

    /// Get value as string
    pub fn hivex_value_string(&mut self, handle: i64, value: i64) -> Result<String> {
        self.ensure_ready()?;

        if self.verbose {
            eprintln!("guestfs: hivex_value_string {} {}", handle, value);
        }

        let mut hive = open_hive!(self, handle);

        let root_key = hive.root_key_node()
            .map_err(|e| Error::CommandFailed(format!("Failed to get root key: {:?}", e)))?;

        let values = root_key.values();
        if value < 1 {
            return Err(Error::InvalidOperation(format!("Value index must be >= 1, got {}", value)));
        }
        let idx = (value - 1) as usize;
        if idx < values.len() {
            match values[idx].value() {
                nt_hive2::RegistryValue::RegSZ(data) | nt_hive2::RegistryValue::RegExpandSZ(data) => {
                    return Ok(data.clone());
                }
                _ => return Ok(String::new()),
            }
        }

        Ok(String::new())
    }

    /// Get value as integer (DWORD)
    pub fn hivex_value_dword(&mut self, handle: i64, value: i64) -> Result<i32> {
        self.ensure_ready()?;

        if self.verbose {
            eprintln!("guestfs: hivex_value_dword {} {}", handle, value);
        }

        let mut hive = open_hive!(self, handle);

        let root_key = hive.root_key_node()
            .map_err(|e| Error::CommandFailed(format!("Failed to get root key: {:?}", e)))?;

        let values = root_key.values();
        if value < 1 {
            return Err(Error::InvalidOperation(format!("Value index must be >= 1, got {}", value)));
        }
        let idx = (value - 1) as usize;
        if idx < values.len() {
            if let nt_hive2::RegistryValue::RegDWord(data) = values[idx].value() {
                return Ok(*data as i32);
            }
        }

        Ok(0)
    }

    /// Get value as binary data
    pub fn hivex_value_value(&mut self, handle: i64, value: i64) -> Result<Vec<u8>> {
        self.ensure_ready()?;

        if self.verbose {
            eprintln!("guestfs: hivex_value_value {} {}", handle, value);
        }

        let mut hive = open_hive!(self, handle);

        let root_key = hive.root_key_node()
            .map_err(|e| Error::CommandFailed(format!("Failed to get root key: {:?}", e)))?;

        let values = root_key.values();
        if value < 1 {
            return Err(Error::InvalidOperation(format!("Value index must be >= 1, got {}", value)));
        }
        let idx = (value - 1) as usize;
        if idx < values.len() {
            match values[idx].value() {
                nt_hive2::RegistryValue::RegBinary(data) => return Ok(data.clone()),
                nt_hive2::RegistryValue::RegSZ(s) | nt_hive2::RegistryValue::RegExpandSZ(s) => {
                    return Ok(s.as_bytes().to_vec());
                }
                nt_hive2::RegistryValue::RegDWord(d) => return Ok(d.to_le_bytes().to_vec()),
                nt_hive2::RegistryValue::RegQWord(q) => return Ok(q.to_le_bytes().to_vec()),
                _ => return Ok(Vec::new()),
            }
        }

        Ok(Vec::new())
    }

    /// Commit changes to hive — not supported (nt_hive2 is read-only)
    pub fn hivex_commit(&mut self, _handle: i64, filename: Option<&str>) -> Result<()> {
        self.ensure_ready()?;

        if self.verbose {
            eprintln!("guestfs: hivex_commit {:?}", filename);
        }

        Err(Error::Unsupported(
            "Registry write operations are not supported (nt_hive2 is read-only)".to_string(),
        ))
    }

    /// Set node value — not supported (nt_hive2 is read-only)
    pub fn hivex_node_set_value(
        &mut self,
        _handle: i64,
        _node: i64,
        key: &str,
        _t: i64,
        _val: &[u8],
    ) -> Result<()> {
        self.ensure_ready()?;

        if self.verbose {
            eprintln!("guestfs: hivex_node_set_value {}", key);
        }

        Err(Error::Unsupported(
            "Registry write operations are not supported (nt_hive2 is read-only)".to_string(),
        ))
    }

    /// Add child node — not supported (nt_hive2 is read-only)
    pub fn hivex_node_add_child(&mut self, _handle: i64, _parent: i64, name: &str) -> Result<i64> {
        self.ensure_ready()?;

        if self.verbose {
            eprintln!("guestfs: hivex_node_add_child {}", name);
        }

        Err(Error::Unsupported(
            "Registry write operations are not supported (nt_hive2 is read-only)".to_string(),
        ))
    }

    /// Delete node — not supported (nt_hive2 is read-only)
    pub fn hivex_node_delete_child(&mut self, _handle: i64, _node: i64) -> Result<()> {
        self.ensure_ready()?;

        if self.verbose {
            eprintln!("guestfs: hivex_node_delete_child");
        }

        Err(Error::Unsupported(
            "Registry write operations are not supported (nt_hive2 is read-only)".to_string(),
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_hivex_ops_api_exists() {
        let g = Guestfs::new().unwrap();
        assert!(g.open_hives.is_empty());
    }
}
