# Tasks: security-hygiene-v1

## 1. Frontend dependency bumps

- [x] 1.1 Bump `vitest` to `^3.2.6`, `@vitest/coverage-v8` to `^3.2.6`, and `vite` to `^6.4.3` in `ui/modelgate-web/package.json`; regenerate `package-lock.json` and confirm locked `vitest ≥ 3.2.6`, `vite ≥ 6.4.3`, `@babel/core ≥ 7.29.6`
- [x] 1.2 Run `npm run typecheck`, `npm run test`, `npm run coverage`, and `npm run build` in `ui/modelgate-web`; fix any vitest 3 migration breakage until all pass
- [x] 1.3 Bump `happy-dom` to `^20.10.6` (critical GHSA-37j7-fg3j-429f, surfaced by `npm audit`, not Dependabot) and `npm audit fix` transitive `brace-expansion`; confirm `npm audit` reports zero vulnerabilities

## 2. CI security audit

- [x] 2.1 Add a `security-audit` job (name: Security Audit) to `.github/workflows/ci.yml`: checkout, `dtolnay/rust-toolchain@stable`, `taiki-e/install-action@cargo-audit`, run `cargo audit`; include a comment documenting the `--ignore RUSTSEC-<id>` escape hatch
- [x] 2.2 Add `security-audit` to the CI Gate `needs` list
- [x] 2.3 Run `cargo audit` locally (installed cargo-audit 0.22.2) — found quinn-proto 0.11.14 vulnerable (RUSTSEC-2026-0185, high)
- [x] 2.4 `cargo update -p quinn-proto -p anyhow` (0.11.15 fixes RUSTSEC-2026-0185; anyhow 1.0.103 clears unsound RUSTSEC-2026-0190); `cargo audit` now reports zero vulnerabilities — 2 git2 unsound warnings remain with no upstream fix released
- [x] 2.5 Fix discovered bug: all five `smctl quality` verbs fell back to bare cwd when no `.smctl/workspace.toml` exists, so cargo-ecosystem tools failed from workspace member dirs; added `smctl::find_cargo_root` (walks up to nearest `Cargo.lock`) as the fallback, with unit test, and verified `smctl quality audit --json` from `smctl/` now returns the report shape

## 3. Ignore hygiene

- [x] 3.1 Confirm no `.log` file is tracked (`git ls-files '*.log'` empty), then add `*.log` and `.claude/worktrees/` to `.gitignore`
- [x] 3.2 Verify `git check-ignore proxmox_mcp.log` and `git check-ignore .claude/worktrees/x` both match, and `git status` no longer lists them

## 4. Committed scaffolding and agent guidance

- [x] 4.1 Stage and commit the OpenSpec 1.5.0 scaffolding: `openspec/config.yaml`, `.claude/skills/openspec-*/`, `.claude/commands/opsx/`
- [x] 4.2 Regenerate `AGENTS.md` from `CLAUDE.md`: generic AI-agent preamble, design-skill pointer to `.agents/skills/smallaios-design/SKILL.md`, drift note naming `CLAUDE.md` as source of truth; commit alongside `.agents/`
- [x] 4.3 Confirm `AGENTS.md` contains no `.Codex/` or `Codex.ai` references

## 5. Validation and PR

- [x] 5.1 `openspec validate --all --strict` passes with the new change and delta spec
- [x] 5.2 `cargo build --workspace` and `cargo test --workspace` pass (no Rust surface changed, but gate anyway)
- [x] 5.3 Verify repo settings still read enabled: `gh api repos/SmallAIOS/ModelGate --jq .security_and_analysis` shows secret scanning, push protection, and Dependabot security updates on
- [x] 5.4 Open PR from `change/security-hygiene-v1` into `develop`; confirm the Security Audit job appears and the CI Gate goes green — PR #30, all 11 checks passed, squash-merged 2026-07-03

## 6. Toolchain currency

- [x] 6.1 Bring local toolchain to current stable (1.96.1 via rustup; Homebrew rust removed) to match CI's `dtolnay/rust-toolchain@stable`
- [x] 6.2 Add `rust-toolchain.toml` pinning `channel = "stable"` so the machine-wide nightly default (kept for SmallAIOS kernel targets) never silently builds this workspace; re-run fmt, clippy `-D warnings`, and the full test suite under 1.96.1
