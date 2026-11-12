import { useState } from "react";
import { updateVault } from "../../../api/requests";
import { UnauthorizedError } from "../../../api/exception";
import type { Vault } from "../../../types/secret";
import { SecretEditor } from "./SecretEditor";

interface Props {
  vault: Vault;
  onSignOut: () => void;
  onSaved: (vault: Vault) => void;
}

export function EditVault({ vault, onSignOut, onSaved }: Props) {
  const [name, setName] = useState(vault.name);
  const [pending, setPending] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [message, setMessage] = useState<string | null>(null);

  async function save() {
    if (pending) return;
    setPending(true); setError(null); setMessage(null);
    try {
      const updated = await updateVault(vault.id, name.trim());
      setName(updated.name);
      setMessage("Saved.");
      onSaved(updated);
    } catch (err) {
      if (err instanceof UnauthorizedError) onSignOut();
      else setError(err instanceof Error ? err.message : "Could not update vault");
    } finally {
      setPending(false);
    }
  }

  return <div className="secrets-content flow-settings-form">
    <form className="secret-set-create" onSubmit={(event) => { event.preventDefault(); void save(); }}>
      <label>Vault name<input required value={name} disabled={pending}
        onChange={(event) => { setName(event.target.value); setError(null); setMessage(null); }} /></label>
      <button type="submit" className="btn-ghost" disabled={pending || name.trim() === vault.name}>{pending ? "Saving…" : "Save"}</button>
    </form>
    {error && <p className="secrets-error" role="alert">{error}</p>}
    {message && <p className="secrets-success" role="status">{message}</p>}
    <SecretEditor ownerId={vault.id} onSignOut={onSignOut} />
  </div>;
}
