//! `smctl quality audit` — RUSTSEC advisory check via `cargo audit --json`.
//!
//! Wraps the `cargo-audit` binary and converts its JSON output into a
//! crate-local [`AuditReport`] struct. Emits MSGID-tagged tracing events at
//! start, per-advisory, and on completion/failure.

use std::path::Path;
use std::process::Command;
use std::time::Instant;

use serde::{Deserialize, Serialize};
use smctl_log::MsgId;
use thiserror::Error;

/// Severity levels reported by cargo-audit / RUSTSEC.
///
/// Matches the severity vocabulary the RUSTSEC database uses. Case-insensitive
/// on input; serialises in lowercase for stable JSON.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum AdvisorySeverity {
    /// Below the usual publication threshold — informational only.
    None,
    /// Low-risk advisory — unlikely to be exploitable in practice.
    Low,
    /// Published advisory without a CVSS score — treat as at-least warning.
    Warning,
    /// Medium-severity advisory — patchable vulnerability.
    Medium,
    /// High-severity advisory — patch promptly.
    High,
    /// Critical advisory — patch immediately.
    Critical,
}

impl AdvisorySeverity {
    /// Parse a RUSTSEC severity string. Unknown values downgrade to
    /// `Warning` so that an unrecognised label never silently passes a fail
    /// threshold.
    fn parse(raw: &str) -> Self {
        match raw.to_ascii_lowercase().as_str() {
            "critical" => Self::Critical,
            "high" => Self::High,
            "medium" | "moderate" => Self::Medium,
            "low" => Self::Low,
            "none" => Self::None,
            _ => Self::Warning,
        }
    }
}

/// A single RUSTSEC advisory in the resulting report.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct Advisory {
    /// The crate name the advisory applies to.
    pub crate_name: String,
    /// RUSTSEC / CVE identifier (e.g. `RUSTSEC-2024-0001`).
    pub advisory_id: String,
    /// Short human summary drawn from the advisory metadata.
    pub title: String,
    /// RUSTSEC-reported severity. See [`AdvisorySeverity`].
    pub severity: AdvisorySeverity,
    /// Version of the crate currently in the lockfile.
    pub installed: String,
    /// Versions the advisory lists as fixed, if any.
    pub patched: Vec<String>,
}

/// Structured result of a `smctl quality audit` run.
///
/// Serialises to stable JSON. Consumers should treat the shape as a
/// public contract.
#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq, Eq)]
pub struct AuditReport {
    /// Root of the workspace (or crate) that was audited.
    pub root: String,
    /// Elapsed wall time of the audit run in milliseconds.
    pub duration_ms: u128,
    /// Number of advisories surfaced by cargo-audit. Same as `advisories.len()`.
    pub advisory_count: usize,
    /// The advisories themselves, one entry per affected crate.
    pub advisories: Vec<Advisory>,
    /// True when the audit found at least one advisory at or above the
    /// configured fail threshold. The caller decides exit status from this.
    pub fail: bool,
}

/// Errors raised by [`run_audit`].
///
/// Every variant carries a three-part message (what happened / what it means
/// / what to do next) so that CLI callers can surface it directly without
/// rewrapping.
#[derive(Debug, Error)]
pub enum AuditError {
    #[error(
        "cargo-audit is not installed on PATH. smctl quality audit cannot run without it. Install it with: cargo install cargo-audit"
    )]
    ToolMissing,

    #[error(
        "cargo-audit exited abnormally ({status}). The audit result is unreliable. Re-run `cargo audit --json` directly to inspect the failure: {stderr}"
    )]
    ToolFailed { status: String, stderr: String },

    #[error(
        "cargo-audit produced JSON that does not match the expected schema. The installed cargo-audit may be a different major version than this build targets. Upgrade it with: cargo install --force cargo-audit. Parser error: {source}"
    )]
    MalformedJson {
        #[source]
        source: serde_json::Error,
    },
}

/// Invoke `cargo audit --json` in `root`, parse the output, emit MSGIDs.
///
/// This is the primary public entry point of the crate. The returned report
/// is independent of CLI formatting concerns — the caller renders it human-
/// readable or as JSON.
pub fn run_audit(root: &Path) -> Result<AuditReport, AuditError> {
    let root_str = root.display().to_string();
    let started = Instant::now();

    tracing::info!(
        msgid = %MsgId::QualityCheckStarted,
        verb = "audit",
        repo = %root_str,
        scope = "workspace",
        "audit started"
    );

    let raw = invoke_cargo_audit(root)?;
    let parsed: CargoAuditOutput =
        serde_json::from_slice(&raw).map_err(|source| AuditError::MalformedJson { source })?;

    let advisories = parsed.into_advisories();
    for advisory in &advisories {
        tracing::error!(
            msgid = %MsgId::DependencyVulnerability,
            crate_name = %advisory.crate_name,
            advisory_id = %advisory.advisory_id,
            severity = ?advisory.severity,
            installed = %advisory.installed,
            patched = %advisory.patched.join(","),
            "dependency vulnerability found"
        );
    }

    let duration_ms = started.elapsed().as_millis();
    let advisory_count = advisories.len();

    Ok(AuditReport {
        root: root_str,
        duration_ms,
        advisory_count,
        advisories,
        fail: false,
    })
}

/// Decide whether the report should fail the check, given a minimum
/// severity. Any advisory at or above `threshold` flips `fail` to true.
///
/// Emits the terminal completion MSGID (either `QualityCheckCompleted` or
/// `QualityCheckFailed`). The caller should invoke this exactly once per
/// run, after [`run_audit`] returns.
pub fn finalise_report(mut report: AuditReport, threshold: AdvisorySeverity) -> AuditReport {
    let fail = report.advisories.iter().any(|a| a.severity >= threshold);
    report.fail = fail;

    if fail {
        let remediation = report
            .advisories
            .iter()
            .map(|a| format!("cargo update -p {}", a.crate_name))
            .collect::<Vec<_>>()
            .join("; ");
        tracing::error!(
            msgid = %MsgId::QualityCheckFailed,
            verb = "audit",
            duration_ms = report.duration_ms as u64,
            violation_count = report.advisory_count,
            remediation = %remediation,
            "audit failed"
        );
    } else {
        tracing::info!(
            msgid = %MsgId::QualityCheckCompleted,
            verb = "audit",
            duration_ms = report.duration_ms as u64,
            violation_count = 0u64,
            "audit completed"
        );
    }

    report
}

/// Shell out to `cargo audit --json`. Returns the raw stdout bytes on
/// success; maps ENOENT to [`AuditError::ToolMissing`] so the CLI surface
/// can render the three-part install message.
fn invoke_cargo_audit(root: &Path) -> Result<Vec<u8>, AuditError> {
    let output = Command::new("cargo")
        .arg("audit")
        .arg("--json")
        .current_dir(root)
        .output()
        .map_err(|e| {
            if e.kind() == std::io::ErrorKind::NotFound {
                AuditError::ToolMissing
            } else {
                AuditError::ToolFailed {
                    status: e.kind().to_string(),
                    stderr: e.to_string(),
                }
            }
        })?;

    // cargo-audit returns non-zero on advisories, but still emits JSON to
    // stdout. Treat empty stdout + failure as tool-missing (the shim
    // `cargo audit` produces "no such subcommand" via cargo itself).
    if output.stdout.is_empty() {
        let stderr = String::from_utf8_lossy(&output.stderr).to_string();
        if !output.status.success()
            && (stderr.contains("no such command")
                || stderr.contains("no such subcommand")
                || stderr.contains("is not installed"))
        {
            return Err(AuditError::ToolMissing);
        }
        return Err(AuditError::ToolFailed {
            status: output
                .status
                .code()
                .map_or_else(|| "signal".to_string(), |c| c.to_string()),
            stderr,
        });
    }

    Ok(output.stdout)
}

/// Detect whether cargo-audit is available on PATH.
///
/// Used by the CLI surface to skip gracefully in environments without the
/// tool installed, surfacing the three-part remediation message instead of
/// attempting an audit that will fail.
pub fn cargo_audit_available() -> bool {
    // `cargo audit --version` is cheap and exits non-zero if the subcommand
    // is missing. Any I/O error means cargo itself is missing; report as
    // unavailable rather than panicking.
    match Command::new("cargo").arg("audit").arg("--version").output() {
        Ok(out) => out.status.success(),
        Err(_) => false,
    }
}

// ── cargo-audit JSON shape ────────────────────────────────────────────
//
// This is the subset of the cargo-audit `--json` output we depend on. The
// full schema carries more fields; we deserialise only what we need so that
// forward-compatible additions do not break parsing.

#[derive(Debug, Deserialize)]
struct CargoAuditOutput {
    #[serde(default)]
    vulnerabilities: CargoAuditVulnerabilities,
}

#[derive(Debug, Default, Deserialize)]
struct CargoAuditVulnerabilities {
    #[serde(default)]
    list: Vec<CargoAuditVulnerability>,
}

#[derive(Debug, Deserialize)]
struct CargoAuditVulnerability {
    advisory: CargoAuditAdvisory,
    versions: CargoAuditVersions,
    package: CargoAuditPackage,
}

#[derive(Debug, Deserialize)]
struct CargoAuditAdvisory {
    id: String,
    title: String,
    #[serde(default)]
    severity: Option<String>,
}

#[derive(Debug, Deserialize)]
struct CargoAuditVersions {
    #[serde(default)]
    patched: Vec<String>,
}

#[derive(Debug, Deserialize)]
struct CargoAuditPackage {
    name: String,
    version: String,
}

impl CargoAuditOutput {
    fn into_advisories(self) -> Vec<Advisory> {
        self.vulnerabilities
            .list
            .into_iter()
            .map(|v| Advisory {
                crate_name: v.package.name,
                advisory_id: v.advisory.id,
                title: v.advisory.title,
                severity: v
                    .advisory
                    .severity
                    .as_deref()
                    .map_or(AdvisorySeverity::Warning, AdvisorySeverity::parse),
                installed: v.package.version,
                patched: v.versions.patched,
            })
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Empty cargo-audit output (no advisories) deserialises cleanly and
    /// produces a zero-advisory report.
    #[test]
    fn empty_cargo_audit_output_parses() {
        let json = r#"{
            "database": {"advisory_count": 0},
            "vulnerabilities": {
                "found": false,
                "count": 0,
                "list": []
            }
        }"#;

        let parsed: CargoAuditOutput = serde_json::from_str(json).unwrap();
        let advisories = parsed.into_advisories();
        assert_eq!(advisories.len(), 0);
    }

    /// A single advisory parses into a fully populated `Advisory`.
    #[test]
    fn single_advisory_parses() {
        let json = r#"{
            "vulnerabilities": {
                "found": true,
                "count": 1,
                "list": [{
                    "advisory": {
                        "id": "RUSTSEC-2024-0001",
                        "title": "example advisory",
                        "severity": "high"
                    },
                    "versions": {
                        "patched": [">=1.2.3"],
                        "unaffected": []
                    },
                    "package": {
                        "name": "example-crate",
                        "version": "1.2.2"
                    }
                }]
            }
        }"#;

        let parsed: CargoAuditOutput = serde_json::from_str(json).unwrap();
        let advisories = parsed.into_advisories();
        assert_eq!(advisories.len(), 1);

        let a = &advisories[0];
        assert_eq!(a.crate_name, "example-crate");
        assert_eq!(a.advisory_id, "RUSTSEC-2024-0001");
        assert_eq!(a.severity, AdvisorySeverity::High);
        assert_eq!(a.installed, "1.2.2");
        assert_eq!(a.patched, vec![">=1.2.3".to_string()]);
    }

    /// An advisory with no severity field downgrades to `Warning` rather
    /// than silently passing threshold gates.
    #[test]
    fn missing_severity_becomes_warning() {
        let json = r#"{
            "vulnerabilities": {
                "list": [{
                    "advisory": {
                        "id": "RUSTSEC-2024-0002",
                        "title": "no severity"
                    },
                    "versions": {"patched": []},
                    "package": {"name": "c", "version": "0.1.0"}
                }]
            }
        }"#;

        let parsed: CargoAuditOutput = serde_json::from_str(json).unwrap();
        let advisories = parsed.into_advisories();
        assert_eq!(advisories[0].severity, AdvisorySeverity::Warning);
    }

    /// Unknown severity strings also downgrade to `Warning`.
    #[test]
    fn unknown_severity_becomes_warning() {
        assert_eq!(AdvisorySeverity::parse("banana"), AdvisorySeverity::Warning);
        assert_eq!(
            AdvisorySeverity::parse("moderate"),
            AdvisorySeverity::Medium
        );
        assert_eq!(AdvisorySeverity::parse("HIGH"), AdvisorySeverity::High);
    }

    /// The `Advisory` type round-trips through JSON without field drift.
    #[test]
    fn advisory_roundtrips_through_json() {
        let a = Advisory {
            crate_name: "c".to_string(),
            advisory_id: "RUSTSEC-2024-9999".to_string(),
            title: "t".to_string(),
            severity: AdvisorySeverity::Medium,
            installed: "1.0.0".to_string(),
            patched: vec!["^1.1".to_string()],
        };
        let rendered = serde_json::to_string(&a).unwrap();
        let back: Advisory = serde_json::from_str(&rendered).unwrap();
        assert_eq!(a, back);
    }

    /// `finalise_report` flips `fail` when an advisory meets the threshold.
    #[test]
    fn finalise_report_fails_above_threshold() {
        let report = AuditReport {
            root: "/tmp".to_string(),
            duration_ms: 1,
            advisory_count: 1,
            advisories: vec![Advisory {
                crate_name: "c".to_string(),
                advisory_id: "RUSTSEC-2024-0003".to_string(),
                title: "t".to_string(),
                severity: AdvisorySeverity::High,
                installed: "1.0.0".to_string(),
                patched: vec![],
            }],
            fail: false,
        };
        let finalised = finalise_report(report, AdvisorySeverity::Warning);
        assert!(finalised.fail);
    }

    /// `finalise_report` leaves `fail` false when every advisory is below
    /// the threshold.
    #[test]
    fn finalise_report_passes_below_threshold() {
        let report = AuditReport {
            root: "/tmp".to_string(),
            duration_ms: 1,
            advisory_count: 1,
            advisories: vec![Advisory {
                crate_name: "c".to_string(),
                advisory_id: "RUSTSEC-2024-0004".to_string(),
                title: "t".to_string(),
                severity: AdvisorySeverity::Low,
                installed: "1.0.0".to_string(),
                patched: vec![],
            }],
            fail: false,
        };
        let finalised = finalise_report(report, AdvisorySeverity::High);
        assert!(!finalised.fail);
    }
}
