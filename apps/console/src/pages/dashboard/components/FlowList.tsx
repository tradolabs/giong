import { useState } from "react";
import type { Flow } from "../../../types/flow";

interface Props {
  flows: Flow[];
  loading: boolean;
  error: string | null;
  onAddFlow: () => void;
  onSettings: (flow: Flow) => void;
}

export function FlowList({ flows, loading, error, onSettings, onAddFlow }: Props) {
  const [query, setQuery] = useState("");
  const visible = flows.filter(flow => flow.name.toLowerCase().includes(query.trim().toLowerCase()));
  return (
    <section className="flow-list-screen" aria-label="Flows">
      <header className="screen-header">
        <h1>Flows <span className="pane-count">{flows.length}</span></h1>
        <button type="button" className="btn-primary" onClick={onAddFlow}>Create flow</button>
      </header>
      <div className="flow-list-toolbar">
        <input type="search" aria-label="Search flows" placeholder="Search flows" value={query} onChange={event => setQuery(event.target.value)} />
      </div>
      {loading && <p role="status">Loading flows…</p>}
      {error && <p className="ui-error" role="alert">{error}</p>}
      {!loading && visible.length === 0 && (
        <div className="empty-state">
          <p className="empty-state-title">{flows.length ? "No matching flows" : "No flows yet"}</p>
        </div>
      )}
      <div className="flow-list-grid">
        {visible.map(flow => (
          <article key={flow.id} className="flow-list-card">
            <a className="flow-list-open" href={`#/flows/${encodeURIComponent(flow.id)}`}>
              <span className="flow-list-symbol" aria-hidden="true">↗</span>
              <h2>{flow.name}</h2>
              <span>{flow.nodes.length} {flow.nodes.length === 1 ? "node" : "nodes"}</span>
            </a>
            <footer>
              <time dateTime={flow.updated_at.replace(" ", "T") + "Z"}>
                {new Date(flow.updated_at.replace(" ", "T") + "Z").toLocaleDateString()}
              </time>
              <button type="button" className="btn-ghost" aria-label={`Settings for ${flow.name}`} onClick={() => onSettings(flow)}>Settings</button>
            </footer>
          </article>
        ))}
      </div>
    </section>
  );
}
