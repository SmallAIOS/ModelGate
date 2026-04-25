# smctl-copy-v1 — Tasks

## Status-vocabulary substitutions (`smctl/src/main.rs`)

- [x] Line 993–995: `"ok"` / `"MISSING"` → `"present"` / `"absent"` (document presence)
- [x] Line 1000: `"validation: PASS"` → `"validation: passed"`
- [x] Line 1008: `"validation: FAIL"` → `"validation: failed"`
- [x] Line 1102: `"build FAILED"` → `"build failed"`
- [x] Line 1230: `"build FAILED"` → `"build failed"`

## Forbidden-glyph removals (`smctl/src/main.rs`)

- [x] Line 489 (workspace status): drop `\u{2713}` / `\u{2717}`; rely on adjacent `clean` / `dirty` word
- [x] Line 638 (flow init): replace glyph with canonical `passed` / `failed` word
- [x] Line 1095 (build results): replace glyph with canonical `passed` / `failed` word

## Tests

- [x] Update `test_spec_ff` assertion from `proposal=ok` to `proposal=present`
- [x] `cargo test --workspace` — all 79 tests pass
- [x] `cargo clippy --workspace --all-targets -- -D warnings` clean
- [x] `cargo fmt --check` clean

## Spec Documents

- [x] Author `proposal.md`
- [x] Author `design.md`
- [x] Author `tasks.md` (this file)

## Archive

- [ ] Run `smctl spec archive smctl-copy-v1` when merged to `develop`
