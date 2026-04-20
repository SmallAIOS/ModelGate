//! MSGID catalog for smctl-logging-v1.
//!
//! The canonical catalog lives at
//! `openspec/changes/smctl-logging-v1/specs/logging.md`. This enum MUST
//! stay in sync with that table. MSGIDs are immutable once published.

use std::fmt;

use crate::severity::Severity;

/// Canonical MSGID catalog. Zero-padded four-digit form with the
/// `SMCTL-` prefix is produced by the `Display` impl.
///
/// Range allocations (see `smctl-logging-v1/specs/logging.md`):
///
/// - `SMCTL-0001` .. `SMCTL-0099` — smctl core (workspace, spec, flow, build)
/// - `SMCTL-0200` .. `SMCTL-0299` — smctl-mcp (see `smctl-mcp-v1/specs/mcp-server-impl.md`)
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
            | MsgId::McpTransportFatal => Severity::Error,
            MsgId::McpClientDisconnected => Severity::Warning,
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
        assert_eq!(MsgId::McpServerStarted.to_string(), "SMCTL-0200");
        assert_eq!(MsgId::McpTransportFatal.to_string(), "SMCTL-0206");
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
        ] {
            let code = id.code();
            assert!(
                (200..=299).contains(&code),
                "MCP MSGID {id:?} code {code} outside reserved 200..=299"
            );
        }
    }
}
