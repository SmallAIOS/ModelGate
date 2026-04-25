import { useMemo, useState } from 'react';
import { useQuery } from '@tanstack/react-query';

import { api } from '../api';
import type { InferenceResult, Model } from '../api';
import { ErrorBox } from '../components/ErrorBox';
import { JsonEditor } from '../components/JsonEditor';
import { useTestInference } from '../hooks/mutations';

const DEFAULT_PAYLOAD = '{\n  "prompt": ""\n}\n';

export function InferenceScreen() {
  const models = useQuery<Model[]>({
    queryKey: ['models'],
    queryFn: () => api.listModels(),
  });
  const inference = useTestInference();

  const [model, setModel] = useState<string>('');
  const [payloadText, setPayloadText] = useState<string>(DEFAULT_PAYLOAD);

  // Default the model select to the first available model when the
  // list arrives.
  if (!model && models.data && models.data.length > 0) {
    setModel(models.data[0].name);
  }

  const parseError = useMemo(() => parseJsonError(payloadText), [payloadText]);

  return (
    <section className="panel">
      <header className="panel__header">
        <h2>Inference</h2>
      </header>

      {models.isError ? (
        <ErrorBox title="Could not list models" error={models.error} />
      ) : !models.data || models.data.length === 0 ? (
        <p className="muted">
          Register a model first — there's nothing to run inference against yet.
        </p>
      ) : (
        <div className="inference">
          <div className="inference__input">
            <label className="form__row">
              <span>Model</span>
              <select
                value={model}
                onChange={(e) => setModel(e.target.value)}
                disabled={inference.isPending}
              >
                {models.data.map((m) => (
                  <option key={m.name} value={m.name}>
                    {m.name}
                  </option>
                ))}
              </select>
            </label>
            <JsonEditor
              value={payloadText}
              onChange={setPayloadText}
              parseError={parseError}
              disabled={inference.isPending}
            />
            <div className="form__actions">
              <button
                type="button"
                className="btn btn--primary"
                disabled={!model || !!parseError || inference.isPending}
                onClick={() => {
                  try {
                    const payload = JSON.parse(payloadText);
                    inference.mutate({ model, payload });
                  } catch {
                    // parseError already shown by JsonEditor
                  }
                }}
              >
                {inference.isPending ? 'Running…' : 'Run inference'}
              </button>
            </div>
          </div>
          <div className="inference__result">
            <ResultPane result={inference.data} pending={inference.isPending} />
          </div>
        </div>
      )}
    </section>
  );
}

function ResultPane({
  result,
  pending,
}: {
  result: InferenceResult | undefined;
  pending: boolean;
}) {
  if (pending) return <p className="muted">Running inference…</p>;
  if (!result) return <p className="muted">Run inference to see a result.</p>;
  return (
    <div className="inference__result-panel">
      <dl className="kv">
        <dt>Model</dt>
        <dd>{result.model}</dd>
        <dt>Latency</dt>
        <dd>{result.latency_ms}ms</dd>
        {result.tokens_generated !== null && result.tokens_generated !== undefined && (
          <>
            <dt>Tokens</dt>
            <dd>{result.tokens_generated}</dd>
          </>
        )}
      </dl>
      <pre className="inference__output">
        <code>{JSON.stringify(result.output, null, 2)}</code>
      </pre>
    </div>
  );
}

function parseJsonError(text: string): string | undefined {
  try {
    JSON.parse(text);
    return undefined;
  } catch (e) {
    return e instanceof Error ? e.message : 'invalid JSON';
  }
}
