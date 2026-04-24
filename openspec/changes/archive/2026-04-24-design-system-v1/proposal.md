# design-system-v1 — Proposal

## Why

ModelGate and `smctl` currently have no shared visual or voice contract. As the ecosystem grows — terminal output, forthcoming web surfaces, docs, logos, diagrams — ad-hoc choices will accumulate and drift. A reference design system was drafted greenfield in `ui/`: tokens, rules, logo proposals, icon policy, JSX reference kits, and an Agent Skill manifest for Claude Code. It is not yet adopted, not discoverable by tooling, and not reviewed against the project's engineering needs.

This change adopts the `ui/` artifacts as the canonical design source for any SmallAIOS / ModelGate / `smctl` interface — CLI output styling, future web surfaces, docs, logos, slides — without committing to ship a web dashboard. The JSX kits in `ui_kits/` remain reference fixtures.

Driving concerns:

- **Coherence.** `smctl` CLI output (prompts, tables, status lines) should share vocabulary with any future web or doc surface. Status terms (`clean` / `dirty`, `ahead N` / `behind N`, `pending` / `running` / `passed` / `failed`) are already in the CLI spec; the design system formalizes them as the project-wide lexicon.
- **Voice.** Error messages, confirmations, and empty states across `smctl` should follow a single rubric (what happened → what it means → what to do next). Today each crate improvises.
- **Brand substitution risk.** The draft uses IBM Plex Sans / JetBrains Mono and a net-new logo mark. Those are explicitly flagged as placeholders in `ui/README.md`. We need a change document that records this so later revisions can supersede without re-litigating.
- **Tooling.** `ui/SKILL.md` is a Claude Code Agent Skill but lives where Claude Code does not auto-discover it. Moving a pointer into `.claude/skills/` makes the design rules load for any contributor using Claude Code.

## What Changes

1. **Adopt `ui/` as the design source of truth** — `colors_and_type.css`, `README.md`, `assets/`, `preview/`, `ui_kits/` become the canonical reference. No code move.
2. **Install the `smallaios-design` skill** at `.claude/skills/smallaios-design/` so Claude Code loads the rules automatically. The skill references `ui/` for assets rather than duplicating them.
3. **Declare the voice / lexicon contract** — lift the status vocabulary, casing rules, and error-message structure from `ui/README.md` into the spec so future CLI copy changes can cite it.
4. **Declare the logging contract** — all log output conforms to RFC 5424 (syslog). Severity names, MSGID stability, STRUCTURED-DATA usage, and transport RFCs (5425 / 5426 / 6587) are first-class spec content. Aerospace and automotive deployments expect standards-conformant logs for SIEM ingestion and certification audit.
5. **Record substitution flags** — fonts and logo mark are provisional. Future changes (`design-system-v2`, `brand-v1`) supersede.
6. **Defer web dashboard** — explicitly out of scope. A later `modelgate-web-v1` change decides backend, framework, and shipping commitment.

## Capabilities

### New Capabilities

- `design-system` — Canonical tokens, voice rules, iconography policy, logo assets for all SmallAIOS / ModelGate / `smctl` surfaces.

### Modified Capabilities

- `smctl-cli` — CLI copy (error messages, empty states, status vocabulary) must conform to the voice rules declared here. No behavioral change; spec cross-reference only.

## Impact

### Repository Home

Design source stays in `ModelGate/ui/`. Skill pointer lands in `ModelGate/.claude/skills/smallaios-design/`.

### New Files

```
ModelGate/
├── .claude/
│   └── skills/
│       └── smallaios-design/
│           └── SKILL.md                       # Pointer skill; references ../../ui/
├── ui/                                         # (already present — now canonical)
│   ├── README.md
│   ├── SKILL.md                                # Portable copy retained for external use
│   ├── colors_and_type.css
│   ├── assets/
│   ├── preview/
│   └── ui_kits/
└── openspec/changes/design-system-v1/
    ├── proposal.md
    ├── design.md
    ├── tasks.md
    └── specs/
        └── design-system.md
```

### Affected Repos

| Repository | Impact |
|---|---|
| `SmallAIOS/ModelGate` | Home — design system lives here, skill loads here |
| `SmallAIOS/SmallAIOS` | Downstream — CLI output, docs, and diagrams should cite the voice / lexicon rules |

### Dependencies

None at runtime. Design-time only:

- Google Fonts (IBM Plex Sans, JetBrains Mono) — **placeholder**, flagged for brand revision
- Lucide icons (CDN, ISC license) — **substitute** for bespoke set
- No new Rust crates, no new build steps

### Out of Scope (deferred to later changes)

- Shipping a ModelGate web dashboard (`modelgate-web-v1`)
- Commissioning a bespoke brand typeface
- Approving a final logo mark
- Commissioning per-bus-protocol glyphs (CAN / ARINC 429 / MIL-STD-1553 / SpaceWire / DDS)

## References

- `ui/README.md` — full design system draft (source document for this change)
- `ui/SKILL.md` — portable Agent Skill manifest
- `ui/colors_and_type.css` — token contract
- `openspec/changes/smctl-tool-v1/specs/cli-interface.md` — canonical CLI surface that must conform to the voice rules
- [Lucide icons](https://lucide.dev) (ISC)
- [IBM Plex Sans](https://github.com/IBM/plex) (SIL OFL 1.1)
- [JetBrains Mono](https://github.com/JetBrains/JetBrainsMono) (SIL OFL 1.1)
