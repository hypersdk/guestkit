// SPDX-License-Identifier: LGPL-3.0-or-later
//! Export module for generating reports in various formats
//!
//! This module provides functionality to export inspection results
//! to different formats including HTML, PDF, and Markdown.

pub mod html;

pub use html::{HtmlExporter, HtmlExportOptions};
