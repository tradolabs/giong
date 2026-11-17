import { useEffect, useState } from "react";
import { getApiErrorMessage } from "../../../api/errors";
import { fetchVaults, updateFlow } from "../../../api/requests";
import type { Vault } from "../../../types/secret";
import type { Flow } from "../../../types/flow";
import { SecretEditor } from "./SecretEditor";
import { InheritedVaultSecrets } from "./InheritedVaultSecrets";

interface Props {
  flow: Flow;
  onSignOut: () => void;
  onSaved: (flow: Flow) => void;
}

export function FlowSecrets({ flow, onSignOut, onSaved }: Props) {
  const [vaults, setVaults] = useState<Vault[]>([]);
  const [inherited, setInherited] = useState<string[]>(
    flow.inherited_vault_ids,
  );
  const [loading, setLoading] = useState(true);
  const [pending, setPending] = useState(false);
  const [message, setMessage] = useState<string | null>(null);
  const [error, setError] = useState<string | null>(null);
  useEffect(() => {
    let active = true;
    fetchVaults()
      .then((data) => {
        if (active) setVaults(data);
      })
      .catch((err: unknown) => {
        if (!active) return;
        setError(getApiErrorMessage(err, onSignOut, "Could not load vaults"));
      })
      .finally(() => {
        if (active) setLoading(false);
      });
    return () => {
      active = false;
    };
  }, [onSignOut]);

  async function save() {
    if (pending) return;
    setPending(true);
    setError(null);
    setMessage(null);
    try {
      onSaved(await updateFlow(flow.id, { name: flow.name, inherited_vault_ids: inherited }));
      setMessage("Saved.");
    } catch (err) {
      setError(
        getApiErrorMessage(err, onSignOut, "Could not save inherited vaults"),
      );
    } finally {
      setPending(false);
    }
  }

  return (
    <div className="ui-dialog-content flow-settings-form">
      {flow.vault_id && (
        <SecretEditor
          key={flow.vault_id}
          ownerId={flow.vault_id}
          onSignOut={onSignOut}
        />
      )}
      <h3>Inherited vaults</h3>
      <p className="ui-hint">Attach shared vaults to use their secrets. Each reference must specify its vault; keys without a vault use only the flow vault.</p>
      {loading && <p role="status">Loading vaults…</p>}
      <div className="secret-set-choices">
        {vaults
          .filter(vault => vault.id !== flow.vault_id)
          .map(vault => (
            <label key={vault.id} className="secret-set-choice">
              <input
                type="checkbox"
                checked={inherited.includes(vault.id)}
                disabled={loading || pending}
                onChange={(event) => {
                  setMessage(null);
                  setError(null);
                  setInherited(current =>
                    event.target.checked
                      ? [...current, vault.id]
                      : current.filter(id => id !== vault.id),
                  );
                }}
              />
              <span>
                {vault.name}
              </span>
            </label>
          ))}
      </div>
      <button
        type="button"
        className="btn-ghost"
        disabled={pending || loading}
        onClick={() => {
          void save();
        }}
      >
        {pending ? "Saving…" : "Save inherited vaults"}
      </button>
      {!loading && inherited.map(vaultId => (
        <InheritedVaultSecrets
          key={vaultId}
          vaultId={vaultId}
          name={vaults.find(vault => vault.id === vaultId)?.name ?? vaultId}
          onSignOut={onSignOut}
        />
      ))}
      {message && (
        <p className="ui-success" role="status">
          {message}
        </p>
      )}
      {error && (
        <p className="ui-error" role="alert">
          {error}
        </p>
      )}
    </div>
  );
}
