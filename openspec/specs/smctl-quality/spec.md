# smctl-quality Specification

## Purpose

`smctl-quality` provides the engineering-quality verb surface under `smctl quality <verb>`. Each verb wraps a Cargo-ecosystem tool, runs it across the active workspace, and returns a structured report. The crate is a log producer only — it never installs its own subscriber.

## Requirements

### Requirement: Five engineering-quality verbs

`smctl quality` SHALL expose five verbs: `audit` (advisory database), `deps` (unused dependencies), `unsafe` (unsafe-code geiger count), `dsm` (module dependency structure matrix), and `complexity` (per-function cyclomatic and cognitive complexity).

#### Scenario: Help lists every verb

- **WHEN** the operator runs `smctl quality --help`
- **THEN** stdout MUST list `audit`, `deps`, `unsafe`, `dsm`, and `complexity`

### Requirement: Tool-missing remediation

Each verb SHALL detect the absence of its underlying cargo plugin and emit a structured `tool_missing` error containing both the missing tool name and the install command.

#### Scenario: cargo-audit not installed

- **WHEN** the operator runs `smctl quality audit --json` on a host without `cargo-audit`
- **THEN** stdout MUST contain a JSON object whose `error` field is `"tool_missing"`
- **AND** the `remediation` field MUST contain `"cargo install cargo-audit"`
- **AND** the process MUST exit with the general-error code

### Requirement: TTY-aware JSON fallback

Each verb SHALL emit JSON when its stdout is not a TTY, regardless of the `--json` flag. Human-formatted output is reserved for interactive sessions.

#### Scenario: Piped audit produces JSON

- **WHEN** the operator runs `smctl quality audit | tee report.txt`
- **THEN** the captured output MUST be valid JSON

### Requirement: Configurable failure threshold

Each verb that produces a count or severity SHALL accept a `--fail-on-*` flag that controls when a non-zero exit code is returned. Setting the flag to `0` disables the gate (report-only mode).

#### Scenario: Audit gates on warning severity by default

- **WHEN** the operator runs `smctl quality audit` without `--fail-on`
- **AND** the report contains an advisory at `Warning` severity or higher
- **THEN** the process MUST exit with the general-error code

#### Scenario: Disabling the unsafe-code gate

- **WHEN** the operator runs `smctl quality unsafe --fail-on-count 0`
- **THEN** the process MUST exit with the success code regardless of the unsafe-site count
