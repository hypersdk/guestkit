// SPDX-License-Identifier: LGPL-3.0-or-later
//! PyO3 Python bindings for guestctl
//!
//! Build with: cargo build --release --features python-bindings

#[cfg(feature = "python-bindings")]
use pyo3::prelude::*;

#[cfg(feature = "python-bindings")]
use crate::converters::DiskConverter as RustDiskConverter;
#[cfg(feature = "python-bindings")]
use std::path::Path;

/// Python wrapper for disk conversion
#[cfg(feature = "python-bindings")]
#[pyclass]
struct DiskConverter {
    converter: RustDiskConverter,
}

#[cfg(feature = "python-bindings")]
#[pymethods]
impl DiskConverter {
    #[new]
    fn new() -> Self {
        Self {
            converter: RustDiskConverter::new(),
        }
    }

    /// Convert disk image format
    ///
    /// # Arguments
    ///
    /// * `source` - Source disk image path
    /// * `output` - Output disk image path
    /// * `format` - Output format (qcow2, raw, vmdk, vdi)
    /// * `compress` - Enable compression (default: false)
    /// * `flatten` - Flatten snapshot chains (default: true)
    ///
    /// # Returns
    ///
    /// Dictionary with conversion results
    ///
    /// # Examples
    ///
    /// ```python
    /// from guestctl import DiskConverter
    ///
    /// converter = DiskConverter()
    /// result = converter.convert(
    ///     "/path/to/source.vmdk",
    ///     "/path/to/output.qcow2",
    ///     "qcow2",
    ///     compress=True
    /// )
    ///
    /// if result["success"]:
    ///     print(f"Converted: {result['output_size']} bytes")
    /// ```
    #[pyo3(signature = (source, output, format="qcow2", compress=false, flatten=true))]
    fn convert(
        &self,
        source: String,
        output: String,
        format: &str,
        compress: bool,
        flatten: bool,
    ) -> PyResult<Py<PyAny>> {
        let result = self
            .converter
            .convert(
                Path::new(&source),
                Path::new(&output),
                format,
                compress,
                flatten,
            )
            .map_err(|e| PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(e.to_string()))?;

        Python::attach(|py| {
            let dict = pyo3::types::PyDict::new(py);
            dict.set_item("source_path", result.source_path.to_str())?;
            dict.set_item("output_path", result.output_path.to_str())?;
            dict.set_item("source_format", result.source_format.as_str())?;
            dict.set_item("output_format", result.output_format.as_str())?;
            dict.set_item("output_size", result.output_size)?;
            dict.set_item("duration_secs", result.duration_secs)?;
            dict.set_item("success", result.success)?;
            dict.set_item("error", result.error)?;
            Ok(dict.into())
        })
    }

    /// Detect disk image format
    ///
    /// # Arguments
    ///
    /// * `image` - Disk image path
    ///
    /// # Returns
    ///
    /// Format string (qcow2, raw, vmdk, etc.)
    fn detect_format(&self, image: String) -> PyResult<String> {
        let format = self
            .converter
            .detect_format(Path::new(&image))
            .map_err(|e| PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(e.to_string()))?;

        Ok(format.as_str().to_string())
    }

    /// Get disk image information
    ///
    /// # Arguments
    ///
    /// * `image` - Disk image path
    ///
    /// # Returns
    ///
    /// Dictionary with disk image metadata
    fn get_info(&self, image: String) -> PyResult<Py<PyAny>> {
        let info = self
            .converter
            .get_info(Path::new(&image))
            .map_err(|e| PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(e.to_string()))?;

        Python::attach(|py| {
            let json_str = serde_json::to_string(&info)
                .map_err(|e| PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(e.to_string()))?;

            let json_module = py.import("json")?;
            let loads = json_module.getattr("loads")?;
            let result = loads.call1((json_str,))?;
            Ok(result.into())
        })
    }
}

/// Python wrapper for Guestfs handle
#[cfg(feature = "python-bindings")]
#[pyclass]
struct Guestfs {
    handle: crate::guestfs::Guestfs,
}

#[cfg(feature = "python-bindings")]
#[pymethods]
impl Guestfs {
    /// Create a new Guestfs handle
    ///
    /// # Examples
    ///
    /// ```python
    /// from guestctl import Guestfs
    ///
    /// g = Guestfs()
    /// g.add_drive_ro("/path/to/disk.qcow2")
    /// g.launch()
    /// roots = g.inspect_os()
    /// for root in roots:
    ///     print(f"Found OS: {g.inspect_get_distro(root)}")
    /// g.shutdown()
    /// ```
    #[new]
    fn new() -> PyResult<Self> {
        let handle = crate::guestfs::Guestfs::new()
            .map_err(|e| PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(e.to_string()))?;

        Ok(Self { handle })
    }

    /// Add a disk image (read-only)
    ///
    /// # Arguments
    ///
    /// * `filename` - Path to disk image
    fn add_drive_ro(&mut self, filename: String) -> PyResult<()> {
        self.handle
            .add_drive_ro(&filename)
            .map_err(|e| PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(e.to_string()))
    }

    /// Add a disk image (read-write)
    ///
    /// # Arguments
    ///
    /// * `filename` - Path to disk image
    fn add_drive(&mut self, filename: String) -> PyResult<()> {
        self.handle
            .add_drive(&filename)
            .map_err(|e| PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(e.to_string()))
    }

    /// Launch the backend (analyze disk)
    fn launch(&mut self) -> PyResult<()> {
        self.handle
            .launch()
            .map_err(|e| PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(e.to_string()))
    }

    /// Shutdown the backend
    fn shutdown(&mut self) -> PyResult<()> {
        self.handle
            .shutdown()
            .map_err(|e| PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(e.to_string()))
    }

    /// Enable/disable verbose output
    ///
    /// # Arguments
    ///
    /// * `verbose` - Enable verbose mode
    fn set_verbose(&mut self, verbose: bool) {
        self.handle.set_verbose(verbose);
    }

    // === Inspection API ===

    /// Inspect operating systems in the disk image
    ///
    /// # Returns
    ///
    /// List of root devices (e.g., ["/dev/sda1"])
    fn inspect_os(&mut self) -> PyResult<Vec<String>> {
        self.handle
            .inspect_os()
            .map_err(|e| PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(e.to_string()))
    }

    /// Get OS type
    ///
    /// # Arguments
    ///
    /// * `root` - Root device from inspect_os()
    ///
    /// # Returns
    ///
    /// OS type (e.g., "linux", "windows")
    fn inspect_get_type(&mut self, root: String) -> PyResult<String> {
        self.handle
            .inspect_get_type(&root)
            .map_err(|e| PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(e.to_string()))
    }

    /// Get distribution name
    ///
    /// # Arguments
    ///
    /// * `root` - Root device from inspect_os()
    ///
    /// # Returns
    ///
    /// Distribution name (e.g., "fedora", "ubuntu")
    fn inspect_get_distro(&mut self, root: String) -> PyResult<String> {
        self.handle
            .inspect_get_distro(&root)
            .map_err(|e| PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(e.to_string()))
    }

    /// Get major version number
    ///
    /// # Arguments
    ///
    /// * `root` - Root device from inspect_os()
    fn inspect_get_major_version(&mut self, root: String) -> PyResult<i32> {
        self.handle
            .inspect_get_major_version(&root)
            .map_err(|e| PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(e.to_string()))
    }

    /// Get minor version number
    ///
    /// # Arguments
    ///
    /// * `root` - Root device from inspect_os()
    fn inspect_get_minor_version(&mut self, root: String) -> PyResult<i32> {
        self.handle
            .inspect_get_minor_version(&root)
            .map_err(|e| PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(e.to_string()))
    }

    /// Get hostname
    ///
    /// # Arguments
    ///
    /// * `root` - Root device from inspect_os()
    fn inspect_get_hostname(&mut self, root: String) -> PyResult<String> {
        self.handle
            .inspect_get_hostname(&root)
            .map_err(|e| PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(e.to_string()))
    }

    /// Get architecture
    ///
    /// # Arguments
    ///
    /// * `root` - Root device from inspect_os()
    fn inspect_get_arch(&mut self, root: String) -> PyResult<String> {
        self.handle
            .inspect_get_arch(&root)
            .map_err(|e| PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(e.to_string()))
    }

    /// Get product name
    ///
    /// # Arguments
    ///
    /// * `root` - Root device from inspect_os()
    fn inspect_get_product_name(&mut self, root: String) -> PyResult<String> {
        self.handle
            .inspect_get_product_name(&root)
            .map_err(|e| PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(e.to_string()))
    }

    /// Get package format
    ///
    /// # Arguments
    ///
    /// * `root` - Root device from inspect_os()
    ///
    /// # Returns
    ///
    /// Package format (e.g., "rpm", "deb")
    fn inspect_get_package_format(&mut self, root: String) -> PyResult<String> {
        self.handle
            .inspect_get_package_format(&root)
            .map_err(|e| PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(e.to_string()))
    }

    /// Get package management tool
    ///
    /// # Arguments
    ///
    /// * `root` - Root device from inspect_os()
    fn inspect_get_package_management(&mut self, root: String) -> PyResult<String> {
        self.handle
            .inspect_get_package_management(&root)
            .map_err(|e| PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(e.to_string()))
    }

    /// Get mountpoints
    ///
    /// # Arguments
    ///
    /// * `root` - Root device from inspect_os()
    ///
    /// # Returns
    ///
    /// Dictionary of mountpoint -> device mappings
    fn inspect_get_mountpoints(&mut self, root: String) -> PyResult<Py<PyAny>> {
        let mountpoints = self
            .handle
            .inspect_get_mountpoints(&root)
            .map_err(|e| PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(e.to_string()))?;

        Python::attach(|py| {
            let dict = pyo3::types::PyDict::new(py);
            for (mountpoint, device) in mountpoints {
                dict.set_item(mountpoint, device)?;
            }
            Ok(dict.into())
        })
    }

    // === Device Operations ===

    /// List all devices
    ///
    /// # Returns
    ///
    /// List of device names (e.g., ["/dev/sda", "/dev/sdb"])
    fn list_devices(&mut self) -> PyResult<Vec<String>> {
        self.handle
            .list_devices()
            .map_err(|e| PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(e.to_string()))
    }

    /// List all partitions
    ///
    /// # Returns
    ///
    /// List of partition names (e.g., ["/dev/sda1", "/dev/sda2"])
    fn list_partitions(&mut self) -> PyResult<Vec<String>> {
        self.handle
            .list_partitions()
            .map_err(|e| PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(e.to_string()))
    }

    /// Get device size
    ///
    /// # Arguments
    ///
    /// * `device` - Device name
    ///
    /// # Returns
    ///
    /// Size in bytes
    fn blockdev_getsize64(&mut self, device: String) -> PyResult<i64> {
        self.handle
            .blockdev_getsize64(&device)
            .map_err(|e| PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(e.to_string()))
    }

    // === Filesystem Operations ===

    /// Get filesystem type
    ///
    /// # Arguments
    ///
    /// * `device` - Device name
    fn vfs_type(&mut self, device: String) -> PyResult<String> {
        self.handle
            .vfs_type(&device)
            .map_err(|e| PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(e.to_string()))
    }

    /// Get filesystem label
    ///
    /// # Arguments
    ///
    /// * `device` - Device name
    fn vfs_label(&mut self, device: String) -> PyResult<String> {
        self.handle
            .vfs_label(&device)
            .map_err(|e| PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(e.to_string()))
    }

    /// Get filesystem UUID
    ///
    /// # Arguments
    ///
    /// * `device` - Device name
    fn vfs_uuid(&mut self, device: String) -> PyResult<String> {
        self.handle
            .vfs_uuid(&device)
            .map_err(|e| PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(e.to_string()))
    }

    /// Mount filesystem
    ///
    /// # Arguments
    ///
    /// * `device` - Device to mount
    /// * `mountpoint` - Mount point path
    fn mount(&mut self, device: String, mountpoint: String) -> PyResult<()> {
        self.handle
            .mount(&device, &mountpoint)
            .map_err(|e| PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(e.to_string()))
    }

    /// Mount filesystem read-only
    ///
    /// # Arguments
    ///
    /// * `device` - Device to mount
    /// * `mountpoint` - Mount point path
    fn mount_ro(&mut self, device: String, mountpoint: String) -> PyResult<()> {
        self.handle
            .mount_ro(&device, &mountpoint)
            .map_err(|e| PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(e.to_string()))
    }

    /// Unmount filesystem
    ///
    /// # Arguments
    ///
    /// * `mountpoint` - Mount point path
    fn umount(&mut self, mountpoint: String) -> PyResult<()> {
        self.handle
            .umount(&mountpoint)
            .map_err(|e| PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(e.to_string()))
    }

    // === File Operations ===

    /// Read file contents
    ///
    /// # Arguments
    ///
    /// * `path` - File path in guest
    ///
    /// # Returns
    ///
    /// File contents as bytes
    fn read_file(&mut self, path: String) -> PyResult<Vec<u8>> {
        self.handle
            .read_file(&path)
            .map_err(|e| PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(e.to_string()))
    }

    /// Write file contents
    ///
    /// # Arguments
    ///
    /// * `path` - File path in guest
    /// * `content` - Content to write
    fn write(&mut self, path: String, content: Vec<u8>) -> PyResult<()> {
        self.handle
            .write(&path, &content)
            .map_err(|e| PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(e.to_string()))
    }

    /// Check if path exists
    ///
    /// # Arguments
    ///
    /// * `path` - Path in guest
    fn exists(&mut self, path: String) -> PyResult<bool> {
        self.handle
            .exists(&path)
            .map_err(|e| PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(e.to_string()))
    }

    /// Check if path is a file
    ///
    /// # Arguments
    ///
    /// * `path` - Path in guest
    fn is_file(&mut self, path: String) -> PyResult<bool> {
        self.handle
            .is_file(&path)
            .map_err(|e| PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(e.to_string()))
    }

    /// Check if path is a directory
    ///
    /// # Arguments
    ///
    /// * `path` - Path in guest
    fn is_dir(&mut self, path: String) -> PyResult<bool> {
        self.handle
            .is_dir(&path)
            .map_err(|e| PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(e.to_string()))
    }

    /// List directory contents
    ///
    /// # Arguments
    ///
    /// * `directory` - Directory path in guest
    fn ls(&mut self, directory: String) -> PyResult<Vec<String>> {
        self.handle
            .ls(&directory)
            .map_err(|e| PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(e.to_string()))
    }

    /// Download file from guest
    ///
    /// # Arguments
    ///
    /// * `remotefilename` - File path in guest
    /// * `filename` - Local file path
    fn download(&mut self, remotefilename: String, filename: String) -> PyResult<()> {
        self.handle
            .download(&remotefilename, &filename)
            .map_err(|e| PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(e.to_string()))
    }

    /// Upload file to guest
    ///
    /// # Arguments
    ///
    /// * `filename` - Local file path
    /// * `remotefilename` - File path in guest
    fn upload(&mut self, filename: String, remotefilename: String) -> PyResult<()> {
        self.handle
            .upload(&filename, &remotefilename)
            .map_err(|e| PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(e.to_string()))
    }

    // === Package Management ===

    /// List installed packages
    ///
    /// # Arguments
    ///
    /// * `root` - Root device from inspect_os()
    ///
    /// # Returns
    ///
    /// List of installed packages
    fn inspect_list_applications(&mut self, root: String) -> PyResult<Py<PyAny>> {
        let apps = self
            .handle
            .inspect_list_applications(&root)
            .map_err(|e| PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(e.to_string()))?;

        Python::attach(|py| {
            let list = pyo3::types::PyList::empty(py);
            for app in apps {
                let dict = pyo3::types::PyDict::new(py);
                dict.set_item("app_name", &app.name)?;
                dict.set_item("app_display_name", &app.display_name)?;
                dict.set_item("app_epoch", app.epoch)?;
                dict.set_item("app_version", &app.version)?;
                dict.set_item("app_release", &app.release)?;
                dict.set_item("app_install_path", &app.install_path)?;
                dict.set_item("app_publisher", &app.publisher)?;
                dict.set_item("app_url", &app.url)?;
                dict.set_item("app_description", &app.description)?;
                list.append(dict)?;
            }
            Ok(list.into())
        })
    }

    // === Command Execution ===

    /// Execute a command in the guest
    ///
    /// # Arguments
    ///
    /// * `arguments` - List of command arguments (first is command name)
    ///
    /// # Returns
    ///
    /// Command output as string
    fn command(&mut self, arguments: Vec<String>) -> PyResult<String> {
        let args: Vec<&str> = arguments.iter().map(|s| s.as_str()).collect();
        self.handle
            .command(&args)
            .map_err(|e| PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(e.to_string()))
    }

    /// Execute shell command lines
    ///
    /// # Arguments
    ///
    /// * `command` - Shell command string
    ///
    /// # Returns
    ///
    /// List of output lines
    fn sh_lines(&mut self, command: String) -> PyResult<Vec<String>> {
        self.handle
            .sh_lines(&command)
            .map_err(|e| PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(e.to_string()))
    }

    /// Execute shell command
    ///
    /// # Arguments
    ///
    /// * `command` - Shell command string
    ///
    /// # Returns
    ///
    /// Command output as string
    fn sh(&mut self, command: String) -> PyResult<String> {
        self.handle
            .sh(&command)
            .map_err(|e| PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(e.to_string()))
    }

    // === LVM Operations ===

    /// Scan for LVM volume groups
    fn vgscan(&mut self) -> PyResult<()> {
        self.handle
            .vgscan()
            .map_err(|e| PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(e.to_string()))
    }

    /// List LVM volume groups
    ///
    /// # Returns
    ///
    /// List of volume group names
    fn vgs(&mut self) -> PyResult<Vec<String>> {
        self.handle
            .vgs()
            .map_err(|e| PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(e.to_string()))
    }

    /// List LVM physical volumes
    ///
    /// # Returns
    ///
    /// List of physical volume names
    fn pvs(&mut self) -> PyResult<Vec<String>> {
        self.handle
            .pvs()
            .map_err(|e| PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(e.to_string()))
    }

    /// List LVM logical volumes
    ///
    /// # Returns
    ///
    /// List of logical volume names
    fn lvs(&mut self) -> PyResult<Vec<String>> {
        self.handle
            .lvs()
            .map_err(|e| PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(e.to_string()))
    }

    // === Archive Operations ===

    /// Extract tar archive into guest directory
    ///
    /// # Arguments
    ///
    /// * `tarfile` - Path to tar file on host
    /// * `directory` - Directory in guest to extract to
    fn tar_in(&mut self, tarfile: String, directory: String) -> PyResult<()> {
        self.handle
            .tar_in(&tarfile, &directory)
            .map_err(|e| PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(e.to_string()))
    }

    /// Create tar archive from guest directory
    ///
    /// # Arguments
    ///
    /// * `directory` - Directory in guest to archive
    /// * `tarfile` - Path to tar file on host
    fn tar_out(&mut self, directory: String, tarfile: String) -> PyResult<()> {
        self.handle
            .tar_out(&directory, &tarfile)
            .map_err(|e| PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(e.to_string()))
    }

    /// Extract compressed tar archive into guest directory
    ///
    /// # Arguments
    ///
    /// * `tarfile` - Path to tar.gz file on host
    /// * `directory` - Directory in guest to extract to
    fn tgz_in(&mut self, tarfile: String, directory: String) -> PyResult<()> {
        self.handle
            .tgz_in(&tarfile, &directory)
            .map_err(|e| PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(e.to_string()))
    }

    /// Create compressed tar archive from guest directory
    ///
    /// # Arguments
    ///
    /// * `directory` - Directory in guest to archive
    /// * `tarfile` - Path to tar.gz file on host
    fn tgz_out(&mut self, directory: String, tarfile: String) -> PyResult<()> {
        self.handle
            .tgz_out(&directory, &tarfile)
            .map_err(|e| PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(e.to_string()))
    }

    // === Additional File Operations ===

    /// Read entire file as string
    ///
    /// # Arguments
    ///
    /// * `path` - File path in guest
    ///
    /// # Returns
    ///
    /// File contents as string
    fn cat(&mut self, path: String) -> PyResult<String> {
        self.handle
            .cat(&path)
            .map_err(|e| PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(e.to_string()))
    }

    /// Create directory
    ///
    /// # Arguments
    ///
    /// * `path` - Directory path in guest
    fn mkdir(&mut self, path: String) -> PyResult<()> {
        self.handle
            .mkdir(&path)
            .map_err(|e| PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(e.to_string()))
    }

    /// Create directory with parents
    ///
    /// # Arguments
    ///
    /// * `path` - Directory path in guest
    fn mkdir_p(&mut self, path: String) -> PyResult<()> {
        self.handle
            .mkdir_p(&path)
            .map_err(|e| PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(e.to_string()))
    }

    /// Remove file
    ///
    /// # Arguments
    ///
    /// * `path` - File path in guest
    fn rm(&mut self, path: String) -> PyResult<()> {
        self.handle
            .rm(&path)
            .map_err(|e| PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(e.to_string()))
    }

    /// Remove directory
    ///
    /// # Arguments
    ///
    /// * `path` - Directory path in guest
    fn rmdir(&mut self, path: String) -> PyResult<()> {
        self.handle
            .rmdir(&path)
            .map_err(|e| PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(e.to_string()))
    }

    /// Remove directory recursively
    ///
    /// # Arguments
    ///
    /// * `path` - Directory path in guest
    fn rm_rf(&mut self, path: String) -> PyResult<()> {
        self.handle
            .rm_rf(&path)
            .map_err(|e| PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(e.to_string()))
    }

    /// Change file permissions
    ///
    /// # Arguments
    ///
    /// * `mode` - Permission mode (octal)
    /// * `path` - File path in guest
    fn chmod(&mut self, mode: i32, path: String) -> PyResult<()> {
        self.handle
            .chmod(mode, &path)
            .map_err(|e| PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(e.to_string()))
    }

    /// Change file owner
    ///
    /// # Arguments
    ///
    /// * `owner` - New owner UID
    /// * `group` - New group GID
    /// * `path` - File path in guest
    fn chown(&mut self, owner: i32, group: i32, path: String) -> PyResult<()> {
        self.handle
            .chown(owner, group, &path)
            .map_err(|e| PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(e.to_string()))
    }

    /// Get file stat information
    ///
    /// # Arguments
    ///
    /// * `path` - File path in guest
    ///
    /// # Returns
    ///
    /// Dictionary with stat information
    fn stat(&mut self, path: String) -> PyResult<Py<PyAny>> {
        let stat = self
            .handle
            .stat(&path)
            .map_err(|e| PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(e.to_string()))?;

        Python::attach(|py| {
            let dict = pyo3::types::PyDict::new(py);
            dict.set_item("dev", stat.dev)?;
            dict.set_item("ino", stat.ino)?;
            dict.set_item("mode", stat.mode)?;
            dict.set_item("nlink", stat.nlink)?;
            dict.set_item("uid", stat.uid)?;
            dict.set_item("gid", stat.gid)?;
            dict.set_item("rdev", stat.rdev)?;
            dict.set_item("size", stat.size)?;
            dict.set_item("blksize", stat.blksize)?;
            dict.set_item("blocks", stat.blocks)?;
            dict.set_item("atime", stat.atime)?;
            dict.set_item("mtime", stat.mtime)?;
            dict.set_item("ctime", stat.ctime)?;
            Ok(dict.into())
        })
    }

    /// Get filesystem statistics
    ///
    /// # Arguments
    ///
    /// * `path` - Path in guest filesystem
    ///
    /// # Returns
    ///
    /// Dictionary with filesystem statistics
    fn statvfs(&mut self, path: String) -> PyResult<Py<PyAny>> {
        let statvfs = self
            .handle
            .statvfs(&path)
            .map_err(|e| PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(e.to_string()))?;

        Python::attach(|py| {
            let dict = pyo3::types::PyDict::new(py);
            for (key, value) in statvfs {
                dict.set_item(key, value)?;
            }
            Ok(dict.into())
        })
    }

    // === Checksum Operations ===

    /// Calculate file checksum
    ///
    /// # Arguments
    ///
    /// * `csumtype` - Checksum type (md5, sha1, sha256, etc.)
    /// * `path` - File path in guest
    ///
    /// # Returns
    ///
    /// Checksum as hex string
    fn checksum(&mut self, csumtype: String, path: String) -> PyResult<String> {
        self.handle
            .checksum(&csumtype, &path)
            .map_err(|e| PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(e.to_string()))
    }

    // === Unmount Operations ===

    /// Unmount all filesystems
    fn umount_all(&mut self) -> PyResult<()> {
        self.handle
            .umount_all()
            .map_err(|e| PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(e.to_string()))
    }

    /// Synchronize filesystem
    fn sync(&mut self) -> PyResult<()> {
        self.handle
            .sync()
            .map_err(|e| PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(e.to_string()))
    }

    // Context manager support
    /// Enter context manager
    ///
    /// # Examples
    ///
    /// ```python
    /// from guestctl import Guestfs
    ///
    /// with Guestfs() as g:
    ///     g.add_drive_ro("/path/to/disk.qcow2")
    ///     g.launch()
    ///     roots = g.inspect_os()
    ///     # ... operations
    ///     # Automatic cleanup on exit
    /// ```
    fn __enter__(slf: PyRef<Self>) -> PyRef<Self> {
        slf
    }

    /// Exit context manager
    #[pyo3(signature = (_exc_type=None, _exc_value=None, _traceback=None))]
    fn __exit__(
        &mut self,
        _exc_type: Option<&Bound<'_, PyAny>>,
        _exc_value: Option<&Bound<'_, PyAny>>,
        _traceback: Option<&Bound<'_, PyAny>>,
    ) -> PyResult<bool> {
        self.shutdown()?;
        Ok(false)
    }
}

/* TODO: Async Python API - Waiting for pyo3-asyncio PyO3 0.22 support
 *
 * AsyncGuestfs will be enabled once pyo3-asyncio releases support for PyO3 0.22+
 * Track progress at: https://github.com/awestlake87/pyo3-asyncio/issues
 *
/// Async Python wrapper for Guestfs handle
///
/// Provides non-blocking operations for concurrent VM inspection.
#[cfg(feature = "python-bindings")]
#[pyclass]
struct AsyncGuestfs {
    handle: std::sync::Arc<tokio::sync::Mutex<crate::guestfs::Guestfs>>,
}

#[cfg(feature = "python-bindings")]
#[pymethods]
impl AsyncGuestfs {
*/
/*
    /// Create a new AsyncGuestfs handle
    ///
    /// # Examples
    ///
    /// ```python
    /// import asyncio
    /// from guestctl import AsyncGuestfs
    ///
    /// async def main():
    ///     async with AsyncGuestfs() as g:
    ///         await g.add_drive_ro("/path/to/disk.qcow2")
    ///         await g.launch()
    ///         roots = await g.inspect_os()
    ///         for root in roots:
    ///             print(f"Found OS: {await g.inspect_get_distro(root)}")
    ///
    /// asyncio.run(main())
    /// ```
    #[new]
    fn new() -> PyResult<Self> {
        let handle = crate::guestfs::Guestfs::new()
            .map_err(|e| PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(e.to_string()))?;

        Ok(Self {
            handle: std::sync::Arc::new(tokio::sync::Mutex::new(handle)),
        })
    }

    /// Context manager entry
    fn __aenter__<'p>(slf: pyo3::Py<Self>, py: pyo3::Python<'p>) -> PyResult<pyo3::Bound<'p, pyo3::types::PyAny>> {
        pyo3_asyncio_0_21::tokio::future_into_py(py, async move {
            Ok(slf)
        })
    }

    /// Context manager exit
    fn __aexit__<'p>(
        slf: pyo3::Py<Self>,
        py: pyo3::Python<'p>,
        _exc_type: Option<&pyo3::Bound<'_, pyo3::types::PyAny>>,
        _exc_value: Option<&pyo3::Bound<'_, pyo3::types::PyAny>>,
        _traceback: Option<&pyo3::Bound<'_, pyo3::types::PyAny>>,
    ) -> PyResult<pyo3::Bound<'p, pyo3::types::PyAny>> {
        pyo3_asyncio_0_21::tokio::future_into_py(py, async move {
            Python::attach(|py| {
                let binding = slf.bind(py).borrow_mut();
                let handle = binding.handle.clone();
                drop(binding);

                tokio::spawn(async move {
                    let mut h = handle.lock().await;
                    let _ = h.shutdown();
                });

                Ok(false)
            })
        })
    }

    /// Add a disk image (read-only) - async version
    fn add_drive_ro<'p>(
        &self,
        py: pyo3::Python<'p>,
        filename: String,
    ) -> PyResult<pyo3::Bound<'p, pyo3::types::PyAny>> {
        let handle = self.handle.clone();
        pyo3_asyncio_0_21::tokio::future_into_py(py, async move {
            let mut h = handle.lock().await;
            h.add_drive_ro(&filename)
                .map_err(|e| PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(e.to_string()))
        })
    }

    /// Add a disk image (read-write) - async version
    fn add_drive<'p>(
        &self,
        py: pyo3::Python<'p>,
        filename: String,
    ) -> PyResult<pyo3::Bound<'p, pyo3::types::PyAny>> {
        let handle = self.handle.clone();
        pyo3_asyncio_0_21::tokio::future_into_py(py, async move {
            let mut h = handle.lock().await;
            h.add_drive(&filename)
                .map_err(|e| PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(e.to_string()))
        })
    }

    /// Launch the backend (analyze disk) - async version
    fn launch<'p>(&self, py: pyo3::Python<'p>) -> PyResult<pyo3::Bound<'p, pyo3::types::PyAny>> {
        let handle = self.handle.clone();
        pyo3_asyncio_0_21::tokio::future_into_py(py, async move {
            let mut h = handle.lock().await;
            h.launch()
                .map_err(|e| PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(e.to_string()))
        })
    }

    /// Shutdown the backend - async version
    fn shutdown<'p>(&self, py: pyo3::Python<'p>) -> PyResult<pyo3::Bound<'p, pyo3::types::PyAny>> {
        let handle = self.handle.clone();
        pyo3_asyncio_0_21::tokio::future_into_py(py, async move {
            let mut h = handle.lock().await;
            h.shutdown()
                .map_err(|e| PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(e.to_string()))
        })
    }

    /// Inspect operating systems in the disk image - async version
    fn inspect_os<'p>(&self, py: pyo3::Python<'p>) -> PyResult<pyo3::Bound<'p, pyo3::types::PyAny>> {
        let handle = self.handle.clone();
        pyo3_asyncio_0_21::tokio::future_into_py(py, async move {
            let mut h = handle.lock().await;
            h.inspect_os()
                .map_err(|e| PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(e.to_string()))
        })
    }

    /// Get OS type - async version
    fn inspect_get_type<'p>(
        &self,
        py: pyo3::Python<'p>,
        root: String,
    ) -> PyResult<pyo3::Bound<'p, pyo3::types::PyAny>> {
        let handle = self.handle.clone();
        pyo3_asyncio_0_21::tokio::future_into_py(py, async move {
            let mut h = handle.lock().await;
            h.inspect_get_type(&root)
                .map_err(|e| PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(e.to_string()))
        })
    }

    /// Get distribution name - async version
    fn inspect_get_distro<'p>(
        &self,
        py: pyo3::Python<'p>,
        root: String,
    ) -> PyResult<pyo3::Bound<'p, pyo3::types::PyAny>> {
        let handle = self.handle.clone();
        pyo3_asyncio_0_21::tokio::future_into_py(py, async move {
            let mut h = handle.lock().await;
            h.inspect_get_distro(&root)
                .map_err(|e| PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(e.to_string()))
        })
    }

    /// Get major version - async version
    fn inspect_get_major_version<'p>(
        &self,
        py: pyo3::Python<'p>,
        root: String,
    ) -> PyResult<pyo3::Bound<'p, pyo3::types::PyAny>> {
        let handle = self.handle.clone();
        pyo3_asyncio_0_21::tokio::future_into_py(py, async move {
            let mut h = handle.lock().await;
            h.inspect_get_major_version(&root)
                .map_err(|e| PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(e.to_string()))
        })
    }

    /// Get minor version - async version
    fn inspect_get_minor_version<'p>(
        &self,
        py: pyo3::Python<'p>,
        root: String,
    ) -> PyResult<pyo3::Bound<'p, pyo3::types::PyAny>> {
        let handle = self.handle.clone();
        pyo3_asyncio_0_21::tokio::future_into_py(py, async move {
            let mut h = handle.lock().await;
            h.inspect_get_minor_version(&root)
                .map_err(|e| PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(e.to_string()))
        })
    }

    /// Get hostname - async version
    fn inspect_get_hostname<'p>(
        &self,
        py: pyo3::Python<'p>,
        root: String,
    ) -> PyResult<pyo3::Bound<'p, pyo3::types::PyAny>> {
        let handle = self.handle.clone();
        pyo3_asyncio_0_21::tokio::future_into_py(py, async move {
            let mut h = handle.lock().await;
            h.inspect_get_hostname(&root)
                .map_err(|e| PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(e.to_string()))
        })
    }

    /// List filesystems - async version
    fn list_filesystems<'p>(
        &self,
        py: pyo3::Python<'p>,
    ) -> PyResult<pyo3::Bound<'p, pyo3::types::PyAny>> {
        let handle = self.handle.clone();
        pyo3_asyncio_0_21::tokio::future_into_py(py, async move {
            let mut h = handle.lock().await;
            h.list_filesystems()
                .map_err(|e| PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(e.to_string()))
        })
    }

    /// Mount a filesystem - async version
    fn mount<'p>(
        &self,
        py: pyo3::Python<'p>,
        device: String,
        mountpoint: String,
    ) -> PyResult<pyo3::Bound<'p, pyo3::types::PyAny>> {
        let handle = self.handle.clone();
        pyo3_asyncio_0_21::tokio::future_into_py(py, async move {
            let mut h = handle.lock().await;
            h.mount(&device, &mountpoint)
                .map_err(|e| PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(e.to_string()))
        })
    }

    /// List directory contents - async version
    fn ls<'p>(
        &self,
        py: pyo3::Python<'p>,
        directory: String,
    ) -> PyResult<pyo3::Bound<'p, pyo3::types::PyAny>> {
        let handle = self.handle.clone();
        pyo3_asyncio_0_21::tokio::future_into_py(py, async move {
            let mut h = handle.lock().await;
            h.ls(&directory)
                .map_err(|e| PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(e.to_string()))
        })
    }

    /// Read file contents - async version
    fn cat<'p>(
        &self,
        py: pyo3::Python<'p>,
        path: String,
    ) -> PyResult<pyo3::Bound<'p, pyo3::types::PyAny>> {
        let handle = self.handle.clone();
        pyo3_asyncio_0_21::tokio::future_into_py(py, async move {
            let mut h = handle.lock().await;
            h.cat(&path)
                .map_err(|e| PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(e.to_string()))
        })
    }
}
*/

/// Python module definition
#[cfg(feature = "python-bindings")]
#[pymodule]
fn guestctl(m: &pyo3::Bound<'_, pyo3::types::PyModule>) -> PyResult<()> {
    m.add_class::<Guestfs>()?;
    // m.add_class::<AsyncGuestfs>()?;  // TODO: Enable when pyo3-asyncio supports PyO3 0.22+
    m.add_class::<DiskConverter>()?;
    m.add("__version__", env!("CARGO_PKG_VERSION"))?;
    Ok(())
}

// Stub when python-bindings feature is not enabled
#[cfg(not(feature = "python-bindings"))]
pub fn python_bindings_not_available() {
    eprintln!("Python bindings not compiled. Build with --features python-bindings");
}
