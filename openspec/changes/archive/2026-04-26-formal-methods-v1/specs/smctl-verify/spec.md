# smctl-verify Specification (delta)

## ADDED Requirements

### Requirement: Verify command tree

`smctl verify` SHALL expose subcommands `policy`, `model`, `proof`, `protocol`, and `discover`. Each subcommand maps to a verifier domain: Cedar for `policy`, TLA+ for `model`, Lean 4 for `proof`, SPIN/Promela for `protocol`. `discover` enumerates which verifiers are reachable on PATH.

#### Scenario: Help lists every verifier

- **WHEN** the operator runs `smctl verify --help`
- **THEN** stdout MUST list `policy`, `model`, `proof`, `protocol`, and `discover`

#### Scenario: Discover enumerates installed tooling

- **WHEN** the operator runs `smctl verify discover`
- **THEN** stdout MUST list every supported verifier
- **AND** each entry MUST include the resolved binary path or `not installed`
- **AND** each installed entry MUST include a parsed version string

### Requirement: Cedar policy verification end-to-end

`smctl verify policy` SHALL verify Cedar policy sets via the `cedar-policy` Rust SDK. It MUST parse policies declared by the `[verify.policy]` workspace section, run validator + analysis passes, and report each diagnostic with the three-part remediation structure required by the design system.

#### Scenario: Well-formed policy passes

- **WHEN** the operator runs `smctl verify policy` against a workspace whose `[verify.policy] sources` point at well-formed Cedar files
- **THEN** stdout MUST report each policy file, the requirement count, and `passed`
- **AND** the process MUST exit 0
- **AND** the run MUST emit `SMCTL-0501` (started) and `SMCTL-0502` (succeeded) in order

#### Scenario: Malformed policy fails with remediation

- **WHEN** the operator runs `smctl verify policy` against a Cedar file with a syntax error
- **THEN** stderr MUST contain a three-part remediation message naming the file, line, and the parse error
- **AND** the process MUST exit non-zero
- **AND** the run MUST emit `SMCTL-0503` (failed) at Error severity

### Requirement: Verifier-missing detection

Every verifier subcommand SHALL detect a missing tool on PATH and emit a structured `tool_missing` envelope rather than crashing. The envelope MUST include the missing tool name and an install hint.

#### Scenario: TLA+ model verifier missing

- **WHEN** the operator runs `smctl verify model --json` on a host without `tlc` (the TLA+ model checker)
- **THEN** stdout MUST contain a JSON object whose `error` field is `"tool_missing"` and whose `tool` field is `"tlc"`
- **AND** the JSON MUST include an install hint pointing at the official TLA+ Toolbox or `tlaplus-cli`
- **AND** the run MUST emit `SMCTL-0504` (verifier missing) at Warning severity

### Requirement: Workspace.toml verify section

The workspace manifest SHALL accept an optional `[verify]` section that declares per-tool source roots and gating thresholds. The schema MUST permit the following structure:

```toml
[verify.policy]
sources = ["security/policies/*.cedar"]
fail_on = "any"

[verify.model]
specs = ["formal/tla/*.tla"]
fail_on = "any"

[verify.proof]
roots = ["formal/lean"]
fail_on = "any"

[verify.protocol]
specs = ["formal/spin/*.pml"]
fail_on = "any"
```

#### Scenario: Manifest declares Cedar sources

- **WHEN** `workspace.toml` declares `[verify.policy] sources = ["security/policies/*.cedar"]`
- **THEN** `smctl verify policy` MUST glob those paths relative to each registered repo
- **AND** MUST include every match in the verification run

#### Scenario: Missing section uses defaults

- **WHEN** `workspace.toml` does not declare `[verify]`
- **THEN** every verify subcommand MUST report `no sources configured` and exit 0

### Requirement: TTY-aware JSON fallback

Every verify subcommand SHALL emit JSON when stdout is not a TTY, regardless of the `--json` flag. Human-formatted output is reserved for interactive sessions, matching the convention from safety-quality-v1 Decision 9.

#### Scenario: Piped policy output produces JSON

- **WHEN** the operator runs `smctl verify policy | tee report.txt`
- **THEN** the captured output MUST be valid JSON parseable by `serde_json`

### Requirement: MSGID range allocation

The MSGID catalog SHALL reserve `SMCTL-0500..0599` for verify. The first four MSGIDs MUST be:

- `SMCTL-0501 VerifyStarted` — Informational
- `SMCTL-0502 VerifySucceeded` — Informational
- `SMCTL-0503 VerifyFailed` — Error
- `SMCTL-0504 VerifierMissing` — Warning

#### Scenario: VerifyStarted has the right code

- **WHEN** the operator inspects `MsgId::VerifyStarted.code()`
- **THEN** the returned value MUST be `501`
- **AND** the value MUST satisfy `(500..=599).contains(&code)`
