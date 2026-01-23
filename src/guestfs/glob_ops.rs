// SPDX-License-Identifier: LGPL-3.0-or-later
//! Glob and wildcard operations compatible with libguestfs
//!
//! This implementation provides glob pattern matching functionality.

use crate::core::{Error, Result};
use crate::guestfs::Guestfs;

impl Guestfs {
    /// Expand glob pattern
    ///
    /// Compatible with libguestfs g.glob_expand()
    pub fn glob_expand(&mut self, pattern: &str) -> Result<Vec<String>> {
        self.ensure_ready()?;

        if self.verbose {
            eprintln!("guestfs: glob_expand {}", pattern);
        }

        let host_pattern = if pattern.starts_with('/') {
            let root_mountpoint = self.mounted.values().next()
                .ok_or_else(|| Error::InvalidState("No filesystem mounted".to_string()))?;
            let pattern_clean = pattern.trim_start_matches('/');
            format!("{}/{}", root_mountpoint, pattern_clean)
        } else {
            pattern.to_string()
        };

        let mut matches = Vec::new();

        // Use glob crate for pattern matching
        match glob::glob(&host_pattern) {
            Ok(paths) => {
                for entry in paths {
                    if let Ok(path) = entry {
                        // Convert back to guest path
                        if let Some(path_str) = path.to_str() {
                            matches.push(path_str.to_string());
                        }
                    }
                }
            }
            Err(e) => {
                return Err(Error::CommandFailed(format!("Glob pattern error: {}", e)));
            }
        }

        Ok(matches)
    }

    /// List files matching pattern
    ///
    /// Compatible with libguestfs g.ls0()
    pub fn ls0(&mut self, dir: &str, filenames: &str) -> Result<()> {
        self.ensure_ready()?;

        if self.verbose {
            eprintln!("guestfs: ls0 {} {}", dir, filenames);
        }

        let entries = self.ls(dir)?;

        // Write to file with NUL separators
        let host_file = self.resolve_guest_path(filenames)?;

        let mut content = String::new();
        for entry in entries {
            content.push_str(&entry);
            content.push('\0');
        }

        std::fs::write(&host_file, content.as_bytes())
            .map_err(|e| Error::Io(e))?;

        Ok(())
    }

    /// Recursive file listing
    ///
    /// Compatible with libguestfs g.find0()
    pub fn find0_impl(&mut self, directory: &str, files: &str) -> Result<()> {
        self.ensure_ready()?;

        if self.verbose {
            eprintln!("guestfs: find0 {} {}", directory, files);
        }

        let all_files = self.find(directory)?;

        let host_file = self.resolve_guest_path(files)?;

        let mut content = String::new();
        for file in all_files {
            content.push_str(&file);
            content.push('\0');
        }

        std::fs::write(&host_file, content.as_bytes())
            .map_err(|e| Error::Io(e))?;

        Ok(())
    }

    /// Match files against pattern
    ///
    /// Compatible with libguestfs g.grep_impl()
    pub fn grep_lines(&mut self, regex: &str, path: &str) -> Result<Vec<String>> {
        self.ensure_ready()?;

        if self.verbose {
            eprintln!("guestfs: grep_lines {} {}", regex, path);
        }

        let content = self.cat(path)?;

        let re = regex::Regex::new(regex)
            .map_err(|e| Error::InvalidFormat(format!("Invalid regex: {}", e)))?;

        let matching_lines: Vec<String> = content
            .lines()
            .filter(|line| re.is_match(line))
            .map(|line| line.to_string())
            .collect();

        Ok(matching_lines)
    }

    /// Read directory entries
    ///
    /// Compatible with libguestfs g.readdir()
    pub fn readdir(&mut self, dir: &str) -> Result<Vec<(String, u8)>> {
        self.ensure_ready()?;

        if self.verbose {
            eprintln!("guestfs: readdir {}", dir);
        }

        let host_path = self.resolve_guest_path(dir)?;

        let mut entries = Vec::new();

        for entry in std::fs::read_dir(&host_path).map_err(|e| Error::Io(e))? {
            let entry = entry.map_err(|e| Error::Io(e))?;
            let name = entry.file_name().to_string_lossy().to_string();

            let file_type = entry.file_type().map_err(|e| Error::Io(e))?;

            let type_code = if file_type.is_file() {
                b'r' // Regular file
            } else if file_type.is_dir() {
                b'd' // Directory
            } else if file_type.is_symlink() {
                b'l' // Symlink
            } else {
                b'u' // Unknown
            };

            entries.push((name, type_code));
        }

        Ok(entries)
    }

    /// Case insensitive path lookup
    ///
    /// Compatible with libguestfs g.case_sensitive_path()
    pub fn case_sensitive_path(&mut self, path: &str) -> Result<String> {
        self.ensure_ready()?;

        if self.verbose {
            eprintln!("guestfs: case_sensitive_path {}", path);
        }

        // Check if path exists as-is
        if self.exists(path)? {
            return Ok(path.to_string());
        }

        // Try to find case-insensitive match
        let parts: Vec<&str> = path.split('/').filter(|s| !s.is_empty()).collect();
        let mut current = String::from("/");

        for part in parts {
            if let Ok(entries) = self.ls(&current) {
                // Look for case-insensitive match
                if let Some(matched) = entries.iter().find(|e| e.to_lowercase() == part.to_lowercase()) {
                    if current == "/" {
                        current = format!("/{}", matched);
                    } else {
                        current = format!("{}/{}", current, matched);
                    }
                } else {
                    return Err(Error::NotFound(format!("Path not found: {}", path)));
                }
            } else {
                return Err(Error::NotFound(format!("Path not found: {}", path)));
            }
        }

        Ok(current)
    }

    /// List extended attributes with values
    ///
    /// Compatible with libguestfs g.lxattrlist()
    pub fn lxattrlist(&mut self, path: &str, names: &[&str]) -> Result<Vec<String>> {
        self.ensure_ready()?;

        if self.verbose {
            eprintln!("guestfs: lxattrlist {} {:?}", path, names);
        }

        let mut results = Vec::new();

        for name in names {
            let file_path = if path.ends_with('/') {
                format!("{}{}", path, name)
            } else {
                format!("{}/{}", path, name)
            };

            if let Ok(xattrs) = self.lgetxattrs(&file_path) {
                for attr in xattrs {
                    results.push(format!("{}: {}", file_path, attr));
                }
            }
        }

        Ok(results)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_glob_ops_api_exists() {
        let mut g = Guestfs::new().unwrap();
        // API structure tests
    }
}
