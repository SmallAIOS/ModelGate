# OpenSpec aggregate — Design Document

## Context

`smctl-spec`'s public API today takes a single `openspec_dir: &Path`:

```rust
pub fn list_specs(openspec_dir: &Path) -> Result<Vec<SpecInfo>>;
pub fn validate(openspec_dir: &Path, name: &str) -> Result<...>;
pub fn archive(openspec_dir: &Path, name: &str) -> Result<PathBuf>;
```

The `smctl` CLI feeds this from `manifest.spec.openspec_dir` joined to the workspace root. That works for a single-repo workspace but produces `no specs found` against any multi-repo workspace because each repo carries its own `openspec/`.

The fix has three layers — crate API, CLI dispatch, MCP tool schema. The crate API is the foundation; if it's right, the other two follow naturally.

## Goals / Non-Goals

### Goals

1. `smctl spec list` enumerates every active spec across every registered repo, prefixed with the repo name.
2. Single-spec subcommands (`validate`, `apply`, `archive`, `status`, `ff`) resolve a name across repos with explicit disambiguation rules.
3. `smctl spec new` writes into a specified repo (`--repo <name>`, or the `smctl_home` repo, or — in single-repo workspaces — the only entry).
4. `smctl_spec_list` / `_validate` / `_archive` MCP tools inherit the aggregation.
5. Backward compatibility for repos that have a workspace-level `openspec/` (a synthetic `_workspace` repo).
6. Every change covered by unit tests in `smctl-spec` plus an integration test in `smctl/tests/cli.rs`.

### Non-Goals

1. Renaming specs across repos (no current API).
2. Cross-repo spec dependencies (specs reference each other by relative path; nothing changes).
3. Per-repo `openspec_dir` overrides — workspace.toml has one `[spec] openspec_dir`, applied to every repo. A future change can add per-repo overrides if real workspaces want them.
4. Watching the openspec tree for changes (smctl-mcp already polls; no need for `subscribe`).

## Decisions

### Decision 1: Crate API takes a slice of `(name, root)` pairs, not the manifest

**Choice:** New `smctl-spec` API surface accepts `&[(repo_name, openspec_dir)]` rather than `&WorkspaceManifest`. The CLI is responsible for assembling the slice from the manifest.

**Rationale:** Keeps `smctl-spec` independent of `smctl-workspace`'s manifest schema. Tests can pass arbitrary fixture trees without constructing a `WorkspaceManifest`. The CLI's manifest-walking logic stays in `main.rs` where it already lives for every other subcommand.

**Trade-off:** Slightly more verbose call sites. Acceptable — there are five of them, all in `smctl/src/main.rs`.

### Decision 2: Spec name resolution rules

**Choice:** Single-spec subcommands accept either bare names or `repo:name` qualified names. Resolution is:

1. If the input contains `:`, split into `(repo, name)` and look up directly.
2. If the input is bare and **exactly one** registered repo holds a spec with that name, use it.
3. If the input is bare and **multiple** repos hold a spec with that name, error with a remediation clause that lists the matches and tells the operator to qualify with `repo:name`.
4. If no repo holds the name, error with a remediation clause that runs `smctl spec list` to enumerate.

**Rationale:** Matches how `kubectl` resolves resource names against namespaces. The unambiguous-bare-name case keeps the common path short; the disambiguation path stays explicit. Operators don't need to memorise `repo:` prefixes when working in a single-repo workspace.

### Decision 3: `spec new` defaults to the `smctl_home` repo

**Choice:** `smctl spec new <name>` writes into the repo whose `[[repos]]` entry has `smctl_home = true`. When no `smctl_home` is declared, the operator must pass `--repo <name>` explicitly. Single-repo workspaces (one `[[repos]]` entry, no `smctl_home`) default to that repo.

**Rationale:** The shakedown showed every smctl-using workspace declares one repo as the smctl host. That's the natural place for new specs about smctl itself. Specs that affect a sibling repo go through `--repo`, which is no worse than today's status quo (operators already cd into the target repo).

### Decision 4: Synthetic `_workspace` repo for legacy layouts

**Choice:** When the workspace root has its own `openspec/` directory and that directory contains active changes, the aggregator inserts a synthetic repo entry named `_workspace` whose `openspec_dir` is the workspace-level path.

**Rationale:** ModelGate (this repo) currently runs in a single-repo workspace mode where `.smctl/workspace.toml` is absent and `openspec/` lives at the repo root. The synthetic entry preserves that path during migration. Once a workspace registers an explicit `[[repos]]` entry covering the same path, the synthetic entry is dropped (de-duplicated by absolute path).

**Trade-off:** A small amount of legacy plumbing. Documented as deprecated; can be removed once every workspace has migrated. Concrete deprecation timeline: when SmallAIOS itself opens a manifest declaring its own repo entry, this repo follows.

### Decision 5: JSON output gains a top-level `repo` field

**Choice:** `smctl spec list --json` and the MCP `smctl_spec_list` tool return a flat array of `{ repo, name, phase, tasks_total, tasks_done, validation, ... }`. No grouping.

**Rationale:** Flat is easier for downstream tooling. Callers who want grouping can `jq 'group_by(.repo)'`. Matches how `smctl workspace status --json` already returns a flat array of repo statuses.

### Decision 6: `spec list` human output is grouped by repo

**Choice:** When stdout is a TTY, `spec list` prints one section per repo with a heading line:

```
ModelGate
  feature-x          phase=ff      tasks 3/12
  feature-y          phase=apply   tasks 8/8

SmallAIOS
  kernel-thing       phase=ff      tasks 0/5
```

**Rationale:** Operators read this output, not machines. The grouping reflects the mental model. The flat JSON shape stays for scripts.

### Decision 7: `spec archive` uses the qualified name

**Choice:** `smctl spec archive <name>` requires the same disambiguation as `validate` etc. After archiving, the synthetic prefix in the output reads `<repo>:<name>` so the operator sees which repo's tree was modified.

**Rationale:** Specs are archived into the repo that owns them; the qualifier is information the operator needs to confirm the right repo was touched. Symmetric with the rest of the single-spec verbs.

## Risks / Trade-offs

| Risk | Mitigation |
|---|---|
| Existing single-repo workspaces break on upgrade | Synthetic `_workspace` repo (Decision 4); call out as a smoke test in tasks.md |
| Disambiguation errors are noisy when two repos declare the same spec name | Error message lists every match with the qualified form; remediation tells the operator how to retry |
| `smctl-spec` tests grow combinatorially with multi-repo fixtures | Helper in tests sets up two-repo trees; reuse across tests |
| MCP schema change is a breaking-ish API tweak for any client that called the old shape | Old tools accept the old call shape (no `repo` field) — falls through to the unambiguous-bare-name resolution. New `repo` field is optional. |

## Open Questions

1. Should `spec list` filter on `--phase <name>` so operators can see "only `ff`" or "only `apply`"? Out of scope for this change; trivial follow-up.
2. Should the synthetic `_workspace` repo emit a deprecation warning? Yes, once per process, but it's easy enough to add later.
3. Does `spec ff <name>` need a `--repo` flag too? Yes — adding it for symmetry with the other verbs. No new behaviour, just consistency.
