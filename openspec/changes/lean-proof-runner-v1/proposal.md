# lean-proof-runner-v1

## Why

`smctl verify proof` is the last exit-code shell-out in the verify surface: it runs `lake build <root>` from smctl's cwd and reports pass/fail from the exit code alone. That model is wrong twice over for the real proof corpus (loose `.lean` files with no lakefile, as shipped in the SmallAIOS kernel's `formal/lean4/`): `lake build` is the wrong invocation for loose files, and Lean's exit code ignores warnings — so a proof containing `sorry` reports as verified. The kernel corpus contains a live example today (`CapabilityNonForgery.lean:144`). TLA+ (`tla-plus-runner-v1`) and SPIN (`spin-protocol-runner-v1`) already received deep runners; this change completes the set.

## What Changes

- `smctl verify proof` becomes a deep runner: each configured root is classified as a Lake package (contains `lakefile.lean` or `lakefile.toml`) or a loose-file tree. Lake packages run `lake build` with text-log parsing; loose trees produce one source row per `.lean` file, checked via `lean --json` with structured per-message parsing.
- Incomplete proofs are detected and **fail** their source row: a `hasSorry`-tagged message (Lean ≥ 4.19) or a `declaration uses 'sorry'` / `` declaration uses `sorry` `` warning text marks the proof incomplete. Exit code 0 no longer implies verified.
- Structured results: `SourceRow.detail` gains a `Proof` variant (error/warning/sorry counts plus an optional failure with kind, location, and message excerpt), with parity to the TLA+/SPIN detail objects in `--json` output.
- Failure diagnostics follow the three-part structure with bounded message excerpts and an executable reproduce command.
- Tool discovery is hardened: the version probe requires exit success (an elan shim with no configured toolchain spawns fine but exits non-zero), `lean` joins `lake` as a probed binary, `SMCTL_VERIFY_LEAN_BIN` / `SMCTL_VERIFY_LAKE_BIN` overrides enable hermetic tests, and the `tool_missing` envelope names the specific missing tool.
- MSGID `SMCTL-0507 ProofIncomplete` (Error) is allocated; evidenced proof errors reuse `SMCTL-0505`, unparseable output reuses `SMCTL-0506`.
- Stale documentation is corrected: the capability spec Purpose paragraph (still lists SPIN as an exit-code wrapper), the README verify rows, and the `lean.rs` / `shell.rs` doc comments.

## Capabilities

### New Capabilities

None.

### Modified Capabilities

- `smctl-verify`: adds a "Lean deep proof verification" requirement (root classification, per-file structured checking, sorry-as-failure semantics, discovery hardening, reproduce commands) and the `SMCTL-0507` row in the MSGID allocation requirement; rewrites the stale Purpose paragraph.

## Impact

- **`smctl-verify` crate**: `lean.rs` rewritten from a 37-line `Shell` config into a deep runner; new `lean_out.rs` parser module (JSON message parsing, lake text-log parsing, sorry classification, excerpt rendering); `lib.rs` gains `VerifyDetail::Proof` (field names kept disjoint from the Model/Protocol variants for untagged deserialization).
- **`smctl-log`**: `MsgId::ProofIncomplete` = SMCTL-0507, severity Error, plus catalog-spec row.
- **`smctl` CLI**: `tool_missing` envelope dispatch learns the proof verifier's two-tool helper; no new flags (existing `--json` / `--strict` / `--verifier` cover the surface).
- **`smctl-mcp`**: report flows through serde unchanged; the `smctl_verify_proof` tool description is refreshed.
- **No `workspace.toml` schema change**: `[verify.proof] roots` / `fail_on` keep their shape; root classification is automatic.
- **Docs**: README verify table, capability-spec Purpose paragraph.
- **Accepted limitations carried over from the TLA+/SPIN precedents**: `discover` takes no manifest context, MCP returns the bare report without the `tool_missing` envelope, tool output is captured rather than streamed, no timeout knob. New Lean-specific accepted limitations: elan may auto-download a toolchain on first invocation (network, stderr noise — tolerated by the parser), and `#print axioms` auditing is deferred.
