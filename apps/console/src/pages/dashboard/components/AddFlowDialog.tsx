import { useEffect, useState } from "react";
import { getApiErrorMessage } from "../../../api/errors";
import { createFlow, fetchVaults } from "../../../api/requests";
import { Dialog } from "../../../components/ui/Dialog";
import type { Flow } from "../../../types/flow";
import type { Vault } from "../../../types/secret";
import "./FlowSettingsDialog.css";

interface Props {
  onClose: () => void;
  onSignOut: () => void;
  onCreated: (flow: Flow) => void;
}

export function AddFlowDialog({ onClose, onSignOut, onCreated }: Props) {
  const [name, setName] = useState("");
  const [vaults, setVaults] = useState<Vault[]>([]);
  const [inherited, setInherited] = useState<string[]>([]);
  const [loading, setLoading] = useState(true);
  const [pending, setPending] = useState(false);
  const [error, setError] = useState<string | null>(null);

  useEffect(() => {
    let active = true;
    fetchVaults()
      .then((data) => {
        if (active) setVaults(data.slice().sort((a, b) => a.name.localeCompare(b.name)));
      })
      .catch((err: unknown) => {
        if (active) setError(getApiErrorMessage(err, onSignOut, "Could not load vaults"));
      })
      .finally(() => { if (active) setLoading(false); });
    return () => { active = false; };
  }, [onSignOut]);

  async function save() {
    if (pending || !name.trim()) return;
    setPending(true);
    setError(null);
    try {
      const saved = await createFlow({ name: name.trim(), inherited_vault_ids: inherited });
      onClose();
      onCreated(saved);
    } catch (err) {
      setError(getApiErrorMessage(err, onSignOut, "Could not create flow"));
    } finally {
      setPending(false);
    }
  }

  return (
    <Dialog title="Create flow" closeDisabled={pending} onClose={onClose} className="create-flow-dialog">
      <form
        className="ui-dialog-content flow-settings-form"
        onSubmit={(event) => { event.preventDefault(); void save(); }}
      >
        <label>
          Flow name
          <input required autoFocus value={name} disabled={pending} onChange={event => setName(event.target.value)} />
        </label>
        {loading && <p role="status">Loading vaults…</p>}
        {vaults.length > 0 && (
          <fieldset className="create-flow-vaults" disabled={pending}>
            <legend>Inherited vaults</legend>
            <div className="secret-set-choices">
              {vaults.map(vault => (
                <label key={vault.id} className="secret-set-choice">
                  <input
                    type="checkbox"
                    checked={inherited.includes(vault.id)}
                    onChange={event => setInherited(current => event.target.checked ? [...current, vault.id] : current.filter(id => id !== vault.id))}
                  />
                  <span>{vault.name}</span>
                </label>
              ))}
            </div>
          </fieldset>
        )}
        {error && <p role="alert" className="ui-error">{error}</p>}
        <div className="ui-form-actions">
          <button type="button" className="btn-ghost" disabled={pending} onClick={onClose}>Cancel</button>
          <button className="btn-primary" disabled={pending || loading || !name.trim()} type="submit">{pending ? "Creating…" : "Create flow"}</button>
        </div>
      </form>
    </Dialog>
  );
}
