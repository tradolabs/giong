import { type ReactNode, useEffect, useState } from "react";
import { getApiErrorMessage } from "../../../api/errors";
import { fetchVaults } from "../../../api/requests";
import type { Vault } from "../../../types/secret";
import { SettingsTabs } from "./SettingsTabs";
import { Vaults } from "./Vaults";
import "./FlowSettingsDialog.css";

interface Props {
  tab: "vault" | "plugins";
  children: ReactNode;
  onSignOut: () => void;
}

export function GlobalSettings({ tab, children, onSignOut }: Props) {
  const [vaults, setVaults] = useState<Vault[]>([]);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);

  useEffect(() => {
    if (tab !== "vault") return;
    let active = true;
    setLoading(true);
    setError(null);
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
  }, [onSignOut, tab]);
  return (
    <section className="global-settings ui-settings" aria-label="Settings">
      <header className="screen-header"><h1>Settings</h1></header>
      <div className="settings-surface settings-layout">
        <SettingsTabs
          id="global-settings"
          orientation="vertical"
          tabs={[{ id: "vault", label: "Vaults" }, { id: "plugins", label: "Plugins" }]}
          selected={tab}
          onSelect={tab => { window.location.hash = tab === "plugins" ? "/settings/plugins" : "/settings"; }}
        />
        <section
          className="settings-vault-panel"
          role="tabpanel"
          id="global-settings-vault-panel"
          aria-labelledby="global-settings-vault-tab"
          hidden={tab !== "vault"}
        >
          {tab === "vault" && (
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
          )}
        </section>
        <section
          className="settings-vault-panel"
          role="tabpanel"
          id="global-settings-plugins-panel"
          aria-labelledby="global-settings-plugins-tab"
          hidden={tab !== "plugins"}
        >
          {tab === "plugins" && children}
        </section>
      </div>

    </section>
  );
}
