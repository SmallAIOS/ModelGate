# Voice and Lexicon Audit — smctl

Audit of user-facing copy in `smctl/`, `smctl-workspace/`, `smctl-flow/`, `smctl-spec/`, `smctl-build/` against the voice contract in `specs/design-system.md`.

This audit is a **report only**. Code changes are explicitly out of scope for `design-system-v1` (per `tasks.md`). Findings below should be addressed in follow-up changes or issues.

Audit date: 2026-04-19.

## Summary

| Category | Count | Severity |
|---|---|---|
| Status-vocabulary synonyms | 6 | High |
| Thin error messages (no remediation) | 15+ | High |
| Unicode check/cross marks (spec-silent) | 3 locations | Medium |
| Person violations (`we` / `I` / `our`) | 0 | — |
| Emoji / exclamation points | 0 | — |
| Product-name casing mistakes | 0 | — |

## Status-vocabulary synonyms

The spec's canonical terms are `clean`/`dirty`, `ahead N`/`behind N`, `pending`/`running`/`passed`/`failed`, `active`/`archived`, `verified`/`unverified`, `present`/`absent`. These use synonyms instead:

- `smctl/src/main.rs:993-995` — `"ok"` / `"MISSING"` for document presence. Should be `present` / `absent`.
- `smctl/src/main.rs:1000` — `"validation: PASS"`. Should be `passed`.
- `smctl/src/main.rs:1008` — `"validation: FAIL"`. Should be `failed`.
- `smctl/src/main.rs:1102, 1230` — `"build FAILED"` (2x). Should be `failed` (lowercase).

All uppercase status labels violate sentence-case. Flat map to the canonical terms.

## Thin error messages

Each of these fails the three-part rule — (a) what happened, (b) what it means, (c) what to do next as an executable command. Most have (a) only.

- `smctl-workspace/src/lib.rs:230` — `"repo '{name}' already exists in workspace"`. No action. Suggest `--force` or removal command.
- `smctl-workspace/src/lib.rs:254` — `"repo '{name}' not found in workspace"`. Suggest `smctl workspace status` to list.
- `smctl-workspace/src/lib.rs:395-400` — `"failed to add worktree for {} at {}: {}"`. Depends on upstream git error quality; may be acceptable.
- `smctl-workspace/src/lib.rs:425, 470` — `"worktree set '{name}' does not exist"`. Suggest `smctl worktree list`.
- `smctl-spec/src/lib.rs:40` — `"spec '{name}' already exists at {}"`. No action.
- `smctl-spec/src/lib.rs:168, 189, 281` — `"spec '{name}' not found"` (3 sites). Suggest `smctl spec list`.
- `smctl/src/main.rs:1020` — `"spec '{spec_name}' has no tasks.md"`. Suggest running `smctl spec ff` or re-scaffolding.
- `smctl/src/lib.rs:115` — `"unknown config key: {key}"`. Suggest `smctl config --help` or listing known keys.
- `smctl-build/src/lib.rs:315` — `"empty command"`. No remediation; suggest which `workspace.toml` field to inspect.

## Unicode check/cross marks

The spec explicitly permits box-drawing (`•`, `─`, `│`, `└─`, `├─`) as structural terminal output but is silent on `✓` / `✗`. These three sites use them:

- `smctl/src/main.rs:489` — 3 uses in workspace status output
- `smctl/src/main.rs:638` — flow init status
- `smctl/src/main.rs:1095` — build output

**Recommendation:** add an explicit line to `specs/design-system.md` covering whether `✓` / `✗` are permitted. Options: (a) allow them as status glyphs since they are widely recognized, (b) forbid them in favor of the canonical word-status vocabulary, (c) permit only when paired with the word label. Suggest option (c).

## Clean areas

No violations found in:

- **Person.** The codebase consistently avoids `we` / `I` / `our` / `let's`. Messages are imperative and report state.
- **Emoji and exclamation points.** None found in user-facing strings.
- **Product-name casing.** `SmallAIOS`, `ModelGate`, `smctl` are all cased canonically in the sites inspected.
- **clap help strings.** Sentence case, imperative, concise.
- **Button / verb forms.** No gerunds or bare nouns observed in action surfaces.

## Follow-up

These findings should be addressed under a successor change, not this one. Suggested grouping:

1. **`smctl-copy-v1`** (or inline into `smctl-tool-v1` follow-ups) — replace the six status synonyms with canonical terms. Low-risk, string-only.
2. **`smctl-errors-v1`** — add remediation hints to the 15+ thin error messages. Higher risk: changes exact error text, may affect scripts or tests that grep on messages.
3. **Spec amendment** — decide `✓` / `✗` policy in `specs/design-system.md` before the errors work lands.

None of the above are blocking for `design-system-v1` adoption.
