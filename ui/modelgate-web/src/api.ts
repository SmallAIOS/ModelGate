// Typed client for the modelgate-web /api/* surface. Mirrors the
// smctl_gate Rust types declared in smctl-gate/src/lib.rs.

export type HealthStatus = {
  status: string;
  version: string;
  uptime_secs: number;
  model_count: number;
};

export type Model = {
  name: string;
  format: string;
  size_bytes: number;
  registered_at: string;
  status: string;
};

export type Route = {
  model: string;
  endpoint: string;
  active: boolean;
  request_count: number;
};

export type LogEntry = {
  timestamp: string;
  level: string;
  message: string;
  fields: unknown;
};

export type InferenceResult = {
  model: string;
  output: unknown;
  latency_ms: number;
  tokens_generated: number | null;
};

/**
 * Error shape returned by the modelgate-web proxy. Matches the JSON
 * body in openspec/changes/modelgate-web-v1/specs/web-server.md.
 */
export class GateApiError extends Error {
  kind: string;
  status: number;
  body: unknown;

  constructor(kind: string, status: number, message: string, body: unknown) {
    super(message);
    this.kind = kind;
    this.status = status;
    this.body = body;
  }
}

async function request<T>(path: string, init?: RequestInit): Promise<T> {
  const resp = await fetch(path, init);
  if (!resp.ok) {
    let body: unknown = undefined;
    let kind = 'http_error';
    let message = `HTTP ${resp.status}`;
    try {
      body = await resp.json();
      if (body && typeof body === 'object') {
        const b = body as Record<string, unknown>;
        if (typeof b.error === 'string') kind = b.error;
        if (typeof b.message === 'string') message = b.message;
      }
    } catch {
      // Non-JSON body — keep defaults.
    }
    throw new GateApiError(kind, resp.status, message, body);
  }
  if (resp.status === 204) {
    return undefined as T;
  }
  return (await resp.json()) as T;
}

export class GateApi {
  constructor(private readonly base = '/api') {}

  health(): Promise<HealthStatus> {
    return request<HealthStatus>(`${this.base}/health`);
  }

  listModels(): Promise<Model[]> {
    return request<Model[]>(`${this.base}/models`);
  }

  removeModel(name: string): Promise<void> {
    return request<void>(`${this.base}/models/${encodeURIComponent(name)}`, {
      method: 'DELETE',
    });
  }

  listRoutes(): Promise<Route[]> {
    return request<Route[]>(`${this.base}/routes`);
  }

  setRoute(model: string, endpoint: string): Promise<Route> {
    return request<Route>(`${this.base}/routes`, {
      method: 'PUT',
      headers: { 'content-type': 'application/json' },
      body: JSON.stringify({ model, endpoint }),
    });
  }

  testInference(model: string, payload: unknown): Promise<InferenceResult> {
    return request<InferenceResult>(
      `${this.base}/inference/${encodeURIComponent(model)}`,
      {
        method: 'POST',
        headers: { 'content-type': 'application/json' },
        body: JSON.stringify(payload),
      },
    );
  }
}

export const api = new GateApi();
