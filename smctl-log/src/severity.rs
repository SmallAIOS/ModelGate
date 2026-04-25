//! RFC 5424 severity levels and the mapping from `tracing::Level`.
//!
//! Mapping is declared in the spec at
//! `openspec/changes/smctl-logging-v1/specs/logging.md` and MUST NOT
//! drift from the table there.

use tracing_core::Level;

/// RFC 5424 severity. Values are the numeric codes from the RFC.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[repr(u8)]
pub enum Severity {
    Emergency = 0,
    Alert = 1,
    Critical = 2,
    Error = 3,
    Warning = 4,
    Notice = 5,
    Informational = 6,
    Debug = 7,
}

impl Severity {
    pub fn as_u8(self) -> u8 {
        self as u8
    }
}

/// Map a `tracing::Level` to its RFC 5424 severity.
///
/// `TRACE` and `DEBUG` both collapse to `Debug` (7). Callers needing
/// `Emergency` / `Alert` / `Critical` / `Notice` must construct a
/// `Severity` directly — `tracing::Level` does not express them.
pub fn from_tracing_level(level: Level) -> Severity {
    match level {
        Level::ERROR => Severity::Error,
        Level::WARN => Severity::Warning,
        Level::INFO => Severity::Informational,
        Level::DEBUG => Severity::Debug,
        Level::TRACE => Severity::Debug,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn tracing_levels_map_to_spec_table() {
        assert_eq!(from_tracing_level(Level::ERROR), Severity::Error);
        assert_eq!(from_tracing_level(Level::WARN), Severity::Warning);
        assert_eq!(from_tracing_level(Level::INFO), Severity::Informational);
        assert_eq!(from_tracing_level(Level::DEBUG), Severity::Debug);
        assert_eq!(from_tracing_level(Level::TRACE), Severity::Debug);
    }

    #[test]
    fn severity_numeric_codes_match_rfc() {
        assert_eq!(Severity::Emergency.as_u8(), 0);
        assert_eq!(Severity::Alert.as_u8(), 1);
        assert_eq!(Severity::Critical.as_u8(), 2);
        assert_eq!(Severity::Error.as_u8(), 3);
        assert_eq!(Severity::Warning.as_u8(), 4);
        assert_eq!(Severity::Notice.as_u8(), 5);
        assert_eq!(Severity::Informational.as_u8(), 6);
        assert_eq!(Severity::Debug.as_u8(), 7);
    }
}
