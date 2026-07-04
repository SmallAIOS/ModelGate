# smctl-verify delta — lean-proof-runner-v1

## ADDED Requirements

### Requirement: Lean deep proof verification

`smctl verify proof` SHALL classify each configured `[verify.proof]` root automatically: a directory containing `lakefile.lean` or `lakefile.toml` is a Lake package; any other matched directory is a loose-file tree; a glob pattern matching an individual file yields a loose-file row unless the file sits inside a Lake package, in which case the enclosing package is checked once instead. Loose trees SHALL produce one source row per `.lean` file (recursive discovery, skipping hidden directories including `.lake/`, guarding against symlink cycles); nested directories carrying a lakefile SHALL be checked as Lake packages, not expanded as loose files. Each loose file is checked by `lean --json` with the working directory set to the root so elan toolchain resolution follows the corpus. Lake packages SHALL produce one source row per package, checked by `lake build` run inside the package directory with its replayed compiler messages parsed. Targets are deduplicated across overlapping patterns. All tool output MUST be captured, never inherited. A directory the walk cannot read, a root that classifies to nothing checkable, and a matched path that is neither file nor directory MUST each fail the run with a diagnostic — silence is never a pass. A source row SHALL fail when any error-severity message is reported or when any message marks a proof incomplete — via the `hasSorry` message kind or the `declaration uses 'sorry'` / `` declaration uses `sorry` `` warning text — regardless of the tool's exit code. Proof rows SHALL attach a structured `detail` object carrying `errors`, `warnings`, and `sorries` counts plus an optional failure (`kind` of `error` for a positioned proof error, `sorry` for an admitted proof, or `build` for environment-level failures; optional `file:line:col` location; message excerpt), using field names disjoint from the model and protocol detail variants. Every failed row MUST carry a three-part diagnostic whose remediation is an executable reproduce command.

#### Scenario: Loose root yields per-file rows

- **WHEN** `[verify.proof] roots` names a directory with no lakefile that contains three `.lean` files
- **THEN** the report MUST contain three source rows
- **AND** each row's reproduce command MUST invoke `lean` on that row's file

#### Scenario: Proof containing sorry fails

- **WHEN** a checked file produces a warning message tagged `hasSorry` (or carrying the sorry warning text) and the tool exits 0
- **THEN** the source row MUST report `failed`
- **AND** the row's detail MUST count at least one sorry and carry a failure of kind `sorry` with the message's `file:line:col` location
- **AND** MSGID `SMCTL-0507` MUST be emitted for the row

#### Scenario: Proof error reports its location

- **WHEN** a checked file produces an error-severity message with a position
- **THEN** the source row MUST report `failed` with a failure of kind `error` and the `file:line:col` location
- **AND** MSGID `SMCTL-0505` MUST be emitted for the row

#### Scenario: Clean proof passes with zeroed counts

- **WHEN** `lean --json` exits 0 with no warning or error messages for a file
- **THEN** the source row MUST report `passed` with detail counts `errors=0`, `warnings=0`, `sorries=0`

#### Scenario: Lake package builds as one row

- **WHEN** a configured root contains `lakefile.toml`
- **THEN** the report MUST contain exactly one row for that root, produced by `lake build` run inside the package directory
- **AND** replayed compiler messages MUST populate the row's counts and failure

#### Scenario: Nested Lake package inside a loose tree builds with lake

- **WHEN** a loose root contains both bare `.lean` files and a subdirectory carrying `lakefile.toml`
- **THEN** the subdirectory MUST produce one Lake-package row and its sources MUST NOT appear as loose-file rows

#### Scenario: Empty root fails loudly

- **WHEN** a configured root matches a directory containing no `.lean` files and no lakefile
- **THEN** the run MUST report `failed` with a diagnostic naming the root
- **AND** the exit code MUST be non-zero

#### Scenario: Kind-less messages still classify sorries

- **WHEN** an older toolchain emits JSON messages without the `kind` field
- **THEN** sorry classification MUST fall back to the warning text and the row MUST still report `failed`

#### Scenario: Unparseable failure falls back to exit code

- **WHEN** the tool exits non-zero and no message can be parsed from its output
- **THEN** the source row MUST report `failed` quoting the head of the captured output
- **AND** MSGID `SMCTL-0506` MUST be emitted for the row

### Requirement: Proof tool discovery and overrides

Proof tool probes SHALL require the probe process to exit successfully; a resolvable binary that exits non-zero (such as an elan shim with no configured toolchain) MUST be treated as not installed rather than parsed for a version. `smctl verify discover` SHALL probe `lean` for the proof verifier and report its resolved path and parsed version. At run time each target SHALL gate on the tool it needs — `lean` for loose files, `lake` for Lake packages. When no configured target can be checked, the run reports `tool_missing` and the missing tool MUST be named specifically in the `tool_missing` JSON envelope with the elan install hint. In a mixed corpus where one tool is present and the other absent, the checkable targets MUST still run and each unverifiable target MUST fail its row with a note naming the missing tool — a missing tool never silently skips proofs. The environment overrides `SMCTL_VERIFY_LEAN_BIN` and `SMCTL_VERIFY_LAKE_BIN` SHALL take precedence over PATH resolution for their respective tools.

#### Scenario: Broken shim is not installed

- **WHEN** `lean` resolves to a shim that exits non-zero from `lean --version`
- **THEN** `smctl verify discover` MUST report the proof verifier as `not installed`

#### Scenario: Missing lean is named for loose corpora

- **WHEN** every configured root classifies as a loose-file tree and `lean` is unavailable while `lake` is present
- **AND** the operator runs `smctl verify proof --json`
- **THEN** stdout MUST be the `tool_missing` envelope with `tool` set to `lean` and an elan install hint

#### Scenario: Override binary is honored

- **WHEN** `SMCTL_VERIFY_LEAN_BIN` points at an executable script
- **THEN** the runner MUST invoke that script instead of any PATH-resolved `lean`

#### Scenario: Mixed corpus with one tool missing fails only the unverifiable rows

- **WHEN** the configured roots yield both loose files and a Lake package, `lean` is installed, and `lake` is not
- **THEN** the loose rows MUST be checked normally
- **AND** the package row MUST report `failed` with a note naming `lake`
- **AND** the run MUST NOT report `tool_missing`

## MODIFIED Requirements

### Requirement: MSGID range allocation

The MSGID catalog SHALL reserve `SMCTL-0500..0599` for verify. The allocated MSGIDs MUST be:

- `SMCTL-0501 VerifyStarted` — Informational
- `SMCTL-0502 VerifySucceeded` — Informational
- `SMCTL-0503 VerifyFailed` — Error
- `SMCTL-0504 VerifierMissing` — Warning
- `SMCTL-0505 VerifyCounterExample` — Error
- `SMCTL-0506 VerifyOutputUnparsed` — Warning
- `SMCTL-0507 ProofIncomplete` — Error

#### Scenario: VerifyStarted has the right code

- **WHEN** the operator inspects `MsgId::VerifyStarted.code()`
- **THEN** the returned value MUST be `501`
- **AND** the value MUST satisfy `(500..=599).contains(&code)`

#### Scenario: VerifyCounterExample has the right code

- **WHEN** the operator inspects `MsgId::VerifyCounterExample.code()`
- **THEN** the returned value MUST be `505`
- **AND** the default severity MUST be Error

#### Scenario: ProofIncomplete has the right code

- **WHEN** the operator inspects `MsgId::ProofIncomplete.code()`
- **THEN** the returned value MUST be `507`
- **AND** the default severity MUST be Error
- **AND** the code MUST satisfy `(500..=599).contains(&code)`
