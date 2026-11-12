import type { Flow } from "../../../types/flow";

interface FlowPaneProps {
  flows: Flow[];
  selectedFlowId: string | null;
  onSelectFlow: (id: string) => void;
  onAddFlow: () => void;
  onSettings: (flow: Flow) => void;
}

export function FlowPane({ flows, selectedFlowId, onSelectFlow, onSettings, onAddFlow }: FlowPaneProps) {
  return (
    <section className="pane flow-pane" aria-label="Flows">
      <div className="pane-header">
        <h2 className="pane-title"><span className="pane-index" aria-hidden="true">01</span>Flows <span className="pane-count">{flows.length}</span></h2>
        <button type="button" className="btn-ghost" onClick={onAddFlow}>Add flow</button>
      </div>
      <div className="pane-body">
        {flows.map((flow) => (
          <div key={flow.id} className={`list-item flow-card ${selectedFlowId === flow.id ? "active" : ""}`}>
            <button type="button" className="flow-card-select"
              aria-pressed={selectedFlowId === flow.id}
              onClick={() => onSelectFlow(flow.id)}>
              <span className="flow-card-heading">
                <span className="flow-card-symbol" aria-hidden="true">
                  <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="1.5" strokeLinejoin="round">
                    <path d="m12 3 9 5-9 5-9-5 9-5Z" /><path d="m3 12 9 5 9-5M3 16l9 5 9-5" />
                  </svg>
                </span>
                <span className="item-name" title={flow.name}>{flow.name}</span>
              </span>
              <span className="item-meta">{`${flow.nodes.length} ${flow.nodes.length === 1 ? "node" : "nodes"}`}</span>
            </button>
            <button type="button" className="flow-card-settings" title="Flow settings"
              aria-label={`Settings for ${flow.name}`} onClick={() => onSettings(flow)}>
              <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="1.5" strokeLinecap="round" aria-hidden="true">
                <path d="M4 7h16M4 17h16" /><circle cx="9" cy="7" r="3" fill="var(--bg-surface)" />
                <circle cx="15" cy="17" r="3" fill="var(--bg-surface)" />
              </svg>
            </button>
          </div>
        ))}

        {flows.length === 0 && (
          <div className="empty-state">
            <p className="empty-state-title">No flows yet</p>
            <button type="button" className="btn-ghost" onClick={onAddFlow}>Add your first flow</button>
          </div>
        )}
      </div>
    </section>
  );
}
