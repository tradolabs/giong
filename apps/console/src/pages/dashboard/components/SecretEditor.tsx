import { useEffect, useRef, useState } from "react";
import { createSecret, deleteSecret, fetchSecrets, updateSecret,
  createSetSecret, deleteSetSecret, fetchSetSecrets, updateSetSecret } from "../../../api/requests";
import { UnauthorizedError } from "../../../api/exception";
import type { Secret } from "../../../types/secret";

interface Props {
  ownerId: string;
  scope: "stack" | "set";
  revision?: number;
  onSignOut: () => void;
}
interface Row {
  id: string;
  key: string;
  value: string;
  saved: boolean;
  changed: boolean;
  pending?: boolean;
  error?: string;
  message?: string;
}
let nextId = 0;
function newRow(key = ""): Row {
  return { id: `secret-${++nextId}`, key, value: "", saved: false, changed: false };
}

export function SecretEditor({ ownerId, scope, revision = 0, onSignOut }: Props) {
  const active = useRef(true);
  const requests = useRef(new Set<string>());
  const [rows, setRows] = useState<Row[]>([]);
  const [inherited, setInherited] = useState<Secret[]>([]);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);

  useEffect(() => {
    active.current = true;
    let current = true;
    setLoading(true); setError(null);
    (scope === "stack" ? fetchSecrets : fetchSetSecrets)(ownerId).then((secrets) => {
      if (!current) return;
      setInherited(secrets.filter((secret) => secret.source));
      const local = secrets.filter((secret) => !secret.source);
      setRows((previous) => {
        const saved = local.map((secret) => {
          const draft = previous.find((row) => row.saved && row.key === secret.key);
          return draft ?? { ...newRow(secret.key), saved: true };
        });
        const drafts = previous.filter((row) => !row.saved);
        return saved.length || drafts.length ? [...saved, ...drafts] : [newRow()];
      });
    }).catch((err: unknown) => {
      if (!current) return;
      if (err instanceof UnauthorizedError) onSignOut();
      else setError(err instanceof Error ? err.message : "Could not load secrets");
    }).finally(() => { if (current) setLoading(false); });
    return () => { current = false; active.current = false; };
  }, [ownerId, scope, revision, onSignOut]);

  function change(id: string, changes: Partial<Row>) {
    setRows((current) => current.map((row) => row.id === id ? { ...row, ...changes } : row));
  }
  function edit(id: string, changes: Partial<Row>) {
    change(id, { ...changes, changed: true, error: undefined, message: undefined });
  }
  async function perform(row: Row, action: () => Promise<Secret[]>) {
    if (requests.current.has(row.id)) return false;
    requests.current.add(row.id);
    change(row.id, { pending: true, error: undefined, message: undefined });
    try {
      const result = await action();
      if (!active.current) return false;
      setInherited(result.filter((secret) => secret.source));
      return true;
    } catch (err) {
      if (err instanceof UnauthorizedError) onSignOut();
      else if (active.current) change(row.id, { error: err instanceof Error ? err.message : "Could not update secret" });
      return false;
    } finally {
      requests.current.delete(row.id);
      if (active.current) change(row.id, { pending: false });
    }
  }
  async function save(row: Row) {
    if (rows.some((other) => other.id !== row.id && other.key === row.key)) {
      change(row.id, { error: "This key is already in the list." }); return;
    }
    const create = scope === "stack" ? createSecret : createSetSecret;
    const update = scope === "stack" ? updateSecret : updateSetSecret;
    if (await perform(row, () => row.saved ? update(ownerId, row.key, row.value)
      : create(ownerId, row.key, row.value))) {
      change(row.id, { saved: true, changed: false, value: "", message: "Saved." });
    }
  }
  async function remove(row: Row) {
    if (!row.saved || await perform(row, () => (scope === "stack" ? deleteSecret : deleteSetSecret)(ownerId, row.key))) {
      setRows((current) => current.filter((entry) => entry.id !== row.id));
    }
  }

  return <div className="secret-editor">
    {loading && <p role="status">Loading secrets…</p>}
    {inherited.length > 0 && <section className="inherited-secrets" aria-label="Inherited secrets">
      <h3>Inherited secrets</h3>
      {inherited.map((secret) => <div className="inherited-secret" key={secret.key}>
        <div><code>{secret.key}</code><small>{secret.source}</small></div>
        <button type="button" className="btn-ghost" disabled={rows.some((row) => row.key === secret.key)}
          onClick={() => setRows((current) => [...current, newRow(secret.key)])}>Override</button>
      </div>)}
    </section>}
    <p className="secrets-hint">Values are available as environment variables and read-only files at /run/secrets/KEY. The runner plugin chooses how to use them.</p>
    <div className="secret-rows">
      {rows.map((row) => <form key={row.id} className="secret-row" aria-label={row.key || "New secret"}
        aria-busy={row.pending} onSubmit={(event) => { event.preventDefault(); void save(row); }}>
        <label>Key<input value={row.key} required readOnly={row.saved} disabled={loading || row.pending}
          onChange={(event) => edit(row.id, { key: event.target.value })}
          placeholder="API_TOKEN or KUBECONFIG" autoComplete="off" spellCheck={false} /></label>
        <label>Value<textarea rows={4} value={row.value} disabled={loading || row.pending} spellCheck={false}
          placeholder={row.saved ? "Enter a replacement to update" : "Paste a value or file contents"}
          onChange={(event) => edit(row.id, { value: event.target.value })} /></label>
        <label className="secret-upload">Or upload a file<input type="file" disabled={loading || row.pending}
          onChange={async (event) => {
            const input = event.currentTarget;
            const file = input.files?.[0];
            if (!file) return;
            if (file.size > 512 * 1024) { change(row.id, { error: "File must be smaller than 512 KiB." }); return; }
            try { const value = await file.text(); if (active.current) edit(row.id, { value }); }
            catch { if (active.current) change(row.id, { error: "Could not read file." }); }
            input.value = "";
          }} /></label>
        <div className="secret-row-actions">
          <button type="submit" className="btn-ghost" disabled={loading || row.pending || (row.saved && !row.changed)}>{row.pending ? "Saving…" : "Save"}</button>
          <button type="button" className="btn-ghost" disabled={loading || row.pending} onClick={() => { void remove(row); }}>Remove</button>
        </div>
        {row.error && <p className="secrets-error" role="alert">{row.error}</p>}
        {row.message && <p className="secrets-success" role="status">{row.message}</p>}
      </form>)}
    </div>
    <p className="secrets-hint">Saved values stay hidden. Enter a replacement to update a secret.</p>
    {error && <p className="secrets-error" role="alert">{error}</p>}
    <button type="button" className="btn-ghost" disabled={loading} onClick={() => setRows((current) => [...current, newRow()])}>Add secret</button>
  </div>;
}
