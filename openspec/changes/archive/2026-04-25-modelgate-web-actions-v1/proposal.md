# ModelGate web actions — Proposal

## Why

`modelgate-web-v1` shipped the read-only dashboard: Overview, Models table, Routes table, Policy and Terminal placeholders. The full mutating surface already exists on the Rust side — `/api/models` (POST), `/api/models/{name}` (DELETE), `/api/routes` (PUT), `/api/inference/{model}` (POST). The browser never calls any of it. Operators who want to register a model still drop back to `smctl gate models add ./model.onnx`.

Problems this solves:

- **The dashboard is decorative.** It surfaces state but cannot change it. A reviewer who opens it during a demo gets a "looks nice, but what does it do?" reaction.
- **Two UX paths for the same task.** `smctl gate` is the source of truth for actions; the dashboard reads the same data but hands you back to the CLI for any change. That's worth fixing now because the divergence will only grow.
- **No confirmation flow precedent.** Once we add destructive actions (`Remove model`), every later destructive action in the SPA needs a consistent pattern.

## What Changes

Add four interactive components to `ui/modelgate-web/`:

1. **Register model** — file picker on the Models screen that streams to `POST /api/models` with a progress indicator.
2. **Remove model** — row-level action on the Models table with a confirmation dialog. Calls `DELETE /api/models/{name}`.
3. **Set route** — small form on the Policy screen that PUTs `/api/routes` with `{model, endpoint}`. Model dropdown is populated from `useModels()`.
4. **Test inference** — a JSON editor + "Run inference" button on a new Inference screen. Posts to `/api/inference/{model}` and renders the result.

No new Rust code. No new MSGIDs. The Rust crate already handles every error variant the browser can hit.

## Capabilities

### New Capabilities

- Confirmation-dialog primitive (`<ConfirmDialog>`) reused by every destructive action
- Toast / status-line surface for action feedback (`<Toaster>`, `useToast()`)
- Mutation hooks (`useRegisterModel`, `useRemoveModel`, `useSetRoute`, `useTestInference`) backed by React Query
- Shared `<JsonEditor>` for the inference payload (textarea + JSON-validate-on-submit; full editor is out of scope)

### Modified Capabilities

- `ModelsScreen` — adds the register button and per-row remove action
- `PolicyScreen` (renamed `RoutesScreen` in the rail; "Policy" framing dropped now that there's a real form to put there)

## Impact

### New Files

```
ui/modelgate-web/src/
├── components/
│   ├── ConfirmDialog.tsx
│   ├── Toaster.tsx
│   └── JsonEditor.tsx
├── hooks/
│   ├── useToast.ts
│   └── mutations.ts          # all four mutation hooks
└── screens/
    ├── InferenceScreen.tsx
    └── RegisterModelDialog.tsx
```

### Modified Files

- `ui/modelgate-web/src/App.tsx` — split out the inline screens into `screens/`, mount the toaster
- `ui/modelgate-web/src/api.ts` — already exports `removeModel`, `setRoute`, `testInference`; add `registerModel(file)` (multipart)
- `ui/modelgate-web/src/styles.css` — minor additions for dialog overlay, toast, form rows

### Dependencies

No new deps. `@tanstack/react-query` is already on the tree; mutations use its `useMutation`.

## Non-Goals

1. **No optimistic updates.** Mutations invalidate the relevant query and let React Query refetch. Optimistic UI is correctness debt I don't want to take on for a v1.
2. **No bulk actions.** Per-row only; a bulk "remove these three" lands in a future change.
3. **No streaming progress for the inference response.** Single-shot POST/JSON. Streaming inference is a separate spec once ModelGate exposes a streaming endpoint.
4. **No drag-and-drop file upload.** Single file picker in v1.
5. **No file-size pre-flight against the upstream's max-upload limit.** ModelGate hasn't declared one.

## References

- Mutating routes: `openspec/changes/archive/2026-04-25-modelgate-web-v1/specs/web-server.md`
- API client: `ui/modelgate-web/src/api.ts`
- Voice rules: `openspec/changes/archive/2026-04-24-design-system-v1/specs/design-system.md`
