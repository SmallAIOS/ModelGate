# smctl-errors-v1 — Tasks

## Spec Documents

- [x] Author `proposal.md`
- [x] Author `design.md`
- [x] Author `tasks.md` (this file)

## Implementation — per audit site

Grouped by crate. Sites are drawn from `openspec/changes/design-system-v1/voice-audit.md` § Thin error messages. Line numbers are from the time of audit (2026-04-19); re-confirm during implementation.

### `smctl-workspace/src/lib.rs`

- [x] Line 230 — `"repo '{name}' already exists in workspace"` → add remediation (`smctl workspace remove <name>` or pick a different name)
- [x] Line 254 — `"repo '{name}' not found in workspace"` → add remediation (`smctl workspace status` to list)
- [x] Line 395–400 — worktree add failure — confirm the upstream git error is actionable; if not, wrap with remediation
- [x] Line 425 — `"worktree set '{name}' does not exist"` → add remediation (`smctl worktree list`)
- [x] Line 470 — same as 425 (second call site)

### `smctl-spec/src/lib.rs`

- [x] Line 40 — `"spec '{name}' already exists"` → add remediation (`smctl spec archive <name>` or use a different name)
- [x] Line 168 — `"spec '{name}' not found"` → add remediation (`smctl spec list`)
- [x] Line 189 — same as 168 (second call site)
- [x] Line 281 — same as 168 (third call site)

### `smctl-build/src/lib.rs`

- [x] Line 315 — `"empty command"` → add remediation (specify `build_command` in `workspace.toml` or remove the repo)

### `smctl/src/lib.rs`

- [x] Line 115 — `"unknown config key: {key}"` → add remediation (`smctl config --help` for valid keys)

### `smctl/src/main.rs`

- [x] Line 1020 — `"spec '{spec_name}' has no tasks.md"` → add remediation (`smctl spec ff <name>` to regenerate, or reinitialize with `smctl spec new`)

## Test updates

- [x] Grep tests for full-string matches on current error text and switch to stable-substring matches
- [x] Add or extend a test that asserts on the remediation clause for at least one representative error (e.g. `"smctl spec list"` appears in the `spec not found` error)
- [x] `cargo test --workspace` — all tests pass
- [x] `cargo clippy --workspace --all-targets -- -D warnings` clean
- [x] `cargo fmt --check` clean

## Scope expansion — sites found during implementation

After the initial audit-scoped pass landed, three adjacent thin or malformed errors surfaced during review. Decision 5 in `design.md` has been amended to allow in-scope uptake when the site is a direct peer of audited sites and the remediation shape is already covered by the catalog. The three sites below are now in scope.

- [x] `smctl-build/src/lib.rs` — `find_repo` lookup in build planning: add remediation pointing at `smctl workspace status` / `smctl workspace add`
- [x] `smctl-flow/src/lib.rs` — `find_branch` check in feature-start base validation: add remediation pointing at `smctl flow init` and `git branch -a`
- [x] `smctl-build/src/lib.rs` — `run_cmd` command-failed: collapse the multi-line `anyhow::bail!` to single-line period-separated form per Decision 3; extract first stderr line as a hint while pointing the operator to the direct command for full output

## Out-of-scope follow-ups (file as separate issues, do NOT expand scope here)

- [ ] Error-type refactor (`thiserror` + structured `SmctlError`) — own change
- [ ] Lint / CI check for three-part rubric on new error sites — revisit if drift
- [ ] Localization scaffolding — revisit only if a non-English consumer appears

## Archive

- [ ] Run `smctl spec archive smctl-errors-v1` when merged to `develop`
