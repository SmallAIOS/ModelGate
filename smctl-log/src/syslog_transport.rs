//! Unix-socket syslog transport for `smctl-log`.
//!
//! On Unix, `open_unix_socket` probes the standard local syslog socket
//! paths (`/dev/log`, `/var/run/syslog`, `/var/run/log`) via the
//! `syslog` crate and returns a connected datagram backend wrapped in
//! a `MakeWriter` adapter that `tracing-subscriber::fmt::Layer` can
//! use directly.
//!
//! On Windows (and any non-Unix target) `open_unix_socket` always
//! returns `Err`: local syslog does not exist. Callers must fall back
//! to stderr and emit a one-time WARN.
//!
//! The RFC 5424 wire format is produced by our own `Rfc5424`
//! formatter; the `syslog` crate is used only for transport, not for
//! formatting.

#[cfg(target_family = "unix")]
mod imp {
    use std::io;
    use std::sync::{Arc, Mutex};

    use syslog::{Formatter3164, LoggerBackend};
    use tracing_subscriber::fmt::MakeWriter;

    /// Shared, thread-safe handle to a connected Unix syslog socket.
    #[derive(Clone)]
    pub struct SyslogWriter {
        inner: Arc<Mutex<LoggerBackend>>,
    }

    /// Per-event writer handed out by `MakeWriter`. Buffers the event
    /// bytes in memory and flushes to the syslog datagram on drop so
    /// each `tracing` event is one syslog message.
    pub struct SyslogEventWriter {
        buf: Vec<u8>,
        inner: Arc<Mutex<LoggerBackend>>,
    }

    impl io::Write for SyslogEventWriter {
        fn write(&mut self, data: &[u8]) -> io::Result<usize> {
            self.buf.extend_from_slice(data);
            Ok(data.len())
        }

        fn flush(&mut self) -> io::Result<()> {
            Ok(())
        }
    }

    impl Drop for SyslogEventWriter {
        fn drop(&mut self) {
            if self.buf.is_empty() {
                return;
            }
            // Strip a single trailing newline — syslog datagrams are
            // framed at the datagram boundary; the \n our RFC 5424
            // formatter appends is redundant on the wire.
            let payload: &[u8] = if self.buf.ends_with(b"\n") {
                &self.buf[..self.buf.len() - 1]
            } else {
                &self.buf[..]
            };
            if let Ok(mut backend) = self.inner.lock() {
                // Best-effort: a failing syslog write MUST NOT panic or
                // propagate. The spec's error-handling rule says log
                // pipeline failures are warnings, not faults.
                let _ = io::Write::write_all(&mut *backend, payload);
            }
        }
    }

    impl<'a> MakeWriter<'a> for SyslogWriter {
        type Writer = SyslogEventWriter;
        fn make_writer(&'a self) -> Self::Writer {
            SyslogEventWriter {
                buf: Vec::new(),
                inner: Arc::clone(&self.inner),
            }
        }
    }

    /// Open a connection to the local syslog Unix socket. The `syslog`
    /// crate probes `/dev/log`, `/var/run/syslog`, and `/var/run/log`
    /// in turn.
    ///
    /// A dummy `Formatter3164` is passed because `syslog::unix`
    /// requires one; we discard it immediately and keep only the
    /// transport backend — RFC 5424 formatting is done by
    /// `smctl_log::formatter::Rfc5424`.
    pub fn open_unix_socket() -> Result<SyslogWriter, syslog::Error> {
        let dummy = Formatter3164 {
            facility: syslog::Facility::LOG_LOCAL0,
            hostname: None,
            process: "smctl".to_string(),
            pid: std::process::id(),
        };
        let logger = syslog::unix(dummy)?;
        Ok(SyslogWriter {
            inner: Arc::new(Mutex::new(logger.backend)),
        })
    }
}

#[cfg(not(target_family = "unix"))]
mod imp {
    use std::io;

    use tracing_subscriber::fmt::MakeWriter;

    /// Non-Unix stub. Kept so the public type exists on every platform
    /// and the `init` code path can reference it unconditionally.
    #[derive(Clone)]
    pub struct SyslogWriter;

    pub struct SyslogEventWriter;

    impl io::Write for SyslogEventWriter {
        fn write(&mut self, data: &[u8]) -> io::Result<usize> {
            Ok(data.len())
        }
        fn flush(&mut self) -> io::Result<()> {
            Ok(())
        }
    }

    impl<'a> MakeWriter<'a> for SyslogWriter {
        type Writer = SyslogEventWriter;
        fn make_writer(&'a self) -> Self::Writer {
            SyslogEventWriter
        }
    }

    /// Windows has no Unix syslog socket. Always fails; callers
    /// fall back to stderr.
    pub fn open_unix_socket() -> Result<SyslogWriter, io::Error> {
        Err(io::Error::new(
            io::ErrorKind::Unsupported,
            "syslog is not supported on this platform",
        ))
    }
}

pub use imp::{SyslogWriter, open_unix_socket};
