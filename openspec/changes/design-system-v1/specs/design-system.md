# Design System Specification

## Overview

This specification declares the canonical design contract for SmallAIOS, ModelGate, and `smctl` surfaces: CLI output, docs, slides, logos, and any future web UI. The source artifacts live in `ui/` at the repo root. This document is the *contract* those artifacts serve; when the two disagree, this spec wins and `ui/` is corrected.

The design system is **terminal-first, monospace-forward, instrumentation-grade**. It derives its character from the product: a safety-critical Rust unikernel for AI inference, a CLI control plane following the `*ctl` convention, and an MCP-exposed toolset for AI coding assistants.

## Scope

This spec covers:

- **Tokens** — color, type, spacing, radius, shadow, motion
- **Voice and lexicon** — casing, person, status vocabulary, error-message structure
- **Iconography** — set, size, stroke, color rules
- **Brand assets** — logo marks and their provisional status
- **Skill integration** — Claude Code skill location and contents

This spec does **not** cover:

- Specific UI component APIs (buttons, tables, inputs) — those live in the referenced HTML / JSX previews in `ui/preview/` and `ui/ui_kits/`
- Frontend toolchain (framework, bundler, routing) — deferred to `modelgate-web-v1`
- CLI rendering implementation (terminal color libraries, width detection) — `smctl`'s concern, not this spec's

## Tokens

The authoritative token file is `ui/colors_and_type.css`. Consumers **MUST** `@import` or `<link>` it at the document root; tokens **MUST NOT** be re-declared or overridden inline.

### Color

- Neutrals: 7-step cool gray ramp plus `--bg-0` / `--bg-1` / `--bg-2` surface steps and `--fg-0` through `--fg-3` text steps.
- Dark mode: declared via the same variables under a prefers-color-scheme or explicit `.dark` selector override in the same file.
- Signal colors:
  - `--sig-ok` — signal green (phosphor-CRT vibe). Reserved for `verified`, `online`, `passed`.
  - `--sig-warn` — signal amber. Reserved for warnings.
  - `--sig-err` — restrained red. Reserved for failures.
- Spot: `--ion` (`#3451FF`). Reserved for active / focus states, selected rows, primary buttons. **MUST NOT** appear in decoration.

### Type

- Display + body sans: IBM Plex Sans. **Placeholder** — superseded by `design-system-v2` when a bespoke typeface is commissioned.
- Monospace: JetBrains Mono. Six-digit numerals, slashed zero, `iI1l` and `oO0` disambiguation required. **Placeholder** with the same supersession note.
- Numerals use tabular-nums in tables and dense labels.
- Scale: `12 / 13 / 14 / 16 / 20 / 28 / 40` (px). No axes beyond weight `400 / 500 / 600`. No italic. No display script.

### Spacing

- 8px grid with 4px half-step. Scale: `4 8 12 16 24 32 48 64 96`.
- Dense tables: `8 / 12`. Card padding: `16 / 24`. Section rhythm: `32 / 48`.

### Radius

- `0` — tables and data chrome
- `4px` — inputs and buttons
- `6px` — cards and overlays
- `8px` — modals (maximum)
- Pill shape — reserved for status dots and segmented controls only

### Shadow

Two elevations only:

- `--shadow-1` — hovering overlays (tooltips, menus)
- `--shadow-2` — modals

Shadows are tight and cool; closer to a 1px hard line plus an 8px 8% shade. No soft drop shadows.

### Motion

- Easing: `cubic-bezier(0.2, 0, 0, 1)` for UI transitions; `linear` for progress indicators.
- Durations: `120ms` micro, `200ms` standard, `320ms` large. No longer.
- Forbidden: bounces, springs, confetti, shimmer loaders. Use a 1Hz caret-blink skeleton instead.

## Voice and Lexicon

Copy reads like instrumentation: factual, verb-first, present tense. The reader is a senior engineer whose time is worth more than the author's.

### Person

- Address the operator as **you**.
- The system **MUST NOT** refer to itself in first person.
- **MUST NOT** use "we".

### Casing

- **Command names and flags**: lowercase, monospace. In prose, render in monospace even inside sans headings.
- **Product names**: exactly as canonical — `SmallAIOS`, `ModelGate`, `smctl`. `smctl` stays lowercase at sentence start; prefer rewording.
- **UI labels**: Sentence case. Never Title Case.
- **Acronyms**: canonical casing — `ONNX`, `MCP`, `CAN`, `TLS`, `QUIC`, `TLA+`, `DDS`, `ARINC`, `MIL-STD-1553`, `SpaceWire`.
- **Button labels**: imperative verbs — `Start build`, `Archive spec`. Never gerunds (`Building…`) or bare nouns (`Build`).

### Emoji and ornament

**MUST NOT** appear anywhere: CLI output, web UI, docs, commit messages authored through this system, error text. Unicode box-drawing characters (`•`, `─`, `│`, `└─`, `├─`) are permitted in terminal output as structural, not decorative.

### Numbers and units

- Bytes: `15 MB`, `< 8 MB`. Space before unit.
- Durations: `< 50 ms`, `230 µs`. Prefer SI.
- Counts: comma thousands separators — `4,143 tests`.
- Versions: `v0.1.0`. Never `V0.1` or `ver 0.1`.

### Status vocabulary

Borrow the CLI's existing terms; do not invent synonyms.

| Term | Meaning |
|---|---|
| `clean` / `dirty` | Worktree state |
| `ahead N` / `behind N` | Relative to upstream |
| `pending` / `running` / `passed` / `failed` | Task or build state |
| `active` / `archived` | Spec state |
| `verified` / `unverified` | Signature / formal-proof state |
| `present` / `absent` | Resource existence |

Any new status term added to the system **MUST** be declared here before use in copy.

### Error messages

Three-part structure. **MUST** include all three:

1. What happened (subject + past participle)
2. What it means (plain-language consequence)
3. What to do next (an executable command, not advice)

Canonical form:

> **Workspace validation failed.** `.smctl/workspace.toml` is missing a `[flow]` section.
> Run `smctl config edit` and add a `[flow]` block, or re-initialize with `smctl workspace init --force`.

### Canonical copy patterns

- Empty state: `"No active specs. Run smctl spec new <name> to create one."`
- Confirmation: `"This will merge feature/gpu-accel into develop in 2 repos. Continue? [y/N]"`
- Success line: `"Archived 2026-04-17-gpu-accel-v1. 4 commits merged to develop."`
- Destructive: `"Force-remove worktree gpu-accel? Uncommitted changes in 1 repo will be lost. [y/N]"`

## Iconography

### Set

Canonical icon set: **Lucide** (ISC). Substitute for a future bespoke set; superseded when bespoke glyphs are commissioned.

Bespoke-glyph candidates (deferred):

- Bus protocols: CAN, ARINC 429, ARINC 664, MIL-STD-1553, SpaceWire, DDS
- SmallAIOS-specific: syscall, formal-proof, unikernel-boot

### Size

- `16px` — inside buttons, dense rows
- `20px` — navigation
- `24px` — empty states, headers
- **MUST NOT** exceed `32px` in UI

### Stroke and color

- Stroke-only. Never filled variants.
- Stroke width: `1.5px` on a `24px` grid.
- Color: `currentColor`. **MUST NOT** embed color in the SVG.

### Labeling

Every icon **MUST** be redundantly labeled by adjacent text or `aria-label`. Decorative-only icons are forbidden.

## Brand Assets

### Logos

Assets live at:

- `ui/assets/logo-mark.svg` — compact glyph (3×3 inset-square grid forming an "S" channel)
- `ui/assets/logo-wordmark.svg` — horizontal lockup
- `ui/assets/logo-stacked.svg` — vertical lockup

All three are **placeholder proposals**. Superseded by `brand-v1` when an approved identity is adopted. Any external document embedding these marks **SHOULD** carry a line noting they are provisional.

Rules:

- Single-color. **MUST NOT** use gradients or bevels.
- **MUST** work on any background.
- Clear space: one mark-height around the glyph.

## Layout (web surfaces only)

Applies only when a web surface is eventually shipped under a future change. Declared here so the contract exists.

- Top chrome: fixed `64px`.
- Left rail: fixed `240px` on desktop, collapsible to `56px`.
- Status line: pinned `24px` bottom bar in monospace. Shows workspace, branch, last-sync time. Always visible.
- Content max-width: `1440px`. Tables extend full-width within their card.
- Cards: `1px --fg-3` border, `6px` radius, no shadow, `--bg-1` background, `16 / 24` padding.
- Translucency / blur: used **only** on the command palette and top-bar chrome.

## Claude Code Skill

A pointer skill **MUST** exist at `.claude/skills/smallaios-design/SKILL.md`. It **MUST**:

- Declare `name: smallaios-design`
- Set `user-invocable: true`
- Reference `ui/README.md` and `ui/colors_and_type.css` by repo-relative path
- List the reflexive design principles (terminal-first, 8px grid, sentence case, no emoji, etc.)
- Not duplicate the full README; act as a pointer

## Conformance

This spec is editorially enforced. PRs touching CLI copy, docs, or any surface that renders product-visible strings **SHOULD** cite this spec in the description when introducing new status terms, error messages, or button labels.

## Provisional and Substitution Flags

The following pieces are adopted as placeholders and will be superseded:

| Item | Substitute | Supersedes via |
|---|---|---|
| Body sans | IBM Plex Sans | `design-system-v2` (bespoke typeface decision) |
| Monospace | JetBrains Mono | `design-system-v2` |
| Logo marks | Three net-new SVGs in `ui/assets/` | `brand-v1` |
| Icon set | Lucide (CDN, ISC) | Future bespoke-glyph commission change |
| Font delivery | Google Fonts CDN | Vendored-local when any offline / air-gapped surface ships |

## References

- `ui/README.md` — narrative source document; this spec is the contract form
- `ui/SKILL.md` — portable Agent Skill manifest (kept alongside the design assets for external use)
- `ui/colors_and_type.css` — authoritative token file
- `openspec/changes/smctl-tool-v1/specs/cli-interface.md` — CLI surface that must conform to voice rules
- [Lucide icons](https://lucide.dev) — ISC license
- [IBM Plex Sans](https://github.com/IBM/plex) — SIL OFL 1.1
- [JetBrains Mono](https://github.com/JetBrains/JetBrainsMono) — SIL OFL 1.1
