# Proposal: security-hygiene-v1

## Why

The frontend toolchain carries one critical (vitest: UI server arbitrary file read and execute) and one high (vite: `server.fs.deny` bypass) advisory, with five further open Dependabot alerts across the two manifests — while the Rust workspace has no advisory scanning at all, in CI or locally. Alongside that, repo hygiene has drifted: the OpenSpec 1.5.0 scaffolding is uncommitted, `AGENTS.md` is a broken machine transform of `CLAUDE.md` with a nonexistent `.Codex/` path, and unignored local files (`proxmox_mcp.log`, `.claude/worktrees/`) sit one `git add .` away from a public repo.

## What Changes

- Bump `vitest` `^2.1.5` → `^3.2.6` and `@vitest/coverage-v8` to match (closes critical GHSA-5xrq-8626-4rwp) in `ui/modelgate-web`.
- Bump `vite` `^5.4.11` → `^6.4.3` (closes high GHSA-fx2h-pf6j-xcff and medium GHSA-v6wh-96g9-6wx3).
- Regenerate `package-lock.json` so transitive `@babel/core` reaches ≥ 7.29.6 (closes low GHSA-4x5r-pxfx-6jf8). Together these close all 7 open Dependabot alerts.
- Bump `happy-dom` `^15.11.7` → `^20.10.6` and refresh transitive `brace-expansion` — `npm audit` flags a critical happy-dom VM-context escape (GHSA-37j7-fg3j-429f) and a moderate brace-expansion DoS (GHSA-jxxr-4gwj-5jf2) that Dependabot had not surfaced; after all bumps `npm audit` reports zero vulnerabilities.
- Add a Security Audit job to CI that runs `cargo audit` (RustSec advisory database) across the workspace and wire it into the CI Gate, giving the Rust dependency tree the advisory coverage npm already gets from Dependabot.
- Update `Cargo.lock`: `quinn-proto` 0.11.14 → 0.11.15 (RUSTSEC-2026-0185, high — remote memory exhaustion) and `anyhow` 1.0.101 → 1.0.103 (clears unsound RUSTSEC-2026-0190), found by the first local `cargo audit` run.
- Fix a bug the audit work exposed: all five `smctl quality` verbs fell back to the bare current directory when no `.smctl/workspace.toml` exists, so their cargo-ecosystem tools failed from any workspace member directory. New `smctl::find_cargo_root` walks up to the nearest `Cargo.lock` instead.
- Add `.gitignore` entries for `*.log` and `.claude/worktrees/`.
- Commit the OpenSpec 1.5.0 generated scaffolding: `openspec/config.yaml`, `.claude/skills/openspec-*/`, `.claude/commands/opsx/`.
- Regenerate `AGENTS.md` from `CLAUDE.md` addressed to AI coding agents generally, pointing at the real `.agents/skills/smallaios-design/` skill copy; commit `.agents/`.
- Record the repo-settings baseline applied 2026-07-02: secret scanning, push protection, and Dependabot security updates enabled on GitHub.

## Capabilities

### New Capabilities

- `repo-security`: security baseline for the ModelGate repository — dependency-advisory gates (RustSec via CI, npm via Dependabot with security updates enabled), GitHub secret-scanning posture, and ignore hygiene that keeps local logs and worktrees out of the public repo.

### Modified Capabilities

<!-- none — no existing capability's requirements change; dependency versions, CI wiring, and committed scaffolding are implementation-level -->

## Impact

- `ui/modelgate-web/package.json` and `package-lock.json` — vitest 3 is a major bump; the test suite and coverage run must pass under it before merge.
- `.github/workflows/ci.yml` — one new job plus a CI Gate dependency; PRs will now fail on RustSec advisories.
- `.gitignore`, `AGENTS.md`, `.agents/`, `openspec/config.yaml`, `.claude/commands/`, `.claude/skills/openspec-*/` — new or corrected committed files.
- GitHub repository settings — already applied out-of-band; the spec records them so drift is detectable.
