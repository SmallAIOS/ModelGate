# ModelGate web actions — Specification

## Components

### `<ConfirmDialog>`

Controlled confirmation dialog for destructive actions.

```tsx
type ConfirmDialogProps = {
  open: boolean;
  title: string;
  body: React.ReactNode;
  /** Equivalent CLI command, rendered as a <code> block. */
  cliEquivalent?: string;
  confirmLabel: string;       // e.g. "Remove"
  cancelLabel?: string;       // default "Cancel"
  /** When true, renders the confirm button in destructive style. */
  destructive?: boolean;
  onConfirm: () => void;
  onCancel: () => void;
};
```

- `aria-modal="true"`, `role="dialog"`, focus trap inside the dialog.
- Pressing `Escape` calls `onCancel`. Clicking the backdrop calls `onCancel`.
- Confirm button is the default focus.

### `<Toaster>` and `useToast()`

Single mount point in `App.tsx`. Bottom-right anchored. Caps at 4 visible toasts; older toasts shift up.

```ts
type Toast = {
  kind: 'success' | 'error' | 'info';
  message: string;
  /** Optional secondary line — usually a remediation clause. */
  detail?: string;
};

function useToast(): {
  show: (toast: Toast) => void;
};
```

- Auto-dismiss after 5 seconds (success/info), 8 seconds (error).
- Click any toast to dismiss immediately.
- Errors render with the same vocabulary as the Rust-side remediation clauses ("ModelGate unreachable. Start ModelGate and retry, or pass --url ...").

### `<JsonEditor>`

```tsx
type JsonEditorProps = {
  value: string;
  onChange: (next: string) => void;
  /** Surfaced when JSON.parse(value) throws. */
  parseError?: string;
};
```

- Monospace `<textarea>`, sized to 12 rows by default.
- Caller is responsible for parsing on submit; the component does not auto-validate, but the design system styles the border red when `parseError` is non-empty.

## Mutation Hooks

All hooks live in `src/hooks/mutations.ts`. Each calls the typed `api` client (or raw `fetch` for multipart), invalidates the relevant query keys on success, and pushes a toast on success / failure.

```ts
useRegisterModel(): UseMutationResult<Model, GateApiError, File>;
// On success: invalidates ['models']. Toast: "Model X registered".

useRemoveModel(): UseMutationResult<void, GateApiError, string>;
// Argument is the model name. On success: invalidates ['models'].

useSetRoute(): UseMutationResult<Route, GateApiError, { model: string; endpoint: string }>;
// On success: invalidates ['routes'].

useTestInference(): UseMutationResult<InferenceResult, GateApiError, { model: string; payload: unknown }>;
// On success: no cache invalidation. Caller renders the result inline.
```

## Screen Wiring

### `ModelsScreen`

- Section header gains a `Register model` button (top-right).
  - Opens `<RegisterModelDialog>`: file picker, "Upload" submits, progress bar fed by `fetch` upload events. On success, dialog closes, toast appears.
- Each row gains a `Remove` action button.
  - Opens `<ConfirmDialog>` with title "Remove model", body "Remove `<name>` from this ModelGate instance? This cannot be undone.", `cliEquivalent` = `smctl gate models remove <name>`, destructive style.
  - On confirm, the row's button enters a loading state and the mutation runs.

### `RoutesScreen` (renamed from the old Policy + Routes combo)

- Header gains a `Set route` button.
  - Opens an inline form (no separate dialog — routes are a less-destructive change). Two fields: `Model` (select, populated from `useModels()`) and `Endpoint` (text). `Save route` submits.
  - Toast on success/failure. Form resets on success.

### `InferenceScreen`

- New screen. Two-column layout: input on the left, result on the right.
- Left column:
  - Model select (from `useModels()`).
  - `<JsonEditor>` for the payload, default value `{"prompt": ""}`.
  - `Run inference` button (disabled when JSON parse fails).
- Right column:
  - When idle: empty state, `Run inference to see a result`.
  - On success: pretty-printed `InferenceResult` (model, latency, tokens, output JSON).
  - On error: error box with remediation hint.

## Voice & Accessibility Rules

Every label in this change passes the existing `design-system-v1` voice rules. Specific points relevant to the new components:

- **Buttons** — imperative verbs only: `Register model`, `Remove model`, `Set route`, `Run inference`, `Cancel`. Never `Registering…`; the loading state hides the verb behind a small spinner without changing the label text.
- **Toasts** — sentence case, one short sentence per line. Errors lead with what failed (`Could not register model.`) and follow with a remediation line where one is meaningful.
- **Dialogs** — body text addresses the operator as `you`, never `we`.
- **Empty states** — `No models registered`, `No routes configured`, `Run inference to see a result`. Match the CLI's wording.

Accessibility:

- Every actionable element is `<button>` or `<a>`. No divs with `onClick`.
- Confirm dialogs trap focus and announce themselves to screen readers via `role="dialog"` + `aria-modal` + `aria-labelledby`.
- The toaster region uses `role="status"` + `aria-live="polite"` so screen readers receive the message without a focus jump.
