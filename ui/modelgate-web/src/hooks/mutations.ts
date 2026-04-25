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

export type RegisterModelArgs = {
  file: File;
  onProgress?: (sent: number, total: number) => void;
};

/**
 * Register a new model from a file. Multipart upload via XHR because
 * `fetch` does not have a stable cross-browser progress API for upload
 * bodies; XHR's `xhr.upload.onprogress` does. See
 * modelgate-web-actions-v1/design.md Decision 6.
 *
 * Invalidates `['models']` on success.
 */
export function useRegisterModel() {
  const qc = useQueryClient();
  const { show } = useToast();
  return useMutation<Model, GateApiError, RegisterModelArgs>({
    mutationFn: ({ file, onProgress }) => uploadModelXhr(file, onProgress),
    onSuccess: (model) => {
      qc.invalidateQueries({ queryKey: ['models'] });
      show({ kind: 'success', message: `Model '${model.name}' registered` });
    },
    onError: (err) => {
      show({ kind: 'error', message: 'Could not register model.', detail: err.message });
    },
  });
}

function uploadModelXhr(
  file: File,
  onProgress?: (sent: number, total: number) => void,
): Promise<Model> {
  return new Promise((resolve, reject) => {
    const xhr = new XMLHttpRequest();
    xhr.open('POST', '/api/models');

    xhr.upload.onprogress = (e) => {
      if (e.lengthComputable && onProgress) {
        onProgress(e.loaded, e.total);
      }
    };

    xhr.onload = () => {
      if (xhr.status >= 200 && xhr.status < 300) {
        try {
          resolve(JSON.parse(xhr.responseText) as Model);
        } catch {
          reject(
            new GateApiError(
              'upstream_parse_error',
              xhr.status,
              'response was not valid JSON',
              null,
            ),
          );
        }
      } else {
        let kind = 'http_error';
        let message = `HTTP ${xhr.status}`;
        let body: unknown = undefined;
        try {
          body = JSON.parse(xhr.responseText);
          if (body && typeof body === 'object') {
            const b = body as Record<string, unknown>;
            if (typeof b.error === 'string') kind = b.error;
            if (typeof b.message === 'string') message = b.message;
          }
        } catch {
          // non-JSON body — keep defaults
        }
        reject(new GateApiError(kind, xhr.status, message, body));
      }
    };

    xhr.onerror = () => {
      reject(new GateApiError('network_error', 0, 'network error during upload', null));
    };

    const form = new FormData();
    form.append('file', file, file.name);
    xhr.send(form);
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
