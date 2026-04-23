//! smctl-quality — engineering-quality toolchain surface.
//!
//! This crate provides the machinery behind `smctl quality <verb>`. Each
//! verb wraps a Cargo-ecosystem tool (cargo-audit, cargo-modules,
//! rust-code-analysis, cargo-machete, cargo-geiger, ...) and returns a
//! structured report that serialises to stable JSON.
//!
//! The crate is a log producer only: it emits tracing events via the
//! `smctl-log` MSGID catalog but never installs its own subscriber. The
//! `smctl` binary (or any embedder) owns the subscriber.
//!
//! Current scope: the `audit`, `deps`, `unsafe`, `dsm`, and `complexity`
//! verbs are implemented end-to-end. The remaining verbs are reserved in
//! the MSGID catalog but deferred to follow-up changes.

pub mod audit;
pub mod complexity;
pub mod deps;
pub mod dsm;
#[path = "unsafe_code.rs"]
pub mod unsafe_scan;

pub use audit::{
    Advisory, AdvisorySeverity, AuditError, AuditReport, cargo_audit_available, finalise_report,
    run_audit,
};
pub use complexity::{
    ComplexityError, ComplexityReport, FinalizedReport, FunctionMetric,
    cargo_rust_code_analysis_available, run_complexity,
};
pub use deps::{DepsError, DepsReport, UnusedDependency, cargo_machete_available, run_deps};
pub use dsm::{DsmCycle, DsmError, DsmReport, cargo_modules_available, run_dsm};
pub use unsafe_scan::{UnsafeCrate, UnsafeError, UnsafeReport, cargo_geiger_available, run_unsafe};
