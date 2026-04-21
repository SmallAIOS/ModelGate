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
/// - `SMCTL-0400` .. `SMCTL-0499` — smctl-quality (see `safety-quality-v1/specs/quality-toolchain.md`)
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
            | MsgId::QualityCheckFailed
            | MsgId::DsmCycleDetected
            | MsgId::DependencyVulnerability => Severity::Error,
            MsgId::ComplexityThresholdExceeded
            | MsgId::DependencyUnused
            | MsgId::FerroceneIncompatibility => Severity::Warning,
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
}
