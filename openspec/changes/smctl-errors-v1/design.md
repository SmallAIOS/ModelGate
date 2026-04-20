# smctl-errors-v1 — Design Document

## Context

The design-system voice audit identified 15+ thin error messages across four `smctl-*` crates. Each violates the three-part error-message rubric by omitting the remediation clause. This change restructures those messages to conform.

The goal is not mechanical text rewriting; it is thinking through what an operator (or AI assistant) actually needs when each error fires. A good remediation is an executable `smctl` subcommand invocation, not a sentence beginning "try to …".

## Goals / Non-Goals

### Goals

1. Every audit-flagged error message carries all three parts.
2. Remediation hints reference real subcommands that exist today (no inventing `smctl foo bar` that isn't in the CLI).
3. Tests continue to pass, with any text assertions updated to stable substrings.

### Non-Goals

1. Not introducing a new error type system or abandoning `anyhow`. `thiserror` + `anyhow::Error` composition stays as-is.
2. Not rewording success paths or non-error copy. That's `smctl-copy-v1`.
3. Not adding i18n.
4. Not adding a lint or CI check to enforce the three-part rubric. Future work if drift becomes a problem.

## Decisions

### Decision 1: Inline string formatting, no helper abstraction

**Choice:** Rewrite each error message inline at the bail / context site. Do not introduce a `format_error(fact, meaning, remedy)` helper.

**Rationale:** There are fewer than 20 sites, each with different remedies tailored to the operation. A helper would force a single template and either lose nuance or become a lowest-common-denominator string. Reviewers looking at the diff should see each new message in full.

**Alternative considered:** A `format_error!(…)` macro enforcing the three-part shape. Over-engineering at this scale; revisit if sites proliferate.

### Decision 2: Remediations reference executable commands

**Choice:** The "what to do next" segment names an exact `smctl` invocation.

Example — spec not found:

> Before: `"spec 'foo' not found"`
> After:  `"spec 'foo' not found. Run `smctl spec list` to see active specs, or `smctl spec new foo` to create it."`

**Rationale:** Operators are senior engineers; they don't want prose telling them to "check your configuration." They want a command to run. This also serves the MCP / AI-assistant consumer: a machine can see the backtick-delimited command and either surface it to its user or execute it.

### Decision 3: Multi-part messages join with periods, not newlines

**Choice:** Keep each error on a single line. Use periods as separators between the fact, the meaning, and the remedy.

**Rationale:** `anyhow::bail!` produces a single string; multi-line formatting gets mangled by context wrappers and log formatters. A single well-punctuated sentence composes cleanly when `.context(...)` wraps it downstream. The design-system canonical form uses two lines for visual impact, but at the Rust-string level single-line is more robust.

### Decision 4: Assertion strategy — stable substring, not full match

**Choice:** Where tests assert on error wording, use `predicate::str::contains("stable phrase")` with a deliberately chosen substring: the subject noun phrase (e.g. `"spec 'foo' not found"`) or the remediation command (`"smctl spec list"`). Not the full string.

**Rationale:** Full-string matching is brittle; stable-substring matching lets copy polish happen without test churn. Document the chosen substring in a one-line test comment so the intent is explicit.

### Decision 5: Scope boundary — audit-driven, with a narrow expansion allowance

**Choice (v1.0 — initial):** Process each file in the audit list; stop at the audit list. Do not expand scope to "every error message in the codebase" even if non-flagged errors look thin while the file is open.

**Rationale:** Scope creep on mass-rewording changes explodes review burden. The audit is the contract.

**Amendment (v1.1 — after the first implementation pass):** When a peer of an audited site surfaces during the review pass AND its remediation shape is already covered by the catalog in this document, it is in scope. This is deliberately narrow: the surfaced site must be in a file already touched by the change, must be a thin or malformed error by the rubric, and must not require a new remediation category. Anything outside that envelope remains out of scope and gets filed as a follow-up issue.

**Why amend rather than spin a separate change:** A `smctl-errors-v2` for three sites would be more ceremony than signal. The amendment keeps the spec tree honest about what actually landed, and the implementation commit is clearly bounded.

**What the amendment does NOT permit:** Rewriting error types, introducing helpers, touching files not already audited, inventing new remediation shapes, or "while we're here" polishing of non-error strings.

## Remediation Catalog (preliminary)

Non-exhaustive — implementer refines as they read the code. These are the patterns each remediation should match:

| Error class | Remediation shape |
|---|---|
| Resource not found (`spec 'X' not found`) | `Run `smctl <domain> list` to see active items, or `smctl <domain> new X` to create it.` |
| Resource already exists (`spec 'X' already exists`) | `Run `smctl <domain> archive X` first, or use a different name.` |
| Malformed config (`unknown config key`) | `Run `smctl config --help` to see valid keys.` |
| Missing workspace state (`failed to load workspace.toml`) | `Run `smctl workspace init` from the workspace root, or pass `--workspace <path>` to an existing workspace.` |
| Missing document (`spec 'X' has no tasks.md`) | `Run `smctl spec ff X` to regenerate missing scaffolds.` |
| Empty build command | `Set a non-empty `build_command` for this repo in `workspace.toml`, or remove the repo with `smctl workspace remove <name>`.` |

## Risks / Trade-offs

- **Test churn.** Tests asserting on exact error text must be updated. Mitigate with stable-substring matching (Decision 4).
- **Redundancy with `tracing` events.** When `smctl-logging-v1` emits a `SMCTL-0099` event for an error, the message text duplicates the error shown to stdout. Accepted — the event goes to logs, the message goes to the operator; they serve different consumers.
- **Wording drift from the spec rubric.** Over time new error messages may slip back to thin form. Mitigation: reviewers check new error paths against the rubric; revisit with a lint if drift becomes a pattern.

## Migration Plan

Not applicable. Error text changes are not a migration — callers using `--json` aren't affected, plaintext consumers adjust on their own. No config, no data, no API surface changes.

## Open Questions

1. **Are any current tests matching on full error strings?** Answer during implementation by grepping. Expected: yes, at least in `smctl/tests/cli.rs`. Budget time for 2–4 test updates.
2. **Do we want to emit a `SMCTL-0099` event for each updated error path, or rely on the top-level main-error catch?** Leaning: the top-level catch already covers it; per-site events would be redundant. Revisit if a specific site needs finer telemetry.
3. **Localization hook.** Should remediation strings live in a table keyed by error variant, to make i18n easier later? Not in v1 — premature abstraction.
