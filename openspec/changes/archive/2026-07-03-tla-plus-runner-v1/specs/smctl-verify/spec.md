# smctl-verify Delta Specification

## ADDED Requirements

### Requirement: TLA+ deep model checking

`smctl verify model` SHALL run TLC against each configured `.tla` source and parse its output into structured results. The invocation MUST run in the spec file's directory, MUST pass `-config <stem>.cfg` when a same-named config file sits beside the spec, MUST direct TLC's metadata scratch (`-metadir`) into a temporary directory outside the target repo, and MUST use `-workers auto` unless `[verify.model] workers` overrides it.

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

### Requirement: Captured tool output

Shell-out verifiers SHALL capture child stdout and stderr rather than inheriting smctl's stdio. Tool output MUST never interleave with smctl's own stdout; when a run fails, the leading lines of captured output MUST be folded into the failure note so signal is preserved.

#### Scenario: Piped model output stays valid JSON while TLC runs

- **WHEN** the operator runs `smctl verify model | tee report.txt` and TLC actually executes
- **THEN** the captured output MUST be valid JSON parseable by `serde_json`, containing no raw TLC lines outside the report object

## MODIFIED Requirements

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
