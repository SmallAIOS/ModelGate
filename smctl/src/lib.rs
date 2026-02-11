use serde::Serialize;

/// Output format for smctl commands.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum OutputFormat {
    Human,
    Json,
}

/// Format a serializable value according to the output format.
pub fn format_output<T: Serialize + std::fmt::Display>(value: &T, format: OutputFormat) -> String {
    match format {
        OutputFormat::Human => value.to_string(),
        OutputFormat::Json => {
            serde_json::to_string_pretty(value).unwrap_or_else(|_| value.to_string())
        }
    }
}

/// Format a serializable value for JSON output, with a human fallback closure.
pub fn format_output_with<T: Serialize, F: FnOnce(&T) -> String>(
    value: &T,
    format: OutputFormat,
    human_fmt: F,
) -> String {
    match format {
        OutputFormat::Human => human_fmt(value),
        OutputFormat::Json => serde_json::to_string_pretty(value)
            .unwrap_or_else(|e| format!("{{\"error\": \"{e}\"}}")),
    }
}

/// Exit codes for smctl.
pub mod exit_code {
    pub const SUCCESS: i32 = 0;
    pub const GENERAL_ERROR: i32 = 1;
    pub const USAGE_ERROR: i32 = 2;
    pub const GIT_ERROR: i32 = 3;
    pub const WORKSPACE_ERROR: i32 = 4;
    pub const SPEC_ERROR: i32 = 5;
    pub const BUILD_ERROR: i32 = 6;
    pub const NETWORK_ERROR: i32 = 7;
    pub const DRY_RUN: i32 = 10;
}
