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

## Out-of-scope follow-ups (file as separate issues, do NOT expand scope here)

- [ ] Error-type refactor (`thiserror` + structured `SmctlError`) — own change
- [ ] Lint / CI check for three-part rubric on new error sites — revisit if drift
- [ ] Localization scaffolding — revisit only if a non-English consumer appears
- [ ] `smctl-build/src/lib.rs:143` — `.with_context(|| format!("repo '{name}' not found"))` on build-plan lookup. Thin error, not in the original audit. Belongs to a `smctl-errors-v2` follow-up pass.
- [ ] `smctl-flow/src/lib.rs:324` — `"base branch '{base}' not found in {}"` context on `find_branch`. Thin error, not in the original audit. Belongs to a `smctl-errors-v2` follow-up pass.
- [ ] `smctl-build/src/lib.rs:327` — `"{}: command '{}' failed:\n{}"` multi-line with embedded stderr. Preserves git-style output but drifts from the single-line period-separated convention (Decision 3). Could be reformatted in a follow-up.

## Archive

- [ ] Run `smctl spec archive smctl-errors-v1` when merged to `develop`
