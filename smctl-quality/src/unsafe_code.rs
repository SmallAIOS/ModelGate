//! `smctl quality unsafe` — unsafe-code tracking via `cargo geiger`.
//!
//! Wraps the `cargo-geiger` binary and converts its output into a crate-local
//! [`UnsafeReport`] struct. Emits MSGID-tagged tracing events at start, per
//! crate with unsafe code (SMCTL-0430), and on completion/failure.
//!
//! Scope: geiger emits a rich tree of per-crate, per-category counts. We
//! parse only the per-crate summary — the top-level figure plus the per-
//! crate unsafe totals. Representing geiger's full tree is out of scope for
//! this verb; a future pass can extend the report type if needed.

use std::path::Path;
use std::process::Command;
use std::time::Instant;

use serde::{Deserialize, Serialize};
use smctl_log::MsgId;
use thiserror::Error;

/// Per-crate unsafe counts, collapsed into a single number so the JSON
/// report stays small. Callers who need the full category breakdown can
/// run `cargo geiger --output-format Json` directly.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct UnsafeCrate {
    /// The crate that contains unsafe code.
    pub crate_name: String,
    /// Version of that crate.
    pub version: String,
    /// Total number of unsafe code sites reported by cargo-geiger, summed
    /// across the functions / expressions / impls / traits / methods
    /// categories.
    pub unsafe_count: u64,
}

/// Structured result of a `smctl quality unsafe` run.
///
/// Serialises to stable JSON. Consumers should treat the shape as a
/// public contract.
#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq, Eq)]
pub struct UnsafeReport {
    /// Root of the workspace (or crate) that was scanned.
    pub root: String,
    /// Elapsed wall time of the unsafe scan in milliseconds.
    pub duration_ms: u128,
    /// Number of crates with at least one unsafe site. Same as `crates.len()`.
    pub crate_count: usize,
    /// Total unsafe site count summed across all crates.
    pub total_unsafe: u64,
    /// Per-crate breakdown, one entry per crate with a non-zero unsafe
    /// count.
    pub crates: Vec<UnsafeCrate>,
    /// True when the report crossed the configured fail threshold. The
    /// caller decides exit status from this.
    pub fail: bool,
}

/// Errors raised by [`run_unsafe`].
///
/// Every variant carries a three-part message (what happened / what it means
/// / what to do next) so that CLI callers can surface it directly without
/// rewrapping.
#[derive(Debug, Error)]
pub enum UnsafeError {
    #[error(
        "cargo-geiger is not installed on PATH. smctl quality unsafe cannot run without it. Install it with: cargo install cargo-geiger"
    )]
    ToolMissing,

    #[error(
        "cargo-geiger exited abnormally ({status}). The unsafe scan result is unreliable. Re-run `cargo geiger` directly to inspect the failure: {stderr}"
    )]
    ToolFailed { status: String, stderr: String },

    #[error(
        "cargo-geiger produced JSON that does not match the expected schema. The installed cargo-geiger may be a different major version than this build targets. Upgrade it with: cargo install --force cargo-geiger. Parser error: {source}"
    )]
    MalformedJson {
        #[source]
        source: serde_json::Error,
    },
}

/// Invoke `cargo geiger --output-format Json` in `root`, parse the output,
/// emit MSGIDs.
///
/// This is the primary public entry point of the unsafe module. The
/// returned report is independent of CLI formatting concerns — the caller
/// renders it human-readable or as JSON.
pub fn run_unsafe(root: &Path) -> Result<UnsafeReport, UnsafeError> {
    let root_str = root.display().to_string();
    let started = Instant::now();

    tracing::info!(
        msgid = %MsgId::QualityCheckStarted,
        verb = "unsafe",
        repo = %root_str,
        scope = "workspace",
        "unsafe started"
    );

    let raw = invoke_cargo_geiger(root)?;
    let crates = parse_geiger_output(&raw)?;
    let total_unsafe: u64 = crates.iter().map(|c| c.unsafe_count).sum();

    for c in &crates {
        tracing::info!(
            msgid = %MsgId::UnsafeBlockFound,
            crate_name = %c.crate_name,
            version = %c.version,
            line_count = c.unsafe_count,
            "unsafe block found"
        );
    }

    let duration_ms = started.elapsed().as_millis();
    let crate_count = crates.len();

    Ok(UnsafeReport {
        root: root_str,
        duration_ms,
        crate_count,
        total_unsafe,
        crates,
        fail: false,
    })
}

/// Decide whether the report should fail the check, given a minimum
/// unsafe-site count. The report fails when `total_unsafe >=
/// fail_on_count`. A `fail_on_count` of 0 disables the gate (useful for
/// report-only runs on the kernel side, where some unsafe is expected).
///
/// Emits the terminal completion MSGID (either `QualityCheckCompleted` or
/// `QualityCheckFailed`). The caller should invoke this exactly once per
/// run, after [`run_unsafe`] returns.
pub fn finalise_report(mut report: UnsafeReport, fail_on_count: u64) -> UnsafeReport {
    let fail = fail_on_count > 0 && report.total_unsafe >= fail_on_count;
    report.fail = fail;

    if fail {
        let remediation = report
            .crates
            .iter()
            .map(|c| format!("review unsafe in `{}`", c.crate_name))
            .collect::<Vec<_>>()
            .join("; ");
        tracing::error!(
            msgid = %MsgId::QualityCheckFailed,
            verb = "unsafe",
            duration_ms = report.duration_ms as u64,
            violation_count = report.total_unsafe,
            remediation = %remediation,
            "unsafe failed"
        );
    } else {
        tracing::info!(
            msgid = %MsgId::QualityCheckCompleted,
            verb = "unsafe",
            duration_ms = report.duration_ms as u64,
            violation_count = report.total_unsafe,
            "unsafe completed"
        );
    }

    report
}

/// Shell out to `cargo geiger --output-format Json`. Returns the raw
/// stdout bytes on success; maps ENOENT to [`UnsafeError::ToolMissing`]
/// so the CLI surface can render the three-part install message.
fn invoke_cargo_geiger(root: &Path) -> Result<Vec<u8>, UnsafeError> {
    let output = Command::new("cargo")
        .arg("geiger")
        .arg("--output-format")
        .arg("Json")
        .current_dir(root)
        .output()
        .map_err(|e| {
            if e.kind() == std::io::ErrorKind::NotFound {
                UnsafeError::ToolMissing
            } else {
                UnsafeError::ToolFailed {
                    status: e.kind().to_string(),
                    stderr: e.to_string(),
                }
            }
        })?;

    if output.stdout.is_empty() {
        let stderr = String::from_utf8_lossy(&output.stderr).to_string();
        if !output.status.success()
            && (stderr.contains("no such command")
                || stderr.contains("no such subcommand")
                || stderr.contains("is not installed"))
        {
            return Err(UnsafeError::ToolMissing);
        }
        return Err(UnsafeError::ToolFailed {
            status: output
                .status
                .code()
                .map_or_else(|| "signal".to_string(), |c| c.to_string()),
            stderr,
        });
    }

    Ok(output.stdout)
}

/// Parse cargo-geiger JSON output into per-crate unsafe totals.
///
/// cargo-geiger emits a tree of packages with `unsafety.used` counts for
/// each category (functions, exprs, item_impls, item_traits, methods).
/// We sum those categories per crate and retain only crates with a
/// non-zero total.
fn parse_geiger_output(raw: &[u8]) -> Result<Vec<UnsafeCrate>, UnsafeError> {
    let parsed: GeigerOutput =
        serde_json::from_slice(raw).map_err(|source| UnsafeError::MalformedJson { source })?;
    Ok(parsed.into_crates())
}

/// Detect whether cargo-geiger is available on PATH.
///
/// Used by the CLI surface to skip gracefully in environments without the
/// tool installed, surfacing the three-part remediation message instead of
/// attempting a run that will fail.
pub fn cargo_geiger_available() -> bool {
    match Command::new("cargo")
        .arg("geiger")
        .arg("--version")
        .output()
    {
        Ok(out) => out.status.success(),
        Err(_) => false,
    }
}

// ── cargo-geiger JSON shape ──────────────────────────────────────────
//
// This is the subset of the cargo-geiger `--output-format Json` output we
// depend on. geiger emits a `packages` array where each entry carries
// package metadata and `unsafety` counts with `used` and `unused` groups,
// each subdivided by category. We only need `used` sums.

#[derive(Debug, Deserialize)]
struct GeigerOutput {
    #[serde(default)]
    packages: Vec<GeigerPackage>,
}

#[derive(Debug, Deserialize)]
struct GeigerPackage {
    package: GeigerPackageId,
    unsafety: GeigerUnsafety,
}

#[derive(Debug, Deserialize)]
struct GeigerPackageId {
    id: GeigerPackageIdInner,
}

#[derive(Debug, Deserialize)]
struct GeigerPackageIdInner {
    name: String,
    version: String,
}

#[derive(Debug, Deserialize)]
struct GeigerUnsafety {
    used: GeigerCategories,
}

#[derive(Debug, Default, Deserialize)]
struct GeigerCategories {
    #[serde(default)]
    functions: GeigerCount,
    #[serde(default)]
    exprs: GeigerCount,
    #[serde(default)]
    item_impls: GeigerCount,
    #[serde(default)]
    item_traits: GeigerCount,
    #[serde(default)]
    methods: GeigerCount,
}

#[derive(Debug, Default, Deserialize)]
struct GeigerCount {
    #[serde(default)]
    unsafe_: u64,
    #[serde(default, rename = "unsafe")]
    unsafe_alt: u64,
}

impl GeigerCount {
    fn value(&self) -> u64 {
        // Either naming convention may appear depending on serde config;
        // take whichever is non-zero.
        if self.unsafe_alt != 0 {
            self.unsafe_alt
        } else {
            self.unsafe_
        }
    }
}

impl GeigerCategories {
    fn total(&self) -> u64 {
        self.functions.value()
            + self.exprs.value()
            + self.item_impls.value()
            + self.item_traits.value()
            + self.methods.value()
    }
}

impl GeigerOutput {
    fn into_crates(self) -> Vec<UnsafeCrate> {
        self.packages
            .into_iter()
            .filter_map(|p| {
                let total = p.unsafety.used.total();
                if total == 0 {
                    None
                } else {
                    Some(UnsafeCrate {
                        crate_name: p.package.id.name,
                        version: p.package.id.version,
                        unsafe_count: total,
                    })
                }
            })
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Empty cargo-geiger output (no packages) deserialises cleanly and
    /// produces a zero-crate report.
    #[test]
    fn empty_geiger_output_parses() {
        let json = br#"{"packages": []}"#;
        let crates = parse_geiger_output(json).unwrap();
        assert_eq!(crates.len(), 0);
    }

    /// A single package with unsafe counts in multiple categories sums
    /// correctly.
    #[test]
    fn single_package_parses() {
        let json = br#"{
            "packages": [{
                "package": {"id": {"name": "demo", "version": "1.2.3"}},
                "unsafety": {
                    "used": {
                        "functions": {"unsafe": 2},
                        "exprs": {"unsafe": 5},
                        "item_impls": {"unsafe": 0},
                        "item_traits": {"unsafe": 0},
                        "methods": {"unsafe": 1}
                    },
                    "unused": {
                        "functions": {"unsafe": 0},
                        "exprs": {"unsafe": 0},
                        "item_impls": {"unsafe": 0},
                        "item_traits": {"unsafe": 0},
                        "methods": {"unsafe": 0}
                    }
                }
            }]
        }"#;
        let crates = parse_geiger_output(json).unwrap();
        assert_eq!(crates.len(), 1);
        assert_eq!(crates[0].crate_name, "demo");
        assert_eq!(crates[0].version, "1.2.3");
        assert_eq!(crates[0].unsafe_count, 2 + 5 + 1);
    }

    /// Packages with zero unsafe across every category are filtered out
    /// of the report.
    #[test]
    fn zero_unsafe_packages_are_dropped() {
        let json = br#"{
            "packages": [{
                "package": {"id": {"name": "safe", "version": "0.1.0"}},
                "unsafety": {
                    "used": {
                        "functions": {"unsafe": 0},
                        "exprs": {"unsafe": 0},
                        "item_impls": {"unsafe": 0},
                        "item_traits": {"unsafe": 0},
                        "methods": {"unsafe": 0}
                    },
                    "unused": {
                        "functions": {"unsafe": 0},
                        "exprs": {"unsafe": 0},
                        "item_impls": {"unsafe": 0},
                        "item_traits": {"unsafe": 0},
                        "methods": {"unsafe": 0}
                    }
                }
            }]
        }"#;
        let crates = parse_geiger_output(json).unwrap();
        assert_eq!(crates.len(), 0);
    }

    /// `UnsafeCrate` round-trips through JSON without field drift.
    #[test]
    fn unsafe_crate_roundtrips_through_json() {
        let c = UnsafeCrate {
            crate_name: "c".to_string(),
            version: "1.0.0".to_string(),
            unsafe_count: 7,
        };
        let rendered = serde_json::to_string(&c).unwrap();
        let back: UnsafeCrate = serde_json::from_str(&rendered).unwrap();
        assert_eq!(c, back);
    }

    /// `finalise_report` flips `fail` when the total meets the threshold.
    #[test]
    fn finalise_report_fails_at_or_above_threshold() {
        let report = UnsafeReport {
            root: "/tmp".to_string(),
            duration_ms: 1,
            crate_count: 1,
            total_unsafe: 3,
            crates: vec![UnsafeCrate {
                crate_name: "c".to_string(),
                version: "1.0.0".to_string(),
                unsafe_count: 3,
            }],
            fail: false,
        };
        let finalised = finalise_report(report, 1);
        assert!(finalised.fail);
    }

    /// `finalise_report` leaves `fail` false below the threshold and when
    /// `fail_on_count` is 0.
    #[test]
    fn finalise_report_passes_below_threshold() {
        let report = UnsafeReport {
            root: "/tmp".to_string(),
            duration_ms: 1,
            crate_count: 0,
            total_unsafe: 0,
            crates: vec![],
            fail: false,
        };
        let finalised = finalise_report(report, 1);
        assert!(!finalised.fail);

        // fail_on_count = 0 disables the gate even with findings.
        let report = UnsafeReport {
            total_unsafe: 99,
            crates: vec![UnsafeCrate {
                crate_name: "k".to_string(),
                version: "0.1.0".to_string(),
                unsafe_count: 99,
            }],
            crate_count: 1,
            ..UnsafeReport::default()
        };
        let finalised = finalise_report(report, 0);
        assert!(!finalised.fail);
    }
}
