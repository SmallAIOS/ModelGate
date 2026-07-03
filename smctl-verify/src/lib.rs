//! smctl-verify — formal-verification surface for `smctl`.
//!
//! This crate backs the `smctl verify <verb>` command tree. Each
//! verifier (Cedar / TLA+ / Lean 4 / SPIN/Promela / discover) is an
//! implementation of the [`Verifier`] trait. The CLI dispatches into
//! a registry that owns one boxed implementation per supported tool.
//!
//! Status: trait + context + report + registry. Per-tool runners
//! land in subsequent commits on this branch (per
//! `openspec/changes/formal-methods-v1/tasks.md`).

use std::collections::BTreeMap;
use std::path::PathBuf;

use serde::{Deserialize, Serialize};

pub mod cedar;
pub mod lean;
pub mod registry;
pub mod shell;
pub mod spin;
pub mod tla;
pub mod tlc;

pub use cedar::CedarVerifier;
pub use lean::LeanVerifier;
pub use registry::Registry;
pub use spin::SpinVerifier;
pub use tla::TlaVerifier;

// --- Trait ---

/// One verifier domain. The registry owns a boxed `dyn Verifier` per
/// supported tool; the CLI dispatches to the implementation whose
/// [`Verifier::name`] matches the operator's chosen subcommand.
pub trait Verifier: Send + Sync {
    /// Stable identifier — `"policy"`, `"model"`, `"proof"`,
    /// `"protocol"`. Matches the CLI subcommand name.
    fn name(&self) -> &'static str;

    /// Probe whether the underlying tool is reachable. Pure-Rust
    /// verifiers (e.g. Cedar) always return [`DiscoveryResult::Found`];
    /// shell-out verifiers consult `PATH` and parse a version string.
    fn discover(&self) -> DiscoveryResult;

    /// Run the verifier against a context. The implementation MUST NOT
    /// install its own tracing subscriber; emit events with the
    /// `tracing` macros and let `smctl-log` route them.
    fn run(&self, ctx: &VerifyContext) -> VerifyReport;
}

// --- Inputs ---

/// Per-run inputs the CLI assembles before calling [`Verifier::run`].
/// Owned data — no borrowed lifetimes — so the trait is dyn-friendly
/// and the struct can travel across thread boundaries if a future
/// version dispatches verifiers concurrently.
#[derive(Debug, Clone)]
pub struct VerifyContext {
    /// Workspace root — the directory containing `.smctl/workspace.toml`.
    pub workspace_root: PathBuf,

    /// Per-repo absolute paths, keyed by repo name from the manifest.
    /// The verifier glob's `sources` patterns relative to each entry.
    pub repos: BTreeMap<String, PathBuf>,

    /// `[verify.<domain>]` section from the manifest.
    pub manifest: VerifyManifest,

    /// `--strict` flag from the CLI. When set, warnings are promoted
    /// to errors for gating purposes.
    pub strict: bool,

    /// `--verifier <name>` filter. When `Some`, only the named
    /// verifier runs; the registry-walking dispatcher ignores
    /// everything else.
    pub verifier_filter: Option<String>,
}

/// Slice of `workspace.toml` relevant to a single verifier run.
/// The schema is finalised in §6 of `tasks.md`; for now this carries
/// just the source-roots plus a `fail_on` token consistent with
/// the smctl-quality pattern.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct VerifyManifest {
    /// Glob patterns relative to each repo root, e.g.
    /// `["security/policies/*.cedar"]` for the Cedar verifier.
    #[serde(default)]
    pub sources: Vec<String>,

    /// `"any"` — fail when any diagnostic is at warning or higher.
    /// `"error"` — fail only on errors. Default `"any"`.
    #[serde(default = "default_fail_on")]
    pub fail_on: String,

    /// `[verify.model] jar` — path to `tla2tools.jar` for the
    /// `java -jar` fallback. Relative paths resolve against the
    /// workspace root. Only the model verb consults this.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub jar: Option<String>,

    /// `[verify.model] workers` — TLC worker thread count.
    /// `None` means `-workers auto`. Only the model verb consults this.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub workers: Option<u32>,
}

fn default_fail_on() -> String {
    "any".to_string()
}

/// Project a `[verify]` subsection from `workspace.toml` onto the
/// internal [`VerifyManifest`] shape for one verifier verb.
///
/// Each verb's natural field name differs (`sources` for Cedar
/// policies, `specs` for TLA+ and SPIN, `roots` for Lean 4); this
/// helper keeps the per-verb switch in one place so the CLI dispatch
/// and the smctl-mcp run-verifier handler don't carry parallel copies
/// of the same match arms.
///
/// Returns the default manifest (empty sources, `"any"` fail-on) when
/// the section is absent or the verb is unknown.
pub fn manifest_from_workspace(
    section: Option<&smctl_workspace::VerifyManifestSection>,
    verb: &str,
) -> VerifyManifest {
    let Some(s) = section else {
        return VerifyManifest::default();
    };
    match verb {
        "policy" => s
            .policy
            .as_ref()
            .map(|p| VerifyManifest {
                sources: p.sources.clone(),
                fail_on: p.fail_on.clone(),
                ..VerifyManifest::default()
            })
            .unwrap_or_default(),
        "model" => s
            .model
            .as_ref()
            .map(|m| VerifyManifest {
                sources: m.specs.clone(),
                fail_on: m.fail_on.clone(),
                jar: m.jar.clone(),
                workers: m.workers,
            })
            .unwrap_or_default(),
        "proof" => s
            .proof
            .as_ref()
            .map(|p| VerifyManifest {
                sources: p.roots.clone(),
                fail_on: p.fail_on.clone(),
                ..VerifyManifest::default()
            })
            .unwrap_or_default(),
        "protocol" => s
            .protocol
            .as_ref()
            .map(|p| VerifyManifest {
                sources: p.specs.clone(),
                fail_on: p.fail_on.clone(),
                ..VerifyManifest::default()
            })
            .unwrap_or_default(),
        _ => VerifyManifest::default(),
    }
}

// --- Outputs ---

/// Result of a [`Verifier::discover`] probe.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum DiscoveryResult {
    /// Tool is reachable. `path` is the resolved binary (or
    /// `<rust-dep>` for crates we link directly). `version` is a
    /// best-effort string — empty when the tool produces no version
    /// banner.
    Found { path: String, version: String },

    /// Tool is not reachable. `tool` is the binary or crate name we
    /// looked for. `install_hint` carries a remediation pointer the
    /// CLI surfaces verbatim.
    NotInstalled { tool: String, install_hint: String },
}

impl DiscoveryResult {
    pub fn is_installed(&self) -> bool {
        matches!(self, DiscoveryResult::Found { .. })
    }
}

/// Aggregate of one [`Verifier::run`] invocation.
///
/// Each row in `sources` records one input (e.g. one `.cedar` file)
/// and its outcome. `outcome` summarises across rows. `diagnostics`
/// collects per-source detail; the CLI renders these with the
/// design-system three-part remediation structure.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VerifyReport {
    /// Verifier name (`"policy"`, `"model"`, …) — duplicates
    /// `Verifier::name` so the JSON envelope is self-describing.
    pub verifier: String,

    /// Overall outcome across every source.
    pub outcome: Outcome,

    /// One row per source (file, spec, root) that the verifier
    /// processed.
    pub sources: Vec<SourceRow>,

    /// Free-form diagnostic lines. Each line is already shaped per
    /// the three-part remediation rule (what / what-it-means /
    /// what-to-do-next).
    pub diagnostics: Vec<String>,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum Outcome {
    /// Every source passed.
    Passed,

    /// At least one source failed. The CLI exits non-zero.
    Failed,

    /// No sources were configured for this verifier (`[verify.X]`
    /// missing or empty). The CLI prints "no sources configured" and
    /// exits 0.
    NoSources,

    /// Underlying tool isn't reachable. The CLI surfaces
    /// `tool_missing` with the install hint from
    /// [`DiscoveryResult::NotInstalled`].
    ToolMissing,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SourceRow {
    /// Path or identifier the verifier acted on.
    pub source: String,
    /// `Passed` or `Failed`.
    pub outcome: Outcome,
    /// Optional human-readable note (e.g. requirement count, error
    /// summary). Not rendered when empty.
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub note: String,
    /// Structured model-check detail. Populated by the TLA+ runner;
    /// other verifiers omit it, so existing JSON consumers see no
    /// change. Flows through smctl-mcp automatically.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub detail: Option<ModelCheckDetail>,
}

impl SourceRow {
    /// Row without structured detail — the shape every verifier
    /// except the TLA+ runner produces.
    pub fn plain(source: String, outcome: Outcome, note: String) -> Self {
        Self {
            source,
            outcome,
            note,
            detail: None,
        }
    }
}

/// Structured result of one TLC model-checking run, parsed from the
/// tool's output. Attached to [`SourceRow::detail`] by the model verb.
#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq, Eq)]
pub struct ModelCheckDetail {
    /// Total states TLC generated.
    pub states_generated: u64,
    /// Distinct states found.
    pub distinct_states: u64,
    /// States left on the queue when checking stopped.
    pub queue_remaining: u64,
    /// Present when TLC found a property violation.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub violation: Option<Violation>,
}

/// A property violation TLC reported, with its counter-example size.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct Violation {
    /// Which class of property failed.
    pub kind: ViolationKind,
    /// Violated property name when TLC printed one (invariants do;
    /// deadlocks don't).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub property: Option<String>,
    /// Number of states in the counter-example trace.
    pub trace_states: usize,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum ViolationKind {
    /// A safety invariant was violated.
    Invariant,
    /// The model deadlocked.
    Deadlock,
    /// A temporal (liveness) property was violated.
    Liveness,
    /// An ASSUME clause evaluated to false.
    Assumption,
}

impl VerifyReport {
    /// Build an empty report with the given outcome. Convenience for
    /// the `NoSources` and `ToolMissing` early-exits.
    pub fn empty(verifier: &str, outcome: Outcome) -> Self {
        Self {
            verifier: verifier.to_string(),
            outcome,
            sources: Vec::new(),
            diagnostics: Vec::new(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn empty_report_round_trips_through_json() {
        let r = VerifyReport::empty("policy", Outcome::NoSources);
        let s = serde_json::to_string(&r).unwrap();
        let back: VerifyReport = serde_json::from_str(&s).unwrap();
        assert_eq!(back.verifier, "policy");
        assert_eq!(back.outcome, Outcome::NoSources);
        assert!(back.sources.is_empty());
    }

    #[test]
    fn discovery_serialises_with_kind_tag() {
        let found = DiscoveryResult::Found {
            path: "/usr/bin/tlc".into(),
            version: "1.8.0".into(),
        };
        let s = serde_json::to_string(&found).unwrap();
        assert!(s.contains("\"kind\":\"found\""));
        assert!(s.contains("\"path\":\"/usr/bin/tlc\""));

        let missing = DiscoveryResult::NotInstalled {
            tool: "tlc".into(),
            install_hint: "install via the TLA+ Toolbox".into(),
        };
        assert!(!missing.is_installed());
    }

    #[test]
    fn source_row_omits_detail_when_absent_and_includes_when_present() {
        let plain = SourceRow::plain("a.tla".into(), Outcome::Passed, String::new());
        let s = serde_json::to_string(&plain).unwrap();
        assert!(!s.contains("detail"), "detail must be omitted: {s}");

        let with_detail = SourceRow {
            source: "b.tla".into(),
            outcome: Outcome::Failed,
            note: String::new(),
            detail: Some(ModelCheckDetail {
                states_generated: 10,
                distinct_states: 5,
                queue_remaining: 0,
                violation: Some(Violation {
                    kind: ViolationKind::Invariant,
                    property: Some("TypeOK".into()),
                    trace_states: 3,
                }),
            }),
        };
        let s = serde_json::to_string(&with_detail).unwrap();
        assert!(s.contains("\"states_generated\":10"), "{s}");
        assert!(s.contains("\"kind\":\"invariant\""), "{s}");
        assert!(s.contains("\"property\":\"TypeOK\""), "{s}");
    }

    #[test]
    fn manifest_from_workspace_threads_model_jar_and_workers() {
        let section = smctl_workspace::VerifyManifestSection {
            model: Some(smctl_workspace::ModelVerifierSection {
                specs: vec!["formal/tla/*.tla".into()],
                fail_on: "any".into(),
                jar: Some("tools/tla2tools.jar".into()),
                workers: Some(8),
            }),
            ..Default::default()
        };
        let m = manifest_from_workspace(Some(&section), "model");
        assert_eq!(m.jar.as_deref(), Some("tools/tla2tools.jar"));
        assert_eq!(m.workers, Some(8));
        // Other verbs never carry jar/workers.
        let p = manifest_from_workspace(Some(&section), "policy");
        assert_eq!(p.jar, None);
        assert_eq!(p.workers, None);
    }

    #[test]
    fn manifest_default_fail_on_is_any() {
        let m: VerifyManifest = serde_json::from_str("{}").unwrap();
        assert_eq!(m.fail_on, "any");
        assert!(m.sources.is_empty());
    }
}
