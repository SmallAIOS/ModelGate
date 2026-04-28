# OpenSpec aggregate — Tasks

## 1. smctl-spec crate API

- [ ] 1.1 Add `smctl-workspace` as a path dep in `smctl-spec/Cargo.toml`
- [ ] 1.2 Define `RepoSpecInfo { repo, info: SpecInfo }` and `RepoSpecRef { repo, name, openspec_dir }`
- [ ] 1.3 Define `ResolveError { NotFound { name }, Ambiguous { matches: Vec<RepoSpecRef> } }`
- [ ] 1.4 Implement `list_specs_across(repos: &[(String, PathBuf)]) -> Result<Vec<RepoSpecInfo>>`
- [ ] 1.5 Implement `find_spec_in_repos(repos, name) -> Result<RepoSpecRef, ResolveError>` with the four-rule resolution table from design Decision 2
- [ ] 1.6 Implement `archive_in_repo(openspec_dir, name) -> PathBuf` — same shape as `archive` but the function asserts the path stays inside the given openspec dir
- [ ] 1.7 Implement `inject_synthetic_workspace_repo(workspace_root, &mut repos)` — adds the `_workspace` entry when the workspace root has its own `openspec/` and no registered repo already covers it (Decision 4)
- [ ] 1.8 Unit tests with two-repo fixtures: list aggregation, qualified resolve, ambiguous resolve, not-found resolve, archive moves to right tree, synthetic-_workspace add, synthetic-_workspace dedup

## 2. CLI dispatch

- [ ] 2.1 `Commands::Spec` dispatch: build the `Vec<(String, PathBuf)>` from `manifest.repos` once, then pass to every subcommand handler
- [ ] 2.2 `SpecCommands::List` calls `list_specs_across` and renders grouped (TTY) or flat JSON (non-TTY / `--json`)
- [ ] 2.3 `SpecCommands::Validate / Status / Ff / Apply` each call `find_spec_in_repos`, then dispatch into `smctl-spec`'s existing single-root helpers using the resolved `RepoSpecRef.openspec_dir`
- [ ] 2.4 `SpecCommands::Archive` resolves with `find_spec_in_repos`, then calls `archive_in_repo`
- [ ] 2.5 `SpecCommands::New` accepts `--repo <name>`, defaults to the `smctl_home: true` repo, falls back to the only registered repo, errors with remediation when both fall-throughs miss
- [ ] 2.6 Disambiguation error path renders the qualified `repo:name` matches with a remediation clause
- [ ] 2.7 Synthetic `_workspace` repo discovered via `inject_synthetic_workspace_repo` so the migration path is automatic

## 3. MCP tools

- [ ] 3.1 `smctl_spec_list` tool: empty params (workspace already known); returns `[{repo, name, phase, ...}]`
- [ ] 3.2 `smctl_spec_validate` tool: `{ name, repo? }` params; resolves with `find_spec_in_repos`
- [ ] 3.3 `smctl_spec_archive` tool: `{ name, repo? }` params; resolves then calls `archive_in_repo`
- [ ] 3.4 `smctl_spec_new` tool: `{ name, repo? }` — same defaulting rules as the CLI
- [ ] 3.5 stdio_handshake integration test: `smctl_spec_list` against a fixture with two repos returns aggregated entries; `smctl_spec_validate` with bare name resolves; ambiguous bare name returns an error envelope citing both matches

## 4. Workspace.toml & legacy compat

- [ ] 4.1 No schema change to `[spec] openspec_dir` — same value applies across every repo
- [ ] 4.2 Synthetic `_workspace` entry skipped when an explicit repo whose path is `<workspace_root>` is already registered
- [ ] 4.3 Document the legacy fallback in CLAUDE.md "Workspace conventions" subsection

## 5. CLI integration tests

- [ ] 5.1 `smctl/tests/cli.rs`: spec list aggregates two repos
- [ ] 5.2 spec validate with bare name unambiguous
- [ ] 5.3 spec validate with bare name ambiguous → error + matches listed
- [ ] 5.4 spec validate with `repo:name` qualified
- [ ] 5.5 spec archive moves into the correct repo's archive/ tree
- [ ] 5.6 spec new defaults to the smctl_home repo
- [ ] 5.7 spec new with `--repo` overrides
- [ ] 5.8 Synthetic `_workspace` discovered when no manifest is registered

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
