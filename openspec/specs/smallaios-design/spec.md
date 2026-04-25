# smallaios-design Specification

## Purpose

The SmallAIOS design system is the contract every user-facing surface — CLI output, error messages, web dashboard labels, slides, docs — must follow. It declares voice rules, status vocabulary, error-message structure, and product-name casing. Reference artefacts (tokens, iconography, kit) live under `ui/`.

## Requirements

### Requirement: Imperative button voice

User-facing buttons and CLI verbs SHALL use imperative verbs (`Start build`, `Register model`, `Run inference`). Progressive (`Building…`, `Registering…`) is reserved for transient busy-state labels and MUST NOT replace the imperative.

#### Scenario: Register model button

- **WHEN** the dashboard renders the model-registration entry point
- **THEN** the button label MUST read "Register model"
- **AND** the busy-state label MAY read "Uploading…" but MUST NOT change the resting label

### Requirement: Address the operator as `you`

User-facing copy SHALL address the operator as `you`. It MUST NOT use `we`. Sentence-case labels SHALL be used for headings and labels.

#### Scenario: Confirmation dialog body

- **WHEN** a destructive action opens a confirmation dialog
- **THEN** the body text MUST address the operator as "you" (e.g. "Remove `<name>` from this ModelGate instance? This cannot be undone.")
- **AND** MUST NOT use "we"

### Requirement: Three-part error messages

Every user-facing error message SHALL contain three parts: what happened, what it means, what to do next (an executable command or concrete pointer).

#### Scenario: ModelGate unreachable

- **WHEN** `smctl gate status` runs against an unreachable URL
- **THEN** the error body MUST identify the connection-refused condition
- **AND** MUST include a remediation line that names `--url` or `MODELGATE_URL` as the next action

### Requirement: Canonical status vocabulary

User-facing status labels SHALL reuse the canonical terms: `clean` / `dirty`, `ahead N` / `behind N`, `pending` / `running` / `passed` / `failed`, `active` / `archived`, `verified` / `unverified`, `present` / `absent`. Introducing new terms requires updating this spec first.

#### Scenario: Empty state copy

- **WHEN** the dashboard renders the Models screen with no rows
- **THEN** the empty state MUST read "No models registered" (matching the CLI's wording exactly)

### Requirement: No emoji, no exclamation points

User-facing strings MUST NOT contain emoji or exclamation points. ASCII-only.

#### Scenario: Audit voice failure

- **WHEN** an automated copy audit walks the SPA's TSX files
- **THEN** the audit MUST flag any string containing an emoji or exclamation point as a violation

### Requirement: Product-name casing

The product names `SmallAIOS`, `ModelGate`, and `smctl` SHALL be cased exactly as written. `smctl` stays lowercase even at sentence start; reword rather than capitalise.

#### Scenario: Lowercase smctl at sentence start

- **WHEN** a documentation sentence begins with the tool name
- **THEN** the author MUST reword to avoid leading-position capitalisation
- **AND** MUST NOT write "Smctl" or "SMCTL"
