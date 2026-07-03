# Design: security-hygiene-v1

## Context

`ui/modelgate-web` pins `vite ^5.4.11` (locked 5.4.21) and `vitest ^2.1.5` (locked 2.1.9). GitHub reports 7 open Dependabot alerts against the app's `package.json` and `package-lock.json`: vitest critical GHSA-5xrq-8626-4rwp (fixed 3.2.6), vite high GHSA-fx2h-pf6j-xcff and medium GHSA-v6wh-96g9-6wx3 (both fixed 6.4.3), and transitive `@babel/core` low GHSA-4x5r-pxfx-6jf8 (fixed 7.29.6). The Rust workspace (11 crates) has no RustSec advisory coverage anywhere: `cargo-audit` is not installed locally and CI has no audit job — Dependabot only watches the npm ecosystem here. CI is a single workflow (`ci.yml`) with jobs fanning into a `CI Gate` job that PR branch protection keys on.

Out-of-band (applied 2026-07-02, recorded here): secret scanning, push protection, and Dependabot security updates were enabled on the GitHub repo via `gh api`.

## Goals / Non-Goals

**Goals:**

- Close all 7 open Dependabot alerts by moving to first-patched versions.
- Give the Rust dependency tree standing advisory coverage in CI, gating PRs.
- Keep local artifacts (logs, Claude-managed worktrees) out of the public repo via `.gitignore`.
- Land the OpenSpec 1.5.0 scaffolding and a corrected `AGENTS.md` in git.

**Non-Goals:**

- Upgrading to vite 7 or vitest 4 — this change takes the smallest jump that clears each advisory; larger upgrades ride a normal upgrade cycle.
- CodeQL / code scanning setup — separate decision, separate change.
- Extending `smctl quality audit` (the operator-facing wrapper) — CI calls `cargo audit` directly.
- Fixing stale prose in `CLAUDE.md` (e.g. the v0.1.0 crate/test counts) — documentation refresh is its own change; `AGENTS.md` is regenerated from `CLAUDE.md` as-is.

## Decisions

**vite `^6.4.3`, not `^7`.** 6.4.3 is the first patched version for both vite advisories. `@vitejs/plugin-react` 4.7.0 and vitest 3.2.x both peer-support vite 5–7, and CI's Node 22 satisfies vite 6's engine floor, so 6.4.3 is the lowest-risk move that clears the range. Vite 7 is deferred (Non-Goal).

**vitest `^3.2.6`, not 4.x.** 3.2.6 is the first patched version for the critical advisory. The 2→3 major is already the risky part of this change; stacking a second major (4.x) on top compounds migration surface for zero additional advisory benefit. `@vitest/coverage-v8` moves in lockstep to `^3.2.6` (its version tracks vitest).

**Transitive `@babel/core` fix via lockfile regeneration.** No direct dependency exists; refreshing the lock during the bump pulls ≥ 7.29.6. No manifest pin needed.

**`happy-dom` `^20.10.6` (found during implementation).** `npm audit` flags 15.11.7 with a critical VM-context escape (GHSA-37j7-fg3j-429f) that Dependabot had not surfaced; the fix line is 20.x. It is a devDependency used only as the vitest DOM environment, and the test suite passes under 20.10.6, so the major bump rides along. Transitive `brace-expansion` (moderate GHSA-jxxr-4gwj-5jf2) is fixed by `npm audit fix`. Exit state: `npm audit` reports zero vulnerabilities.

**`cargo audit` in CI via `taiki-e/install-action@cargo-audit`.** Prebuilt binary in seconds, and the workflow already uses `taiki-e/install-action` for `cargo-llvm-cov` — same trust surface, no new action vendor. Alternatives: `cargo install cargo-audit` (compiles for minutes on every run), `rustsec/audit-check` action (wraps the same tool with less control and slower maintenance cadence). The job needs only `Cargo.lock` (committed), so it runs from checkout with no `needs:` — parallel with `check-format` — and is added to the `CI Gate` `needs` list so advisories block PRs. `smctl quality audit` stays the operator-facing wrapper; CI uses the primitive directly rather than building `smctl` first.

**`.gitignore` gets `*.log` and `.claude/worktrees/`.** No `.log` file is currently tracked (verify with `git ls-files '*.log'` before landing), so the broad pattern is safe and covers future MCP/tool logs, not just `proxmox_mcp.log`. `.claude/worktrees/` is Claude Code's managed-worktree location; the existing `.worktrees/` entry does not cover it.

**`AGENTS.md` regenerated from `CLAUDE.md`, addressed generically.** The current file is a sed transform ("Codex (Codex.ai/code)", `.Codex/` paths that don't exist). Regenerate with the same content but: title/preamble addressed to AI coding agents generally, and the design-skill pointer aimed at the real cross-tool copy `.agents/skills/smallaios-design/SKILL.md`. Commit `.agents/` alongside. Alternative considered: delete both — rejected because `AGENTS.md` is the emerging cross-tool convention and the repo already maintains the `.agents/` skill copy. Drift risk vs `CLAUDE.md` is accepted and noted in the file header.

**OpenSpec scaffolding committed as generated.** `openspec/config.yaml`, `.claude/skills/openspec-*/` (10 skills, `generatedBy: "1.5.0"`), `.claude/commands/opsx/` (10 commands). Regenerable via `openspec update`, but committing keeps every contributor's agent tooling on the same version.

**Discovered during implementation — Rust side.** The first local `cargo audit` run (cargo-audit 0.22.2) found `quinn-proto` 0.11.14 vulnerable to RUSTSEC-2026-0185 (high, remote memory exhaustion; fixed 0.11.15) — a semver-compatible `cargo update` closes it, alongside `anyhow` → 1.0.103 for the unsound RUSTSEC-2026-0190 warning. Two `git2` 0.20.4 unsound warnings (RUSTSEC-2026-0183/0184) have no released fix; they are warnings, not vulnerabilities, and do not fail `cargo audit`. Installing cargo-audit also exposed a latent bug: with no `.smctl/workspace.toml` on the machine, every `smctl quality` verb fell back to the bare cwd, so cargo-ecosystem tools broke from workspace member directories (the `test_quality_audit_json_output_is_structurally_valid` test only passes on machines *without* cargo-audit). Fix: `smctl::find_cargo_root` walks up to the nearest `Cargo.lock` and replaces the bare-cwd fallback in the five quality verbs. No delta spec needed — the existing `smctl-quality` spec already requires verbs to run "across the active workspace"; this makes the implementation conform. `smctl verify`'s cwd fallback is intentional (anonymous single-repo mode) and is untouched.

**Toolchain currency and `rust-toolchain.toml` pin.** Local rust was behind CI: the Homebrew rustc was 1.94.1 (later removed in favor of rustup), while CI's `dtolnay/rust-toolchain@stable` resolves current stable — 1.96.1 as of 2026-06-26 — so local clippy could pass while CI clippy fails on newer lints. Worse, the machine-wide rustup default is a pinned nightly (2026-02-01, kept for SmallAIOS kernel targets: `aarch64-unknown-none`, UEFI, RISC-V), which silently built this workspace on nightly. Added `rust-toolchain.toml` with `channel = "stable"`: every rustup-driven build of this repo now uses current stable, matching CI, without touching the machine default the kernel work depends on. The pin floats on the stable channel rather than a fixed version, so it never goes stale. Ferrocene parity (build with stock rustc) is unaffected — stable is stock; Ferrocene stays opt-in. All checks re-verified under 1.96.1 before pushing.

## Risks / Trade-offs

- [vitest 2→3 breaking changes (config, coverage API)] → Run `npm run test`, `npm run coverage`, and `npm run build` locally before pushing; the suite is small and `happy-dom` 15.x is a supported vitest 3 environment.
- [`cargo audit` fails CI on a future advisory with no upstream fix] → Documented escape hatch: `--ignore RUSTSEC-<id>` flag in the workflow step with a comment requiring a linked tracking issue; the gate stays honest without permanently muting the tool.
- [`*.log` ignore hides a log a future change intends to commit] → Narrow with `!path/to/file.log` if that ever happens; today zero tracked `.log` files exist.
- [`AGENTS.md` drifts from `CLAUDE.md`] → Accepted; the regeneration note in the file states the source of truth.

## Migration Plan

Single PR from `change/security-hygiene-v1` into `develop` per git flow. No runtime deployment surface. Rollback is `git revert` of the merge commit; GitHub settings changes are independent and reversible in repo settings.

## Open Questions

None.
