---
name: smallaios-design
description: Use this skill to generate well-branded interfaces and assets for SmallAIOS / ModelGate / smctl, either for production or throwaway prototypes / mocks. Contains design guidelines, colors, type, fonts, assets, and UI kit components for a safety-critical, terminal-first Rust unikernel and its CLI control plane.
user-invocable: true
---

Read the `README.md` file within this skill, and explore the other available files.

Key files:
- `README.md` — product context, content fundamentals, visual foundations, iconography
- `colors_and_type.css` — CSS variables for color, type, spacing, shadow, radius, motion (import once at the root of any page)
- `assets/` — logo mark, wordmark, stacked lockup (SVG)
- `ui_kits/smctl_cli/` — high-fidelity terminal recreation of the `smctl` CLI
- `ui_kits/modelgate_web/` — net-new web dashboard for a ModelGate instance
- `preview/` — atomic design-system cards (type, colors, spacing, components, brand)

If creating visual artifacts (slides, mocks, throwaway prototypes, etc.), copy the assets you need out of this skill folder and create static HTML files for the user to view. Always `@import` or `<link>` `colors_and_type.css` at the root so tokens are consistent.

If working on production code, copy assets and read the rules in `README.md` to become an expert in designing with this brand.

If the user invokes this skill without any other guidance, ask them what they want to build or design, ask clarifying questions, and act as an expert designer who outputs HTML artifacts _or_ production code, depending on the need.

**Design principles to apply reflexively:**
- Terminal-first, monospace-forward. Rules, not cards. Measured grids.
- Neutral palette. Color only for status signals + one ion blue for active/focus.
- Sentence case everywhere. Imperative verbs on buttons. `you`, never `we`. No emoji, no exclamation points.
- 8px grid. 1px borders. Two shadow elevations. Tight motion (120/200/320 ms).
- Icons: Lucide-style, 1.5px stroke, 24px grid, stroke-only.
