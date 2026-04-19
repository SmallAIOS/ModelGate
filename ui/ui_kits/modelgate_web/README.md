# ModelGate web — UI kit

A **net-new** web dashboard for a ModelGate instance. No production web UI exists — this is a designer's proposal drawn against the contract in `ModelGate/openspec/changes/smctl-tool-v1/specs/cli-interface.md`. Every surface here maps 1:1 to a `smctl gate` subcommand.

## Files

| File | Purpose |
|---|---|
| `index.html` | Full dashboard demo |
| `Shell.jsx` | Chrome — `TopBar`, `LeftRail`, `StatusLine`, `Icon`, `LogoMark` |
| `Screens.jsx` | Views — `OverviewScreen`, `ModelsScreen`, `PolicyScreen`, `TerminalScreen` + shared atoms (`Badge`, `Btn`, `Table`, `Section`, `KVGrid`) |
| `App.jsx` | Route switching, command palette (⌘K) |

## Screens

- **Overview** — workspace summary: stats, linked repos, recent builds, open alerts. Entry point.
- **Models** — registered models table (`smctl gate models list`), filter tabs, register CTA.
- **Policy** — Cedar policy viewer + property analysis (`smctl gate policy analyze`) + signing metadata.
- **Terminal** — an embedded terminal for `smctl` (links to the smctl UI kit).
- **Routes / Boundaries / Settings** — placeholder screens, marked out-of-scope for this pass.

## Interactions

- ⌘K / Ctrl+K opens the command palette (translucent overlay, backdrop blur — the one permitted use of blur per VISUAL FOUNDATIONS).
- Left-rail items persist selection in `localStorage`.
- Alert bell shows an amber dot when there are open alerts.

## Scope caveats

- **No real data.** Every number, name, and status is illustrative.
- **Routes / Boundaries / Settings** are intentionally stubbed. Flag anything beyond overview / models / policy if the team wants to prioritize it.
- **Icons** are hand-drawn Lucide lookalikes inlined into `Shell.jsx` so the kit is self-contained. Swap for the real Lucide package in production.
