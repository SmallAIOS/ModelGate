# repo-security Delta Specification

## ADDED Requirements

### Requirement: RustSec advisory gate in CI

CI SHALL run `cargo audit` against the committed `Cargo.lock` on every pull request and on every push to `main` or `develop`, and the CI Gate SHALL depend on the audit job so that an unaddressed RustSec advisory blocks merging.

#### Scenario: Pull request triggers the audit

- **WHEN** a pull request targeting `develop` or `main` is opened or updated
- **THEN** the ModelGate CI workflow MUST run a Security Audit job that executes `cargo audit`

#### Scenario: Advisory blocks the gate

- **WHEN** `cargo audit` exits non-zero because a dependency matches a RustSec advisory
- **THEN** the Security Audit job MUST fail
- **AND** the CI Gate job MUST fail, blocking the pull request

#### Scenario: Advisory suppression is traceable

- **WHEN** an advisory has no upstream fix and must be temporarily ignored
- **THEN** the workflow MUST carry an explicit `--ignore RUSTSEC-<id>` flag with a comment linking a tracking reference, rather than disabling the job

### Requirement: npm advisory posture

The repository SHALL keep Dependabot alerts and Dependabot security updates enabled for the npm ecosystem, and direct dependencies with a published advisory SHALL be moved to at least the first patched version.

#### Scenario: No open critical or high alerts after this change

- **WHEN** the dependency bumps in this change land on `develop`
- **THEN** `vitest` MUST resolve to ≥ 3.2.6, `vite` to ≥ 6.4.3, and transitive `@babel/core` to ≥ 7.29.6 in `ui/modelgate-web/package-lock.json`
- **AND** the corresponding Dependabot alerts MUST close on the next scan

#### Scenario: Future npm advisory arrives as a fix PR

- **WHEN** a new advisory is published against a locked npm dependency
- **THEN** Dependabot MUST be able to open an automated security-update pull request (the feature is enabled at the repo level)

### Requirement: Secret scanning enabled

The GitHub repository SHALL have secret scanning and secret-scanning push protection enabled.

#### Scenario: Settings are verifiable

- **WHEN** an operator runs `gh api repos/SmallAIOS/ModelGate --jq .security_and_analysis`
- **THEN** `secret_scanning.status` and `secret_scanning_push_protection.status` MUST both read `enabled`

### Requirement: Local artifact ignore hygiene

The repository SHALL gitignore local log files (`*.log`) and Claude-managed worktrees (`.claude/worktrees/`), and no `.log` file SHALL be tracked in git.

#### Scenario: Stray MCP log stays untracked

- **WHEN** a tool writes `proxmox_mcp.log` (or any `*.log`) into the repository root
- **THEN** `git check-ignore` MUST match it
- **AND** `git status` MUST NOT list it as untracked

#### Scenario: Claude worktrees stay untracked

- **WHEN** Claude Code creates a worktree under `.claude/worktrees/`
- **THEN** `git check-ignore` MUST match the path
