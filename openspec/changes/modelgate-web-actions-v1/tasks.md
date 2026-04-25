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

- [x] Rename the rail entry from "Policy" to "Routes" — Policy framing dropped, PolicyScreen.tsx removed (resurrect when Cedar viewer ships).
- [x] Add `Set route` button + inline form (model select + endpoint text).
- [x] On submit, run `useSetRoute`. Form closes on success.
- [x] Toast on success / failure. Routes table refetches.

## Inference screen

- [x] Add `Inference` to the rail.
- [x] New `<InferenceScreen>` — two-column layout per `specs/actions.md`.
- [x] `<JsonEditor>` component for the input — textarea wrapper, parent owns parse + parseError plumbing.
- [x] `Run inference` button runs `useTestInference`. Right column renders the result with model / latency / tokens / output.

## Voice & A11y

- [x] Manual voice review of every new string — imperative buttons (`Register model`, `Remove model`, `Set route`, `Save route`, `Run inference`, `Upload`), sentence-case labels and table headers, no emoji, no exclamation points. Empty / pending states reuse CLI vocabulary verbatim.
- [x] Accessibility check — `aria-modal` + `aria-labelledby` on dialogs with initial focus on the confirm button, `aria-live="polite"` + `role="status"` on the toaster, `role="progressbar"` with valuemin/valuemax/valuenow on the upload progress bar, every interactive element a `<button>` or `<select>` / `<input>` inside a `<label>`.

## Tests

- [x] Vitest for `api.ts`: success + GateApiError parsing for `removeModel`, `setRoute`, `testInference`, plus health/listModels/non-JSON-body paths. Closes the deferred row from `modelgate-web-v1/tasks.md`. (registerModel is XHR, covered by the mutation hook tests when those land.)
- [ ] Vitest for the mutation hooks — assert the right query key is invalidated and the right toast is pushed on success/failure. Deferred to a follow-up commit; needs a React test renderer + provider plumbing.

## Verify

- [x] `npm run typecheck` clean
- [x] `npm run build` clean — bundle 200 KB / 62.6 KB gzipped, under the 2 MB budget
- [x] `cargo build --workspace` clean — 201 workspace tests still pass after the dist/ refresh
- [x] `npm test` clean — 9 vitest cases pass
- [x] `cargo clippy --workspace -- -D warnings` clean
- [x] `cargo fmt --check` clean
- [ ] Manual smoke against the running `smctl gate web` end-to-end — pending a real ModelGate. Wiremock-backed Rust path tests (covered in `modelgate-web-v1`) exercise every `/api/*` route the SPA calls.
