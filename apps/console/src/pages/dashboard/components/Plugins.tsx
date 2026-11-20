import { type SubmitEvent, useEffect, useState } from "react";
import { getApiErrorMessage } from "../../../api/errors";
import { fetchPlugins, installPlugin, removePlugin } from "../../../api/requests";
import type { Flow } from "../../../types/flow";
import type { PluginInventory, PluginStatus } from "../../../types/plugin";
import "./Plugins.css";

interface Props {
  flows: Flow[];
  revision: number;
  flowsLoading: boolean;
  flowsError: string | null;
  onRefresh: () => void;
  onSignOut: () => void;
}

type PluginUsage = Map<Flow, number>;

const statusLabels: Record<PluginStatus, string> = {
  installed: "Installed",
  missing: "Image missing",
  unknown: "Unknown",
};

export function Plugins({ revision, flows, flowsLoading, flowsError, onRefresh, onSignOut }: Props) {
  const [search, setSearch] = useState("");
  const [inventory, setInventory] = useState<PluginInventory | null>(null);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);
  const [actionError, setActionError] = useState<string | null>(null);
  const [busy, setBusy] = useState(false);
  const [image, setImage] = useState("");
  const [pull, setPull] = useState(false);

  useEffect(() => {
    const controller = new AbortController();
    setLoading(true);
    setError(null);
    void fetchPlugins(controller.signal)
      .then(data => {
        if (!controller.signal.aborted) setInventory(data);
      })
      .catch((error: unknown) => {
        if (!controller.signal.aborted)
          setError(getApiErrorMessage(error, onSignOut, "Could not load plugins"));
      })
      .finally(() => {
        if (!controller.signal.aborted) setLoading(false);
      });
    return () => controller.abort();
  }, [revision, onSignOut]);

  async function install(event: SubmitEvent<HTMLFormElement>) {
    event.preventDefault();
    setBusy(true);
    setActionError(null);
    try {
      await installPlugin(image.trim(), pull);
      setImage("");
      onRefresh();
    } catch (error) {
      setActionError(getApiErrorMessage(error, onSignOut, "Could not install plugin"));
    } finally {
      setBusy(false);
    }
  }

  async function remove(kind: string) {
    setBusy(true);
    setActionError(null);
    try {
      await removePlugin(kind);
      onRefresh();
    } catch (error) {
      setActionError(getApiErrorMessage(error, onSignOut, "Could not remove plugin"));
    } finally {
      setBusy(false);
    }
  }

  const registered = new Map(inventory?.plugins.map(plugin => [plugin.kind, plugin]));
  const usage = new Map<string, PluginUsage>();
  const usageAvailable = !flowsLoading && !flowsError;

  if (usageAvailable) {
    for (const flow of flows) {
      for (const node of flow.nodes) {
        const flowCounts = usage.get(node.type) ?? new Map<Flow, number>();
        usage.set(node.type, flowCounts);
        flowCounts.set(flow, (flowCounts.get(flow) ?? 0) + 1);
      }
    }
  }

  const plugins = Array.from(new Set([...registered.keys(), ...usage.keys()])).sort((a, b) => a.localeCompare(b));
  const query = search.trim().toLowerCase();
  const visible = plugins.filter(kind => kind.toLowerCase().includes(query));
  const refreshing = loading || flowsLoading;

  return (
    <section className="ui-dialog-content flow-settings-form" aria-labelledby="plugins-title">
      <header className="section-heading">
        <h2 id="plugins-title">Plugins</h2>
        <button type="button" className="btn-ghost" disabled={refreshing || busy} onClick={onRefresh}>
          Refresh
        </button>
      </header>
      <form className="plugin-install" onSubmit={event => { void install(event); }}>
        <label htmlFor="plugin-image">Docker image</label>
        <div className="plugin-install-controls">
          <input id="plugin-image" value={image} onChange={event => setImage(event.target.value)} required />
          <button type="submit" className="btn-primary" disabled={busy || !image.trim()}>
            {busy ? "Working…" : "Install"}
          </button>
        </div>
        <label className="plugin-install-option">
          <input type="checkbox" checked={pull} onChange={event => setPull(event.target.checked)} />
          Pull from registry
        </label>
      </form>
      <div className="flow-list-toolbar">
        <input
          type="search"
          aria-label="Search plugins"
          placeholder="Search plugins"
          value={search}
          onChange={event => setSearch(event.target.value)}
        />
      </div>
      {loading && <p className="ui-empty" role="status">Loading plugins…</p>}
      {error && <p className="ui-error" role="alert">{error}</p>}
      {actionError && <p className="ui-error" role="alert">{actionError}</p>}
      {inventory && !inventory.docker_available && <p className="ui-empty" role="status">Docker is unavailable.</p>}
      {flowsError && <p className="ui-error" role="alert">{flowsError}</p>}
      {!loading && !error && (
        visible.length > 0 ? (
          <div className="plugins-table-container">
            <table aria-label="Plugins" className="plugins-table">
              <thead>
                <tr>
                  <th scope="col">Plugin</th>
                  <th scope="col">Status</th>
                  <th scope="col">Used by</th>
                  <th scope="col" className="plugin-actions" aria-label="Actions" />
                </tr>
              </thead>
              <tbody>
                {visible.map(kind => {
                  const plugin = registered.get(kind);
                  const status = plugin?.status ?? (inventory?.docker_available ? "missing" : "unknown");
                  const flowCounts = usage.get(kind);
                  return (
                    <tr key={kind}>
                      <th scope="row">
                        <code>{kind}</code>
                        {plugin && <span className="plugin-image">{plugin.image}{plugin.version && ` · ${plugin.version}`}</span>}
                      </th>
                      <td>
                        <span className={`plugin-status ${status}`}>
                          {!plugin && status === "missing" ? "Not installed" : statusLabels[status]}
                        </span>
                      </td>
                      <td>
                        {flowCounts ? (
                          <ul className="plugin-flow-list">
                            {Array.from(flowCounts, ([flow, count]) => (
                              <li key={flow.id}>
                                <a href={`#/flows/${encodeURIComponent(flow.id)}`}>{flow.name}</a>
                                <span>{count} {count === 1 ? "node" : "nodes"}</span>
                              </li>
                            ))}
                          </ul>
                        ) : flowsLoading ? "Loading…" : flowsError ? "Unavailable" : "No flows"}
                      </td>
                      <td className="plugin-actions">
                        {plugin && (
                          <button type="button" className="btn-ghost plugin-remove" aria-label={`Remove ${kind}`} disabled={busy} onClick={() => { void remove(kind); }}>
                            Remove
                          </button>
                        )}
                      </td>
                    </tr>
                  );
                })}
              </tbody>
            </table>
          </div>
        ) : (
          <div className="empty-state">
            <p className="empty-state-title">
              {plugins.length > 0 ? "No matching plugins" : "No plugins found"}
            </p>
          </div>
        )
      )}
    </section>
  );
}
