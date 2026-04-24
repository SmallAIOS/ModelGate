# SmallAIOS Design System

A design system for **SmallAIOS** — a minimal, secure, Rust-based operating system kernel purpose-built for AI inference — and **ModelGate / smctl**, its developer control-plane CLI and (forthcoming) web interface.

This system is greenfield: no production UI exists yet. The foundations here are derived from the product's character — a safety-critical, formally-verified unikernel that boots directly to ONNX inference, with ~46 syscalls, targeting aerospace/automotive workloads (CAN, ARINC 429/664, MIL-STD-1553, SpaceWire, DDS). The design vocabulary is therefore **terminal-first, monospace-forward, high-density, instrumentation-grade** — closer to an ATC console or a network operator's NOC than to a consumer SaaS dashboard.

---

## Sources

All sources are GitHub repositories read directly via the GitHub connector. The reader is not assumed to have access.

| Source | What it gave us |
|---|---|
| `github.com/SmallAIOS/SmallAIOS` (branch `develop`) | Product positioning, crate layout, formal-verification framing, CAN/ARINC/MIL-STD-1553 domain language, CLI patterns (`just`, env vars), Gitflow conventions |
| `github.com/SmallAIOS/ModelGate` (branch `main`) | `smctl` CLI specification — command tree, global flags, exit codes, JSON output format, MCP server mode |
| `ModelGate/openspec/changes/smctl-tool-v1/specs/cli-interface.md` | Canonical command tree for `smctl` |
| `ModelGate/openspec/changes/smctl-tool-v1/design.md` | Design rationale (workspace manifest, worktrees, gate subcommands, MCP mapping) |
| `SmallAIOS/CLAUDE.md` | Layer model, build toolchain, formal-verification surface area |

No Figma, brand book, logos, or sample decks were provided. All visual decisions in this system are net-new, designed against the product's stated character and the CLI contract.

---

## Index

**Foundations**
- `colors_and_type.css` — CSS variables for color, type, spacing, shadow, radius, motion. Includes a dark-mode override and semantic type classes (`.ds-h1`, `.ds-body`, `.ds-mono`, `.ds-label`, `.ds-kbd`).
- Fonts load from Google Fonts (IBM Plex Sans + JetBrains Mono). See Caveats for the substitution flag.

**Preview cards** (seen in the Design System tab)
- `preview/type-*.html`, `preview/color-*.html`, `preview/spacing-*.html`, `preview/components-*.html`, `preview/brand-*.html`

**Assets**
- `assets/logo-mark.svg`, `assets/logo-wordmark.svg`, `assets/logo-stacked.svg`

**UI kits** (recreations + net-new surfaces)
- `ui_kits/smctl_cli/` — terminal recreation: 6-scene click-through (`workspace status` → `spec ff` → `build` → `feat` → `gate status` → `done`) with keyboard nav. Components: `Prompt`, `Line`, `Caret`, `WorkspaceStatus`, `SpecValidation`, `BuildOutput`, `GateStatus`, `ConfirmLine`.
- `ui_kits/modelgate_web/` — net-new web dashboard for a ModelGate instance. Screens: Overview, Models, Policy, Terminal. Chrome: `TopBar` (64px), `LeftRail` (240px), `StatusLine` (24px, vim-style). ⌘K command palette.

**Handoff**
- `SKILL.md` — portable Agent Skill manifest so this system can be used in Claude Code
- `README.md` — this file

---

## Product context

**SmallAIOS.** A clean-room `#![no_std]` Rust unikernel that boots directly to ONNX inference. ~46 syscalls vs Linux's ~450. Six architectures (x86-64, AArch64, RISC-V, NVIDIA, AMD, Intel GPU). Post-quantum crypto default (ML-KEM-768 + ML-DSA-65). 19 TLA+ models, Lean 4 proofs, MC/DC coverage targets for DO-178C DAL A. Release binary < 15 MB.

**ModelGate.** The model-gateway + developer-tooling hub. Houses `smctl`, the CLI.

**smctl.** One binary, hierarchical subcommands on the `*ctl` convention (kubectl, systemctl). Commands span `workspace`, `worktree`, `flow` (gitflow), `spec` (OpenSpec), `build`, `gate` (ModelGate control-plane), `serve` (MCP server), `config`. Every subcommand has a 1:1 MCP tool equivalent.

**Who uses it.** Kernel engineers, formal-methods practitioners, embedded/aerospace systems engineers, and AI coding assistants (via MCP). Not consumer users. Not marketing.

---

## CONTENT FUNDAMENTALS

### Voice

Precise, declarative, low-affect. The product is a flight control. Copy should read like instrumentation: factual, verb-first, present tense. No cheerleading, no emoji, no exclamation points. The reader is a senior engineer whose time is worth more than yours.

### Casing

- **Command names and flags** are always lowercase, monospace, and not code-formatted in headings (use the sans heading font, but in monospace): `smctl workspace status`.
- **Product names** — `SmallAIOS`, `ModelGate`, `smctl` — are cased exactly as defined. `smctl` stays lowercase even at sentence start (prefer rewording so it doesn't).
- **UI labels** use Sentence case, never Title Case. "Workspace status", not "Workspace Status".
- **Acronyms** (ONNX, MCP, CAN, TLS, QUIC, TLA+, DDS) retain canonical casing.
- **Button labels** are imperative verbs: `Start build`, `Archive spec`, not `Build` or `Archiving…`.

### Person

Use **you** for the operator. Never **we**. Never **your friends at**. The system never refers to itself in first person; it reports state.

- ✅ "Workspace not found. Run `smctl workspace init` to create one."
- ❌ "I couldn't find your workspace — let's create one together!"

### Emoji and ornament

Not used. Anywhere. Not in the CLI, not in the web UI, not in docs.

### Numbers and units

- Bytes: `15 MB`, `< 8 MB`. Space before unit.
- Durations: `< 50 ms`, `230 µs`. Prefer SI.
- Counts: `4,143 tests`, `21 crates`, `29 operators`. Comma thousands separators.
- Versions: `v0.1.0`, never `V0.1` or `ver 0.1`.

### Status language

State is binary and unambiguous. Borrow the CLI's vocabulary:

| Term | Meaning |
|---|---|
| `clean` / `dirty` | repo worktree state |
| `ahead N` / `behind N` | relative to upstream |
| `pending` / `running` / `passed` / `failed` | task or build state |
| `active` / `archived` | spec state |
| `verified` / `unverified` | signature / formal proof state |
| `present` / `absent` | resource existence |

### Error messages

Structured. Three parts: what happened, what it means, what to do next.

> **Workspace validation failed.** `.smctl/workspace.toml` is missing a `[flow]` section.
> Run `smctl config edit` and add a `[flow]` block, or re-initialize with `smctl workspace init --force`.

### Copy examples (canonical)

- Empty state: "No active specs. Run `smctl spec new <name>` to create one."
- Confirmation: "This will merge `feature/gpu-accel` into `develop` in 2 repos. Continue? [y/N]"
- Success line: "Archived 2026-04-17-gpu-accel-v1. 4 commits merged to develop."
- Destructive: "Force-remove worktree `gpu-accel`? Uncommitted changes in 1 repo will be lost. [y/N]"

---

## VISUAL FOUNDATIONS

### Motifs

- **Terminal-native.** Monospace as the workhorse face. The sans face is reserved for display and dense chrome labels.
- **Rules, not cards.** Primary structure is 1px horizontal rules on a flat background. Cards exist but are rare; when used, they are 1px-bordered with no shadow, no radius above `6px`.
- **Measured grids.** Everything lives on an 8px grid with a 4px half-step. Tables are the native layout.
- **Instrumentation.** Small-caps labels, numeric badges, status dots, uptime counters, ring-style progress — borrow from NOC/telemetry UIs.
- **Two-tone neutrals.** The palette is intentionally near-monochromatic. Color is reserved for status signals.

### Colors

Defined in `colors_and_type.css`. Base neutral is a cool paper white with graphite text. Dark mode is a true dark (not navy, not charcoal-with-blue) paired with off-white text. Accent is a single **signal green** (`--sig-ok`) reminiscent of phosphor CRTs and status LEDs; it is used sparingly for "verified/online/passed." Secondary accent is a **signal amber** for warnings. Failures use a restrained red (`--sig-err`). The rest is 7 steps of gray.

There is one spot color — **ion** (`#3451FF`) — reserved for active/focus states, selected rows, and primary buttons. It never appears in decoration.

### Type

- **Display + body sans:** IBM Plex Sans (substitute for a bespoke sans — see Caveats). Geometric-humanist, unmistakably technical, high legibility at small sizes.
- **Monospace:** JetBrains Mono. Six-digit numerals, slashed zero, clear disambiguation of `iI1l` and `oO0`. Used for CLI output, tables of numbers, IDs, branch names, hashes.
- **Display numerals:** IBM Plex Sans at tabular-nums.

Scale is a restrained 7-step ramp (12 / 13 / 14 / 16 / 20 / 28 / 40). No display-script, no italic, no variable axes beyond weight (400, 500, 600).

### Spacing

8px grid with 4px half-step. Scale: `4 8 12 16 24 32 48 64 96`. Dense tables use `8/12`. Card padding is `16/24`. Section rhythm is `32/48`.

### Backgrounds

- Flat. No gradients anywhere except one reserved use: **protection gradients** under translucent top bars, 0% → 12% opacity of `--bg-1`.
- No hand-drawn illustrations. No patterns. No textures.
- Imagery, when present, is black-and-white (duotone `ink` + `paper`) or a single hairline schematic. Never color photography. Never stock.

### Animation

- Minimal. Motion communicates state change, not delight.
- **Easing:** `cubic-bezier(0.2, 0, 0, 1)` (ease-out-expo-ish) for most UI transitions. `linear` for progress indicators.
- **Durations:** `120ms` micro (hover, focus ring), `200ms` standard (panel open), `320ms` large (route change). Never longer.
- No bounces. No springs. No confetti. No loading shimmer (prefer a solid caret-blinking skeleton line at 1Hz, like a terminal).

### Hover / press / focus

- **Hover:** background step +1 on interactive rows (e.g., `--bg-1` → `--bg-2`). Borders gain `--fg-2` on buttons. No elevation change.
- **Press:** background step +2, no scale change. Buttons darken; do not shrink.
- **Focus:** 2px `--ion` ring, offset `2px`, always visible (not `:focus-visible`-only on keyboard flows). Non-negotiable — this is an accessibility-sensitive engineering tool.
- **Disabled:** 0.4 opacity on the whole control.

### Borders

- 1px, `--fg-3`. Never thicker. Never dashed in UI chrome (dashes are reserved for skeleton/placeholder states).
- Corner radii: `0` for tables and data chrome, `4px` for inputs and buttons, `6px` for cards and overlays, `8px` max for modals. Nothing is pill-shaped except status dots and segmented controls.

### Shadows

Two elevations only. `--shadow-1` for hovering overlays (tooltips, menus). `--shadow-2` for modals. Shadows are tight and cool — not soft drop shadows; closer to a 1px hard line plus an 8px 8% shade.

### Transparency / blur

Used in exactly one place: the command palette and top-bar chrome, which sit over content with `backdrop-filter: blur(16px) saturate(120%)` and a 70% background. Not elsewhere — translucency is a cue that the layer is modal or non-persistent.

### Layout rules

- Fixed 64px top chrome; fixed 240px left rail on desktop (collapsible to 56px).
- Content max-width `1440px` on wide screens; tables extend full-width within their card.
- A monospace "status line" is pinned to the bottom 24px of app screens (like `vim`): current workspace, current branch, last-sync time. Always visible in the web UI.

### Cards

When used: 1px `--fg-3` border, `6px` radius, no shadow, `--bg-1` background. Internal padding `16/24`. Header is a 40px-tall monospaced label row with a trailing `•` status dot.

### Imagery color vibe

Cool. Black-and-white with one touch of ion blue. Never warm, never cinematic grain, never HDR. When a product shot or diagram is needed, it is rendered as a 1px-stroke schematic in `--fg-1` on `--bg-0`.

---

## ICONOGRAPHY

The codebase ships no icon set. For this design system we adopt **Lucide** (ISC license) as the canonical icon system, loaded from CDN.

Rationale: Lucide is a fork of Feather — 1.5px strokes, 24px grid, square caps, round joins, geometric construction. It reads as "technical CAD line" rather than "marketing illustration," matching the instrumentation motif. It covers everything `smctl` needs (`terminal`, `git-branch`, `git-merge`, `package`, `cpu`, `hard-drive`, `shield-check`, `activity`, `pause`, `play`, `check`, `x`, `circle-dot`, `chevron-*`).

**Usage rules:**
- Size `16px` inside buttons and dense rows, `20px` in nav, `24px` in empty states and headers. Never above `32px` in UI.
- Stroke inherits `currentColor`. Color never embedded.
- Never filled variants in this system. Stroke-only, consistent.
- Never decorative — every icon must be redundantly labeled by adjacent text or an `aria-label`.
- **Emoji:** never. Unicode pictographs: never. Exception: `•` `─` `│` `└─` `├─` box-drawing for terminal output, which is canonical not decorative.

**Substitution flag:** Lucide is a substitute for a bespoke set. If the team wants custom icons (e.g. per-bus-protocol glyphs for CAN/ARINC/1553), those should be commissioned as 24px 1.5px-stroke SVGs matching the Lucide grid.

### Logos

`assets/logo-mark.svg`, `assets/logo-wordmark.svg`, `assets/logo-stacked.svg`. The mark is a compact glyph that reads as both a chip/die and a schematic gate — a 3×3 grid of inset squares forming a stylized "S" channel. Single-color, works on any background. No gradients, no bevels.

---

## CAVEATS

1. **Font substitution.** No bespoke brand typeface exists. We use **IBM Plex Sans** and **JetBrains Mono** from Google Fonts — both excellent technical faces but likely not the final choice. Flag any brand-level type decision as a future revision.
2. **Logo is net-new.** No logo was provided. The mark in `assets/` is a designer proposal, not an approved identity.
3. **No provided screenshots / designs.** The CLI terminal UI kit is a straightforward recreation of `smctl`'s actual output contract. The web dashboard (`ui_kits/modelgate_web`) is a net-new proposal drawn against the CLI spec.
4. **No sample decks** were provided, so no `slides/` folder is included.
5. **Icon set** is a substitute (Lucide CDN). See ICONOGRAPHY.
