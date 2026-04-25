# ModelGate web actions — Tasks

Each block ships as one commit on `change/modelgate-web-actions-v1`.

## Foundation

- [x] Add `<Toaster>` + `useToast()` context to `ui/modelgate-web/src/`. Mount in `App.tsx`. Style via `styles.css`.
- [x] Add `<ConfirmDialog>` controlled component (focus trap via initial-focus + escape/backdrop close + `aria-modal`).
- [x] Split inline screen components out of `App.tsx` into `src/screens/{Overview,Models,Routes,Policy,Terminal}Screen.tsx`. App.tsx keeps the shell + router only.

## API Client

- [x] Multipart upload lives in `useRegisterModel` (XHR with `upload.onprogress`) rather than on the typed client — keeps `api.ts` JSON-only per design Decision 6.
- [x] Existing JSON methods unchanged: `removeModel`, `setRoute`, `testInference`.

## Mutation Hooks

- [x] `src/hooks/mutations.ts` — `useRegisterModel`, `useRemoveModel`, `useSetRoute`, `useTestInference`. Each invalidates the right query key on success and pushes a toast.

## Models screen — Remove action

- [x] Add a `Remove` button to each `ModelsScreen` row.
- [x] On click, open a `<ConfirmDialog>` with the CLI equivalent.
- [x] On confirm, run `useRemoveModel`. Row enters loading state until the mutation resolves.
- [x] Toast on success / failure. List refetches.

## Models screen — Register action

- [x] Add `Register model` button to the screen header.
- [x] Open `<RegisterModelDialog>` (file picker + Upload). Progress bar fed by XHR `upload.onprogress`.
- [x] On submit, run `useRegisterModel`. Dialog stays open while uploading; closes on success; stays open on error so the operator can pick a different file.
- [x] Toast on success / failure. List refetches.

## Routes screen

- [ ] Rename the rail entry from "Policy" to "Routes" — Policy framing is gone now that there's a real form.
- [ ] Add `Set route` button + inline form (model select + endpoint text).
- [ ] On submit, run `useSetRoute`. Form resets on success.
- [ ] Toast on success / failure. Routes table refetches.

## Inference screen

- [ ] Add `Inference` to the rail.
- [ ] New `<InferenceScreen>` — two-column layout per `specs/actions.md`.
- [ ] `<JsonEditor>` component for the input.
- [ ] `Run inference` button runs `useTestInference`. Right column renders the result or the error.

## Voice & A11y

- [ ] Manual voice review of every new string — imperative buttons, sentence-case labels, no emoji, no exclamation points.
- [ ] Accessibility check — focus trap in dialogs, `aria-live` on toaster, every interactive element a `<button>` or `<a>`.

## Tests

- [ ] Vitest for `api.ts`: success + GateApiError parsing for `registerModel`, `removeModel`, `setRoute`, `testInference` against a mocked `fetch`. Closes the deferred row from `modelgate-web-v1/tasks.md`.
- [ ] Vitest for the mutation hooks — assert the right query key is invalidated and the right toast is pushed on success/failure.

## Verify

- [ ] `npm run typecheck` clean
- [ ] `npm run build` clean and under the 2 MB budget
- [ ] `cargo build --workspace` still clean (no Rust changes expected, but a sanity rebuild)
- [ ] Manual smoke against the running `smctl gate web` end-to-end: register a small file, remove it, set a route, run an inference.
