import { useEffect, useState } from "react";
import { fetchSecretSets, inheritSecretSets } from "../../../api/requests";
import { UnauthorizedError } from "../../../api/exception";
import type { SecretSet } from "../../../types/secret";
import type { Stack } from "../../../types/stack";
import { SecretEditor } from "./SecretEditor";

interface Props { stack: Stack; onSignOut: () => void; onSaved: (stack: Stack) => void }

export function StackSecrets({ stack, onSignOut, onSaved }: Props) {
  const [sets, setSets] = useState<SecretSet[]>([]);
  const [selected, setSelected] = useState<string[]>(stack.inherited_secret_set_ids ?? []);
  const [loading, setLoading] = useState(true);
  const [pending, setPending] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [revision, setRevision] = useState(0);
  const [saved, setSaved] = useState(false);
  useEffect(() => {
    let active = true;
    fetchSecretSets().then((data) => { if (active) setSets(data); }).catch((err: unknown) => {
      if (!active) return;
      if (err instanceof UnauthorizedError) onSignOut();
      else setError(err instanceof Error ? err.message : "Could not load secret sets");
    }).finally(() => { if (active) setLoading(false); });
    return () => { active = false; };
  }, [onSignOut]);
  function move(index: number, offset: number) {
    const next = [...selected];
    [next[index], next[index + offset]] = [next[index + offset], next[index]];
    setSelected(next); setSaved(false);
  }
  async function save() {
    if (pending) return;
    setPending(true); setError(null); setSaved(false);
    try {
      await inheritSecretSets(stack.id, selected);
      onSaved({ ...stack, inherited_secret_set_ids: selected });
      setRevision((value) => value + 1); setSaved(true);
    } catch (err) {
      if (err instanceof UnauthorizedError) onSignOut();
      else setError(err instanceof Error ? err.message : "Could not save inheritance");
    } finally { setPending(false); }
  }
  return <div className="secrets-content stack-settings-form">
    <section className="secret-inheritance">
      <h3>Secret sets</h3>
      <p className="secrets-hint">Select sets from global Settings. Later sets override earlier ones; stack secrets override inherited values.</p>
      {loading && <p role="status">Loading secret sets…</p>}
      {!loading && sets.length === 0 && <p className="secrets-hint">Create a secret set in global Settings to share secrets across stacks.</p>}
      <div className="secret-set-choices">{sets.map((set) => <label key={set.id} className="secret-set-choice">
        <input type="checkbox" checked={selected.includes(set.id)} disabled={loading || pending}
          onChange={(event) => {
            setSelected((current) => event.target.checked ? [...current, set.id] : current.filter((id) => id !== set.id));
            setSaved(false);
          }} /><span>{set.name}</span>
      </label>)}</div>
      {selected.length > 0 && <ol className="secret-set-order">{selected.map((id, index) => <li key={id}>
        <span>{sets.find((set) => set.id === id)?.name ?? "Unavailable set"}</span>
        <button type="button" className="btn-ghost" aria-label="Move earlier" disabled={pending || index === 0} onClick={() => move(index, -1)}>↑</button>
        <button type="button" className="btn-ghost" aria-label="Move later" disabled={pending || index === selected.length - 1} onClick={() => move(index, 1)}>↓</button>
      </li>)}</ol>}
      <button type="button" className="btn-ghost" disabled={pending || loading} onClick={() => { void save(); }}>{pending ? "Saving…" : "Save inheritance"}</button>
      {saved && <p className="secrets-success" role="status">Inheritance saved.</p>}
      {error && <p className="secrets-error" role="alert">{error}</p>}
    </section>
    <SecretEditor ownerId={stack.id} scope="stack" revision={revision} onSignOut={onSignOut} />
  </div>;
}
