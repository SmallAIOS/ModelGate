//! MCP resources exposed by `smctl-mcp`.
//!
//! Resources are the read-only half of the MCP surface: they publish
//! snapshots of workspace state under the `smctl://` URI scheme. Tools
//! (defined in [`crate::server`]) drive workspace mutations; resources
//! let clients poll the resulting state without a tool round-trip.
//!
//! The five resources defined here match `smctl-mcp-v1/tasks.md`:
//!
//! - `smctl://workspace/config` — manifest TOML (application/toml)
//! - `smctl://workspace/status` — live per-repo status (application/json)
//! - `smctl://flow/branches` — active flow branches (application/json)
//! - `smctl://spec/list` — every spec with phase + task progress (application/json)
//! - `smctl://spec/{name}/tasks` — per-spec task progress, templated URI
//!
//! The static-URI resources are advertised through `resources/list`;
//! `smctl://spec/{name}/tasks` is advertised through
//! `resources/templates/list` because its URI is parameterized.
//!
//! Error payloads follow the `design-system-v1` three-part rubric (fact
//! → meaning → executable `smctl` remediation) — clients relay these
//! strings verbatim, so the voice contract applies at the edge.

use std::path::{Path, PathBuf};

use rmcp::ErrorData;
use rmcp::model::{
    Annotated, RawResource, RawResourceTemplate, ReadResourceResult, Resource, ResourceContents,
    ResourceTemplate,
};

/// MIME types declared in the resource metadata and attached to the
/// returned [`ResourceContents`]. The TOML payload carries
/// `application/toml`; everything else is JSON.
pub mod mime {
    pub const TOML: &str = "application/toml";
    pub const JSON: &str = "application/json";
}

/// Static URIs advertised via `resources/list`.
pub const URI_WORKSPACE_CONFIG: &str = "smctl://workspace/config";
pub const URI_WORKSPACE_STATUS: &str = "smctl://workspace/status";
pub const URI_FLOW_BRANCHES: &str = "smctl://flow/branches";
pub const URI_SPEC_LIST: &str = "smctl://spec/list";

/// Templated URI advertised via `resources/templates/list`.
pub const URI_TEMPLATE_SPEC_TASKS: &str = "smctl://spec/{name}/tasks";

/// Static resources exposed by the server.
///
/// Sentence-case descriptions, imperative / declarative voice, no
/// emoji — these strings ship to MCP clients verbatim.
pub fn static_resources() -> Vec<Resource> {
    vec![
        Annotated::new(
            RawResource::new(URI_WORKSPACE_CONFIG, "workspace config")
                .with_title("Workspace manifest")
                .with_description(
                    "The workspace.toml manifest for the active workspace. \
                     Read-only snapshot of the configured repos, flow prefixes, \
                     worktree base, and spec directory.",
                )
                .with_mime_type(mime::TOML),
            None,
        ),
        Annotated::new(
            RawResource::new(URI_WORKSPACE_STATUS, "workspace status")
                .with_title("Workspace status")
                .with_description(
                    "Per-repo status across the workspace: branch, clean or dirty, \
                     ahead and behind counts, modified-file count. Equivalent to \
                     the `smctl_workspace_status` tool, delivered as a resource.",
                )
                .with_mime_type(mime::JSON),
            None,
        ),
        Annotated::new(
            RawResource::new(URI_FLOW_BRANCHES, "flow branches")
                .with_title("Active flow branches")
                .with_description(
                    "Feature, release, and hotfix branches currently present in \
                     each workspace repo. Classified against the configured flow \
                     prefixes.",
                )
                .with_mime_type(mime::JSON),
            None,
        ),
        Annotated::new(
            RawResource::new(URI_SPEC_LIST, "spec list")
                .with_title("OpenSpec changes")
                .with_description(
                    "Every OpenSpec change in the workspace with its phase and \
                     task-completion counts. Archived specs are included.",
                )
                .with_mime_type(mime::JSON),
            None,
        ),
    ]
}

/// Templated resources exposed by the server.
///
/// `spec/{name}/tasks` is parameterized by spec name and cannot be
/// enumerated statically; advertise it as a template so clients know
/// how to construct a readable URI.
pub fn resource_templates() -> Vec<ResourceTemplate> {
    vec![Annotated::new(
        RawResourceTemplate::new(URI_TEMPLATE_SPEC_TASKS, "spec tasks")
            .with_title("Spec task progress")
            .with_description(
                "Task-completion counts and the raw tasks.md body for a single \
                 OpenSpec change. Substitute {name} with the spec identifier \
                 (matching the directory under openspec/changes/).",
            )
            .with_mime_type(mime::JSON),
        None,
    )]
}

/// Classifier for `resources/read` dispatch failures. Used to populate
/// the `error_kind` STRUCTURED-DATA field on `SMCTL-0208` and to pick
/// the right JSON-RPC error code on the wire.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ReadErrorKind {
    /// URI did not match any static or templated resource.
    UnknownUri,
    /// URI matched the spec template but the spec name was empty or
    /// contained a path separator.
    InvalidUri,
    /// Workspace manifest was not found or failed to parse.
    ManifestMissing,
    /// Underlying smctl library call returned an error.
    UpstreamFailed,
    /// Payload could not be serialized to JSON.
    SerializationFailed,
}

impl ReadErrorKind {
    /// Stable snake_case identifier for logging.
    pub fn as_str(self) -> &'static str {
        match self {
            Self::UnknownUri => "unknown_uri",
            Self::InvalidUri => "invalid_uri",
            Self::ManifestMissing => "manifest_missing",
            Self::UpstreamFailed => "upstream_failed",
            Self::SerializationFailed => "serialization_failed",
        }
    }
}

/// Parse a `smctl://spec/{name}/tasks` URI and return the spec name.
/// Returns `None` if the URI does not match the template shape.
/// Rejects empty names and names containing path separators so the
/// upstream spec loader cannot be driven into filesystem traversal.
pub fn parse_spec_tasks_uri(uri: &str) -> Option<&str> {
    let rest = uri.strip_prefix("smctl://spec/")?;
    let name = rest.strip_suffix("/tasks")?;
    if name.is_empty() || name.contains('/') {
        return None;
    }
    Some(name)
}

/// Wrap a JSON value into the `ReadResourceResult` envelope with the
/// given URI and MIME type.
pub fn json_result(uri: &str, value: &serde_json::Value) -> Result<ReadResourceResult, ErrorData> {
    let text = serde_json::to_string_pretty(value).map_err(|e| {
        read_failed(
            uri,
            format!(
                "Resource serialization failed. {e}. \
                 Retry the read or run the equivalent `smctl` subcommand to reproduce."
            ),
        )
    })?;
    Ok(ReadResourceResult::new(vec![
        ResourceContents::TextResourceContents {
            uri: uri.to_string(),
            mime_type: Some(mime::JSON.to_string()),
            text,
            meta: None,
        },
    ]))
}

/// Wrap a TOML string into the `ReadResourceResult` envelope.
pub fn toml_result(uri: &str, content: String) -> ReadResourceResult {
    ReadResourceResult::new(vec![ResourceContents::TextResourceContents {
        uri: uri.to_string(),
        mime_type: Some(mime::TOML.to_string()),
        text: content,
        meta: None,
    }])
}

/// Construct an internal-error `ErrorData` with a three-part message.
/// Used when the read handler itself fails (not for unknown-URI cases,
/// which should use [`ErrorData::resource_not_found`]).
pub fn read_failed(uri: &str, message: impl Into<String>) -> ErrorData {
    ErrorData::internal_error(format!("[{uri}] {}", message.into()), None)
}

/// Read `.smctl/workspace.toml` from the given root. Returns a
/// three-part error payload when the manifest is missing.
pub fn read_workspace_toml(root: &Path) -> Result<String, ErrorData> {
    let path: PathBuf = root.join(".smctl").join("workspace.toml");
    std::fs::read_to_string(&path).map_err(|e| {
        ErrorData::resource_not_found(
            format!(
                "Workspace manifest not found at {}. {e}. \
                 Run `smctl workspace init` in {} to create it.",
                path.display(),
                root.display()
            ),
            None,
        )
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn static_resources_cover_the_declared_uris() {
        let uris: Vec<_> = static_resources()
            .into_iter()
            .map(|r| r.raw.uri.clone())
            .collect();
        for expected in [
            URI_WORKSPACE_CONFIG,
            URI_WORKSPACE_STATUS,
            URI_FLOW_BRANCHES,
            URI_SPEC_LIST,
        ] {
            assert!(
                uris.iter().any(|u| u == expected),
                "static resources should include {expected}, got {uris:?}"
            );
        }
    }

    #[test]
    fn templates_advertise_spec_tasks() {
        let templates = resource_templates();
        assert_eq!(templates.len(), 1);
        assert_eq!(templates[0].raw.uri_template, URI_TEMPLATE_SPEC_TASKS);
        assert_eq!(templates[0].raw.mime_type.as_deref(), Some(mime::JSON));
    }

    #[test]
    fn parse_spec_tasks_uri_accepts_simple_name() {
        assert_eq!(
            parse_spec_tasks_uri("smctl://spec/smctl-mcp-v1/tasks"),
            Some("smctl-mcp-v1")
        );
    }

    #[test]
    fn parse_spec_tasks_uri_rejects_nested_path() {
        assert_eq!(
            parse_spec_tasks_uri("smctl://spec/foo/bar/tasks"),
            None,
            "nested spec names must be rejected to avoid path traversal"
        );
    }

    #[test]
    fn parse_spec_tasks_uri_rejects_other_shapes() {
        assert!(parse_spec_tasks_uri("smctl://spec/list").is_none());
        assert!(parse_spec_tasks_uri("smctl://spec//tasks").is_none());
        assert!(parse_spec_tasks_uri("smctl://workspace/status").is_none());
    }

    #[test]
    fn read_error_kind_as_str_is_stable_snake_case() {
        assert_eq!(ReadErrorKind::UnknownUri.as_str(), "unknown_uri");
        assert_eq!(ReadErrorKind::InvalidUri.as_str(), "invalid_uri");
        assert_eq!(ReadErrorKind::ManifestMissing.as_str(), "manifest_missing");
        assert_eq!(ReadErrorKind::UpstreamFailed.as_str(), "upstream_failed");
    }
}
