# smctl-verify Delta Specification

## ADDED Requirements

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

## MODIFIED Requirements

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
