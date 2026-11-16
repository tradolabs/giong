import { useEffect, useState } from "react";
import { getApiErrorMessage } from "../../../api/errors";
import { fetchVaults } from "../../../api/requests";
import type { Vault } from "../../../types/secret";
import { SettingsTabs } from "./SettingsTabs";
import { Vaults } from "./Vaults";
import "./FlowSettingsDialog.css";

interface Props {
  onSignOut: () => void;
}

export function GlobalSettings({ onSignOut }: Props) {
  const [vaults, setVaults] = useState<Vault[]>([]);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);

  useEffect(() => {
    let active = true;
    fetchVaults()
      .then((data) => {
        if (active)
          setVaults(data.slice().sort((a, b) => a.name.localeCompare(b.name)));
      })
      .catch((err: unknown) => {
        if (active)
          setError(getApiErrorMessage(err, onSignOut, "Could not load vaults"));
      })
      .finally(() => {
        if (active) setLoading(false);
      });
    return () => { active = false; };
  }, [onSignOut]);


  return (
    <section className="global-settings ui-settings" aria-label="Setting">
      <header className="screen-header"><h1>Setting</h1></header>
      <div className="settings-surface settings-layout">
      <SettingsTabs
        id="global-settings"
        orientation="vertical"
        tabs={[{ id: "vault", label: "Vaults" }]}
        selected="vault"
        onSelect={() => {}}
      />
      <section
        className="settings-vault-panel"
        role="tabpanel"
        id="global-settings-vault-panel"
        aria-labelledby="global-settings-vault-tab"
      >
          <Vaults
            vaults={vaults}
            loading={loading}
            error={error}
            onError={setError}
            onCreated={(vault) => {
              setVaults(current =>
                [...current, vault].sort((a, b) => a.name.localeCompare(b.name)),
              );
            }}
            onUpdated={(updated) => {
              setVaults(current =>
                current
                  .map(vault => vault.id === updated.id ? updated : vault)
                  .sort((a, b) => a.name.localeCompare(b.name)),
              );
            }}
            onDeleted={(id) => {
              setVaults(current => current.filter(vault => vault.id !== id));
            }}
            onSignOut={onSignOut}
          />
      </section>
      </div>

    </section>
  );
}
