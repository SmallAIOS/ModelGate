export type JsonEditorProps = {
  value: string;
  onChange: (next: string) => void;
  /** Surfaced when JSON.parse(value) threw. */
  parseError?: string;
  rows?: number;
  disabled?: boolean;
};

/**
 * Textarea-backed JSON editor. The component itself does not validate;
 * the parent calls `JSON.parse(value)` on submit, catches the error,
 * and feeds it back as `parseError`. The design-system styles the
 * border red when `parseError` is non-empty.
 *
 * See modelgate-web-actions-v1/design.md Decision 4 for why this is a
 * textarea rather than CodeMirror or Monaco.
 */
export function JsonEditor({
  value,
  onChange,
  parseError,
  rows = 12,
  disabled = false,
}: JsonEditorProps) {
  return (
    <div className="json-editor">
      <textarea
        className={`json-editor__textarea${parseError ? ' json-editor__textarea--invalid' : ''}`}
        value={value}
        onChange={(e) => onChange(e.target.value)}
        rows={rows}
        spellCheck={false}
        disabled={disabled}
        aria-invalid={parseError ? 'true' : undefined}
      />
      {parseError && <p className="json-editor__error">{parseError}</p>}
    </div>
  );
}
