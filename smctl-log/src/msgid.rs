//! MSGID catalog for smctl-logging-v1.
//!
//! The canonical catalog lives at
//! `openspec/changes/smctl-logging-v1/specs/logging.md`. This enum MUST
//! stay in sync with that table. MSGIDs are immutable once published.

use std::fmt;

use crate::severity::Severity;

/// Canonical MSGID catalog. Zero-padded four-digit form with the
/// `SMCTL-` prefix is produced by the `Display` impl.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum MsgId {
    WorkspaceInitialized,
    SpecCreated,
    SpecArchived,
    FeatureStarted,
    FeatureFinished,
    BuildStarted,
    BuildCompleted,
    BuildFailed,
    Uncategorized,
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
        }
    }

    /// Default severity for this MSGID. Callers MAY override by passing
    /// an explicit severity to `smctl_log::emit!`, but these are the
    /// defaults declared in the spec.
    pub fn default_severity(self) -> Severity {
        match self {
            MsgId::BuildFailed | MsgId::Uncategorized => Severity::Error,
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
}
