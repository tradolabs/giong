import { useEffect, useRef, useState } from "react";
import { fetchVaults } from "../../../api/requests";
import { UnauthorizedError } from "../../../api/exception";
import type { Vault } from "../../../types/secret";
import { Vaults } from "./Vaults";
import { EditVault } from "./EditVault";
import "./SecretsDialog.css";
import "./FlowSettingsDialog.css";

interface Props { onClose: () => void; onSignOut: () => void }

export function GlobalSettingsDialog({ onClose, onSignOut }: Props) {
  const dialog = useRef<HTMLDialogElement>(null);
  const [vaults, setVaults] = useState<Vault[]>([]);
  const [editingId, setEditingId] = useState<string | null>(null);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);

  useEffect(() => { dialog.current?.showModal(); }, []);

  useEffect(() => {
    let active = true;
    fetchVaults().then((data) => {
      if (active) setVaults(data.slice().sort((a, b) => a.name.localeCompare(b.name)));
    }).catch((err: unknown) => {
      if (!active) return;
      if (err instanceof UnauthorizedError) onSignOut();
      else setError(err instanceof Error ? err.message : "Could not load vaults");
    }).finally(() => { if (active) setLoading(false); });
    return () => { active = false; };
  }, [onSignOut]);

  const editingVault = vaults.find((vault) => vault.id === editingId) ?? null;

  return <dialog ref={dialog} className="secrets-dialog" onCancel={event => { event.preventDefault(); onClose(); }}>
    <header className="secrets-header">
      <h2>{editingVault ? `Edit vault: ${editingVault.name}` : "Vaults"}</h2>
      <div className="secret-row-actions">
        {editingVault && <button type="button" className="btn-ghost" onClick={() => setEditingId(null)}>Back</button>}
        <button type="button" onClick={onClose}>Close</button>
      </div>
    </header>
    {editingVault ? (
      <EditVault
        key={editingVault.id}
        vault={editingVault}
        onSignOut={onSignOut}
        onSaved={(updated) => {
          setVaults((current) => current.map((v) => v.id === updated.id ? updated : v).sort((a, b) => a.name.localeCompare(b.name)));
        }}
      />
    ) : (
      <Vaults
        vaults={vaults}
        loading={loading}
        error={error}
        onError={setError}
        onCreated={(vault) => {
          setVaults((current) => [...current, vault].sort((a, b) => a.name.localeCompare(b.name)));
        }}
        onDeleted={(id) => {
          setVaults((current) => current.filter((v) => v.id !== id));
        }}
        onEdit={(vault) => {
          setError(null);
          setEditingId(vault.id);
        }}
        onSignOut={onSignOut}
      />
    )}
  </dialog>;
}
