// SPDX-License-Identifier: Apache-2.0
//! Cutover Passport — versioned, CI-gateable migration assurance artifact.
//!
//! GuestKit certifies cutover readiness; HyperSDK / hyper2kvm handle export
//! and convert/deploy. Red Hat virt-v2v/MTV remain convert-first; this artifact
//! is the gate they cannot skip.

use crate::assurance::copilot::{build_evidence_digest, EvidenceDigest};
use crate::boot::BootabilityReport;
use crate::cli::plan::FixPlan;
use crate::evidence::EvidenceSnapshot;
use crate::migration::{MigrationAssessment, MigrationRepairPlanner, RepairOptions, ReadinessLevel};
use crate::VERSION;
use anyhow::{bail, Context, Result};
use chrono::Utc;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::fs::File;
use std::io::{Read, Write};
use std::path::{Path, PathBuf};

/// Schema version for Cutover Passport JSON documents.
pub const PASSPORT_SCHEMA_VERSION: &str = "1";

/// Options for emitting a Cutover Passport.
#[derive(Debug, Clone, Default)]
pub struct PassportEmitOptions {
    pub verbose: bool,
    /// Include a truncated SHA-256 of the image file contents (slow on large disks).
    pub content_hash: bool,
    /// Optional host path for VirtIO driver tree (feeds repair planner notes).
    pub virtio_win_dir: Option<PathBuf>,
    /// Optional agent-proxy base URL (e.g. `http://127.0.0.1:8765`) for live attestation.
    pub live_url: Option<String>,
    /// Path to Ed25519 signing key (64-byte seed hex, or raw 32-byte seed file).
    /// Requires `--features agent`.
    pub sign_key: Option<PathBuf>,
}

/// Image identity fingerprint.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ImageFingerprint {
    pub path: String,
    pub size_bytes: u64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub content_sha256: Option<String>,
}

/// Condensed scores for CI gates.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PassportScores {
    pub boot: f64,
    pub migration: f64,
    pub readiness: ReadinessLevel,
}

/// Failed / blocking check summary (stable for auditors).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PassportFinding {
    pub id: String,
    pub title: String,
    pub message: String,
    pub severity: String,
}

/// Digest of the associated FixPlan (full plan may live in a companion file).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlanDigest {
    pub profile: String,
    pub operation_count: usize,
    pub sha256: String,
    pub operation_ids: Vec<String>,
}

/// Optional policy verdict snapshot.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PolicyVerdict {
    pub passed: bool,
    pub summary: String,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub failures: Vec<String>,
}

/// Windows offline readiness flags (Phase 2).
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct WindowsPassportFlags {
    pub is_windows: bool,
    /// Active BitLocker protection — hard blocker for cutover.
    pub bitlocker_blocker: bool,
    pub bitlocker_detected: bool,
    pub rdp_enabled: bool,
    pub virtio_driver_count: usize,
    /// True when offline day-0 path looks ready (no BitLocker blocker + VirtIO
    /// and/or RDP signals / plan ops).
    pub windows_offline_ready: bool,
    /// Multi-partition layout: separate System Reserved or ESP boot volume.
    #[serde(default)]
    pub system_reserved_layout: bool,
    /// BCD located (OS volume, System Reserved, or ESP).
    #[serde(default)]
    pub bcd_store_found: bool,
}

/// Optional live agent attestation (Phase 3).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LiveAttestation {
    pub source: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub readiness_score: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub detail: Option<serde_json::Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
}

/// Suite handoff — convert/deploy lives in sibling products.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SuiteHandoff {
    pub export: String,
    pub convert_deploy: String,
    pub assurance: String,
    pub next_step: String,
}

/// Cryptographic signature over the canonical unsigned payload.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PassportSignature {
    pub algorithm: String,
    pub public_key_hex: String,
    pub signature_hex: String,
}

/// Versioned Cutover Passport document.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CutoverPassport {
    pub schema_version: String,
    pub kind: String,
    pub generated_at: String,
    pub tool_version: String,
    pub target: String,
    pub image: ImageFingerprint,
    pub evidence_schema: String,
    pub evidence_digest: EvidenceDigest,
    pub scores: PassportScores,
    pub critical_blockers: Vec<PassportFinding>,
    pub recommended_actions: Vec<String>,
    pub fix_plan: PlanDigest,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub policy: Option<PolicyVerdict>,
    pub windows: WindowsPassportFlags,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub live_attestation: Option<LiveAttestation>,
    pub suite: SuiteHandoff,
    /// True when BitLocker or readiness blocks cutover regardless of score.
    pub hard_blocked: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub signature: Option<PassportSignature>,
}

/// Options for verifying a passport.
#[derive(Debug, Clone, Default)]
pub struct PassportVerifyOptions {
    pub fail_below: Option<f64>,
    /// Require a valid Ed25519 signature (needs `--features agent`).
    pub require_signature: bool,
    /// Optional public key hex to verify against (otherwise uses embedded pubkey).
    pub public_key: Option<String>,
}

impl Default for SuiteHandoff {
    fn default() -> Self {
        Self {
            export: "HyperSDK (hyperctl / hypervisord)".into(),
            convert_deploy: "hyper2kvm (h2kvmctl)".into(),
            assurance: "GuestKit Cutover Passport".into(),
            next_step: "After passport verify passes, convert/deploy with hyper2kvm \
                 (or continue HyperSDK job pipeline). Do not skip this gate for MTV/virt-v2v imports."
                .into(),
        }
    }
}

/// Build a Cutover Passport from an offline disk image.
pub fn emit_passport(
    image: &Path,
    target: &str,
    opts: &PassportEmitOptions,
) -> Result<(CutoverPassport, FixPlan)> {
    let (evidence, assessment) =
        crate::assurance::run_migrate_assess(image, target, opts.verbose)?;
    let boot = crate::assurance::run_doctor(image, target, false, opts.verbose)?;

    let (plan, _notes) = MigrationRepairPlanner::from_assessment(
        &assessment,
        &evidence,
        &RepairOptions {
            include_destructive: false,
            virtio_win_dir: opts.virtio_win_dir.clone(),
        },
    );

    let fingerprint = image_fingerprint(image, opts.content_hash)?;
    let windows = windows_flags(&evidence, &plan, &assessment);
    let hard_blocked = windows.bitlocker_blocker
        || matches!(assessment.readiness, ReadinessLevel::Blocked)
        || !boot.bootability.blockers.is_empty();

    let live_attestation = match &opts.live_url {
        Some(url) => Some(fetch_live_attestation(url)?),
        None => online_correlation_attestation(&assessment),
    };

    let mut passport = CutoverPassport {
        schema_version: PASSPORT_SCHEMA_VERSION.into(),
        kind: "guestkit.cutover_passport".into(),
        generated_at: Utc::now().to_rfc3339(),
        tool_version: VERSION.into(),
        target: target.into(),
        image: fingerprint,
        evidence_schema: evidence.schema_version.to_string(),
        evidence_digest: build_evidence_digest(&evidence),
        scores: PassportScores {
            boot: boot.bootability.score,
            migration: assessment.overall_score,
            readiness: assessment.readiness,
        },
        critical_blockers: collect_blockers(&boot.bootability, &assessment),
        recommended_actions: assessment.recommended_actions.clone(),
        fix_plan: plan_digest(&plan)?,
        policy: None,
        windows,
        live_attestation,
        suite: SuiteHandoff::default(),
        hard_blocked,
        signature: None,
    };

    if let Some(key_path) = &opts.sign_key {
        passport.signature = Some(sign_passport(&passport, key_path)?);
    }

    Ok((passport, plan))
}

/// Verify a passport document for CI gates.
pub fn verify_passport(passport: &CutoverPassport, opts: &PassportVerifyOptions) -> Result<()> {
    if passport.schema_version != PASSPORT_SCHEMA_VERSION {
        bail!(
            "unsupported passport schema_version {} (expected {})",
            passport.schema_version,
            PASSPORT_SCHEMA_VERSION
        );
    }

    if passport.hard_blocked {
        bail!(
            "passport hard-blocked (BitLocker and/or boot/migration blockers) — cutover refused"
        );
    }

    if passport.windows.bitlocker_blocker {
        bail!("BitLocker protection is active — suspend/escrow before cutover");
    }

    if let Some(threshold) = opts.fail_below {
        let score = passport.scores.boot.min(passport.scores.migration);
        if score < threshold {
            bail!(
                "passport score {:.0} (min of boot {:.0} / migration {:.0}) below --fail-below {threshold}",
                score,
                passport.scores.boot,
                passport.scores.migration
            );
        }
    }

    if opts.require_signature || passport.signature.is_some() {
        verify_signature(passport, opts.public_key.as_deref())?;
    }

    Ok(())
}

/// Write passport JSON (+ optional plan YAML) and optional tar.gz bundle.
pub fn write_passport_outputs(
    passport: &CutoverPassport,
    plan: &FixPlan,
    output: &Path,
    bundle: bool,
) -> Result<()> {
    if let Some(parent) = output.parent() {
        if !parent.as_os_str().is_empty() {
            std::fs::create_dir_all(parent)?;
        }
    }

    let json = serde_json::to_string_pretty(passport)?;
    std::fs::write(output, &json)
        .with_context(|| format!("write passport {}", output.display()))?;

    let plan_path = companion_plan_path(output);
    let plan_yaml = serde_yaml::to_string(plan)?;
    std::fs::write(&plan_path, plan_yaml)
        .with_context(|| format!("write fix plan {}", plan_path.display()))?;

    if bundle {
        let bundle_dir = output.with_extension("passport");
        write_tar_gz_bundle(&bundle_dir, output, &plan_path)?;
    }
    Ok(())
}

fn companion_plan_path(passport_path: &Path) -> PathBuf {
    let stem = passport_path
        .file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or("passport");
    passport_path
        .parent()
        .unwrap_or_else(|| Path::new("."))
        .join(format!("{stem}.fix-plan.yaml"))
}

/// Bundle as a directory containing passport JSON + companion FixPlan YAML.
fn write_tar_gz_bundle(bundle_path: &Path, passport: &Path, plan: &Path) -> Result<()> {
    // Prefer a directory bundle (no optional flate2/tar): `<stem>.passport/`
    let dir = if bundle_path.extension().and_then(|e| e.to_str()) == Some("gz") {
        bundle_path.with_extension("").with_extension("passport")
    } else {
        bundle_path.to_path_buf()
    };
    std::fs::create_dir_all(&dir)?;
    let dest_passport = dir.join(passport.file_name().unwrap());
    let dest_plan = dir.join(plan.file_name().unwrap());
    std::fs::copy(passport, &dest_passport)?;
    std::fs::copy(plan, &dest_plan)?;
    Ok(())
}

fn image_fingerprint(image: &Path, content_hash: bool) -> Result<ImageFingerprint> {
    let meta = std::fs::metadata(image)
        .with_context(|| format!("stat image {}", image.display()))?;
    let content_sha256 = if content_hash {
        Some(hash_file_sha256(image)?)
    } else {
        None
    };
    Ok(ImageFingerprint {
        path: image.display().to_string(),
        size_bytes: meta.len(),
        content_sha256,
    })
}

fn hash_file_sha256(path: &Path) -> Result<String> {
    let mut file = File::open(path)?;
    let mut hasher = Sha256::new();
    let mut buf = [0u8; 1024 * 64];
    loop {
        let n = file.read(&mut buf)?;
        if n == 0 {
            break;
        }
        hasher.update(&buf[..n]);
    }
    Ok(format!("{:x}", hasher.finalize()))
}

fn plan_digest(plan: &FixPlan) -> Result<PlanDigest> {
    let bytes = serde_json::to_vec(plan)?;
    let sha = format!("{:x}", Sha256::digest(&bytes));
    Ok(PlanDigest {
        profile: plan.profile.clone(),
        operation_count: plan.operations.len(),
        sha256: sha,
        operation_ids: plan.operations.iter().map(|o| o.id.clone()).collect(),
    })
}

fn collect_blockers(
    boot: &BootabilityReport,
    assessment: &MigrationAssessment,
) -> Vec<PassportFinding> {
    let mut out = Vec::new();
    for b in &boot.blockers {
        out.push(PassportFinding {
            id: b.check_id.clone(),
            title: b.title.clone(),
            message: b.message.clone(),
            severity: "blocker".into(),
        });
    }
    for b in &assessment.critical_blockers {
        if out.iter().any(|f| f.id == b.check_id) {
            continue;
        }
        out.push(PassportFinding {
            id: b.check_id.clone(),
            title: b.title.clone(),
            message: b.message.clone(),
            severity: "blocker".into(),
        });
    }
    out
}

fn windows_flags(
    evidence: &EvidenceSnapshot,
    plan: &FixPlan,
    assessment: &MigrationAssessment,
) -> WindowsPassportFlags {
    let is_windows = evidence.os.os_type.eq_ignore_ascii_case("windows")
        || evidence.windows.is_some();

    let Some(win) = evidence.windows.as_ref() else {
        return WindowsPassportFlags {
            is_windows,
            ..Default::default()
        };
    };

    let bitlocker_blocker = win
        .bitlocker
        .as_ref()
        .map(|b| b.any_protected)
        .unwrap_or(false)
        || assessment.critical_blockers.iter().any(|b| b.check_id == "MIG-W-005");

    let virtio_driver_count = win.virtio_drivers.len();
    let plan_has_driver_inject = plan
        .operations
        .iter()
        .any(|o| matches!(o.op_type, crate::cli::plan::OperationType::DriverInject(_)));
    let plan_has_rdp_or_winrm = plan.operations.iter().any(|o| {
        o.id.contains("rdp")
            || o.id.contains("winrm")
            || o.description.to_lowercase().contains("remote desktop")
            || o.description.to_lowercase().contains("winrm")
    });

    let windows_offline_ready = !bitlocker_blocker
        && (virtio_driver_count > 0 || plan_has_driver_inject)
        && (win.rdp_enabled || plan_has_rdp_or_winrm);

    WindowsPassportFlags {
        is_windows: true,
        bitlocker_blocker,
        bitlocker_detected: win.bitlocker_detected
            || win.bitlocker.as_ref().map(|b| b.any_protected).unwrap_or(false),
        rdp_enabled: win.rdp_enabled,
        virtio_driver_count,
        windows_offline_ready,
        system_reserved_layout: win.system_reserved.is_some(),
        bcd_store_found: win.bcd_store_found,
    }
}

fn online_correlation_attestation(
    assessment: &MigrationAssessment,
) -> Option<LiveAttestation> {
    let oc = assessment.online_correlation.as_ref()?;
    let readiness_score = oc
        .pointer("/payload/heartbeat/readiness_score")
        .and_then(|v| v.as_f64())
        .or_else(|| oc.pointer("/payload/doctor/score").and_then(|v| v.as_f64()));
    Some(LiveAttestation {
        source: "guest_on_disk_cache".into(),
        readiness_score,
        detail: Some(oc.clone()),
        error: None,
    })
}

fn fetch_live_attestation(base_url: &str) -> Result<LiveAttestation> {
    let url = format!(
        "{}/doctor",
        base_url.trim_end_matches('/')
    );
    match http_get_json(&url) {
        Ok(detail) => {
            let readiness_score = detail
                .get("score")
                .and_then(|v| v.as_f64())
                .or_else(|| detail.get("readiness_score").and_then(|v| v.as_f64()))
                .or_else(|| {
                    detail
                        .pointer("/bootability/score")
                        .and_then(|v| v.as_f64())
                });
            Ok(LiveAttestation {
                source: url,
                readiness_score,
                detail: Some(detail),
                error: None,
            })
        }
        Err(e) => Ok(LiveAttestation {
            source: url,
            readiness_score: None,
            detail: None,
            error: Some(e.to_string()),
        }),
    }
}

/// Minimal HTTP/1.1 GET for agent-proxy (no reqwest / url crate required).
fn http_get_json(url: &str) -> Result<serde_json::Value> {
    let rest = url
        .strip_prefix("http://")
        .context("live URL must start with http://")?;
    let (authority, path_and_query) = match rest.split_once('/') {
        Some((a, p)) => (a, format!("/{p}")),
        None => (rest, "/".to_string()),
    };
    let (host, port) = match authority.rsplit_once(':') {
        Some((h, p)) => (h, p.parse::<u16>().context("parse live URL port")?),
        None => (authority, 80),
    };

    let mut stream = std::net::TcpStream::connect((host, port))
        .with_context(|| format!("connect {host}:{port}"))?;
    stream.set_read_timeout(Some(std::time::Duration::from_secs(10)))?;
    stream.set_write_timeout(Some(std::time::Duration::from_secs(10)))?;

    let req = format!(
        "GET {path_and_query} HTTP/1.1\r\nHost: {authority}\r\nConnection: close\r\nAccept: application/json\r\n\r\n"
    );
    stream.write_all(req.as_bytes())?;

    let mut resp = String::new();
    stream.read_to_string(&mut resp)?;
    let body = resp
        .split("\r\n\r\n")
        .nth(1)
        .context("HTTP response missing body")?;
    serde_json::from_str(body).context("parse live doctor JSON")
}

#[cfg(feature = "agent")]
fn signing_payload(passport: &CutoverPassport) -> Result<Vec<u8>> {
    let mut unsigned = passport.clone();
    unsigned.signature = None;
    Ok(serde_json::to_vec(&unsigned)?)
}

#[cfg(feature = "agent")]
fn sign_passport(passport: &CutoverPassport, key_path: &Path) -> Result<PassportSignature> {
    use ed25519_dalek::{Signer, SigningKey};

    let seed = load_ed25519_seed(key_path)?;
    let signing_key = SigningKey::from_bytes(&seed);
    let verifying_key = signing_key.verifying_key();
    let payload = signing_payload(passport)?;
    let sig = signing_key.sign(&payload);

    Ok(PassportSignature {
        algorithm: "ed25519".into(),
        public_key_hex: hex::encode(verifying_key.as_bytes()),
        signature_hex: hex::encode(sig.to_bytes()),
    })
}

#[cfg(not(feature = "agent"))]
fn sign_passport(_passport: &CutoverPassport, _key_path: &Path) -> Result<PassportSignature> {
    bail!("passport signing requires guestkit built with --features agent (Ed25519)")
}

#[cfg(feature = "agent")]
fn verify_signature(passport: &CutoverPassport, public_key: Option<&str>) -> Result<()> {
    use ed25519_dalek::{Signature, Verifier, VerifyingKey};

    let sig = passport
        .signature
        .as_ref()
        .context("passport has no signature")?;
    if sig.algorithm != "ed25519" {
        bail!("unsupported signature algorithm {}", sig.algorithm);
    }
    let pk_hex = public_key.unwrap_or(&sig.public_key_hex);
    let pk_bytes = hex::decode(pk_hex).context("decode public key hex")?;
    let pk_arr: [u8; 32] = pk_bytes
        .as_slice()
        .try_into()
        .map_err(|_| anyhow::anyhow!("public key must be 32 bytes"))?;
    let verifying_key = VerifyingKey::from_bytes(&pk_arr)
        .map_err(|e| anyhow::anyhow!("invalid public key: {e}"))?;

    let sig_bytes = hex::decode(&sig.signature_hex).context("decode signature hex")?;
    let sig_arr: [u8; 64] = sig_bytes
        .as_slice()
        .try_into()
        .map_err(|_| anyhow::anyhow!("signature must be 64 bytes"))?;
    let signature = Signature::from_bytes(&sig_arr);

    let payload = signing_payload(passport)?;
    verifying_key
        .verify(&payload, &signature)
        .map_err(|e| anyhow::anyhow!("passport signature verification failed: {e}"))?;
    Ok(())
}

#[cfg(not(feature = "agent"))]
fn verify_signature(passport: &CutoverPassport, _public_key: Option<&str>) -> Result<()> {
    if passport.signature.is_some() {
        bail!("passport has a signature but this build lacks --features agent to verify Ed25519");
    }
    Ok(())
}

#[cfg(feature = "agent")]
fn load_ed25519_seed(path: &Path) -> Result<[u8; 32]> {
    let raw = std::fs::read(path).with_context(|| format!("read sign key {}", path.display()))?;
    if raw.len() == 32 {
        let mut seed = [0u8; 32];
        seed.copy_from_slice(&raw);
        return Ok(seed);
    }
    let text = String::from_utf8_lossy(&raw);
    let hex_str = text.trim();
    let decoded = hex::decode(hex_str).context("sign key must be 32 raw bytes or 64 hex chars")?;
    let seed: [u8; 32] = decoded
        .as_slice()
        .try_into()
        .map_err(|_| anyhow::anyhow!("sign key hex must decode to 32 bytes"))?;
    Ok(seed)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cli::migrate::plan::MigrationScoreReport;
    use crate::cli::plan::types::{FixPlan, Operation, OperationType, Priority, RegistryEdit};
    use crate::evidence::snapshot::{BitLockerState, OsEvidence, WindowsEvidence};
    use crate::migration::score::{MigrationAssessment, MigrationSubScores, ReadinessLevel};

    fn empty_assessment() -> MigrationAssessment {
        MigrationAssessment {
            target: "kvm".into(),
            live: false,
            assessed_at: Utc::now().to_rfc3339(),
            overall_score: 90.0,
            sub_scores: MigrationSubScores::default(),
            readiness: ReadinessLevel::Ready,
            critical_blockers: vec![],
            recommended_actions: vec![],
            checks: vec![],
            online_correlation: None,
            legacy: MigrationScoreReport {
                score: 90.0,
                driver_injections: vec![],
                required_changes: vec![],
                licensing_warnings: vec![],
                estimated_downtime_minutes: 0,
            },
        }
    }

    #[test]
    fn windows_bitlocker_sets_blocker_and_not_ready() {
        let mut win = WindowsEvidence {
            bitlocker_detected: true,
            rdp_enabled: true,
            bitlocker: Some(BitLockerState {
                any_protected: true,
                ..Default::default()
            }),
            ..Default::default()
        };
        win.virtio_drivers.clear();
        let ev = crate::evidence::snapshot::EvidenceSnapshot {
            schema_version: 5,
            image_path: "win.qcow2".into(),
            collected_at: Utc::now().to_rfc3339(),
            root: "/".into(),
            os: OsEvidence {
                os_type: "windows".into(),
                ..Default::default()
            },
            storage: Default::default(),
            boot: Default::default(),
            network: Default::default(),
            packages: Default::default(),
            security: Default::default(),
            vm_tools: Default::default(),
            systemd: None,
            windows: Some(win),
            kubevirt: None,
            cloud_init: None,
            network_probes: None,
            snapshot_readiness: None,
            process: None,
            hardware: None,
            linux_migration: None,
            online_cache: None,
        };
        let plan = FixPlan::new("win.qcow2".into(), "migration-repair".into());
        let assessment = empty_assessment();
        let flags = windows_flags(&ev, &plan, &assessment);
        assert!(flags.bitlocker_blocker);
        assert!(!flags.windows_offline_ready);
    }

    #[test]
    fn plan_digest_is_stable() {
        let mut plan = FixPlan::new("vm.qcow2".into(), "migration-repair".into());
        plan.add_operation(Operation {
            id: "op-1".into(),
            op_type: OperationType::RegistryEdit(RegistryEdit {
                key: r"HKLM\SYSTEM".into(),
                value: "Test".into(),
                data_type: "dword".into(),
                current_data: serde_json::json!(0),
                new_data: serde_json::json!(1),
            }),
            priority: Priority::High,
            description: "test".into(),
            risk: Priority::Low,
            reversible: true,
            depends_on: vec![],
            validation: None,
            undo: None,
        });
        let d1 = plan_digest(&plan).unwrap();
        let d2 = plan_digest(&plan).unwrap();
        assert_eq!(d1.sha256, d2.sha256);
        assert_eq!(d1.operation_count, 1);
    }

    #[test]
    fn verify_fails_when_hard_blocked() {
        let passport = CutoverPassport {
            schema_version: PASSPORT_SCHEMA_VERSION.into(),
            kind: "guestkit.cutover_passport".into(),
            generated_at: Utc::now().to_rfc3339(),
            tool_version: "0.0.0".into(),
            target: "kvm".into(),
            image: ImageFingerprint {
                path: "x".into(),
                size_bytes: 1,
                content_sha256: None,
            },
            evidence_schema: "1".into(),
            evidence_digest: EvidenceDigest {
                os: "linux".into(),
                architecture: "x86_64".into(),
                bootloader: "grub".into(),
                root_filesystem: "ext4".into(),
                kernel_count: 1,
                fstab_entries: 1,
                virtio_modules_loaded: true,
                vm_tools: vec![],
                selinux: "disabled".into(),
            },
            scores: PassportScores {
                boot: 95.0,
                migration: 95.0,
                readiness: ReadinessLevel::Ready,
            },
            critical_blockers: vec![],
            recommended_actions: vec![],
            fix_plan: PlanDigest {
                profile: "migration-repair".into(),
                operation_count: 0,
                sha256: "abc".into(),
                operation_ids: vec![],
            },
            policy: None,
            windows: WindowsPassportFlags::default(),
            live_attestation: None,
            suite: SuiteHandoff::default(),
            hard_blocked: true,
            signature: None,
        };
        let err = verify_passport(&passport, &PassportVerifyOptions::default()).unwrap_err();
        assert!(err.to_string().contains("hard-blocked"));
    }

    #[test]
    fn verify_fail_below_uses_min_score() {
        let mut passport = CutoverPassport {
            schema_version: PASSPORT_SCHEMA_VERSION.into(),
            kind: "guestkit.cutover_passport".into(),
            generated_at: Utc::now().to_rfc3339(),
            tool_version: "0.0.0".into(),
            target: "kvm".into(),
            image: ImageFingerprint {
                path: "x".into(),
                size_bytes: 1,
                content_sha256: None,
            },
            evidence_schema: "1".into(),
            evidence_digest: EvidenceDigest {
                os: "linux".into(),
                architecture: "x86_64".into(),
                bootloader: "grub".into(),
                root_filesystem: "ext4".into(),
                kernel_count: 1,
                fstab_entries: 1,
                virtio_modules_loaded: true,
                vm_tools: vec![],
                selinux: "disabled".into(),
            },
            scores: PassportScores {
                boot: 90.0,
                migration: 70.0,
                readiness: ReadinessLevel::ReadyWithWarnings,
            },
            critical_blockers: vec![],
            recommended_actions: vec![],
            fix_plan: PlanDigest {
                profile: "migration-repair".into(),
                operation_count: 0,
                sha256: "abc".into(),
                operation_ids: vec![],
            },
            policy: None,
            windows: WindowsPassportFlags::default(),
            live_attestation: None,
            suite: SuiteHandoff::default(),
            hard_blocked: false,
            signature: None,
        };
        let opts = PassportVerifyOptions {
            fail_below: Some(80.0),
            ..Default::default()
        };
        assert!(verify_passport(&passport, &opts).is_err());
        passport.scores.migration = 85.0;
        assert!(verify_passport(&passport, &opts).is_ok());
    }

    #[test]
    fn suite_handoff_names_hypersdk_and_hyper2kvm() {
        let h = SuiteHandoff::default();
        assert!(h.export.contains("HyperSDK"));
        assert!(h.convert_deploy.contains("hyper2kvm"));
        assert!(h.assurance.contains("Passport"));
    }
}
