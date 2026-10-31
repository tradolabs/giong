import { useEffect, useRef, useState } from "react";
import { createSecret, deleteSecret, fetchSecrets, updateSecret } from "../../../api/requests";
import { UnauthorizedError } from "../../../api/exception";
import type { Secret } from "../../../types/secret";
import type { Stack } from "../../../types/stack";
import "./SecretsDialog.css";

interface SecretsDialogProps {
  stack: Stack;
  onClose: () => void;
  onSignOut: () => void;
}

interface SecretRow {
  id: string;
  key: string;
  value: string;
  saved: boolean;
  changed: boolean;
  pending?: boolean;
  error?: string;
  message?: string;
}

let nextRowId = 0;

function newRow(): SecretRow {
  return { id: `new-${++nextRowId}`, key: "", value: "", saved: false, changed: false };
}

function savedRow(key: string): SecretRow {
  return { id: key, key, value: "", saved: true, changed: false };
}

export function SecretsDialog({ stack, onClose, onSignOut }: SecretsDialogProps) {
  const dialog = useRef<HTMLDialogElement>(null);
  const active = useRef(true);
  const pending = useRef(new Set<string>());
  const [rows, setRows] = useState<SecretRow[]>([]);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);

  useEffect(() => {
    active.current = true;
    const element = dialog.current;
    element?.showModal();
    fetchSecrets(stack.id)
      .then((keys) => {
        if (active.current) setRows(keys.length ? keys.map((secret) => savedRow(secret.key)) : [newRow()]);
      })
      .catch((err: unknown) => {
        if (!active.current) return;
        if (err instanceof UnauthorizedError) onSignOut();
        else setError(err instanceof Error ? err.message : "Could not load credentials");
      })
      .finally(() => { if (active.current) setLoading(false); });
    return () => {
      active.current = false;
      element?.close();
    };
  }, [stack.id, onSignOut]);

  function setRow(id: string, changes: Partial<SecretRow>) {
    setRows((current) => current.map((row) => row.id === id ? { ...row, ...changes } : row));
  }

  async function update(row: SecretRow, action: () => Promise<Secret[]>) {
    if (pending.current.has(row.id)) return false;
    pending.current.add(row.id);
    setRow(row.id, { pending: true, error: undefined, message: undefined });
    try {
      await action();
      return active.current;
    } catch (err) {
      if (err instanceof UnauthorizedError) onSignOut();
      else if (active.current) setRow(row.id, { error: err instanceof Error ? err.message : "Could not update credential" });
    } finally {
      pending.current.delete(row.id);
      if (active.current) setRow(row.id, { pending: false });
    }
    return false;
  }

  function changeRow(id: string, field: "key" | "value", value: string) {
    setRow(id, { [field]: value, changed: true, error: undefined, message: undefined });
  }

  async function saveRow(row: SecretRow) {
    if (loading || pending.current.has(row.id)) return;
    if (rows.some((item) => item.id !== row.id && item.key === row.key)) {
      setRow(row.id, { error: "This key is already in the list." });
      return;
    }
    if (await update(row, () => (row.saved ? updateSecret : createSecret)(stack.id, row.key, row.value))) {
      setRow(row.id, { ...savedRow(row.key), id: row.id, message: "Saved." });
    }
  }

  async function removeRow(row: SecretRow) {
    if (pending.current.has(row.id)) return;
    if (!row.saved || await update(row, () => deleteSecret(stack.id, row.key))) {
      setRows((current) => current.filter((item) => item.id !== row.id));
    }
  }

  return (
    <dialog ref={dialog} className="secrets-dialog" aria-labelledby="secrets-title"
      onCancel={(event) => { event.preventDefault(); onClose(); }}>
      <header className="secrets-header">
        <div>
          <h2 id="secrets-title">Secrets</h2>
          <p>{stack.name}</p>
        </div>
        <button type="button" className="btn-ghost" onClick={onClose}>Close</button>
      </header>
      <div className="secrets-content">
        <p className="secrets-hint">Credentials are available as environment variables on the next run.</p>
        {loading && <p role="status">Loading credentials…</p>}
        <div className="secrets-rows">
          {rows.map((row) => (
            <form key={row.id} className="secrets-row" aria-label={row.key || "New credential"} aria-busy={row.pending}
              onSubmit={(event) => { event.preventDefault(); void saveRow(row); }}>
              <label>Key
                <input value={row.key} required readOnly={row.saved} disabled={row.pending || loading}
                  onChange={(event) => changeRow(row.id, "key", event.target.value)}
                  pattern="[A-Za-z_][A-Za-z0-9_]*" title="Use letters, digits, and underscores; start with a letter or underscore."
                  placeholder="API_TOKEN" autoComplete="off" autoCapitalize="none" spellCheck={false} />
              </label>
              <label>Value
                <input type="password" value={row.value} disabled={row.pending || loading}
                  onChange={(event) => changeRow(row.id, "value", event.target.value)}
                  placeholder={row.saved ? "Unchanged" : "Enter value"}
                  autoComplete="off" spellCheck={false} aria-describedby="credentials-format" />
              </label>
              <div className="secret-row-actions">
                <button type="submit" className="btn-ghost" disabled={row.pending || loading || (row.saved && !row.changed)}
                  aria-label={`Save ${row.key || "new credential"}`}>
                  {row.pending ? "…" : "Save"}
                </button>
                <button type="button" className="btn-ghost" disabled={row.pending || loading}
                  aria-label={`Remove ${row.key || "new credential"}`} onClick={() => { void removeRow(row); }}>
                  Remove
                </button>
              </div>
              {row.error && <p className="secrets-error secret-row-message" role="alert">{row.error}</p>}
              {row.message && <p className="secrets-success secret-row-message" role="status">{row.message}</p>}
            </form>
          ))}
        </div>
        <p id="credentials-format" className="secrets-hint">Saved values stay hidden. Enter a new value to update a credential.</p>
        {error && <p className="secrets-error" role="alert">{error}</p>}
        <div className="secrets-actions">
          <button type="button" className="btn-ghost" disabled={loading}
            onClick={() => setRows((current) => [...current, newRow()])}>Add credential</button>
        </div>
      </div>
    </dialog>
  );
}
