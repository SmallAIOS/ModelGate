# OpenSpec aggregate — Proposal

## Why

`smctl spec` operates on a single workspace-level `openspec/` directory: `smctl_spec::list_specs` is invoked with `root.join(&manifest.spec.openspec_dir)` from one place. That model breaks the moment a workspace registers more than one repo. The `smallaios-integration-v1` shakedown ([archived 2026-04-25](../archive/2026-04-25-smallaios-integration-v1)) ran the canonical `smctl spec list` against an integration workspace containing both ModelGate and SmallAIOS — each with its own `openspec/changes/` tree — and got back `no specs found`. The findings file flagged it as the only release-blocker-grade gap surfaced by that exercise.

Concrete problems:

- **`spec list` is silent on real specs.** Every active proposal in every registered repo is invisible.
- **`spec validate <name>` can't resolve a spec by name** — it expects the workspace root to own the openspec tree, not the repos.
- **`spec archive <name>` writes to the wrong place** — silently moves files under the workspace root if they happen to exist there, otherwise errors.
- **`smctl-mcp`'s `smctl_spec_list` MCP tool inherits the same blindness.**

The fix is architectural, not cosmetic: the openspec tree lives in each repo, never in the workspace.

## What Changes

Make `smctl spec` aggregate across each registered repo's own `<repo>/openspec/` directory. The output prefixes each spec name with `<repo>:` so callers can disambiguate. Single-spec subcommands (`validate`, `apply`, `archive`, `status`, `ff`) accept either the bare name (when unambiguous) or the qualified `repo:name` form.

- New `smctl-spec` capability spec declaring per-repo aggregation as the canonical behaviour.
- `smctl-spec` crate: new `list_specs_across` / `find_spec_in_repos` / `archive_in_repo` API surface that takes a list of repo roots rather than one workspace root. The single-root helpers stay for callers that already know the repo (e.g. `smctl spec new` always scaffolds into the active repo).
- `smctl` CLI: every spec subcommand reads `manifest.repos`, walks each repo's `openspec_dir`, and aggregates. Single-spec verbs resolve names with disambiguation rules.
- `smctl spec new <name> [--repo <r>]`: defaults to the `smctl_home: true` repo when no flag is set; errors with a remediation clause when no `smctl_home` is declared and `--repo` is missing.
- `smctl-mcp`: `smctl_spec_list` returns the aggregated list with repo prefixes; `smctl_spec_validate` accepts both bare and qualified names; `smctl_spec_archive` likewise.
- Output formats: human shows a per-repo header; `--json` returns `[{ "repo": "...", "name": "...", "phase": "...", ... }]` with the repo as a top-level field.
- Backward compatibility: a workspace-level `openspec/` (the legacy layout this repo currently uses) is still discovered as a synthetic repo named `_workspace`, so single-repo workspaces don't lose access to their specs during migration.

## Capabilities

### New Capabilities

- `smctl-spec`: aggregating spec discovery, name resolution across repos (bare + qualified), per-repo archive, single-source-of-truth crate API for list/validate/apply/archive/status/ff.

### Modified Capabilities

- `smctl-cli`: gains the per-repo spec dispatch behaviour as MODIFIED Requirements on the existing subcommand surface. New `--repo <name>` flag on the spec subcommand tree.
- `smctl-mcp`: `smctl_spec_list` / `smctl_spec_validate` / `smctl_spec_archive` now take optional `repo` parameters and return aggregated results.

## Impact

### New Files

```
openspec/changes/openspec-aggregate-v1/
├── .openspec.yaml
├── proposal.md
├── design.md
├── specs/
│   ├── smctl-spec/spec.md   (ADDED — net-new capability)
│   ├── smctl-cli/spec.md    (MODIFIED — spec subcommand surface)
│   └── smctl-mcp/spec.md    (MODIFIED — three tool schemas)
└── tasks.md
```

### Modified Files

- `smctl-spec/src/lib.rs` — new public functions, existing single-root helpers stay
- `smctl-spec/Cargo.toml` — add `smctl-workspace` as a path dep so spec resolution can read the manifest
- `smctl/src/main.rs` — every `Commands::Spec` arm
- `smctl-mcp/src/server.rs` — `spec_list` / `spec_validate` / `spec_archive` tool handlers + their input schema structs
- `openspec/specs/smctl-spec/spec.md` — new capability spec, lifted at archive time
- `README.md` — `spec` section gains the per-repo behaviour and the `repo:name` qualifier syntax

### Out of Scope

- Cross-repo spec dependencies (a spec in one repo referencing a spec in another). Specs already point at sibling specs by relative path; nothing here changes that.
- Renaming specs across repos. `smctl-spec` doesn't currently expose rename; this change keeps that gap.
- Migrating ModelGate's existing `openspec/` to a "real" repo entry. ModelGate is its own repo and its `openspec/` is the per-repo tree this proposal makes canonical. The synthetic `_workspace` repo is purely for legacy / single-repo workspaces that haven't registered the repo yet.

## References

- Integration shakedown that surfaced this: [`openspec/changes/archive/2026-04-25-smallaios-integration-v1/specs/integration-checklist.md`](../archive/2026-04-25-smallaios-integration-v1/specs/integration-checklist.md), Phase 4.
- Current `smctl-spec` API: [`smctl-spec/src/lib.rs`](../../../smctl-spec/src/lib.rs).
- `smctl_spec_list` MCP tool: [`smctl-mcp/src/server.rs`](../../../smctl-mcp/src/server.rs).
