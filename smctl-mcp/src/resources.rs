//! MCP resources exposed by `smctl-mcp`.
//!
//! Resources are the read-only half of the MCP surface: they publish
//! snapshots of workspace state under the `smctl://` URI scheme. Tools
//! (defined in [`crate::server`]) drive workspace mutations; resources
//! let clients poll the resulting state without a tool round-trip.
//!
//! Workspace-scoped resources live here in the first commit:
//!
//! - `smctl://workspace/config` — manifest TOML (application/toml)
//! - `smctl://workspace/status` — live per-repo status (application/json)
//! - `smctl://flow/branches` — active flow branches (application/json)
//!
//! Spec-scoped resources (`smctl://spec/list`, the templated
//! `smctl://spec/{name}/tasks`) land in a follow-up commit on the same
//! branch.
//!
//! Error payloads follow the `design-system-v1` three-part rubric (fact
//! → meaning → executable `smctl` remediation) — clients relay these
//! strings verbatim, so the voice contract applies at the edge.

use std::path::{Path, PathBuf};

use rmcp::ErrorData;
use rmcp::model::{
    Annotated, RawResource, ReadResourceResult, Resource, ResourceContents, ResourceTemplate,
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
    ]
}

/// Templated resources exposed by the server.
///
/// No templates are advertised in this commit; spec-tasks lands with
/// the spec-scoped URIs in the follow-up.
pub fn resource_templates() -> Vec<ResourceTemplate> {
    Vec::new()
}

/// Classifier for `resources/read` dispatch failures. Used to populate
/// the `error_kind` STRUCTURED-DATA field on `SMCTL-0208` and to pick
/// the right JSON-RPC error code on the wire.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ReadErrorKind {
    /// URI did not match any static or templated resource.
    UnknownUri,
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
            Self::ManifestMissing => "manifest_missing",
            Self::UpstreamFailed => "upstream_failed",
            Self::SerializationFailed => "serialization_failed",
        }
    }
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
        ] {
            assert!(
                uris.iter().any(|u| u == expected),
                "static resources should include {expected}, got {uris:?}"
            );
        }
    }

    #[test]
    fn templates_are_empty_in_this_commit() {
        assert!(resource_templates().is_empty());
    }

    #[test]
    fn read_error_kind_as_str_is_stable_snake_case() {
        assert_eq!(ReadErrorKind::UnknownUri.as_str(), "unknown_uri");
        assert_eq!(ReadErrorKind::ManifestMissing.as_str(), "manifest_missing");
        assert_eq!(ReadErrorKind::UpstreamFailed.as_str(), "upstream_failed");
    }
}
