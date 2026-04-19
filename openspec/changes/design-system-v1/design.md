# design-system-v1 — Design Document

## Context

A greenfield design system was drafted in `ui/` against SmallAIOS / ModelGate / `smctl` product character: a safety-critical Rust unikernel for AI inference, a CLI control plane, and an AI-assistant integration via MCP. The draft is terminal-first, monospace-forward, instrumentation-grade — closer to an ATC console than a consumer SaaS dashboard. It exists as tokens (CSS variables), voice rules (README), reference HTML previews, JSX UI kits (terminal recreation + proposed web dashboard), three SVG logo proposals, and an Agent Skill manifest.

This change ratifies the draft as the canonical design source without committing to ship the web dashboard. The dashboard JSX stays as reference only.

## Goals / Non-Goals

### Goals

1. One place to look up tokens, voice rules, icon policy, and brand assets.
2. Claude Code auto-loads the design rules when working in this repo.
3. `smctl` CLI copy conforms to a declared voice / lexicon rubric.
4. Provisional pieces (fonts, logo) are explicitly flagged so later revisions can supersede.
5. The design system is portable — contributors without Claude Code can still read `ui/README.md`.

### Non-Goals

1. Not shipping a ModelGate web dashboard. That is `modelgate-web-v1`, a future change.
2. Not commissioning bespoke typefaces or a final logo.
3. Not adding a frontend toolchain (Vite, npm, framework) to the repo.
4. Not adopting the JSX kits as production code. They are fixtures.
5. Not changing existing `smctl` behavior. Voice conformance is an editorial contract, enforced by review, not code.

## Decisions

### Decision 1: `ui/` as canonical, not `.claude/skills/` as canonical

**Choice:** Keep the design source in `ui/`. Install a thin pointer skill at `.claude/skills/smallaios-design/SKILL.md` that references `../../ui/` for assets.

**Rationale:** `ui/` is discoverable by any contributor (GitHub browsing, file search, IDE sidebar). `.claude/skills/` is Claude-Code-specific and invisible to everyone else. Design assets belong where all contributors can find them. Claude Code's skill manifest is lightweight — a pointer is sufficient.

**Alternatives considered:**

- *Move everything to `.claude/skills/smallaios-design/`* — loses discoverability for contributors not using Claude Code. Ties design artifacts to one tool.
- *Duplicate `ui/` into `.claude/skills/`* — two sources drift. Rejected.
- *Skip the skill entirely* — then Claude Code contributors don't get the rules automatically. Rejected.

### Decision 2: Voice / lexicon as editorial contract, not runtime enforcement

**Choice:** The voice rules (casing, person, status vocabulary, error-message structure) are declared in `specs/design-system.md` and enforced by spec review. No lint, no codegen, no runtime check.

**Rationale:** CLI copy is sparse, scattered across crates, and rarely changes. A lint would cost more than the drift it prevents. Voice conformance is a reviewer concern, not a CI concern. If drift becomes a problem we can revisit (e.g., a `smctl-copy` crate that centralizes all user-facing strings).

**Alternatives considered:**

- *Centralize all copy in one crate with compile-time checks* — over-engineered for current surface area. Defer until pain.
- *Codegen error messages from a YAML catalog* — same objection.

### Decision 3: Fonts and logo are provisional, flagged in-spec

**Choice:** IBM Plex Sans, JetBrains Mono, and the three SVG logo marks are adopted as **placeholders**. The spec records them as provisional and names the superseding change type (`brand-v1` for logo, `design-system-v2` for a bespoke typeface decision).

**Rationale:** Calling them placeholders up front prevents future debate about whether the current choices were ever "real." The README caveats section already flags this; the spec makes it formally part of the adoption record.

**Alternatives considered:**

- *Don't adopt the logo at all; leave blank until approved* — but then there is nothing to put on docs or slides today. Worse than a flagged placeholder.
- *Commission the logo now as part of this change* — out of scope; no design budget requested.

### Decision 4: JSX kits stay as reference, not production seed

**Choice:** `ui/ui_kits/modelgate_web/` and `ui/ui_kits/smctl_cli/` remain as static reference files. They are not wired to any build system and not importable.

**Rationale:** The user confirmed these are design reference only. Adopting them as production seed code would force a framework decision (React? Leptos? Dioxus?), a toolchain (Vite, Cargo-leptos), and an auth/transport contract to any backend — none of which belong in a design-system change. `modelgate-web-v1` decides those when the time comes.

**Alternatives considered:**

- *Set up Vite + React scaffolding so the kits run* — scope creep; commits to a stack prematurely.
- *Delete the JSX kits* — loses the most concrete reference artifact of what the design looks like applied. Rejected.

### Decision 5: Icon policy adopts Lucide as substitute

**Choice:** Lucide icons (ISC, CDN-loaded) are the canonical icon set for this version. Bespoke per-bus-protocol glyphs (CAN / ARINC 429 / MIL-STD-1553 / SpaceWire / DDS) are flagged as a future commission.

**Rationale:** Lucide's 1.5px stroke / 24px grid / square-cap / geometric construction reads as "technical CAD line," matching the instrumentation motif. License is permissive. No vendoring burden at design-reference stage.

**Alternatives considered:**

- *Feather* — parent of Lucide, less actively maintained.
- *Tabler / Phosphor* — heavier, more stylized, less engineering-tool feel.
- *Vendor custom-only set now* — premature; no icon inventory pressure yet.

## Risks / Trade-offs

- **Skill discoverability.** Contributors not using Claude Code won't see the skill prompt. Mitigation: `ui/README.md` carries the same rules verbatim.
- **Font licensing in offline / air-gapped builds.** Google Fonts CDN won't work in disconnected environments. Mitigation: when a production web surface ships, vendor the fonts locally. Not a design-system-v1 concern.
- **Logo ambiguity.** Using a placeholder logo in docs and slides may read as "official" externally. Mitigation: add a caveat line in any external doc that embeds the mark, citing this change.
- **Voice enforcement drift.** Without lint, CLI copy can drift. Accepted risk — surface area is small; revisit if incident rate warrants.
- **`ui_kits/` rot.** JSX fixtures may drift from the CLI spec as `smctl` evolves. Accepted — they are reference, not contract. If they become misleading, update or delete in a follow-up.

## Migration Plan

Not applicable. This change is additive. No existing code, docs, or config are modified beyond the new `.claude/skills/` pointer and the spec documents themselves.

## Open Questions

1. Should `ui/preview/*.html` be served as a static site (e.g., via GitHub Pages) so reviewers can view the design cards without cloning? Deferred — answer in a follow-up if anyone asks.
2. Do we want a CI check that fails if `ui/colors_and_type.css` is edited without a version bump comment? Deferred until the token file has more than one consumer.
3. Does SmallAIOS (the kernel repo) want to cite this design system for its own docs, or keep its own? Coordination question for the kernel team; not blocking.
