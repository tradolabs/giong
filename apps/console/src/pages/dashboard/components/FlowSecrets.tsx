import { useEffect, useState } from "react";
import { fetchVaults, updateFlowVaults } from "../../../api/requests";
import { UnauthorizedError } from "../../../api/exception";
import type { Vault } from "../../../types/secret";
import type { Flow } from "../../../types/flow";
import { SecretEditor } from "./SecretEditor";

interface Props { flow: Flow; onSignOut: () => void; onSaved: (flow: Flow) => void }
export function FlowSecrets({ flow, onSignOut, onSaved }: Props) {
  const [vaults, setVaults] = useState<Vault[]>([]);
  const [inherited, setInherited] = useState<string[]>(flow.inherited_vault_ids);
  const [loading, setLoading] = useState(true);
  const [pending, setPending] = useState(false);
  const [message, setMessage] = useState<string | null>(null);
  const [error, setError] = useState<string | null>(null);
  useEffect(() => {
    let active = true;
    fetchVaults().then(data => { if (active) setVaults(data); }).catch((err: unknown) => {
      if (!active) return;
      if (err instanceof UnauthorizedError) onSignOut();
      else setError(err instanceof Error ? err.message : "Could not load vaults");
    }).finally(() => { if (active) setLoading(false); });
    return () => { active = false; };
  }, [onSignOut]);
  async function save() {
    if (pending) return;
    setPending(true); setError(null); setMessage(null);
    try {
      onSaved(await updateFlowVaults(flow.id, inherited));
      setMessage("Inherited vaults saved. New runs will use them.");
    } catch (err) {
      if (err instanceof UnauthorizedError) onSignOut();
      else setError(err instanceof Error ? err.message : "Could not save inherited vaults");
    } finally { setPending(false); }
  }
  return <div className="secrets-content flow-settings-form">
    <h3>Flow secrets</h3>
    {flow.vault_id && <SecretEditor key={flow.vault_id} ownerId={flow.vault_id} onSignOut={onSignOut} />}
    <h3>Inherited vaults</h3>
    <p className="secrets-hint">Later selections override earlier ones. Secrets entered above have highest priority.</p>
    {loading && <p role="status">Loading vaults…</p>}
    <div className="secret-set-choices">{vaults.filter(vault => vault.id !== flow.vault_id).map(vault => <label key={vault.id} className="secret-set-choice">
      <input type="checkbox" checked={inherited.includes(vault.id)} disabled={loading || pending}
        onChange={event => { setMessage(null); setError(null); setInherited(current => event.target.checked ? [...current, vault.id] : current.filter(id => id !== vault.id)); }} />
      <span>{vault.name}{inherited.includes(vault.id) ? ` (${inherited.indexOf(vault.id) + 1})` : ""}</span>
    </label>)}</div>
    <button type="button" className="btn-ghost" disabled={pending || loading} onClick={() => { void save(); }}>{pending ? "Saving…" : "Save inherited vaults"}</button>
    {message && <p className="secrets-success" role="status">{message}</p>}
    {error && <p className="secrets-error" role="alert">{error}</p>}
  </div>;
}
