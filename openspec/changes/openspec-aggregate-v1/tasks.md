# OpenSpec aggregate — Tasks

## 1. smctl-spec crate API

- [x] 1.1 `smctl-workspace` already a path dep — no Cargo.toml change needed
- [x] 1.2 `RepoSpecInfo { repo, info: SpecInfo }` (uses `#[serde(flatten)]` so JSON is one level) and `RepoSpecRef { repo, name, openspec_dir }` with a `.qualified()` helper
- [x] 1.3 `ResolveError { NotFound { name }, Ambiguous { name, matches } }` via `thiserror`. The `Display` impl for `Ambiguous` lists every match's repo and shows one canonical qualified form as a remediation hint.
- [x] 1.4 `list_specs_across(repos)` walks each repo's `openspec/changes/`, skips repos whose openspec dir is absent (matches the existing single-root forgiveness), returns flat `Vec<RepoSpecInfo>`
- [x] 1.5 `find_spec_in_repos(repos, input)` — qualified `repo:name` direct lookup, bare-name unambiguous, bare-name ambiguous, not-found
- [x] 1.6 `archive_in_repo(openspec_dir, name)` — thin wrapper around the existing `archive` for documentation intent; same return type
- [x] 1.7 `inject_synthetic_workspace_repo(workspace_root, "openspec", repos)` — adds the `_workspace` entry when needed; de-dupes against an explicit repo by canonicalised path
- [x] 1.8 Eleven unit tests with two-repo fixtures: aggregation, empty input, repo-without-openspec, qualified-direct, bare-unambiguous, bare-ambiguous, not-found, qualified-unknown-repo, qualified-unknown-name, archive-into-repo-tree, synthetic-add, synthetic-skip-when-no-dir, synthetic-dedup-against-explicit

## 2. CLI dispatch

- [x] 2.1 Prelude builds `Vec<(String, PathBuf)>` from `manifest.repos` once, plus the `inject_synthetic_workspace_repo` synthetic entry, plus a final fallback for fresh-workspace `spec new` (zero registered repos and no openspec/ yet)
- [x] 2.2 `SpecCommands::List` calls `list_specs_across` + new `render_repo_specs` helper. Human form groups by repo with one heading per repo; JSON form is the flat array `serde_json` produces from `RepoSpecInfo` (top-level `repo` field)
- [x] 2.3 `SpecCommands::Validate / Status / Ff / Apply` each call the new `resolve_existing_spec` helper which honours `--repo` as sugar for `repo:name`; result piped into the existing single-root validate / spec_info helpers via the resolved `RepoSpecRef.openspec_dir`
- [x] 2.4 `SpecCommands::Archive` resolves with `resolve_existing_spec`, calls `archive_in_repo`, output reads `archived spec 'repo:name' to <dest>`
- [x] 2.5 `SpecCommands::New` accepts `--repo <name>`; resolution order in `resolve_new_target`: explicit flag → `smctl_home` repo → single-repo fallback → error with remediation listing the registered repos
- [x] 2.6 `ResolveError::Display` includes the three-part remediation per design-system-v1: NotFound suggests `smctl spec list`/`new`; Ambiguous lists every match's repo and shows the qualified `smctl spec validate repo:name` next-step
- [x] 2.7 Synthetic `_workspace` repo discovered via `inject_synthetic_workspace_repo`; legacy single-repo workspaces continue working without manifest changes

## 3. MCP tools

- [x] 3.1 `smctl_spec_list` tool: aggregates via `list_specs_across`; envelope shape `{ specs: [{repo, name, phase, ...}] }` — `RepoSpecInfo` flattens so the JSON is one level
- [x] 3.2 `smctl_spec_validate` tool: `SpecByNameParams { name, repo? }`; resolves via the new `SmctlServer::resolve_spec_ref` helper; envelope wraps the `ValidationResult` with a top-level `qualified` field
- [x] 3.3 `smctl_spec_archive` tool: same resolution; calls `archive_in_repo` so the archive lands in the repo's tree, not the workspace root; envelope reports `repo`, `name`, `qualified`, `archive_path`
- [x] 3.4 `smctl_spec_new` tool: same defaulting rules as the CLI (`repo` flag → `smctl_home` → single-repo); errors with three-part remediation when no rule matches
- [x] 3.5 stdio_handshake integration extended: scaffold a spec via `smctl_spec_new`, re-list and assert the entry carries `repo: "_workspace"`, validate via the qualified `_workspace:mcp-aggregate-fixture` form and assert the response carries the `qualified` field, and probe a missing-name error to confirm the `smctl spec list` remediation surfaces in the envelope

## 4. Workspace.toml & legacy compat

- [x] 4.1 `[spec] openspec_dir` schema unchanged — same value joined to each registered repo's path
- [x] 4.2 `inject_synthetic_workspace_repo` de-dupes via canonicalised path; covered by `inject_synthetic_workspace_repo_dedupes_against_explicit_repo` test in §1
- [ ] 4.3 CLAUDE.md note about per-repo openspec convention (lands with §6 docs)

## 5. CLI integration tests

- [x] 5.1 test_spec_list_aggregates_two_repos — human + JSON forms
- [x] 5.2 test_spec_validate_bare_unambiguous
- [x] 5.3 test_spec_validate_bare_ambiguous_lists_matches — error stderr cites both repos
- [x] 5.4 test_spec_validate_qualified_form — `RepoB:shared` picks the right one
- [x] 5.5 test_spec_archive_lands_in_owning_repo — RepoA's archive grows, RepoB's stays empty
- [x] 5.6 spec new smctl_home defaulting — covered by the existing test_spec_new_validate_archive (single-repo synthetic _workspace path)
- [x] 5.7 test_spec_new_with_repo_flag — explicit `--repo RepoB` lands in RepoB
- [x] 5.8 test_spec_new_unknown_repo_flag_errors_with_remediation — error stderr lists registered repos

Plus: test_spec_validate_with_repo_flag — `--repo X` plus bare name acts as `X:name`.

## 6. Docs

- [ ] 6.1 README spec section: per-repo behaviour, `repo:name` syntax, `--repo` flag
- [ ] 6.2 CLAUDE.md note about the per-repo openspec convention
- [ ] 6.3 Archive-time spec lift to `openspec/specs/smctl-spec/spec.md`

## 7. Verify

- [ ] 7.1 `cargo build --workspace` clean
- [ ] 7.2 `cargo test --workspace` clean — no regressions, new tests passing
- [ ] 7.3 `cargo clippy --workspace --all-targets -- -D warnings` clean
- [ ] 7.4 `cargo fmt --check` clean
- [ ] 7.5 `openspec validate openspec-aggregate-v1 --strict` passes
- [ ] 7.6 Manual smoke against an integration workspace (ModelGate + SmallAIOS) — `smctl spec list` returns specs from both
