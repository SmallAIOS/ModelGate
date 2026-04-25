import { useMutation, useQueryClient } from '@tanstack/react-query';

import type { Model, Route, InferenceResult } from '../api';
import { api, GateApiError } from '../api';
import { useToast } from '../components/Toaster';

/**
 * Remove a model by name. Invalidates `['models']` on success.
 *
 * `mutate(name)` is the imperative entry point.
 */
export function useRemoveModel() {
  const qc = useQueryClient();
  const { show } = useToast();
  return useMutation<void, GateApiError, string>({
    mutationFn: (name) => api.removeModel(name),
    onSuccess: (_void, name) => {
      qc.invalidateQueries({ queryKey: ['models'] });
      show({ kind: 'success', message: `Model '${name}' removed` });
    },
    onError: (err, name) => {
      show({
        kind: 'error',
        message: `Could not remove '${name}'.`,
        detail: err.message,
      });
    },
  });
}

/**
 * Set (or update) a route. Invalidates `['routes']` on success.
 */
export function useSetRoute() {
  const qc = useQueryClient();
  const { show } = useToast();
  return useMutation<Route, GateApiError, { model: string; endpoint: string }>({
    mutationFn: ({ model, endpoint }) => api.setRoute(model, endpoint),
    onSuccess: (route) => {
      qc.invalidateQueries({ queryKey: ['routes'] });
      show({
        kind: 'success',
        message: `Route set: ${route.model} → ${route.endpoint}`,
      });
    },
    onError: (err) => {
      show({ kind: 'error', message: 'Could not set route.', detail: err.message });
    },
  });
}

/**
 * Register a new model from a file. Multipart upload via raw `fetch`
 * because the typed client does not expose a `FormData` entry point —
 * see modelgate-web-actions-v1/design.md Decision 6.
 *
 * Invalidates `['models']` on success.
 */
export function useRegisterModel() {
  const qc = useQueryClient();
  const { show } = useToast();
  return useMutation<Model, GateApiError, File>({
    mutationFn: async (file) => {
      const form = new FormData();
      form.append('file', file, file.name);
      const resp = await fetch('/api/models', { method: 'POST', body: form });
      if (!resp.ok) {
        let kind = 'http_error';
        let message = `HTTP ${resp.status}`;
        let body: unknown = undefined;
        try {
          body = await resp.json();
          if (body && typeof body === 'object') {
            const b = body as Record<string, unknown>;
            if (typeof b.error === 'string') kind = b.error;
            if (typeof b.message === 'string') message = b.message;
          }
        } catch {
          // non-JSON body — keep defaults
        }
        throw new GateApiError(kind, resp.status, message, body);
      }
      return (await resp.json()) as Model;
    },
    onSuccess: (model) => {
      qc.invalidateQueries({ queryKey: ['models'] });
      show({ kind: 'success', message: `Model '${model.name}' registered` });
    },
    onError: (err) => {
      show({ kind: 'error', message: 'Could not register model.', detail: err.message });
    },
  });
}

/**
 * Run a test inference. Does not invalidate any cache — the caller
 * renders the returned `InferenceResult` inline.
 */
export function useTestInference() {
  const { show } = useToast();
  return useMutation<InferenceResult, GateApiError, { model: string; payload: unknown }>({
    mutationFn: ({ model, payload }) => api.testInference(model, payload),
    onError: (err) => {
      show({ kind: 'error', message: 'Inference failed.', detail: err.message });
    },
  });
}
