# smctl-copy-v1 — Proposal

## Why

`design-system-v1` declared the voice and lexicon contract for SmallAIOS, ModelGate, and `smctl`. Its companion `voice-audit.md` surveyed the existing `smctl` CLI output against that contract and flagged nine sites using forbidden forms:

- Six **status-vocabulary synonyms** (`ok`, `MISSING`, `PASS`, `FAIL`, `FAILED`) that should map to the canonical terms declared in the spec (`present` / `absent` / `passed` / `failed`).
- Three **forbidden Unicode pictograph** uses (`✓` / `✗`) that the subsequent spec amendment explicitly banned.

These are editorial violations, not functional bugs. But the design system is only as real as its first application, and leaving the audit's own findings unresolved would suggest the contract is aspirational.

This change closes the gap by applying the contract to the nine flagged sites. It is deliberately a small, tightly scoped change — pure string substitutions, no behavioral shift — so that subsequent deeper work (e.g. `smctl-errors-v1` remediation-hint additions) has a clean baseline.

## What Changes

1. **Status vocabulary.** Replace `ok` / `MISSING` / `PASS` / `FAIL` / `FAILED` in `smctl/src/main.rs` with the canonical terms from the design-system spec.
2. **Forbidden glyphs.** Remove `✓` (U+2713) and `✗` (U+2717) from the three flagged sites. Where a status marker was redundant with an adjacent word (`clean` / `dirty`), drop the glyph entirely. Where the glyph was the only status indicator, substitute the canonical word form (`passed` / `failed`).
3. **Test fixture.** Update the one CLI test (`test_spec_ff` in `smctl/tests/cli.rs`) that asserted on the old `proposal=ok` substring.

## Capabilities

### Modified Capabilities

- `smctl-cli` — User-facing copy now conforms to the `design-system-v1` voice and lexicon rules. No behavioral change. `--json` output shape is unchanged; scripts parsing JSON are unaffected. Scripts that grep plaintext for `PASS` / `FAIL` / `MISSING` were never on a supported contract — `--json` exists for that.

### No new capabilities.

## Impact

### Files Modified

- `smctl/src/main.rs` — 9 string substitutions, 1 removed positional format arg
- `smctl/tests/cli.rs` — 1 test expectation update

### No New Files

### Dependencies

None added.

### Affected Repos

Just ModelGate.

### Risk

Minimal. All edits are string-level. `cargo test` (79 tests) passes; `cargo clippy --workspace --all-targets -- -D warnings` clean; `cargo fmt --check` clean.

## References

- `openspec/changes/design-system-v1/specs/design-system.md` — the voice contract
- `openspec/changes/design-system-v1/voice-audit.md` — the audit that identified the nine sites
- `openspec/changes/design-system-v1/specs/design-system.md` § Emoji and ornament — the `✓` / `✗` prohibition
