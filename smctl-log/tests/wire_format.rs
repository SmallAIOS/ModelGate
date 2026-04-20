//! Integration test — a real `tracing` event goes through the
//! subscriber and produces an RFC 5424 line.
//!
//! Uses `tracing::subscriber::with_default` rather than the library's
//! global `init()` so the test does not collide with other tests in
//! the process.

use std::io::Write;
use std::sync::{Arc, Mutex};

use smctl_log::MsgId;
use smctl_log::formatter::Rfc5424;
use tracing::subscriber::with_default;
use tracing_subscriber::Registry;
use tracing_subscriber::fmt;
use tracing_subscriber::layer::SubscriberExt;

#[derive(Clone)]
struct BufferWriter(Arc<Mutex<Vec<u8>>>);

impl BufferWriter {
    fn new() -> Self {
        Self(Arc::new(Mutex::new(Vec::new())))
    }

    fn contents(&self) -> String {
        String::from_utf8(self.0.lock().unwrap().clone()).unwrap()
    }
}

impl Write for BufferWriter {
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        self.0.lock().unwrap().extend_from_slice(buf);
        Ok(buf.len())
    }

    fn flush(&mut self) -> std::io::Result<()> {
        Ok(())
    }
}

impl<'a> tracing_subscriber::fmt::MakeWriter<'a> for BufferWriter {
    type Writer = BufferWriter;
    fn make_writer(&'a self) -> Self::Writer {
        self.clone()
    }
}

fn run_with_capture<F: FnOnce()>(f: F) -> String {
    let buf = BufferWriter::new();
    let layer = fmt::Layer::default()
        .event_format(Rfc5424::new(16, "smctl-test"))
        .with_writer(buf.clone())
        .with_ansi(false);
    let subscriber = Registry::default().with(layer);
    with_default(subscriber, f);
    buf.contents()
}

#[test]
fn info_event_produces_rfc5424_line() {
    let out = run_with_capture(|| {
        tracing::info!(
            msgid = %MsgId::WorkspaceInitialized,
            path = "/tmp/ws",
            name = "demo",
            "initialized workspace"
        );
    });

    // PRI for facility=local0(16) severity=Informational(6) is 16*8+6=134
    assert!(
        out.starts_with("<134>1 "),
        "expected PRI <134>1, got: {}",
        &out[..out.len().min(50)]
    );
    assert!(out.contains(" smctl-test "), "missing APP-NAME: {out}");
    assert!(out.contains(" SMCTL-0001 "), "missing MSGID: {out}");
    assert!(
        out.contains("[SMCTL@32473"),
        "missing STRUCTURED-DATA open: {out}"
    );
    assert!(
        out.contains("path=\"/tmp/ws\""),
        "missing path field: {out}"
    );
    assert!(out.contains("name=\"demo\""), "missing name field: {out}");
    assert!(
        out.trim_end().ends_with("initialized workspace"),
        "missing or misplaced MSG: {out}"
    );
    assert!(out.ends_with('\n'), "missing trailing newline: {out}");
}

#[test]
fn error_event_uses_severity_3() {
    let out = run_with_capture(|| {
        tracing::error!(
            msgid = %MsgId::BuildFailed,
            repo = "kernel",
            duration_ms = 5123,
            "build failed"
        );
    });

    // local0(16)*8 + Error(3) = 131
    assert!(
        out.starts_with("<131>1 "),
        "expected PRI <131>, got: {}",
        &out[..out.len().min(50)]
    );
    assert!(out.contains(" SMCTL-0008 "), "missing MSGID: {out}");
    assert!(
        out.contains("duration_ms=\"5123\""),
        "missing numeric field: {out}"
    );
}

#[test]
fn event_without_msgid_falls_back_to_0099() {
    let out = run_with_capture(|| {
        tracing::error!("something went wrong");
    });

    assert!(
        out.contains(" SMCTL-0099 "),
        "missing fallback MSGID: {out}"
    );
}

#[test]
fn event_without_structured_fields_uses_nilvalue() {
    let out = run_with_capture(|| {
        tracing::info!(msgid = %MsgId::WorkspaceInitialized, "bare event");
    });

    // Should contain `SMCTL-0001 - bare event` (MSGID, space, nil, space, msg)
    assert!(
        out.contains(" SMCTL-0001 - bare event"),
        "missing nilvalue for empty SD: {out}"
    );
}

#[test]
fn special_characters_in_sd_values_are_escaped() {
    let out = run_with_capture(|| {
        tracing::info!(
            msgid = %MsgId::SpecCreated,
            branch = "feature/has\"quote",
            "quoting"
        );
    });

    assert!(
        out.contains("branch=\"feature/has\\\"quote\""),
        "double-quote not escaped: {out}"
    );
}
