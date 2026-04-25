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
}

fn default_fail_on() -> String {
    "any".to_string()
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
    fn manifest_default_fail_on_is_any() {
        let m: VerifyManifest = serde_json::from_str("{}").unwrap();
        assert_eq!(m.fail_on, "any");
        assert!(m.sources.is_empty());
    }
}
