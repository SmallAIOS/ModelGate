# ModelGate web actions — Design Document

## Context

The dashboard from `modelgate-web-v1` reads ModelGate state. This change makes it write. The Rust `/api/*` surface already covers every mutation; the work is entirely in `ui/modelgate-web/src/`.

## Goals / Non-Goals

### Goals

1. Every destructive action goes through a confirmation dialog with a copy-able command preview (e.g. "this is the same as `smctl gate models remove llama-7b`").
2. Every mutation surfaces success / failure via a non-blocking toast in the status line at the bottom of the shell.
3. React Query owns the cache; mutations call `queryClient.invalidateQueries` and let it refetch — no hand-rolled optimistic updates.
4. Voice rules from `design-system-v1` apply unchanged. Buttons say `Register model` / `Remove model` / `Set route` / `Run inference`. No emoji.

### Non-Goals

1. Optimistic UI.
2. Bulk operations.
3. Streaming inference responses.
4. Drag-and-drop file upload.
5. JSON Schema validation against the model's expected input shape.

## Decisions

### Decision 1: One file per mutation hook, all in `hooks/mutations.ts`

**Choice:** Co-locate the four mutation hooks (`useRegisterModel`, `useRemoveModel`, `useSetRoute`, `useTestInference`) in a single file rather than one file each.

**Rationale:** Each hook is ~10 lines (a `useMutation` call, an `onSuccess` invalidation, an `onError` toast). Splitting them into four files would be more files than insight. They invalidate adjacent query keys (`['models']`, `['routes']`); seeing them together makes the dependency graph visible.

### Decision 2: ConfirmDialog is a controlled component

**Choice:** `<ConfirmDialog open onConfirm onCancel ...>` — caller owns the `open` state. No imperative `confirm()` API.

**Rationale:** Imperative confirms break React's mental model and are hard to type. Controlled-state lets a single dialog instance live near the action it confirms, which keeps the action's labels (title, body, command preview, destructive-button label) co-located with the data it acts on.

### Decision 3: Toasts live in a single global region anchored to the status line

**Choice:** A `<Toaster>` mounted once in `App.tsx`. `useToast()` pushes new toasts to a context-backed queue. Toasts auto-dismiss after 5s, or earlier on user dismiss.

**Rationale:** One mount point means consistent z-index, focus, and a11y. Context API is enough for a queue this small — no new state-management dep. 5s matches the design-system status-line spec for transient messaging.

### Decision 4: `JsonEditor` is a textarea, not Monaco

**Choice:** `<JsonEditor>` is a `<textarea>` with monospace font + on-blur `JSON.parse` validation that surfaces a one-line error.

**Rationale:** Monaco is 2 MB+ gzipped — blows the bundle budget. A textarea covers the inference-test use case (paste a small JSON payload, press Run). When inference inputs grow larger or schema-aware, a future change can swap in CodeMirror 6 (which is ~150 KB) — but that's not v1.

### Decision 5: Confirmation dialog shows the equivalent CLI command

**Choice:** Each destructive action's confirm dialog includes a small `<code>` block with the equivalent `smctl gate ...` invocation.

**Rationale:** Reinforces the CLI as the system of record. A reviewer or operator who likes the SPA but wants to script a change later already sees the canonical invocation. Cost: one extra string per action; benefit: educational and audit-friendly.

### Decision 6: Multipart upload uses `fetch` directly, not the typed client

**Choice:** `useRegisterModel` calls `fetch('/api/models', { method: 'POST', body: formData })` directly. Other mutations go through `api.removeModel` / `api.setRoute` / `api.testInference` on the typed client.

**Rationale:** `fetch` already handles `FormData` natively, with browser-built progress events. Wrapping it in the typed client would force the typed client to expose a low-level FormData entry point, which leaks abstraction. The status quo (typed client for JSON, raw fetch for multipart) keeps the typed client honest.

## Risks / Trade-offs

| Risk | Mitigation |
|---|---|
| Toast spam on rapid mutations | Queue caps at 4 visible at a time; older toasts shift up |
| Long uploads block the dialog | Dialog stays open with a progress bar until POST resolves; cancel button aborts via `AbortController` |
| User confirms remove on stale data | After remove, React Query refetches `['models']` immediately so a vanished name disappears from the table |
| JSON parse error UX | On-blur parse highlights the textarea border; submit button stays disabled until parse succeeds |

## Open Questions

1. Should the toast persist a copy-to-clipboard of the failing curl-equivalent for diagnosability? Inclined yes, but only after the first user request.
2. Where does the inference test go in the rail — its own tab, or under Models? Going with its own tab in v1; cheap to move later.
3. Do we wire keyboard shortcuts (`r` to register, `d` to delete a focused row)? Out of scope; likely a later a11y pass.
