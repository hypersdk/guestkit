// SPDX-License-Identifier: LGPL-3.0-or-later
//! Fluent filesystem operations with type safety

use super::{Guestfs, FilesystemType};
use crate::core::Result;

/// Builder for creating filesystems with a fluent API
///
/// # Examples
///
/// ```no_run
/// # use guestkit::Guestfs;
/// # let mut g = Guestfs::new()?;
/// // Create ext4 filesystem with label
/// g.mkfs("/dev/sda1")
///     .fstype(guestkit::guestfs::FilesystemType::Ext4)
///     .label("rootfs")
///     .blocksize(4096)
///     .create()?;
///
/// // Create BTRFS filesystem
/// g.mkfs("/dev/sda2")
///     .btrfs()
///     .label("data")
///     .create()?;
/// # Ok::<(), Box<dyn std::error::Error>>(())
/// ```
pub struct MkfsBuilder<'a> {
    guestfs: &'a mut Guestfs,
    device: String,
    fstype: Option<FilesystemType>,
    blocksize: Option<i32>,
    label: Option<String>,
    features: Vec<String>,
}

impl<'a> MkfsBuilder<'a> {
    pub(crate) fn new(guestfs: &'a mut Guestfs, device: impl Into<String>) -> Self {
        Self {
            guestfs,
            device: device.into(),
            fstype: None,
            blocksize: None,
            label: None,
            features: Vec::new(),
        }
    }

    /// Set the filesystem type
    pub fn fstype(mut self, fstype: FilesystemType) -> Self {
        self.fstype = Some(fstype);
        self
    }

    /// Create ext2 filesystem
    pub fn ext2(mut self) -> Self {
        self.fstype = Some(FilesystemType::Ext2);
        self
    }

    /// Create ext3 filesystem
    pub fn ext3(mut self) -> Self {
        self.fstype = Some(FilesystemType::Ext3);
        self
    }

    /// Create ext4 filesystem (default for most Linux)
    pub fn ext4(mut self) -> Self {
        self.fstype = Some(FilesystemType::Ext4);
        self
    }

    /// Create XFS filesystem
    pub fn xfs(mut self) -> Self {
        self.fstype = Some(FilesystemType::Xfs);
        self
    }

    /// Create BTRFS filesystem
    pub fn btrfs(mut self) -> Self {
        self.fstype = Some(FilesystemType::Btrfs);
        self
    }

    /// Create VFAT filesystem
    pub fn vfat(mut self) -> Self {
        self.fstype = Some(FilesystemType::Vfat);
        self
    }

    /// Create NTFS filesystem
    pub fn ntfs(mut self) -> Self {
        self.fstype = Some(FilesystemType::Ntfs);
        self
    }

    /// Set block size in bytes
    pub fn blocksize(mut self, size: i32) -> Self {
        self.blocksize = Some(size);
        self
    }

    /// Set filesystem label
    pub fn label(mut self, label: impl Into<String>) -> Self {
        self.label = Some(label.into());
        self
    }

    /// Add filesystem features
    pub fn feature(mut self, feature: impl Into<String>) -> Self {
        self.features.push(feature.into());
        self
    }

    /// Create the filesystem
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - Filesystem type is not specified
    /// - Device doesn't exist
    /// - Creation fails
    pub fn create(self) -> Result<()> {
        let fstype = self.fstype.ok_or_else(|| {
            crate::core::Error::InvalidParameter("filesystem type not specified".into())
        })?;

        self.guestfs.mkfs(
            fstype.as_str(),
            &self.device,
            self.blocksize,
            self.label.as_deref(),
            None, // features - reserved for future use
            None, // sectorsize - reserved for future use
        )
    }
}

/// Builder for mounting filesystems with options
///
/// # Examples
///
/// ```no_run
/// # use guestkit::Guestfs;
/// # let mut g = Guestfs::new()?;
/// // Mount with BTRFS subvolume
/// g.mount_with("/dev/sda1", "/")
///     .option("subvol=@")
///     .option("compress=zstd")
///     .readonly()
///     .perform()?;
/// # Ok::<(), Box<dyn std::error::Error>>(())
/// ```
pub struct MountBuilder<'a> {
    guestfs: &'a mut Guestfs,
    device: String,
    mountpoint: String,
    readonly: bool,
    options: Vec<String>,
}

impl<'a> MountBuilder<'a> {
    pub(crate) fn new(
        guestfs: &'a mut Guestfs,
        device: impl Into<String>,
        mountpoint: impl Into<String>,
    ) -> Self {
        Self {
            guestfs,
            device: device.into(),
            mountpoint: mountpoint.into(),
            readonly: false,
            options: Vec::new(),
        }
    }

    /// Mount in read-only mode
    pub fn readonly(mut self) -> Self {
        self.readonly = true;
        self
    }

    /// Add a mount option
    pub fn option(mut self, opt: impl Into<String>) -> Self {
        self.options.push(opt.into());
        self
    }

    /// Mount with BTRFS subvolume
    pub fn subvolume(mut self, subvol: impl Into<String>) -> Self {
        self.options.push(format!("subvol={}", subvol.into()));
        self
    }

    /// Enable compression (BTRFS/ZFS)
    pub fn compress(mut self, algorithm: impl Into<String>) -> Self {
        self.options.push(format!("compress={}", algorithm.into()));
        self
    }

    /// Perform the mount operation
    pub fn perform(self) -> Result<()> {
        let opts = if self.options.is_empty() {
            None
        } else {
            Some(self.options.join(","))
        };

        self.guestfs.mount(&self.device, &self.mountpoint, opts.as_deref())
    }
}

impl Guestfs {
    /// Start building a filesystem creation operation
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # use guestkit::Guestfs;
    /// # let mut g = Guestfs::new()?;
    /// g.mkfs("/dev/sda1")
    ///     .ext4()
    ///     .label("rootfs")
    ///     .blocksize(4096)
    ///     .create()?;
    /// # Ok::<(), Box<dyn std::error::Error>>(())
    /// ```
    pub fn mkfs<'a>(&'a mut self, device: impl Into<String>) -> MkfsBuilder<'a> {
        MkfsBuilder::new(self, device)
    }

    /// Start building a mount operation with options
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # use guestkit::Guestfs;
    /// # let mut g = Guestfs::new()?;
    /// g.mount_with("/dev/sda1", "/")
    ///     .subvolume("@")
    ///     .readonly()
    ///     .perform()?;
    /// # Ok::<(), Box<dyn std::error::Error>>(())
    /// ```
    pub fn mount_with<'a>(
        &'a mut self,
        device: impl Into<String>,
        mountpoint: impl Into<String>,
    ) -> MountBuilder<'a> {
        MountBuilder::new(self, device, mountpoint)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_mkfs_builder() {
        // Can't actually test creation without a real handle,
        // but we can test builder construction
        let mut g = Guestfs::new().unwrap();

        let builder = g.mkfs("/dev/sda1")
            .ext4()
            .label("test")
            .blocksize(4096);

        assert_eq!(builder.device, "/dev/sda1");
        assert_eq!(builder.fstype, Some(FilesystemType::Ext4));
        assert_eq!(builder.label, Some("test".to_string()));
        assert_eq!(builder.blocksize, Some(4096));
    }

    #[test]
    fn test_mount_builder() {
        let mut g = Guestfs::new().unwrap();

        let builder = g.mount_with("/dev/sda1", "/")
            .subvolume("@home")
            .compress("zstd")
            .readonly();

        assert_eq!(builder.device, "/dev/sda1");
        assert_eq!(builder.mountpoint, "/");
        assert!(builder.readonly);
        assert_eq!(builder.options, vec!["subvol=@home", "compress=zstd"]);
    }
}
