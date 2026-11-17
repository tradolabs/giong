import { useState } from "react";
import { getApiErrorMessage } from "../../../api/errors";
import { deleteVault } from "../../../api/requests";
import type { Vault } from "../../../types/secret";
import { VaultDialog } from "./VaultDialog";

interface Props {
  vaults: Vault[];
  loading: boolean;
  error: string | null;
  onError: (error: string | null) => void;
  onCreated: (vault: Vault) => void;
  onDeleted: (id: string) => void;
  onUpdated: (vault: Vault) => void;
  onSignOut: () => void;
}

export function Vaults({
  vaults,
  loading,
  error,
  onError,
  onCreated,
  onDeleted,
  onUpdated,
  onSignOut,
}: Props) {
  const [editor, setEditor] = useState<{ vault?: Vault } | null>(null);
  const [deletingId, setDeletingId] = useState<string | null>(null);

  async function remove(id: string) {
    if (deletingId) return;
    setDeletingId(id);
    onError(null);
    try {
      await deleteVault(id);
      onDeleted(id);
    } catch (err) {
      onError(getApiErrorMessage(err, onSignOut, "Could not delete vault"));
    } finally {
      setDeletingId(null);
    }
  }

  const busy = deletingId !== null;

  return (
    <div className="ui-dialog-content flow-settings-form">
      <div className="section-heading">
        <h3>Vaults</h3>
        <button type="button" className="btn-primary" disabled={busy || loading} onClick={() => setEditor({})}>Create vault</button>
      </div>
      {loading && <p role="status">Loading vaults…</p>}
      {error && (
        <p className="ui-error" role="alert">
          {error}
        </p>
      )}
      {!loading && vaults.length === 0 && (
        <p className="ui-empty">No vaults yet.</p>
      )}
      {vaults.length > 0 && (
        <div className="vault-list" aria-label="Vaults">
          {vaults.map(vault => (
            <div key={vault.id} className="vault-entry">
              <div className="inherited-secret">
              <div>
                <strong>{vault.name}</strong>
              </div>
              <div className="secret-row-actions">
                <button type="button" className="btn-ghost" disabled={busy} onClick={() => setEditor({ vault })}>Settings</button>
                <button
                  type="button"
                  className="btn-ghost"
                  disabled={busy}
                  onClick={() => {
                    void remove(vault.id);
                  }}
                >
                  {deletingId === vault.id ? "Deleting…" : "Delete"}
                </button>
              </div>
              </div>

            </div>
          ))}
        </div>
      )}
      {editor && (
        <VaultDialog
          vault={editor.vault}
          onClose={() => setEditor(null)}
          onSignOut={onSignOut}
          onSaved={vault => {
            if (editor.vault) onUpdated(vault);
            else onCreated(vault);
            setEditor({ vault });
          }}
        />
      )}
    </div>
  );
}
