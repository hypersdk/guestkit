// SPDX-License-Identifier: LGPL-3.0-or-later
//! PyO3 Python bindings for guestkit
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
    /// from guestkit import DiskConverter
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
    ) -> PyResult<PyObject> {
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

        Python::with_gil(|py| {
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
    fn get_info(&self, image: String) -> PyResult<PyObject> {
        let info = self
            .converter
            .get_info(Path::new(&image))
            .map_err(|e| PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(e.to_string()))?;

        Python::with_gil(|py| {
            let json_str = serde_json::to_string(&info)
                .map_err(|e| PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(e.to_string()))?;

            let json_module = py.import("json")?;
            let loads = json_module.getattr("loads")?;
            let result = loads.call1((json_str,))?;
            Ok(result.into())
        })
    }
}

/// Python module definition
#[cfg(feature = "python-bindings")]
#[pymodule]
fn guestkit_py(_py: Python, m: &PyModule) -> PyResult<()> {
    m.add_class::<DiskConverter>()?;
    m.add("__version__", env!("CARGO_PKG_VERSION"))?;
    Ok(())
}

// Stub when python-bindings feature is not enabled
#[cfg(not(feature = "python-bindings"))]
pub fn python_bindings_not_available() {
    eprintln!("Python bindings not compiled. Build with --features python-bindings");
}
