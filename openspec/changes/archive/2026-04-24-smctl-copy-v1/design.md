# smctl-copy-v1 — Design Document

## Context

`voice-audit.md` in `design-system-v1` identified nine sites in `smctl/src/main.rs` where user-facing copy violates the design system's voice rules. This change is the implementation. It is narrow on purpose.

## Goals / Non-Goals

### Goals

1. Replace status-vocabulary synonyms with canonical terms.
2. Remove forbidden Unicode pictographs.
3. Keep the change purely string-level so it can merge quickly and low-risk.

### Non-Goals

1. Not fixing the thin error messages (lack of remediation hints). That is `smctl-errors-v1`, a larger and more contentious change.
2. Not refactoring the format-output helpers or the dispatch structure. Nothing structural.
3. Not touching the `--json` output shape. Scripts depending on structured output must be unaffected.

## Decisions

### Decision 1: Word form, not bracketed ASCII, in workspace-status rows

**Choice:** For the workspace status output at `smctl/src/main.rs:489`, remove the glyph entirely rather than substituting a bracketed marker. The `state` word (`clean` / `dirty`) already sits next to where the glyph was and is redundant with it.

**Rationale:** The design-system amendment permits `[x] word` when a visual marker is structurally necessary, but in this site the word alone is sufficient and the glyph was always decorative. Removing it reduces line noise; the column alignment still works.

**Alternative considered:** Keep a `[x]` / `[ ]` bracket. Adds visual weight for no information gain. Rejected.

### Decision 2: Word form replaces glyph in flow-init and build-result rows

**Choice:** For `smctl/src/main.rs:638` (flow init) and `smctl/src/main.rs:1095` (build results), replace the glyph with the canonical `passed` / `failed` word. The rows become `  passed repo-a — message` and `  failed repo-b`.

**Rationale:** In both sites the glyph was the ONLY status indicator — removing it would lose information. Using the canonical word keeps information parity while conforming to the glyph prohibition. `passed` and `failed` are both six characters, so column alignment holds.

**Alternative considered:** Use `[x]` / `[ ]` bracketed form. Shorter but less self-documenting; the word form is more accessible at the cost of two characters. Rejected for readability.

### Decision 3: Lowercase everywhere

**Choice:** `build failed` (not `build FAILED`), `validation: passed` (not `validation: PASS`).

**Rationale:** The voice contract says labels are sentence case. Uppercase status words are shouty and violate that rule. Lowercase matches the rest of the output stream.

## Risks / Trade-offs

- **External scripts grepping stdout.** Any script matching `/build FAILED/` or similar in plaintext will miss the new form. Accepted — `--json` is the supported scripting contract; plaintext has never been one. If a consumer breaks, the fix is to switch to JSON, not to revert this.
- **Diff size distracts reviewers from the real content.** Nine sites, but all trivial. Reviewers should focus on test green-ness rather than re-auditing each substitution.

## Migration Plan

Not applicable. Purely additive (from the user's perspective): new canonical terms replace old non-canonical ones. No config change, no flag change, no data migration.

## Open Questions

None. The audit already resolved which terms are canonical; the amendment resolved the `✓` / `✗` question. This change is purely implementation.
