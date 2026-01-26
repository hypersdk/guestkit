// SPDX-License-Identifier: LGPL-3.0-or-later
//! PDF report generation with professional layout

use printpdf::*;
use std::fs::File;
use std::io::BufWriter;
use std::path::Path;

/// PDF export options
#[derive(Debug, Clone)]
pub struct PdfExportOptions {
    /// Include page numbers
    pub include_page_numbers: bool,

    /// Include table of contents
    pub include_toc: bool,

    /// Use color (vs black and white)
    pub use_color: bool,

    /// Paper size (A4, Letter, etc.)
    pub paper_size: PaperSize,

    /// Font size for body text
    pub font_size: f32,
}

impl Default for PdfExportOptions {
    fn default() -> Self {
        Self {
            include_page_numbers: true,
            include_toc: true,
            use_color: true,
            paper_size: PaperSize::A4,
            font_size: 12.0,
        }
    }
}

/// Paper size options
#[derive(Debug, Clone, Copy)]
pub enum PaperSize {
    A4,
    Letter,
    Legal,
}

impl PaperSize {
    fn to_mm(&self) -> (f32, f32) {
        match self {
            PaperSize::A4 => (210.0, 297.0),
            PaperSize::Letter => (215.9, 279.4),
            PaperSize::Legal => (215.9, 355.6),
        }
    }
}

/// Inspection data for PDF export
#[derive(Debug, Clone)]
pub struct InspectionData {
    pub hostname: String,
    pub os_type: String,
    pub distribution: String,
    pub version: String,
    pub architecture: String,
    pub product_name: String,
    pub package_format: String,
    pub package_manager: String,
    pub kernel_version: Option<String>,
    pub total_memory: Option<u64>,
    pub vcpus: Option<u32>,
    pub filesystems: Vec<FilesystemInfo>,
    pub packages: Vec<PackageInfo>,
    pub users: Vec<UserInfo>,
    pub interfaces: Vec<NetworkInterface>,
}

/// Filesystem information
#[derive(Debug, Clone)]
pub struct FilesystemInfo {
    pub device: String,
    pub mountpoint: String,
    pub fstype: String,
    pub size: u64,
    pub used: u64,
    pub available: u64,
}

/// Package information
#[derive(Debug, Clone)]
pub struct PackageInfo {
    pub name: String,
    pub version: String,
    pub arch: String,
}

/// User information
#[derive(Debug, Clone)]
pub struct UserInfo {
    pub username: String,
    pub uid: String,
    pub home: String,
    pub shell: String,
}

/// Network interface information
#[derive(Debug, Clone)]
pub struct NetworkInterface {
    pub name: String,
    pub mac_address: String,
    pub ip_addresses: Vec<String>,
    pub state: String,
}

/// PDF report exporter
pub struct PdfExporter {
    options: PdfExportOptions,
}

impl PdfExporter {
    /// Create a new PDF exporter with default options
    pub fn new(options: PdfExportOptions) -> Self {
        Self { options }
    }

    /// Generate PDF report
    pub fn generate<P: AsRef<Path>>(
        &self,
        output_path: P,
        data: &InspectionData,
    ) -> std::io::Result<()> {
        let (width_mm, height_mm) = self.options.paper_size.to_mm();
        let width_pt = Mm(width_mm).into();
        let height_pt = Mm(height_mm).into();

        let (doc, page1, layer1) = PdfDocument::new(
            &format!("VM Inspection Report - {}", data.hostname),
            width_pt,
            height_pt,
            "Layer 1",
        );

        // Use built-in font (Helvetica)
        let font = doc.add_builtin_font(BuiltinFont::Helvetica)
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e.to_string()))?;

        let bold_font = doc.add_builtin_font(BuiltinFont::HelveticaBold)
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e.to_string()))?;

        let current_layer = doc.get_page(page1).get_layer(layer1);

        // Add title
        let title = format!("VM Inspection Report");
        current_layer.use_text(&title, self.options.font_size + 8.0, Mm(20.0), Mm(height_mm - 30.0), &bold_font);

        let subtitle = format!("{} - {}", data.hostname, chrono::Local::now().format("%Y-%m-%d"));
        current_layer.use_text(&subtitle, self.options.font_size, Mm(20.0), Mm(height_mm - 40.0), &font);

        // Current Y position for content
        let mut y_pos = height_mm - 60.0;

        // System Information Section
        y_pos = self.add_section_header(&current_layer, "System Information", y_pos, &bold_font);
        y_pos = self.add_text_line(&current_layer, &format!("OS Type: {}", data.os_type), y_pos, &font);
        y_pos = self.add_text_line(&current_layer, &format!("Distribution: {}", data.distribution), y_pos, &font);
        y_pos = self.add_text_line(&current_layer, &format!("Version: {}", data.version), y_pos, &font);
        y_pos = self.add_text_line(&current_layer, &format!("Architecture: {}", data.architecture), y_pos, &font);
        y_pos = self.add_text_line(&current_layer, &format!("Product: {}", data.product_name), y_pos, &font);
        y_pos = self.add_text_line(&current_layer, &format!("Package Format: {}", data.package_format), y_pos, &font);
        y_pos = self.add_text_line(&current_layer, &format!("Package Manager: {}", data.package_manager), y_pos, &font);

        if let Some(kernel) = &data.kernel_version {
            y_pos = self.add_text_line(&current_layer, &format!("Kernel: {}", kernel), y_pos, &font);
        }

        if let Some(mem) = data.total_memory {
            y_pos = self.add_text_line(&current_layer, &format!("Memory: {} GB", mem / 1024 / 1024 / 1024), y_pos, &font);
        }

        if let Some(vcpus) = data.vcpus {
            y_pos = self.add_text_line(&current_layer, &format!("vCPUs: {}", vcpus), y_pos, &font);
        }

        y_pos -= 10.0;

        // Filesystems Section
        if !data.filesystems.is_empty() {
            y_pos = self.add_section_header(&current_layer, "Filesystems", y_pos, &bold_font);

            for fs in data.filesystems.iter().take(10) {
                let fs_text = format!("{} -> {} ({}) - {:.1} GB / {:.1} GB",
                    fs.device,
                    fs.mountpoint,
                    fs.fstype,
                    fs.used as f64 / 1024.0 / 1024.0 / 1024.0,
                    fs.size as f64 / 1024.0 / 1024.0 / 1024.0,
                );
                y_pos = self.add_text_line(&current_layer, &fs_text, y_pos, &font);

                // Check if we need a new page
                if y_pos < 30.0 {
                    break;
                }
            }

            y_pos -= 10.0;
        }

        // Packages Section (show count)
        if !data.packages.is_empty() {
            y_pos = self.add_section_header(&current_layer, "Installed Packages", y_pos, &bold_font);
            y_pos = self.add_text_line(&current_layer, &format!("Total packages: {}", data.packages.len()), y_pos, &font);

            // Show first 20 packages
            let packages_to_show = data.packages.iter().take(20);
            for pkg in packages_to_show {
                let pkg_text = format!("{} - {} ({})", pkg.name, pkg.version, pkg.arch);
                y_pos = self.add_text_line(&current_layer, &pkg_text, y_pos, &font);

                if y_pos < 30.0 {
                    break;
                }
            }

            if data.packages.len() > 20 {
                y_pos = self.add_text_line(&current_layer, &format!("... and {} more packages", data.packages.len() - 20), y_pos, &font);
            }

            y_pos -= 10.0;
        }

        // Users Section
        if !data.users.is_empty() {
            y_pos = self.add_section_header(&current_layer, "User Accounts", y_pos, &bold_font);

            for user in data.users.iter().take(15) {
                let user_text = format!("{} (UID: {}) - {} [{}]",
                    user.username,
                    user.uid,
                    user.home,
                    user.shell,
                );
                y_pos = self.add_text_line(&current_layer, &user_text, y_pos, &font);

                if y_pos < 30.0 {
                    break;
                }
            }

            y_pos -= 10.0;
        }

        // Network Interfaces Section
        if !data.interfaces.is_empty() {
            y_pos = self.add_section_header(&current_layer, "Network Interfaces", y_pos, &bold_font);

            for iface in &data.interfaces {
                let iface_text = format!("{} - {} [{}] - {}",
                    iface.name,
                    iface.mac_address,
                    iface.ip_addresses.join(", "),
                    iface.state,
                );
                y_pos = self.add_text_line(&current_layer, &iface_text, y_pos, &font);

                if y_pos < 30.0 {
                    break;
                }
            }
        }

        // Add footer with page number if enabled
        if self.options.include_page_numbers {
            current_layer.use_text("Page 1", self.options.font_size - 2.0, Mm(width_mm / 2.0 - 10.0), Mm(15.0), &font);
        }

        // Save the PDF
        doc.save(&mut BufWriter::new(File::create(output_path)?))
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e.to_string()))?;

        Ok(())
    }

    /// Add a section header
    fn add_section_header(
        &self,
        layer: &PdfLayerReference,
        text: &str,
        y_pos: f32,
        font: &IndirectFontRef,
    ) -> f32 {
        layer.use_text(text, self.options.font_size + 4.0, Mm(20.0), Mm(y_pos), font);
        y_pos - 10.0
    }

    /// Add a text line
    fn add_text_line(
        &self,
        layer: &PdfLayerReference,
        text: &str,
        y_pos: f32,
        font: &IndirectFontRef,
    ) -> f32 {
        layer.use_text(text, self.options.font_size, Mm(25.0), Mm(y_pos), font);
        y_pos - 6.0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_pdf_export_options_default() {
        let options = PdfExportOptions::default();
        assert!(options.include_page_numbers);
        assert!(options.include_toc);
        assert!(options.use_color);
        assert_eq!(options.font_size, 12.0);
    }

    #[test]
    fn test_paper_size_a4() {
        let size = PaperSize::A4.to_mm();
        assert_eq!(size, (210.0, 297.0));
    }

    #[test]
    fn test_paper_size_letter() {
        let size = PaperSize::Letter.to_mm();
        assert_eq!(size, (215.9, 279.4));
    }

    #[test]
    fn test_pdf_exporter_creation() {
        let exporter = PdfExporter::new(PdfExportOptions::default());
        assert_eq!(exporter.options.font_size, 12.0);
    }

    #[test]
    fn test_inspection_data_creation() {
        let data = InspectionData {
            hostname: "test-vm".to_string(),
            os_type: "linux".to_string(),
            distribution: "ubuntu".to_string(),
            version: "22.04".to_string(),
            architecture: "x86_64".to_string(),
            product_name: "Ubuntu".to_string(),
            package_format: "deb".to_string(),
            package_manager: "apt".to_string(),
            kernel_version: Some("5.15.0".to_string()),
            total_memory: Some(8589934592),
            vcpus: Some(4),
            filesystems: vec![],
            packages: vec![],
            users: vec![],
            interfaces: vec![],
        };

        assert_eq!(data.hostname, "test-vm");
        assert_eq!(data.os_type, "linux");
    }
}
