import { useEffect, useState } from "react";
import { getApiErrorMessage } from "../../../api/errors";
import { fetchVaults, updateFlow } from "../../../api/requests";
import type { Vault } from "../../../types/secret";
import type { Flow } from "../../../types/flow";
import { InheritedVaultSecrets } from "./InheritedVaultSecrets";

interface Props {
  flow: Flow;
  onSignOut: () => void;
  onSaved: (flow: Flow) => void;
}

export function FlowSecrets({ flow, onSignOut, onSaved }: Props) {
  const [vaults, setVaults] = useState<Vault[]>([]);
  const [selectedVaultIds, setSelectedVaultIds] = useState<string[]>(
    flow.vault_ids,
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
      onSaved(await updateFlow(flow.id, { name: flow.name, vault_ids: selectedVaultIds }));
      setMessage("Saved.");
    } catch (err) {
      setError(
        getApiErrorMessage(err, onSignOut, "Could not save attached vaults"),
      );
    } finally {
      setPending(false);
    }
  }

  return (
    <div className="ui-dialog-content flow-settings-form">
      <h3>Attached vaults</h3>
      <p className="ui-hint">Attach vaults to use their secrets. Specify the vault name in each secret reference.</p>
      {loading && <p role="status">Loading vaults…</p>}
      <div className="secret-set-choices">
        {vaults.map(vault => (
          <label key={vault.id} className="secret-set-choice">
            <input
              type="checkbox"
              checked={selectedVaultIds.includes(vault.id)}
              disabled={loading || pending}
              onChange={(event) => {
                setMessage(null);
                setError(null);
                setSelectedVaultIds(current =>
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
        {pending ? "Saving…" : "Save attached vaults"}
      </button>
      {!loading && selectedVaultIds.map(vaultId => (
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
