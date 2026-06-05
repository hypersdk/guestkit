// SPDX-License-Identifier: Apache-2.0
//! Export module for generating reports in various formats
//!
//! This module provides functionality to export inspection results
//! to different formats including HTML, PDF, and Markdown.

pub mod html;
pub mod pdf;
pub mod template;

pub use html::{HtmlExportOptions, HtmlExporter};
pub use pdf::{PaperSize, PdfExportOptions, PdfExporter};
pub use template::{create_variable_map, TemplateEngine, TemplateFormat, TemplateLevel};
