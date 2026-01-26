// SPDX-License-Identifier: LGPL-3.0-or-later
//! Export module for generating reports in various formats
//!
//! This module provides functionality to export inspection results
//! to different formats including HTML, PDF, and Markdown.

pub mod html;
pub mod pdf;
pub mod template;

pub use html::{HtmlExporter, HtmlExportOptions};
pub use pdf::{PdfExporter, PdfExportOptions, PaperSize};
pub use template::{TemplateEngine, TemplateFormat, TemplateLevel, create_variable_map};
