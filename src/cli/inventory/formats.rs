// SPDX-License-Identifier: LGPL-3.0-or-later
//! SBOM format converters (SPDX, CycloneDX, CSV)

use super::Inventory;
use anyhow::Result;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// SPDX 2.3 Document
#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SpdxDocument {
    pub spdx_version: String,
    pub data_license: String,
    #[serde(rename = "SPDXID")]
    pub spdxid: String,
    pub name: String,
    pub document_namespace: String,
    pub creation_info: SpdxCreationInfo,
    pub packages: Vec<SpdxPackage>,
    pub relationships: Vec<SpdxRelationship>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SpdxCreationInfo {
    pub created: String,
    pub creators: Vec<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub license_list_version: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SpdxPackage {
    #[serde(rename = "SPDXID")]
    pub spdxid: String,
    pub name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub version_info: Option<String>,
    pub download_location: String,
    pub files_analyzed: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub license_concluded: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub license_declared: Option<String>,
    pub copyright_text: String,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SpdxRelationship {
    pub spdx_element_id: String,
    pub relationship_type: String,
    pub related_spdx_element: String,
}

/// CycloneDX 1.5 BOM
#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CycloneDxBom {
    pub bom_format: String,
    pub spec_version: String,
    pub serial_number: String,
    pub version: u32,
    pub metadata: CdxMetadata,
    pub components: Vec<CdxComponent>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub vulnerabilities: Vec<CdxVulnerability>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CdxMetadata {
    pub timestamp: String,
    pub tools: Vec<CdxTool>,
    pub component: CdxRootComponent,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CdxTool {
    pub vendor: String,
    pub name: String,
    pub version: String,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CdxRootComponent {
    #[serde(rename = "type")]
    pub component_type: String,
    pub name: String,
    pub version: String,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CdxComponent {
    #[serde(rename = "type")]
    pub component_type: String,
    pub bom_ref: String,
    pub name: String,
    pub version: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub purl: Option<String>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub licenses: Vec<CdxLicense>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CdxLicense {
    pub license: CdxLicenseChoice,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CdxLicenseChoice {
    pub id: String,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CdxVulnerability {
    pub id: String,
    pub source: CdxSource,
    pub ratings: Vec<CdxRating>,
    pub affects: Vec<CdxAffect>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CdxSource {
    pub name: String,
    pub url: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CdxRating {
    pub severity: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub score: Option<f64>,
    pub method: String,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CdxAffect {
    #[serde(rename = "ref")]
    pub component_ref: String,
}

/// Convert inventory to SPDX format
pub fn to_spdx(inventory: &Inventory) -> Result<SpdxDocument> {
    let doc_id = format!("SPDXRef-DOCUMENT");
    let namespace = format!(
        "https://guestkit.dev/sbom/{}/{}",
        inventory.image_path.replace('/', "-"),
        chrono::Utc::now().format("%Y%m%d-%H%M%S")
    );

    let mut packages = Vec::new();
    let mut relationships = Vec::new();

    for (idx, pkg) in inventory.packages.iter().enumerate() {
        let pkg_id = format!("SPDXRef-Package-{}", idx);

        packages.push(SpdxPackage {
            spdxid: pkg_id.clone(),
            name: pkg.name.clone(),
            version_info: Some(pkg.version.clone()),
            download_location: "NOASSERTION".to_string(),
            files_analyzed: false,
            license_concluded: pkg.license.clone(),
            license_declared: pkg.license.clone(),
            copyright_text: "NOASSERTION".to_string(),
        });

        relationships.push(SpdxRelationship {
            spdx_element_id: doc_id.clone(),
            relationship_type: "DESCRIBES".to_string(),
            related_spdx_element: pkg_id,
        });
    }

    Ok(SpdxDocument {
        spdx_version: "SPDX-2.3".to_string(),
        data_license: "CC0-1.0".to_string(),
        spdxid: doc_id,
        name: inventory.image_path.clone(),
        document_namespace: namespace,
        creation_info: SpdxCreationInfo {
            created: inventory.scanned_at.clone(),
            creators: vec![format!("Tool: guestkit-{}", env!("CARGO_PKG_VERSION"))],
            license_list_version: Some("3.21".to_string()),
        },
        packages,
        relationships,
    })
}

/// Convert inventory to CycloneDX format
pub fn to_cyclonedx(inventory: &Inventory) -> Result<CycloneDxBom> {
    let serial_number = format!("urn:uuid:{}", Uuid::new_v4());

    let mut components = Vec::new();
    let mut vulnerabilities = Vec::new();

    for pkg in &inventory.packages {
        let bom_ref = format!(
            "pkg:{}/{}/{}@{}",
            pkg.package_type,
            inventory.os_name.to_lowercase(),
            pkg.name,
            pkg.version
        );

        let licenses = if let Some(license) = &pkg.license {
            vec![CdxLicense {
                license: CdxLicenseChoice {
                    id: license.clone(),
                },
            }]
        } else {
            Vec::new()
        };

        components.push(CdxComponent {
            component_type: "library".to_string(),
            bom_ref: bom_ref.clone(),
            name: pkg.name.clone(),
            version: pkg.version.clone(),
            purl: Some(bom_ref.clone()),
            licenses,
        });

        // Add vulnerabilities
        for vuln in &pkg.vulnerabilities {
            vulnerabilities.push(CdxVulnerability {
                id: vuln.cve.clone(),
                source: CdxSource {
                    name: "NVD".to_string(),
                    url: format!("https://nvd.nist.gov/vuln/detail/{}", vuln.cve),
                },
                ratings: vec![CdxRating {
                    severity: vuln.severity.clone(),
                    score: vuln.score,
                    method: "CVSSv3".to_string(),
                }],
                affects: vec![CdxAffect {
                    component_ref: bom_ref.clone(),
                }],
            });
        }
    }

    Ok(CycloneDxBom {
        bom_format: "CycloneDX".to_string(),
        spec_version: "1.5".to_string(),
        serial_number,
        version: 1,
        metadata: CdxMetadata {
            timestamp: inventory.scanned_at.clone(),
            tools: vec![CdxTool {
                vendor: "guestkit".to_string(),
                name: "guestkit".to_string(),
                version: env!("CARGO_PKG_VERSION").to_string(),
            }],
            component: CdxRootComponent {
                component_type: "application".to_string(),
                name: inventory.image_path.clone(),
                version: "1.0.0".to_string(),
            },
        },
        components,
        vulnerabilities,
    })
}

/// Convert inventory to CSV format
pub fn to_csv(inventory: &Inventory) -> Result<String> {
    let mut csv = String::new();

    // Header
    csv.push_str("Package,Version,Type,License,Size,CVEs,Max Severity\n");

    // Data rows
    for pkg in &inventory.packages {
        let size_str = pkg.size
            .map(|s| format_size(s))
            .unwrap_or_else(|| "N/A".to_string());

        let cve_count = pkg.vulnerabilities.len();
        let max_severity = pkg.vulnerabilities
            .iter()
            .map(|v| v.severity.as_str())
            .max_by_key(|s| severity_rank(s))
            .unwrap_or("none");

        csv.push_str(&format!(
            "\"{}\",\"{}\",\"{}\",\"{}\",\"{}\",{},\"{}\"\n",
            pkg.name,
            pkg.version,
            pkg.package_type,
            pkg.license.as_deref().unwrap_or("Unknown"),
            size_str,
            cve_count,
            max_severity
        ));
    }

    Ok(csv)
}

fn format_size(bytes: i64) -> String {
    const KB: i64 = 1024;
    const MB: i64 = KB * 1024;
    const GB: i64 = MB * 1024;

    if bytes >= GB {
        format!("{:.2} GB", bytes as f64 / GB as f64)
    } else if bytes >= MB {
        format!("{:.2} MB", bytes as f64 / MB as f64)
    } else if bytes >= KB {
        format!("{:.2} KB", bytes as f64 / KB as f64)
    } else {
        format!("{} B", bytes)
    }
}

fn severity_rank(severity: &str) -> u8 {
    match severity.to_lowercase().as_str() {
        "critical" => 4,
        "high" => 3,
        "medium" => 2,
        "low" => 1,
        _ => 0,
    }
}
