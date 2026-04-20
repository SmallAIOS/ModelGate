//! RFC 5424 logging for the SmallAIOS / ModelGate / smctl ecosystem.
//!
//! Implements the `design-system-v1` Logging and Telemetry contract.
//! The concrete wire format and MSGID catalog live in
//! `openspec/changes/smctl-logging-v1/specs/logging.md`.
//!
//! # Typical use
//!
//! ```no_run
//! use smctl_log::{LoggingConfig, LogLevel};
//! let config = LoggingConfig { level: LogLevel::Info, ..Default::default() };
//! smctl_log::init(&config).expect("init logging");
//! tracing::info!(msgid = %smctl_log::MsgId::WorkspaceInitialized, path = "/tmp/ws", name = "demo", "initialized workspace");
//! ```

pub mod formatter;
pub mod msgid;
pub mod severity;
pub mod syslog_transport;

use std::fs::OpenOptions;
use std::path::PathBuf;
use std::sync::{Mutex, OnceLock};

use thiserror::Error;
use tracing_subscriber::Registry;
use tracing_subscriber::filter::LevelFilter;
use tracing_subscriber::fmt;
use tracing_subscriber::layer::SubscriberExt;

pub use formatter::Rfc5424;
pub use msgid::MsgId;
pub use severity::Severity;

/// `tracing` level selector. Mirrors `tracing::Level` but avoids forcing
/// callers to depend on `tracing` directly.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum LogLevel {
    Error,
    Warn,
    #[default]
    Info,
    Debug,
    Trace,
}

impl LogLevel {
    fn to_filter(self) -> LevelFilter {
        match self {
            LogLevel::Error => LevelFilter::ERROR,
            LogLevel::Warn => LevelFilter::WARN,
            LogLevel::Info => LevelFilter::INFO,
            LogLevel::Debug => LevelFilter::DEBUG,
            LogLevel::Trace => LevelFilter::TRACE,
        }
    }
}

impl std::str::FromStr for LogLevel {
    type Err = LogError;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_ascii_lowercase().as_str() {
            "error" => Ok(LogLevel::Error),
            "warn" | "warning" => Ok(LogLevel::Warn),
            "info" | "informational" => Ok(LogLevel::Info),
            "debug" => Ok(LogLevel::Debug),
            "trace" => Ok(LogLevel::Trace),
            other => Err(LogError::BadLevel(other.to_string())),
        }
    }
}

/// Subscriber initialization config.
#[derive(Debug, Clone)]
pub struct LoggingConfig {
    /// Minimum severity to emit. Applies uniformly to every transport.
    pub level: LogLevel,

    /// Emit to stderr. Default true.
    pub stderr: bool,

    /// If set, also emit to this file (append-only).
    pub file: Option<PathBuf>,

    /// If true, also emit to the local syslog Unix socket. Unix only.
    /// On Windows, specifying `true` triggers a one-time WARN and the
    /// syslog layer is a no-op; stderr is activated if it was not
    /// already. Default false.
    pub syslog: bool,

    /// Syslog facility for the PRI field. `local0` (16) by default.
    pub facility: u8,

    /// APP-NAME for the RFC 5424 header.
    pub app_name: String,
}

impl Default for LoggingConfig {
    fn default() -> Self {
        Self {
            level: LogLevel::Info,
            stderr: true,
            file: None,
            syslog: false,
            facility: 16, // local0
            app_name: "smctl".to_string(),
        }
    }
}

#[derive(Debug, Error)]
pub enum LogError {
    #[error("unknown log level: {0}")]
    BadLevel(String),

    #[error("failed to open log file at {path}: {source}")]
    OpenFile {
        path: PathBuf,
        #[source]
        source: std::io::Error,
    },

    #[error("tracing subscriber already installed")]
    AlreadyInstalled,
}

static INSTALLED: OnceLock<()> = OnceLock::new();

/// Install the RFC 5424 subscriber as the global default.
///
/// Idempotent: the second and subsequent calls are a no-op and return
/// `Ok(())`. Does not panic on a second call; returns success silently
/// to match the idempotency guarantee in the spec.
///
/// Syslog open-failure handling: per `specs/logging.md`, if the
/// `syslog` transport was requested but the local Unix socket cannot
/// be opened, this function does NOT fail. Instead it forces the
/// stderr transport on, installs the subscriber, and emits a single
/// `SMCTL-0099` WARN describing the fallback. Same policy on Windows,
/// where syslog is unsupported.
pub fn init(config: &LoggingConfig) -> Result<(), LogError> {
    if INSTALLED.get().is_some() {
        return Ok(());
    }

    let fmt_event = Rfc5424::new(config.facility, &config.app_name);

    // Resolve the syslog transport first. If it fails, we flip stderr
    // on and record the reason so we can emit a WARN after the
    // subscriber is installed.
    let mut effective_stderr = config.stderr;
    let mut syslog_fallback_reason: Option<String> = None;
    let syslog_layer = if config.syslog {
        match syslog_transport::open_unix_socket() {
            Ok(writer) => Some(
                fmt::Layer::default()
                    .event_format(fmt_event.clone())
                    .with_writer(writer)
                    .with_ansi(false),
            ),
            Err(err) => {
                effective_stderr = true;
                syslog_fallback_reason = Some(format!("{err}"));
                None
            }
        }
    } else {
        None
    };

    let stderr_layer = effective_stderr.then(|| {
        fmt::Layer::default()
            .event_format(fmt_event.clone())
            .with_writer(std::io::stderr)
            .with_ansi(false)
    });

    let file_layer = if let Some(path) = &config.file {
        if let Some(parent) = path.parent()
            && !parent.as_os_str().is_empty()
        {
            std::fs::create_dir_all(parent).map_err(|source| LogError::OpenFile {
                path: path.clone(),
                source,
            })?;
        }
        let file = OpenOptions::new()
            .create(true)
            .append(true)
            .open(path)
            .map_err(|source| LogError::OpenFile {
                path: path.clone(),
                source,
            })?;
        let writer = Mutex::new(file);
        Some(
            fmt::Layer::default()
                .event_format(fmt_event.clone())
                .with_writer(writer)
                .with_ansi(false),
        )
    } else {
        None
    };

    let subscriber = Registry::default()
        .with(config.level.to_filter())
        .with(stderr_layer)
        .with(file_layer)
        .with(syslog_layer);

    tracing::subscriber::set_global_default(subscriber).map_err(|_| LogError::AlreadyInstalled)?;

    let _ = INSTALLED.set(());

    // One-time WARN after the subscriber is live so it lands on every
    // active transport (including the now-forced stderr). Does not
    // panic, does not propagate.
    if let Some(reason) = syslog_fallback_reason {
        tracing::warn!(
            msgid = %MsgId::Uncategorized,
            reason = %reason,
            "syslog transport unavailable; falling back to stderr"
        );
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn log_level_parses_canonical_names() {
        assert_eq!("error".parse::<LogLevel>().unwrap(), LogLevel::Error);
        assert_eq!("WARN".parse::<LogLevel>().unwrap(), LogLevel::Warn);
        assert_eq!("warning".parse::<LogLevel>().unwrap(), LogLevel::Warn);
        assert_eq!("Info".parse::<LogLevel>().unwrap(), LogLevel::Info);
        assert_eq!("informational".parse::<LogLevel>().unwrap(), LogLevel::Info);
        assert_eq!("debug".parse::<LogLevel>().unwrap(), LogLevel::Debug);
        assert_eq!("trace".parse::<LogLevel>().unwrap(), LogLevel::Trace);
    }

    #[test]
    fn log_level_rejects_garbage() {
        assert!("banana".parse::<LogLevel>().is_err());
    }

    #[test]
    fn default_config_is_stderr_info_local0() {
        let c = LoggingConfig::default();
        assert!(c.stderr);
        assert!(c.file.is_none());
        assert_eq!(c.level, LogLevel::Info);
        assert_eq!(c.facility, 16);
        assert_eq!(c.app_name, "smctl");
    }
}
