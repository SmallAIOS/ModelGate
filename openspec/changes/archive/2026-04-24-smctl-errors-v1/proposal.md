# smctl-errors-v1 — Proposal

## Why

The voice audit in `design-system-v1` catalogued 15+ sites across the `smctl-*` crates where `anyhow::bail!` and equivalent error paths violate the three-part error-message rubric the design system requires:

> 1. What happened
> 2. What it means
> 3. What to do next — as an **executable command**, not vague advice.

Today most of those sites have only part 1. "repo 'foo' not found in workspace" tells the operator the fact but not what to type next. When the tool is being used by AI coding assistants via MCP, thin errors are doubly costly: the assistant guesses at remediation and wastes turns.

Good error copy is the second-cheapest improvement in the tool (after the nine-site copy fix already done as `smctl-copy-v1`) and the one with the highest compounding return: every future error-path addition lands into a better baseline.

## What Changes

1. **Restructure every audit-flagged error message** to include all three parts. The "what to do next" segment is an executable `smctl` invocation the operator can run literally.
2. **Introduce a small helper in `smctl-log`** (or a fresh module) that renders structured errors consistently: `error, meaning, remediation` triples.
3. **Update tests that assert on exact error text** to match the new forms. Prefer matching a stable substring (e.g. the MSGID or a specific phrase) rather than the full string, so future wording tweaks don't cascade.

## Capabilities

### Modified Capabilities

- `smctl-workspace` — Error paths in `add_repo`, `remove_repo`, `add_worktree`, `remove_worktree`, and manifest-load gain remediation hints.
- `smctl-spec` — Error paths in `spec_info`, `validate`, and `archive` gain remediation hints.
- `smctl-build` — Error paths in `build` (empty-command and config-missing cases) gain remediation hints.
- `smctl/src/lib.rs` — `config` set / get error paths gain remediation hints.
- `smctl/src/main.rs` — Any top-level dispatch `.context()` additions needed to thread remediation through.

### No new capabilities.

## Impact

### Files Modified

See `openspec/changes/design-system-v1/voice-audit.md` § "Thin error messages" for the exact file:line inventory. At time of scope:

- `smctl-workspace/src/lib.rs` — 5 sites
- `smctl-spec/src/lib.rs` — 4 sites
- `smctl-build/src/lib.rs` — 1 site
- `smctl/src/lib.rs` — 1 site
- `smctl/src/main.rs` — 1 site (`spec has no tasks.md`)

Exact site counts may shift slightly as the implementer reads current code.

### Dependencies

None added. The helper (if introduced) lives alongside existing error handling; no new crate.

### Risk

Moderate. Error text is sometimes matched by scripts or tests. Mitigations:

- **Tests.** Any `predicate::str::contains` assertion on old error wording must be updated to match a new stable substring.
- **Scripts.** `--json` output surface does not change; callers using JSON are unaffected. Callers grepping plaintext error text are on an unsupported contract.
- **MSGIDs as anchors.** Where a log event accompanies an error, the MSGID (from `smctl-logging-v1`) is the stable anchor; callers can match on that instead of wording.

## Out of Scope

- Restructuring Rust error *types* (introducing a rich `SmctlError` enum, replacing `anyhow::Error` with `thiserror`). That is a larger refactor and a separate change.
- Localization (i18n). English only; add a localization surface later if ever.
- Rewording every `println!` or user-facing string. This change is about **error paths** specifically.
- Removing `.context()` wrappers. If an existing `.context()` call is already providing the three parts, leave it alone.

## References

- `openspec/changes/design-system-v1/voice-audit.md` — the audit inventory this change implements
- `openspec/changes/design-system-v1/specs/design-system.md` § Error messages — the three-part rubric
- `openspec/changes/smctl-copy-v1/` — the precedent small-change that fixed the string-substitution subset
