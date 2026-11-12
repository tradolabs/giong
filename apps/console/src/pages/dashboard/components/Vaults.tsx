import { useState } from "react";
import { createVault, deleteVault } from "../../../api/requests";
import { UnauthorizedError } from "../../../api/exception";
import type { Vault } from "../../../types/secret";

interface Props {
  vaults: Vault[];
  loading: boolean;
  error: string | null;
  onError: (error: string | null) => void;
  onCreated: (vault: Vault) => void;
  onDeleted: (id: string) => void;
  onEdit: (vault: Vault) => void;
  onSignOut: () => void;
}

export function Vaults({ vaults, loading, error, onError, onCreated, onDeleted, onEdit, onSignOut }: Props) {
  const [name, setName] = useState("");
  const [creating, setCreating] = useState(false);
  const [deletingId, setDeletingId] = useState<string | null>(null);

  async function add() {
    if (creating || deletingId) return;
    setCreating(true); onError(null);
    try {
      const vault = await createVault(name.trim());
      setName("");
      onCreated(vault);
    } catch (err) {
      if (err instanceof UnauthorizedError) onSignOut();
      else onError(err instanceof Error ? err.message : "Could not create vault");
    } finally { setCreating(false); }
  }

  async function remove(id: string) {
    if (creating || deletingId) return;
    setDeletingId(id); onError(null);
    try {
      await deleteVault(id);
      onDeleted(id);
    } catch (err) {
      if (err instanceof UnauthorizedError) onSignOut();
      else onError(err instanceof Error ? err.message : "Could not delete vault");
    } finally { setDeletingId(null); }
  }

  const busy = creating || deletingId !== null;

  return <div className="secrets-content flow-settings-form">
    <p className="secrets-hint">Vaults store encrypted secrets. Flows can use a vault and inherit additional vaults.</p>
    <form className="secret-set-create" onSubmit={(event) => { event.preventDefault(); void add(); }}>
      <label>New vault<input required value={name} disabled={busy || loading}
        placeholder="Production Kubernetes" onChange={(event) => setName(event.target.value)} /></label>
      <button type="submit" className="btn-ghost" disabled={busy || loading}>{creating ? "Saving…" : "Create vault"}</button>
    </form>
    {loading && <p role="status">Loading vaults…</p>}
    {error && <p className="secrets-error" role="alert">{error}</p>}
    {!loading && vaults.length === 0 && <p className="secrets-hint">No vaults yet.</p>}
    {vaults.length > 0 && <div className="vault-list" aria-label="Vaults">
      {vaults.map((vault) => <div key={vault.id} className="inherited-secret">
        <div><strong>{vault.name}</strong></div>
        <div className="secret-row-actions">
          <button type="button" className="btn-ghost" disabled={busy} onClick={() => onEdit(vault)}>Edit</button>
          <button type="button" className="btn-ghost" disabled={busy} onClick={() => { void remove(vault.id); }}>
            {deletingId === vault.id ? "Deleting…" : "Delete"}
          </button>
        </div>
      </div>)}
    </div>}
  </div>;
}
