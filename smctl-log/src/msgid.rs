//! MSGID catalog for smctl-logging-v1.
//!
//! The founding catalog lives at
//! `openspec/changes/archive/2026-04-24-smctl-logging-v1/specs/logging.md`
//! (immutable once archived); later ranges carry their per-code rows
//! in their capability specs (e.g. `openspec/specs/smctl-verify/spec.md`
//! for `SMCTL-05xx`). This enum MUST stay in sync with those tables.
//! MSGIDs are immutable once published.

use std::fmt;

use crate::severity::Severity;

/// Canonical MSGID catalog. Zero-padded four-digit form with the
/// `SMCTL-` prefix is produced by the `Display` impl.
///
/// Range allocations (see the archived `smctl-logging-v1` logging.md):
///
/// - `SMCTL-0001` .. `SMCTL-0099` — smctl core (workspace, spec, flow, build)
/// - `SMCTL-0200` .. `SMCTL-0299` — smctl-mcp (see `smctl-mcp-v1/specs/mcp-server-impl.md`)
/// - `SMCTL-0300` .. `SMCTL-0399` — modelgate-web (see `modelgate-web-v1/specs/web-server.md`)
/// - `SMCTL-0400` .. `SMCTL-0499` — smctl-quality (see `safety-quality-v1/specs/quality-toolchain.md`)
/// - `SMCTL-0500` .. `SMCTL-0599` — smctl-verify (see `formal-methods-v1/specs/smctl-verify/spec.md`)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum MsgId {
    // smctl core (0001..0099)
    WorkspaceInitialized,
    SpecCreated,
    SpecArchived,
    FeatureStarted,
    FeatureFinished,
    BuildStarted,
    BuildCompleted,
    BuildFailed,
    Uncategorized,

    // smctl-mcp (0200..0299)
    McpServerStarted,
    McpServerStopped,
    McpToolCalled,
    McpToolCompleted,
    McpToolFailed,
    McpClientDisconnected,
    McpTransportFatal,
    McpResourceRead,
    McpResourceReadFailed,

    // modelgate-web (0300..0399)
    WebServerStarted,
    WebServerStopped,
    WebUpstreamUnreachable,
    WebUpstreamTimeout,

    // smctl-verify (0500..0599)
    VerifyStarted,
    VerifySucceeded,
    VerifyFailed,
    VerifierMissing,
    VerifyCounterExample,
    VerifyOutputUnparsed,
    ProofIncomplete,

    // smctl-quality (0400..0499)
    QualityCheckStarted,
    QualityCheckCompleted,
    QualityCheckFailed,
    DsmCycleDetected,
    ComplexityThresholdExceeded,
    DependencyVulnerability,
    DependencyUnused,
    UnsafeBlockFound,
    FerroceneIncompatibility,
}

impl MsgId {
    pub fn code(self) -> u16 {
        match self {
            MsgId::WorkspaceInitialized => 1,
            MsgId::SpecCreated => 2,
            MsgId::SpecArchived => 3,
            MsgId::FeatureStarted => 4,
            MsgId::FeatureFinished => 5,
            MsgId::BuildStarted => 6,
            MsgId::BuildCompleted => 7,
            MsgId::BuildFailed => 8,
            MsgId::Uncategorized => 99,
            MsgId::McpServerStarted => 200,
            MsgId::McpServerStopped => 201,
            MsgId::McpToolCalled => 202,
            MsgId::McpToolCompleted => 203,
            MsgId::McpToolFailed => 204,
            MsgId::McpClientDisconnected => 205,
            MsgId::McpTransportFatal => 206,
            MsgId::McpResourceRead => 207,
            MsgId::McpResourceReadFailed => 208,
            MsgId::WebServerStarted => 301,
            MsgId::WebServerStopped => 302,
            MsgId::WebUpstreamUnreachable => 303,
            MsgId::WebUpstreamTimeout => 304,
            MsgId::VerifyStarted => 501,
            MsgId::VerifySucceeded => 502,
            MsgId::VerifyFailed => 503,
            MsgId::VerifierMissing => 504,
            MsgId::VerifyCounterExample => 505,
            MsgId::VerifyOutputUnparsed => 506,
            MsgId::ProofIncomplete => 507,
            MsgId::QualityCheckStarted => 400,
            MsgId::QualityCheckCompleted => 401,
            MsgId::QualityCheckFailed => 402,
            MsgId::DsmCycleDetected => 410,
            MsgId::ComplexityThresholdExceeded => 411,
            MsgId::DependencyVulnerability => 420,
            MsgId::DependencyUnused => 421,
            MsgId::UnsafeBlockFound => 430,
            MsgId::FerroceneIncompatibility => 440,
        }
    }

    /// Default severity for this MSGID. Callers MAY override by passing
    /// an explicit severity to `smctl_log::emit!`, but these are the
    /// defaults declared in the spec.
    pub fn default_severity(self) -> Severity {
        match self {
            MsgId::BuildFailed
            | MsgId::Uncategorized
            | MsgId::McpToolFailed
            | MsgId::McpTransportFatal
            | MsgId::McpResourceReadFailed
            | MsgId::QualityCheckFailed
            | MsgId::DsmCycleDetected
            | MsgId::DependencyVulnerability
            | MsgId::VerifyFailed
            | MsgId::VerifyCounterExample
            | MsgId::ProofIncomplete => Severity::Error,
            MsgId::McpClientDisconnected
            | MsgId::ComplexityThresholdExceeded
            | MsgId::DependencyUnused
            | MsgId::FerroceneIncompatibility
            | MsgId::WebUpstreamUnreachable
            | MsgId::WebUpstreamTimeout
            | MsgId::VerifierMissing
            | MsgId::VerifyOutputUnparsed => Severity::Warning,
            MsgId::UnsafeBlockFound => Severity::Notice,
            _ => Severity::Informational,
        }
    }
}

impl fmt::Display for MsgId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "SMCTL-{:04}", self.code())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn display_produces_canonical_zero_padded_form() {
        assert_eq!(MsgId::WorkspaceInitialized.to_string(), "SMCTL-0001");
        assert_eq!(MsgId::BuildFailed.to_string(), "SMCTL-0008");
        assert_eq!(MsgId::Uncategorized.to_string(), "SMCTL-0099");
    }

    #[test]
    fn codes_match_spec_catalog() {
        assert_eq!(MsgId::WorkspaceInitialized.code(), 1);
        assert_eq!(MsgId::SpecCreated.code(), 2);
        assert_eq!(MsgId::SpecArchived.code(), 3);
        assert_eq!(MsgId::FeatureStarted.code(), 4);
        assert_eq!(MsgId::FeatureFinished.code(), 5);
        assert_eq!(MsgId::BuildStarted.code(), 6);
        assert_eq!(MsgId::BuildCompleted.code(), 7);
        assert_eq!(MsgId::BuildFailed.code(), 8);
        assert_eq!(MsgId::Uncategorized.code(), 99);
    }

    #[test]
    fn quality_codes_and_display_match_spec_catalog() {
        assert_eq!(MsgId::QualityCheckStarted.code(), 400);
        assert_eq!(MsgId::QualityCheckCompleted.code(), 401);
        assert_eq!(MsgId::QualityCheckFailed.code(), 402);
        assert_eq!(MsgId::DsmCycleDetected.code(), 410);
        assert_eq!(MsgId::ComplexityThresholdExceeded.code(), 411);
        assert_eq!(MsgId::DependencyVulnerability.code(), 420);
        assert_eq!(MsgId::DependencyUnused.code(), 421);
        assert_eq!(MsgId::UnsafeBlockFound.code(), 430);
        assert_eq!(MsgId::FerroceneIncompatibility.code(), 440);
        assert_eq!(MsgId::QualityCheckStarted.to_string(), "SMCTL-0400");
        assert_eq!(MsgId::FerroceneIncompatibility.to_string(), "SMCTL-0440");
    }

    #[test]
    fn quality_default_severity_matches_spec() {
        assert_eq!(
            MsgId::QualityCheckStarted.default_severity(),
            Severity::Informational
        );
        assert_eq!(
            MsgId::QualityCheckFailed.default_severity(),
            Severity::Error
        );
        assert_eq!(MsgId::DsmCycleDetected.default_severity(), Severity::Error);
        assert_eq!(
            MsgId::DependencyVulnerability.default_severity(),
            Severity::Error
        );
        assert_eq!(
            MsgId::ComplexityThresholdExceeded.default_severity(),
            Severity::Warning
        );
        assert_eq!(
            MsgId::DependencyUnused.default_severity(),
            Severity::Warning
        );
        assert_eq!(MsgId::UnsafeBlockFound.default_severity(), Severity::Notice);
    }

    #[test]
    fn quality_codes_sit_in_reserved_range() {
        for id in [
            MsgId::QualityCheckStarted,
            MsgId::QualityCheckCompleted,
            MsgId::QualityCheckFailed,
            MsgId::DsmCycleDetected,
            MsgId::ComplexityThresholdExceeded,
            MsgId::DependencyVulnerability,
            MsgId::DependencyUnused,
            MsgId::UnsafeBlockFound,
            MsgId::FerroceneIncompatibility,
        ] {
            let code = id.code();
            assert!(
                (400..=499).contains(&code),
                "quality MSGID {id:?} code {code} outside reserved 400..=499"
            );
        }
    }

    #[test]
    fn default_severity_matches_spec() {
        assert_eq!(
            MsgId::WorkspaceInitialized.default_severity(),
            Severity::Informational
        );
        assert_eq!(MsgId::BuildFailed.default_severity(), Severity::Error);
        assert_eq!(MsgId::Uncategorized.default_severity(), Severity::Error);
    }

    #[test]
    fn mcp_codes_and_display_match_spec_catalog() {
        assert_eq!(MsgId::McpServerStarted.code(), 200);
        assert_eq!(MsgId::McpServerStopped.code(), 201);
        assert_eq!(MsgId::McpToolCalled.code(), 202);
        assert_eq!(MsgId::McpToolCompleted.code(), 203);
        assert_eq!(MsgId::McpToolFailed.code(), 204);
        assert_eq!(MsgId::McpClientDisconnected.code(), 205);
        assert_eq!(MsgId::McpTransportFatal.code(), 206);
        assert_eq!(MsgId::McpResourceRead.code(), 207);
        assert_eq!(MsgId::McpResourceReadFailed.code(), 208);
        assert_eq!(MsgId::McpServerStarted.to_string(), "SMCTL-0200");
        assert_eq!(MsgId::McpTransportFatal.to_string(), "SMCTL-0206");
        assert_eq!(MsgId::McpResourceRead.to_string(), "SMCTL-0207");
        assert_eq!(MsgId::McpResourceReadFailed.to_string(), "SMCTL-0208");
    }

    #[test]
    fn mcp_default_severity_matches_spec() {
        assert_eq!(
            MsgId::McpServerStarted.default_severity(),
            Severity::Informational
        );
        assert_eq!(
            MsgId::McpToolCompleted.default_severity(),
            Severity::Informational
        );
        assert_eq!(MsgId::McpToolFailed.default_severity(), Severity::Error);
        assert_eq!(MsgId::McpTransportFatal.default_severity(), Severity::Error);
        assert_eq!(
            MsgId::McpClientDisconnected.default_severity(),
            Severity::Warning
        );
        assert_eq!(
            MsgId::McpResourceRead.default_severity(),
            Severity::Informational
        );
        assert_eq!(
            MsgId::McpResourceReadFailed.default_severity(),
            Severity::Error
        );
    }

    #[test]
    fn web_codes_and_display_match_spec_catalog() {
        assert_eq!(MsgId::WebServerStarted.code(), 301);
        assert_eq!(MsgId::WebServerStopped.code(), 302);
        assert_eq!(MsgId::WebUpstreamUnreachable.code(), 303);
        assert_eq!(MsgId::WebUpstreamTimeout.code(), 304);
        assert_eq!(MsgId::WebServerStarted.to_string(), "SMCTL-0301");
        assert_eq!(MsgId::WebUpstreamTimeout.to_string(), "SMCTL-0304");
    }

    #[test]
    fn web_default_severity_matches_spec() {
        assert_eq!(
            MsgId::WebServerStarted.default_severity(),
            Severity::Informational
        );
        assert_eq!(
            MsgId::WebServerStopped.default_severity(),
            Severity::Informational
        );
        assert_eq!(
            MsgId::WebUpstreamUnreachable.default_severity(),
            Severity::Warning
        );
        assert_eq!(
            MsgId::WebUpstreamTimeout.default_severity(),
            Severity::Warning
        );
    }

    #[test]
    fn web_codes_sit_in_reserved_range() {
        for id in [
            MsgId::WebServerStarted,
            MsgId::WebServerStopped,
            MsgId::WebUpstreamUnreachable,
            MsgId::WebUpstreamTimeout,
        ] {
            let code = id.code();
            assert!(
                (300..=399).contains(&code),
                "web MSGID {id:?} code {code} outside reserved 300..=399"
            );
        }
    }

    #[test]
    fn verify_codes_and_display_match_spec_catalog() {
        assert_eq!(MsgId::VerifyStarted.code(), 501);
        assert_eq!(MsgId::VerifySucceeded.code(), 502);
        assert_eq!(MsgId::VerifyFailed.code(), 503);
        assert_eq!(MsgId::VerifierMissing.code(), 504);
        assert_eq!(MsgId::VerifyCounterExample.code(), 505);
        assert_eq!(MsgId::VerifyOutputUnparsed.code(), 506);
        assert_eq!(MsgId::ProofIncomplete.code(), 507);
        assert_eq!(MsgId::VerifyStarted.to_string(), "SMCTL-0501");
        assert_eq!(MsgId::VerifierMissing.to_string(), "SMCTL-0504");
        assert_eq!(MsgId::VerifyCounterExample.to_string(), "SMCTL-0505");
        assert_eq!(MsgId::ProofIncomplete.to_string(), "SMCTL-0507");
    }

    #[test]
    fn verify_default_severity_matches_spec() {
        assert_eq!(
            MsgId::VerifyStarted.default_severity(),
            Severity::Informational
        );
        assert_eq!(
            MsgId::VerifySucceeded.default_severity(),
            Severity::Informational
        );
        assert_eq!(MsgId::VerifyFailed.default_severity(), Severity::Error);
        assert_eq!(MsgId::VerifierMissing.default_severity(), Severity::Warning);
        assert_eq!(
            MsgId::VerifyCounterExample.default_severity(),
            Severity::Error
        );
        assert_eq!(
            MsgId::VerifyOutputUnparsed.default_severity(),
            Severity::Warning
        );
        assert_eq!(MsgId::ProofIncomplete.default_severity(), Severity::Error);
    }

    #[test]
    fn verify_codes_sit_in_reserved_range() {
        for id in [
            MsgId::VerifyStarted,
            MsgId::VerifySucceeded,
            MsgId::VerifyFailed,
            MsgId::VerifierMissing,
            MsgId::VerifyCounterExample,
            MsgId::VerifyOutputUnparsed,
            MsgId::ProofIncomplete,
        ] {
            let code = id.code();
            assert!(
                (500..=599).contains(&code),
                "verify MSGID {id:?} code {code} outside reserved 500..=599"
            );
        }
    }

    #[test]
    fn mcp_codes_sit_in_reserved_range() {
        for id in [
            MsgId::McpServerStarted,
            MsgId::McpServerStopped,
            MsgId::McpToolCalled,
            MsgId::McpToolCompleted,
            MsgId::McpToolFailed,
            MsgId::McpClientDisconnected,
            MsgId::McpTransportFatal,
            MsgId::McpResourceRead,
            MsgId::McpResourceReadFailed,
        ] {
            let code = id.code();
            assert!(
                (200..=299).contains(&code),
                "MCP MSGID {id:?} code {code} outside reserved 200..=299"
            );
        }
    }
}
