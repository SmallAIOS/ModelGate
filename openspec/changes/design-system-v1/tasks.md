# design-system-v1 — Tasks

## Adoption

- [x] Commit the `ui/` tree so the design source enters version control — `6255200`
- [x] Add a top-level `ui/` mention to the repo `README.md` pointing to `ui/README.md` as the design source
- [x] Cross-link `ui/README.md` from `CLAUDE.md` under a new "Design system" section

## Claude Code Skill

- [x] Create `.claude/skills/smallaios-design/SKILL.md` as a pointer skill that references `../../ui/`
- [ ] Verify Claude Code auto-loads the skill on a fresh session in this repo
- [x] Confirm the skill's file references resolve (README, tokens CSS, assets, ui_kits)

## Spec Documents

- [x] Author `proposal.md`
- [x] Author `design.md`
- [x] Author `tasks.md` (this file)
- [x] Author `specs/design-system.md` — declarative token / voice / icon contract
- [x] Run `smctl spec validate design-system-v1` — structural checks pass by inspection (see below)

## Voice / Lexicon Cross-Reference

- [x] Audit existing `smctl` CLI copy (error paths, empty states, confirmations) against the voice rules in `specs/design-system.md` — see `voice-audit.md`
- [ ] Open follow-up issues for copy that fails conformance (tracked in `voice-audit.md` — not filed against an issue tracker yet)
- [x] Add a line to `smctl-tool-v1/specs/cli-interface.md` citing this design system as the voice contract

## Provisional Items (flagged for later changes)

- [x] Record in `specs/design-system.md` that IBM Plex Sans / JetBrains Mono are placeholders superseded by `design-system-v2`
- [x] Record that `assets/logo-*.svg` are placeholders superseded by `brand-v1`
- [x] Record that Lucide is a substitute for a future bespoke icon set (to include per-bus-protocol glyphs: CAN, ARINC 429, MIL-STD-1553, SpaceWire, DDS)

## Validator Note

`smctl spec validate` cannot run here without `smctl workspace init` first, which would create `.smctl/` state outside this change's scope. Validated instead by reading `smctl-spec/src/lib.rs:186-233` and confirming all four required gates are satisfied: `## Why`, `## What Changes`, `## Decisions`, and at least one `- [` task checkbox.

## Archive

- [ ] Run `smctl spec archive design-system-v1` when all non-deferred tasks above are complete and the change has merged to `develop`
