// SPDX-License-Identifier: LGPL-3.0-or-later
//! Cloud and local disk source abstraction.

pub mod local;
pub mod uri;

pub use local::LocalDiskSource;
pub use uri::{open_disk_source, resolve_to_local_path, DiskSource, DiskSourceMetadata};

#[cfg(feature = "cloud-s3")]
pub mod s3;
#[cfg(feature = "cloud-azure")]
pub mod azure;
#[cfg(feature = "cloud-gcs")]
pub mod gcs;
