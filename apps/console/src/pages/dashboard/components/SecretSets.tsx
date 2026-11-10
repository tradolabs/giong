import { useEffect, useState } from "react";
import { createSecretSet, deleteSecretSet, fetchSecretSets } from "../../../api/requests";
import { UnauthorizedError } from "../../../api/exception";
import type { SecretSet } from "../../../types/secret";
import { SecretEditor } from "./SecretEditor";

export function SecretSets({ onSignOut }: { onSignOut: () => void }) {
  const [sets, setSets] = useState<SecretSet[]>([]);
  const [selected, setSelected] = useState("");
  const [name, setName] = useState("");
  const [loading, setLoading] = useState(true);
  const [pending, setPending] = useState(false);
  const [error, setError] = useState<string | null>(null);
  useEffect(() => {
    let active = true;
    fetchSecretSets().then((data) => {
      if (active) { setSets(data); setSelected(data[0]?.id ?? ""); }
    }).catch((err: unknown) => {
      if (!active) return;
      if (err instanceof UnauthorizedError) onSignOut();
      else setError(err instanceof Error ? err.message : "Could not load secret sets");
    }).finally(() => { if (active) setLoading(false); });
    return () => { active = false; };
  }, [onSignOut]);
  async function add() {
    if (pending) return;
    setPending(true); setError(null);
    try {
      const set = await createSecretSet(name.trim());
      setSets((current) => [...current, set].sort((a, b) => a.name.localeCompare(b.name)));
      setSelected(set.id); setName("");
    } catch (err) {
      if (err instanceof UnauthorizedError) onSignOut();
      else setError(err instanceof Error ? err.message : "Could not create secret set");
    } finally { setPending(false); }
  }
  async function remove() {
    if (pending || !selected) return;
    setPending(true); setError(null);
    try {
      await deleteSecretSet(selected);
      const remaining = sets.filter((set) => set.id !== selected);
      setSets(remaining); setSelected(remaining[0]?.id ?? "");
    } catch (err) {
      if (err instanceof UnauthorizedError) onSignOut();
      else setError(err instanceof Error ? err.message : "Could not delete secret set");
    } finally { setPending(false); }
  }
  return <div className="secrets-content stack-settings-form">
    <p className="secrets-hint">Secret sets share encrypted text values across stacks. Choose which sets to inherit in each stack’s Secrets tab.</p>
    <form className="secret-set-create" onSubmit={(event) => { event.preventDefault(); void add(); }}>
      <label>New secret set<input required value={name} disabled={pending || loading}
        placeholder="Production Kubernetes" onChange={(event) => setName(event.target.value)} /></label>
      <button type="submit" className="btn-ghost" disabled={pending || loading}>{pending ? "Saving…" : "Create set"}</button>
    </form>
    {loading && <p role="status">Loading secret sets…</p>}
    {error && <p className="secrets-error" role="alert">{error}</p>}
    {sets.length > 0 && <>
      <div className="secret-set-create">
        <label>Secret set<select value={selected} disabled={pending}
          onChange={(event) => setSelected(event.target.value)}>
          {sets.map((set) => <option key={set.id} value={set.id}>{set.name}</option>)}
        </select></label>
        <button type="button" className="btn-ghost" disabled={pending} onClick={() => { void remove(); }}>Delete set</button>
      </div>
      <SecretEditor key={selected} ownerId={selected} scope="set" onSignOut={onSignOut} />
    </>}
  </div>;
}
