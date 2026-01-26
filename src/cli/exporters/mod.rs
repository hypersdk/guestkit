// SPDX-License-Identifier: LGPL-3.0-or-later
//! Report export functionality

pub mod html;
pub mod markdown;
pub mod pdf;

use crate::cli::formatters::InspectionReport;
use anyhow::Result;
use std::path::Path;

/// Export format for reports
#[derive(Debug, Clone, Copy)]
pub enum ExportFormat {
    Html,
    Markdown,
    Pdf,
}

impl ExportFormat {
    pub fn from_str(s: &str) -> Result<Self> {
        match s.to_lowercase().as_str() {
            "html" => Ok(ExportFormat::Html),
            "md" | "markdown" => Ok(ExportFormat::Markdown),
            "pdf" => Ok(ExportFormat::Pdf),
            _ => Err(anyhow::anyhow!("Unknown export format: {}", s)),
        }
    }

    #[allow(dead_code)]
    pub fn extension(&self) -> &'static str {
        match self {
            ExportFormat::Html => "html",
            ExportFormat::Markdown => "md",
            ExportFormat::Pdf => "pdf",
        }
    }
}

/// Export an inspection report to a file
pub fn export_report(
    report: &InspectionReport,
    format: ExportFormat,
    output_path: &Path,
) -> Result<()> {
    match format {
        ExportFormat::Html => {
            let content = html::generate_html_report(report)?;
            std::fs::write(output_path, content)?;
        }
        ExportFormat::Markdown => {
            let content = markdown::generate_markdown_report(report)?;
            std::fs::write(output_path, content)?;
        }
        ExportFormat::Pdf => {
            pdf::generate_pdf_to_file(report, output_path)?;
        }
    }

    Ok(())
}
