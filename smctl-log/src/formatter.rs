//! RFC 5424 event formatter for `tracing-subscriber`.
//!
//! Implements `FormatEvent` so it plugs into
//! `tracing_subscriber::fmt::Layer` with any `MakeWriter`. The wire
//! format matches `openspec/changes/smctl-logging-v1/specs/logging.md`
//! exactly.

use std::fmt;

use time::OffsetDateTime;
use time::format_description::well_known::Rfc3339;
use tracing::field::{Field, Visit};
use tracing::{Event, Subscriber};
use tracing_subscriber::fmt::FmtContext;
use tracing_subscriber::fmt::format::{FormatEvent, FormatFields, Writer};
use tracing_subscriber::registry::LookupSpan;

use crate::severity;

/// IANA private Enterprise Number placeholder. Replace with an
/// allocated PEN before any external SIEM ingestion. See
/// `specs/logging.md` Open Questions.
pub const STRUCTURED_DATA_PEN: u32 = 32473;

const SD_ID: &str = "SMCTL";
const NIL: &str = "-";

#[derive(Debug, Clone)]
pub struct Rfc5424 {
    pub facility: u8,
    pub app_name: String,
    pub hostname: String,
    pub pid: u32,
}

impl Rfc5424 {
    pub fn new(facility: u8, app_name: impl Into<String>) -> Self {
        let hostname = gethostname::gethostname()
            .into_string()
            .unwrap_or_else(|_| NIL.to_string());
        Self {
            facility,
            app_name: app_name.into(),
            hostname,
            pid: std::process::id(),
        }
    }
}

impl<S, N> FormatEvent<S, N> for Rfc5424
where
    S: Subscriber + for<'a> LookupSpan<'a>,
    N: for<'a> FormatFields<'a> + 'static,
{
    fn format_event(
        &self,
        _ctx: &FmtContext<'_, S, N>,
        mut writer: Writer<'_>,
        event: &Event<'_>,
    ) -> fmt::Result {
        let meta = event.metadata();
        let severity = severity::from_tracing_level(*meta.level());
        let pri = (self.facility as u16) * 8 + (severity.as_u8() as u16);

        let mut visitor = FieldCollector::default();
        event.record(&mut visitor);

        let msgid = visitor.msgid.unwrap_or_else(|| "SMCTL-0099".to_string());
        let message = visitor.message.unwrap_or_default();

        let timestamp = OffsetDateTime::now_utc()
            .format(&Rfc3339)
            .unwrap_or_else(|_| NIL.to_string());

        let hostname = if self.hostname.is_empty() {
            NIL.to_string()
        } else {
            self.hostname.clone()
        };

        write!(
            writer,
            "<{pri}>1 {ts} {host} {app} {pid} {msgid} ",
            pri = pri,
            ts = timestamp,
            host = hostname,
            app = self.app_name,
            pid = self.pid,
            msgid = msgid,
        )?;

        if visitor.sd.is_empty() {
            writer.write_str(NIL)?;
        } else {
            write!(writer, "[{}@{}", SD_ID, STRUCTURED_DATA_PEN)?;
            for (k, v) in &visitor.sd {
                write!(writer, " {}=\"{}\"", k, escape_sd_value(v))?;
            }
            writer.write_char(']')?;
        }

        if !message.is_empty() {
            writer.write_char(' ')?;
            writer.write_str(&message)?;
        }

        writeln!(writer)
    }
}

#[derive(Default)]
struct FieldCollector {
    msgid: Option<String>,
    message: Option<String>,
    sd: Vec<(String, String)>,
}

impl Visit for FieldCollector {
    fn record_str(&mut self, field: &Field, value: &str) {
        match field.name() {
            "msgid" => self.msgid = Some(value.to_string()),
            "message" => self.message = Some(value.to_string()),
            _ => self.sd.push((field.name().to_string(), value.to_string())),
        }
    }

    fn record_debug(&mut self, field: &Field, value: &dyn fmt::Debug) {
        let rendered = format!("{:?}", value);
        let trimmed = trim_debug_quotes(&rendered);
        match field.name() {
            "msgid" => self.msgid = Some(trimmed.to_string()),
            "message" => self.message = Some(trimmed.to_string()),
            _ => self
                .sd
                .push((field.name().to_string(), trimmed.to_string())),
        }
    }

    fn record_i64(&mut self, field: &Field, value: i64) {
        self.sd.push((field.name().to_string(), value.to_string()));
    }

    fn record_u64(&mut self, field: &Field, value: u64) {
        self.sd.push((field.name().to_string(), value.to_string()));
    }

    fn record_bool(&mut self, field: &Field, value: bool) {
        self.sd.push((field.name().to_string(), value.to_string()));
    }
}

// RFC 5424 § 6.3.3 — inside a PARAM-VALUE, backslash, double-quote,
// and close-bracket MUST be escaped with a preceding backslash.
fn escape_sd_value(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    for c in s.chars() {
        match c {
            '\\' | '"' | ']' => {
                out.push('\\');
                out.push(c);
            }
            _ => out.push(c),
        }
    }
    out
}

// tracing's Debug format wraps string-valued debug output in quotes:
// `record_debug(field, &"foo")` produces `"foo"`. Strip one level.
fn trim_debug_quotes(s: &str) -> &str {
    if s.len() >= 2 && s.starts_with('"') && s.ends_with('"') {
        &s[1..s.len() - 1]
    } else {
        s
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::severity::Severity;

    #[test]
    fn sd_value_escape_applies_to_backslash_quote_bracket() {
        assert_eq!(escape_sd_value("a\\b"), "a\\\\b");
        assert_eq!(escape_sd_value("a\"b"), "a\\\"b");
        assert_eq!(escape_sd_value("a]b"), "a\\]b");
        assert_eq!(escape_sd_value("plain text"), "plain text");
    }

    #[test]
    fn sd_value_escape_handles_all_three_in_one_string() {
        assert_eq!(escape_sd_value("a\\b\"c]d"), "a\\\\b\\\"c\\]d");
    }

    #[test]
    fn debug_quote_trimmer_removes_single_pair() {
        assert_eq!(trim_debug_quotes("\"foo\""), "foo");
        assert_eq!(trim_debug_quotes("foo"), "foo");
        assert_eq!(trim_debug_quotes("\""), "\"");
        assert_eq!(trim_debug_quotes(""), "");
    }

    #[test]
    fn pri_calculation_matches_rfc() {
        // local0 = 16, Informational = 6, PRI = 16*8 + 6 = 134
        let facility: u16 = 16;
        let severity = Severity::Informational.as_u8() as u16;
        assert_eq!(facility * 8 + severity, 134);

        // local1 = 17, Error = 3, PRI = 17*8 + 3 = 139
        assert_eq!(17u16 * 8 + Severity::Error.as_u8() as u16, 139);
    }
}
