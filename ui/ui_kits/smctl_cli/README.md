# smctl — UI kit

A high-fidelity recreation of the `smctl` CLI defined in `ModelGate/openspec/changes/smctl-tool-v1/specs/cli-interface.md`.

This is the canonical surface for SmallAIOS operators. The kit renders the terminal experience — prompt, tree glyphs, status colors, progress, confirmations — exactly as the spec prescribes.

## Files

| File | Purpose |
|---|---|
| `index.html` | Interactive shell demo + scene picker |
| `Terminal.jsx` | Atoms (`Prompt`, `Line`, `Caret`, color spans) and output blocks (`WorkspaceStatus`, `SpecValidation`, `BuildOutput`, `GateStatus`, `ConfirmLine`) |
| `TerminalApp.jsx` | Scenario wiring — walks through 6 commands that cover the hot path |

## Scenes

1. `smctl workspace status` — state of all repos in the workspace
2. `smctl spec ff gpu-accel` — fast-forward spec validation
3. `smctl build --parallel --test` — dependency-ordered build + test
4. `smctl feat gpu-accel` — alias: `flow feature start` + `worktree add`
5. `smctl gate status` — ModelGate health
6. `smctl done gpu-accel` — alias: finish feature, with destructive confirm

## Design rules enforced

- Tree glyphs (`├─`, `└─`) for lists; never bullets.
- Status colors: `ok=#3BCB72`, `err=#E56372`, `warn=#E0A64A`, `ion=#6E83FF`.
- Comma thousands separators, SI units, tabular numerals.
- Confirmation prompts are `text [y/N]` with `y`/`N` capitalized to show the default.
- Error messages are the 3-part pattern from the design system.
