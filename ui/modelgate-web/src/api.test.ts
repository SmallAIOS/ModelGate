import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest';

import { GateApi, GateApiError } from './api';

// Each test sets up its own `fetch` mock and the api under test points
// at a fixed base so the URL assertions are exact.
const BASE = '/test-api';

let fetchMock: ReturnType<typeof vi.fn>;

beforeEach(() => {
  fetchMock = vi.fn();
  vi.stubGlobal('fetch', fetchMock);
});

afterEach(() => {
  vi.unstubAllGlobals();
});

function jsonResponse(status: number, body: unknown): Response {
  return new Response(JSON.stringify(body), {
    status,
    headers: { 'content-type': 'application/json' },
  });
}

describe('GateApi.health', () => {
  it('returns the parsed HealthStatus on 200', async () => {
    fetchMock.mockResolvedValueOnce(
      jsonResponse(200, {
        status: 'healthy',
        version: '0.2.0',
        uptime_secs: 42,
        model_count: 1,
      }),
    );
    const api = new GateApi(BASE);
    const got = await api.health();
    expect(got.status).toBe('healthy');
    expect(got.model_count).toBe(1);
    expect(fetchMock).toHaveBeenCalledWith(`${BASE}/health`, undefined);
  });

  it('throws GateApiError on non-2xx with the parsed body', async () => {
    fetchMock.mockResolvedValueOnce(
      jsonResponse(502, {
        error: 'upstream_unreachable',
        message: 'connection refused: http://x',
      }),
    );
    const api = new GateApi(BASE);
    await expect(api.health()).rejects.toMatchObject({
      kind: 'upstream_unreachable',
      status: 502,
      message: 'connection refused: http://x',
    });
  });

  it('throws GateApiError when the body is not JSON', async () => {
    fetchMock.mockResolvedValueOnce(new Response('plain text', { status: 503 }));
    const api = new GateApi(BASE);
    const err = await api.health().catch((e: unknown) => e);
    expect(err).toBeInstanceOf(GateApiError);
    expect((err as GateApiError).status).toBe(503);
    expect((err as GateApiError).kind).toBe('http_error');
  });
});

describe('GateApi.listModels', () => {
  it('parses an array of Model entries', async () => {
    fetchMock.mockResolvedValueOnce(
      jsonResponse(200, [
        {
          name: 'phi-2',
          format: 'onnx',
          size_bytes: 1024,
          registered_at: '2026-04-25',
          status: 'loaded',
        },
      ]),
    );
    const api = new GateApi(BASE);
    const models = await api.listModels();
    expect(models).toHaveLength(1);
    expect(models[0].name).toBe('phi-2');
  });
});

describe('GateApi.removeModel', () => {
  it('returns undefined on 204 and URL-encodes the name', async () => {
    fetchMock.mockResolvedValueOnce(new Response(null, { status: 204 }));
    const api = new GateApi(BASE);
    await expect(api.removeModel('weird/name')).resolves.toBeUndefined();
    expect(fetchMock).toHaveBeenCalledWith(`${BASE}/models/weird%2Fname`, {
      method: 'DELETE',
    });
  });

  it('throws ModelNotFound-shaped error on 404', async () => {
    fetchMock.mockResolvedValueOnce(
      jsonResponse(404, {
        error: 'model_not_found',
        message: 'model not found: ghost',
        name: 'ghost',
      }),
    );
    const api = new GateApi(BASE);
    await expect(api.removeModel('ghost')).rejects.toMatchObject({
      kind: 'model_not_found',
      status: 404,
    });
  });
});

describe('GateApi.setRoute', () => {
  it('PUTs the {model, endpoint} body and returns the parsed Route', async () => {
    fetchMock.mockResolvedValueOnce(
      jsonResponse(200, {
        model: 'phi-2',
        endpoint: '/v1/chat/completions',
        active: true,
        request_count: 0,
      }),
    );
    const api = new GateApi(BASE);
    const route = await api.setRoute('phi-2', '/v1/chat/completions');
    expect(route.active).toBe(true);

    const [url, init] = fetchMock.mock.calls[0];
    expect(url).toBe(`${BASE}/routes`);
    expect((init as RequestInit).method).toBe('PUT');
    expect(JSON.parse((init as RequestInit).body as string)).toEqual({
      model: 'phi-2',
      endpoint: '/v1/chat/completions',
    });
  });
});

describe('GateApi.testInference', () => {
  it('POSTs the payload to /inference/{model} and returns the result', async () => {
    fetchMock.mockResolvedValueOnce(
      jsonResponse(200, {
        model: 'llama-7b',
        output: { text: 'hello' },
        latency_ms: 42,
        tokens_generated: 3,
      }),
    );
    const api = new GateApi(BASE);
    const got = await api.testInference('llama-7b', { prompt: 'hi' });
    expect(got.model).toBe('llama-7b');
    expect(got.tokens_generated).toBe(3);

    const [url, init] = fetchMock.mock.calls[0];
    expect(url).toBe(`${BASE}/inference/llama-7b`);
    expect(JSON.parse((init as RequestInit).body as string)).toEqual({ prompt: 'hi' });
  });

  it('surfaces 404 model_not_found from the upstream JSON body', async () => {
    fetchMock.mockResolvedValueOnce(
      jsonResponse(404, {
        error: 'model_not_found',
        message: 'model not found: ghost',
      }),
    );
    const api = new GateApi(BASE);
    await expect(
      api.testInference('ghost', { prompt: 'hi' }),
    ).rejects.toMatchObject({
      kind: 'model_not_found',
      status: 404,
    });
  });
});
