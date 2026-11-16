import { useEffect, useRef, useState } from "react";
import {
  createVaultSecret,
  updateVaultSecret,
  deleteVaultSecret,
  fetchVaultSecrets,
} from "../../../api/requests";
import { getApiErrorMessage } from "../../../api/errors";
import type { Secret } from "../../../types/secret";

interface Props {
  ownerId: string;
  revision?: number;
  onSignOut: () => void;
}
interface Draft {
  id?: string;
  key: string;
  value: string;
}

export function SecretEditor({ ownerId, revision = 0, onSignOut }: Props) {
  const generation = useRef(0);
  const busy = useRef(false);
  const fileInput = useRef<HTMLInputElement>(null);
  const [secrets, setSecrets] = useState<Secret[]>([]);
  const [draft, setDraft] = useState<Draft | null>(null);
  const [loading, setLoading] = useState(true);
  const [pending, setPending] = useState(false);
  const [readingFile, setReadingFile] = useState(false);
  const [error, setError] = useState<string | null>(null);

  useEffect(() => {
    const current = ++generation.current;
    busy.current = false;
    setLoading(true);
    setPending(false);
    setReadingFile(false);
    setDraft(null);
    setError(null);
    fetchVaultSecrets(ownerId)
      .then((data) => {
        if (generation.current !== current) return;
        setSecrets(data);
        if (!data.length) setDraft({ key: "", value: "" });
      })
      .catch((err: unknown) => {
        if (generation.current === current)
          setError(getApiErrorMessage(err, onSignOut, "Could not load secrets"));
      })
      .finally(() => {
        if (generation.current === current) setLoading(false);
      });
    return () => { generation.current++; };
  }, [ownerId, revision, onSignOut]);

  async function save() {
    if (!draft || busy.current || readingFile) return;
    if (secrets.some(secret => secret.key === draft.key && secret.id !== draft.id)) {
      setError("This key already exists.");
      return;
    }
    const current = generation.current;
    busy.current = true;
    setPending(true);
    setError(null);
    try {
      const updated = draft.id
        ? await updateVaultSecret(ownerId, draft.id, draft.value)
        : await createVaultSecret(ownerId, draft.key, draft.value);
      if (generation.current !== current) return;
      setSecrets(updated);
      setDraft(null);
    } catch (err) {
      if (generation.current === current)
        setError(getApiErrorMessage(err, onSignOut, "Could not save secret"));
    } finally {
      if (generation.current === current) {
        busy.current = false;
        setPending(false);
      }
    }
  }

  async function remove(secret: Secret) {
    if (busy.current) return;
    const current = generation.current;
    busy.current = true;
    setPending(true);
    setError(null);
    try {
      const updated = await deleteVaultSecret(ownerId, secret.id);
      if (generation.current !== current) return;
      setSecrets(updated);
      if (draft?.id === secret.id) setDraft(null);
    } catch (err) {
      if (generation.current === current)
        setError(getApiErrorMessage(err, onSignOut, "Could not delete secret"));
    } finally {
      if (generation.current === current) {
        busy.current = false;
        setPending(false);
      }
    }
  }

  const disabled = loading || pending || readingFile;
  return (
    <section className="secret-editor" aria-label="Secrets" aria-busy={loading || pending}>
      <div className="section-heading">
        <h3>Secrets</h3>
        <button
          type="button"
          className="btn-ghost"
          disabled={disabled}
          onClick={() => {
            setDraft({ key: "", value: "" });
            setError(null);
            if (fileInput.current) fileInput.current.value = "";
          }}
        >
          Add secret
        </button>
      </div>
      {loading && <p role="status">Loading secrets…</p>}
      {!loading && secrets.length > 0 && (
        <ul className="secret-list">
          {secrets.map(secret => (
            <li key={secret.id}>
              <code>{secret.key}</code>
              <div className="secret-row-actions">
                <button
                  type="button"
                  className="btn-ghost"
                  aria-label={`Edit ${secret.key}`}
                  disabled={disabled}
                  onClick={() => {
                    setDraft({ id: secret.id, key: secret.key, value: "" });
                    setError(null);
                    if (fileInput.current) fileInput.current.value = "";
                  }}
                >Edit</button>
                <button
                  type="button"
                  className="btn-danger"
                  aria-label={`Delete ${secret.key}`}
                  disabled={disabled}
                  onClick={() => { void remove(secret); }}
                >Delete</button>
              </div>
            </li>
          ))}
        </ul>
      )}
      {!loading && !secrets.length && !draft && <p className="ui-empty">No secrets yet.</p>}
      {draft && !loading && (
        <form
          className="secret-form"
          aria-label={draft.id ? `Edit ${draft.key}` : "New secret"}
          onSubmit={(event) => { event.preventDefault(); void save(); }}
        >
          <label>
            Key
            <input
              required
              autoComplete="off"
              spellCheck={false}
              value={draft.key}
              readOnly={Boolean(draft.id)}
              disabled={pending || readingFile}
              onChange={event => setDraft({ ...draft, key: event.target.value })}
            />
          </label>
          <label>
            {draft.id ? "New value" : "Value"}
            <textarea
              rows={3}
              value={draft.value}
              disabled={pending || readingFile}
              spellCheck={false}
              onChange={event => setDraft({ ...draft, value: event.target.value })}
            />
          </label>
          <input
            ref={fileInput}
            type="file"
            aria-label="Choose file for secret value"
            disabled={pending || readingFile}
            onChange={async (event) => {
              const file = event.currentTarget.files?.[0];
              if (!file) return;
              const current = generation.current;
              const selectedDraft = draft;
              if (file.size > 512 * 1024) {
                setError("File must be smaller than 512 KiB.");
                event.currentTarget.value = "";
                return;
              }
              setReadingFile(true);
              setError(null);
              try {
                const value = await file.text();
                if (generation.current === current)
                  setDraft(latest => latest?.id === selectedDraft.id && latest?.key === selectedDraft.key
                    ? { ...latest, value } : latest);
              } catch {
                if (generation.current === current) setError("Could not read file.");
              } finally {
                if (generation.current === current) setReadingFile(false);
              }
            }}
          />
          <div className="ui-form-actions">
            <button type="button" className="btn-ghost" disabled={pending || readingFile} onClick={() => { setDraft(null); setError(null); }}>Cancel</button>
            <button type="submit" className="btn-primary" disabled={pending || readingFile || !draft.key.trim()}>{pending ? "Saving…" : "Save secret"}</button>
          </div>
        </form>
      )}
      {error && <p className="ui-error" role="alert">{error}</p>}
    </section>
  );
}
