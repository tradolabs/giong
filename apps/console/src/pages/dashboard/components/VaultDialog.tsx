import { useId, useState } from "react";
import { getApiErrorMessage } from "../../../api/errors";
import { createVault, updateVault } from "../../../api/requests";
import { Dialog } from "../../../components/ui/Dialog";
import type { Vault } from "../../../types/secret";
import { SettingsTabs } from "./SettingsTabs";
import { SecretEditor } from "./SecretEditor";

interface Props {
  vault?: Vault;
  onClose: () => void;
  onSignOut: () => void;
  onSaved: (vault: Vault) => void;
}

export function VaultDialog({ vault, onClose, onSignOut, onSaved }: Props) {
  const id = useId();
  const [tab, setTab] = useState("general");
  const [message, setMessage] = useState<string | null>(null);
  const [name, setName] = useState(vault?.name ?? "");
  const [pending, setPending] = useState(false);
  const [error, setError] = useState<string | null>(null);

  async function save() {
    if (pending || !name.trim()) return;
    setPending(true);
    setError(null);
    setMessage(null);
    try {
      const updated = vault
        ? await updateVault(vault.id, name.trim())
        : await createVault(name.trim());
      setName(updated.name);
      setMessage("Saved.");
      onSaved(updated);
      if (!vault) setTab("secrets");
    } catch (err) {
      setError(getApiErrorMessage(err, onSignOut, vault ? "Could not update vault" : "Could not create vault"));
    } finally {
      setPending(false);
    }
  }

  return (
    <Dialog title={vault ? "Vault settings" : "Create vault"} onClose={onClose} closeDisabled={pending} className="tabbed-settings-dialog vault-dialog">
      <SettingsTabs
        id={id}
        tabs={[{ id: "general", label: "General" }, { id: "secrets", label: "Secrets" }]}
        selected={tab}
        disabled={pending}
        onSelect={setTab}
      />
      <section className="flow-settings-general" role="tabpanel" id={`${id}-general-panel`} aria-labelledby={`${id}-general-tab`} hidden={tab !== "general"}>
        <form className="flow-general-form" onSubmit={event => { event.preventDefault(); void save(); }}>
          <div className="flow-general-content">
            <label>
              Vault name
              <input required autoFocus value={name} disabled={pending} onChange={event => { setName(event.target.value); setError(null); setMessage(null); }} />
            </label>
            {error && <p className="ui-error" role="alert">{error}</p>}
            {message && <p className="ui-success" role="status">{message}</p>}
          </div>
          <footer className="flow-general-footer">
            <div className="ui-form-actions">
              <button type="button" className="btn-ghost" disabled={pending} onClick={onClose}>Close</button>
              <button type="submit" className="btn-primary" disabled={pending || !name.trim() || name.trim() === vault?.name}>
                {pending ? "Saving…" : vault ? "Save changes" : "Create vault"}
              </button>
            </div>
          </footer>
        </form>
      </section>
      {tab === "secrets" && (
        <section className="ui-dialog-content" role="tabpanel" id={`${id}-secrets-panel`} aria-labelledby={`${id}-secrets-tab`}>
          {vault
            ? <SecretEditor key={vault.id} ownerId={vault.id} onSignOut={onSignOut} />
            : <p className="ui-empty">Create the vault in General to manage secrets.</p>}
        </section>
      )}
    </Dialog>
  );
}
