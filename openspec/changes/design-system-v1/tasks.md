# design-system-v1 — Tasks

## Adoption

- [ ] Commit the `ui/` tree (currently untracked per `git status`) so the design source enters version control
- [ ] Add a top-level `ui/` mention to the repo `README.md` pointing to `ui/README.md` as the design source
- [ ] Cross-link `ui/README.md` from `CLAUDE.md` under a new "Design system" section

## Claude Code Skill

- [x] Create `.claude/skills/smallaios-design/SKILL.md` as a pointer skill that references `../../ui/`
- [ ] Verify Claude Code auto-loads the skill on a fresh session in this repo
- [ ] Confirm the skill's file references resolve (README, tokens CSS, assets, ui_kits)

## Spec Documents

- [x] Author `proposal.md`
- [x] Author `design.md`
- [x] Author `tasks.md` (this file)
- [ ] Author `specs/design-system.md` — declarative token / voice / icon contract
- [ ] Run `smctl spec validate design-system-v1` and resolve any gaps

## Voice / Lexicon Cross-Reference

- [ ] Audit existing `smctl` CLI copy (error paths, empty states, confirmations) against the voice rules in `specs/design-system.md`
- [ ] File follow-up issues for any copy that fails conformance — do not change code in this change
- [ ] Add a line to `smctl-tool-v1/specs/cli-interface.md` (or a successor) citing this design system as the voice contract

## Provisional Items (flagged for later changes)

- [ ] Record in `specs/design-system.md` that IBM Plex Sans / JetBrains Mono are placeholders superseded by `design-system-v2`
- [ ] Record that `assets/logo-*.svg` are placeholders superseded by `brand-v1`
- [ ] Record that Lucide is a substitute for a future bespoke icon set (to include per-bus-protocol glyphs: CAN, ARINC 429, MIL-STD-1553, SpaceWire, DDS)

## Archive

- [ ] Run `smctl spec archive design-system-v1` when all non-deferred tasks above are complete and the change has merged to `develop`
