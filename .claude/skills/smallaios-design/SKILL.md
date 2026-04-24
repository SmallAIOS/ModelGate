---
name: smallaios-design
description: Use this skill to generate well-branded interfaces and assets for SmallAIOS / ModelGate / smctl — production code, static mocks, slides, docs, or CLI copy. Declares the canonical tokens, voice, casing, iconography, and brand assets for a safety-critical terminal-first Rust unikernel and its CLI control plane. Invoke when the user asks to design, style, brand, mock up, or render any user-facing surface in this repo.
type: skill
user-invocable: true
---

# SmallAIOS / ModelGate / smctl design system

The canonical design source lives at the repo root under `ui/`. This file is a thin pointer; do not duplicate `ui/README.md` here.

## Read these first

- `ui/README.md` — full narrative: product context, voice, colors, type, spacing, motion, iconography, caveats
- `ui/colors_and_type.css` — authoritative token contract. `@import` or `<link>` this at the root of any HTML artifact. Do not re-declare tokens inline.
- `openspec/changes/design-system-v1/specs/design-system.md` — the contract form of the same rules. When it disagrees with `ui/`, the spec wins and `ui/` is corrected.

## Assets

- `ui/assets/logo-mark.svg`, `ui/assets/logo-wordmark.svg`, `ui/assets/logo-stacked.svg` — provisional logo proposals. Single-color, no gradients.
- `ui/preview/*.html` — atomic design-system cards (type, color, spacing, components, brand). Reference fixtures; not production code.
- `ui/ui_kits/smctl_cli/` — terminal recreation of `smctl` output (reference, not runnable).
- `ui/ui_kits/modelgate_web/` — net-new dashboard proposal (reference; no web UI is shipping yet).

## When invoked

If the user asks to build or design something:

1. Ask what surface they want (CLI output, static HTML mock, slide, doc diagram, logo application) if it is not clear.
2. Read `ui/README.md` in full before producing output. Every decision below is documented there with rationale.
3. For HTML artifacts, always `<link>` or `@import` `ui/colors_and_type.css` and use its variables. Never hardcode color, type, or spacing values.
4. For CLI copy edits, apply the voice rules reflexively (see below) and cite `design-system-v1/specs/design-system.md` in PR descriptions if introducing new status terms, error messages, or button labels.
5. If generating throwaway prototypes, copy assets out of `ui/` into the target location rather than linking into the skill tree.

## Reflexive design principles

Apply these without being asked:

- **Terminal-first, monospace-forward.** Rules, not cards. Measured grids. Tables are the native layout.
- **8px grid** (4px half-step). Spacing scale: `4 8 12 16 24 32 48 64 96`.
- **Neutral palette.** Color only for status signals (`--sig-ok` green, `--sig-warn` amber, `--sig-err` red) and one `--ion` blue for active / focus. Never decorative color.
- **Two shadow elevations** (`--shadow-1` overlays, `--shadow-2` modals). No soft drop shadows.
- **Motion**: `120 / 200 / 320 ms` with `cubic-bezier(0.2, 0, 0, 1)`. No bounces, springs, confetti, or shimmer.
- **Focus ring**: 2px `--ion`, offset 2px, always visible. Non-negotiable.
- **1px borders**, `--fg-3`. Radii: `0` for tables, `4px` inputs/buttons, `6px` cards, `8px` modals max.
- **Sentence case everywhere.** Never Title Case. Button labels are imperative verbs (`Start build`, not `Building…`).
- **`you`, never `we`.** The system reports state; it does not have a personality.
- **No emoji. No exclamation points.** Anywhere. Unicode box-drawing (`•`, `─`, `│`, `└─`, `├─`) is permitted as structural CLI output only.
- **Icons**: Lucide-style, 1.5px stroke, 24px grid, stroke-only, `currentColor`, always labeled.
- **Status vocabulary**: reuse the canonical terms (`clean` / `dirty`, `ahead N` / `behind N`, `pending` / `running` / `passed` / `failed`, `active` / `archived`, `verified` / `unverified`, `present` / `absent`). Introducing a new term requires updating the spec first.
- **Error messages**: three parts — what happened, what it means, what to do next (an executable command, not advice).
- **Numbers**: space before unit (`15 MB`, `< 50 ms`), comma thousands (`4,143`), SI durations.
- **Logging**: conforms to RFC 5424 (syslog). Severity names are `Emergency` / `Alert` / `Critical` / `Error` / `Warning` / `Notice` / `Informational` / `Debug`. Use `tracing` with an RFC 5424 formatter; never invent a log wire format. STRUCTURED-DATA for contextual fields, not ad-hoc string interpolation. MSGIDs are stable. This governs `tracing` / `log` output, not interactive CLI UX.

## Provisional items

These are placeholders; flag any decision that depends on them:

- Fonts (IBM Plex Sans, JetBrains Mono) — substitute for a future bespoke typeface
- Logo marks in `ui/assets/` — designer proposals, not an approved identity
- Lucide icons — substitute for a future bespoke set including per-bus-protocol glyphs (CAN, ARINC 429, MIL-STD-1553, SpaceWire, DDS)
