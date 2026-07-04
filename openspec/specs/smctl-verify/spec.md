# smctl-verify Specification

## Purpose

`smctl verify` exposes the formal-verification surface: Cedar policy checks run in-process end-to-end; TLA+ model checking runs TLC with deep output parsing (statistics, violations, counter-example traces); SPIN/Promela protocol verification runs the full spin → cc → pan pipeline with trail replay; Lean 4 proof checking classifies each root as a Lake package (`lake build`) or a loose `.lean` tree (`lean --json` per file) and fails proofs that contain `sorry`. Source roots come from `[verify.<domain>]` in `workspace.toml`; MSGIDs live in the reserved `SMCTL-0500..0599` range.

## Requirements
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

Every verifier subcommand SHALL detect a missing tool and emit a structured `tool_missing` envelope rather than crashing. The envelope MUST include the missing tool name and an install hint. For the model domain, the tool counts as missing only when neither a PATH `tlc` binary nor a usable `java` + `tla2tools.jar` combination (via `[verify.model] jar` or `TLA2TOOLS_JAR`) is available.

#### Scenario: TLA+ model verifier missing

- **WHEN** the operator runs `smctl verify model --json` on a host with no `tlc` on PATH and no configured `tla2tools.jar`
- **THEN** stdout MUST contain a JSON object whose `error` field is `"tool_missing"` and whose `tool` field is `"tlc"`
- **AND** the JSON MUST include an install hint naming the PATH binary, the `TLA2TOOLS_JAR` environment variable, and the `[verify.model] jar` workspace field as the three resolution options
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
jar = "tools/tla2tools.jar"   # optional: tla2tools.jar for the java -jar fallback
workers = 4                   # optional: TLC worker threads (default: auto)

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

#### Scenario: Unknown model fields still fail at parse time

- **WHEN** `workspace.toml` declares `[verify.model] jars = ["x"]` (a typo of `jar`)
- **THEN** manifest parsing MUST fail with an error naming the unknown field

### Requirement: TTY-aware JSON fallback

Every verify subcommand SHALL emit JSON when stdout is not a TTY, regardless of the `--json` flag. Human-formatted output is reserved for interactive sessions, matching the convention from safety-quality-v1 Decision 9.

#### Scenario: Piped policy output produces JSON

- **WHEN** the operator runs `smctl verify policy | tee report.txt`
- **THEN** the captured output MUST be valid JSON parseable by `serde_json`

### Requirement: MSGID range allocation

The MSGID catalog SHALL reserve `SMCTL-0500..0599` for verify. The allocated MSGIDs MUST be:

- `SMCTL-0501 VerifyStarted` — Informational
- `SMCTL-0502 VerifySucceeded` — Informational
- `SMCTL-0503 VerifyFailed` — Error
- `SMCTL-0504 VerifierMissing` — Warning
- `SMCTL-0505 VerifyCounterExample` — Error
- `SMCTL-0506 VerifyOutputUnparsed` — Warning

#### Scenario: VerifyStarted has the right code

- **WHEN** the operator inspects `MsgId::VerifyStarted.code()`
- **THEN** the returned value MUST be `501`
- **AND** the value MUST satisfy `(500..=599).contains(&code)`

#### Scenario: VerifyCounterExample has the right code

- **WHEN** the operator inspects `MsgId::VerifyCounterExample.code()`
- **THEN** the returned value MUST be `505`
- **AND** the default severity MUST be Error

### Requirement: TLA+ deep model checking

`smctl verify model` SHALL run TLC against each configured `.tla` source and parse its output into structured results. The invocation MUST run in the spec file's directory, MUST pass `-config <stem>.cfg` when a same-named config file sits beside the spec, MUST direct TLC's metadata scratch (`-metadir`) into a temporary directory outside the target repo, and MUST use `-workers auto` unless `[verify.model] workers` overrides it. The structured per-source `detail` object SHALL be polymorphic across verifier domains: model rows carry model-checking fields, protocol rows carry protocol fields, and each domain's rows serialize only its own field names.

#### Scenario: Passing model reports statistics

- **WHEN** TLC completes a model with no error
- **THEN** the source row MUST report `passed`
- **AND** the row's `detail` object MUST carry `states_generated`, `distinct_states`, and `queue_remaining` parsed from TLC's summary line

#### Scenario: Invariant violation renders a bounded trace

- **WHEN** TLC reports `Error: Invariant <name> is violated`
- **THEN** the source row MUST report `failed`
- **AND** `detail.violation` MUST name the violated property and the counter-example trace length
- **AND** the human diagnostics MUST include a trace excerpt capped to the first 4 and last 2 states with an elision marker and a three-part closing line with the exact TLC command to reproduce the full trace
- **AND** the run MUST emit `SMCTL-0505` at Error severity

#### Scenario: Unparsed failure falls back to exit code

- **WHEN** TLC exits non-zero but no known output pattern matches
- **THEN** the source row MUST still report `failed` (exit code is ground truth)
- **AND** the run MUST emit `SMCTL-0506` at Warning severity
- **AND** the diagnostic MUST quote the first lines of raw TLC output

#### Scenario: Jar fallback when tlc is absent

- **WHEN** no `tlc` binary is on PATH but `java` is present and a `tla2tools.jar` is declared via `[verify.model] jar` or the `TLA2TOOLS_JAR` environment variable
- **THEN** `smctl verify model` MUST run TLC as `java -jar <jar>` and behave identically to the PATH-binary case

#### Scenario: Model detail JSON shape is unchanged by the protocol extension

- **WHEN** a model row with detail is serialized after protocol support lands
- **THEN** the JSON MUST be identical to the pre-protocol shape (no tag or wrapper field introduced)

### Requirement: Captured tool output

Shell-out verifiers SHALL capture child stdout and stderr rather than inheriting smctl's stdio. Tool output MUST never interleave with smctl's own stdout; when a run fails, the leading lines of captured output MUST be folded into the failure note so signal is preserved.

#### Scenario: Piped model output stays valid JSON while TLC runs

- **WHEN** the operator runs `smctl verify model | tee report.txt` and TLC actually executes
- **THEN** the captured output MUST be valid JSON parseable by `serde_json`, containing no raw TLC lines outside the report object

### Requirement: SPIN deep protocol verification

`smctl verify protocol` SHALL run the full SPIN verification pipeline for each configured `.pml` source — `spin -a` code generation, C compilation of the generated pan verifier, and a `pan -a` run with acceptance-cycle detection — entirely inside a per-source temporary work directory so no `pan.*` or `.trail` artifact lands in the operator's repository. Pan output SHALL be parsed into structured results.

#### Scenario: Verified protocol reports statistics

- **WHEN** pan completes with `errors: 0`
- **THEN** the source row MUST report `passed`
- **AND** the row's `detail` object MUST carry `states_stored`, `states_matched`, and `depth_reached` parsed from pan's summary

#### Scenario: Assertion violation renders a bounded trail excerpt

- **WHEN** pan reports a non-zero error count with an assertion violation and writes a `.trail` file
- **THEN** the source row MUST report `failed`
- **AND** `detail.violation` MUST classify the failure and carry the trail step count
- **AND** the runner MUST replay the trail with `spin -t -p` and include a bounded excerpt (first 4 and last 2 steps with an elision marker) in the diagnostics, closed by a three-part line with the exact reproduce commands
- **AND** the run MUST emit `SMCTL-0505` at Error severity

#### Scenario: Acceptance cycle and invalid end state classify distinctly

- **WHEN** pan reports `acceptance cycle` or `invalid end state`
- **THEN** `detail.violation.kind` MUST read `acceptance_cycle` or `invalid_end_state` respectively

#### Scenario: Unparsed pan failure falls back safely

- **WHEN** pan exits non-zero (or reports a non-zero error count) but its output matches no known pattern
- **THEN** the source row MUST still report `failed`
- **AND** the run MUST emit `SMCTL-0506` at Warning severity
- **AND** the diagnostic MUST quote the first lines of raw pan output

#### Scenario: Missing C compiler is distinguishable from missing spin

- **WHEN** `spin` is available but no `cc` is on PATH and the operator runs `smctl verify protocol --json`
- **THEN** stdout MUST contain a JSON object whose `error` field is `"tool_missing"` and whose `tool` field is `"cc"`
- **AND** the install hint MUST name the Xcode Command Line Tools and the distro build-essential package

#### Scenario: Spin syntax error fails at the generation step

- **WHEN** `spin -a` rejects the Promela source
- **THEN** the source row MUST report `failed` with the spin error quoted and a reproduce command
- **AND** the compile and pan steps MUST NOT run for that source
