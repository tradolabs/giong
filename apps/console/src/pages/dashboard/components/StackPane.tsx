import type { Stack } from "../../../types/stack";

interface StackPaneProps {
  stacks: Stack[];
  selectedStackId: string | null;
  onSelectStack: (id: string) => void;
  onAddStack: () => void;
  onSettings: (stack: Stack) => void;
}

export function StackPane({ stacks, selectedStackId, onSelectStack, onSettings, onAddStack }: StackPaneProps) {
  return (
    <section className="pane stack-pane" aria-label="Stacks">
      <div className="pane-header">
        <h2 className="pane-title"><span className="pane-index" aria-hidden="true">01</span>Stacks <span className="pane-count">{stacks.length}</span></h2>
        <button type="button" className="btn-ghost" onClick={onAddStack}>Add stack</button>
      </div>
      <div className="pane-body">
        {stacks.map((stack) => (
          <div key={stack.id} className={`list-item stack-card ${selectedStackId === stack.id ? "active" : ""}`}>
            <button type="button" className="stack-card-select"
              aria-pressed={selectedStackId === stack.id}
              onClick={() => onSelectStack(stack.id)}>
              <span className="stack-card-heading">
                <span className="stack-card-symbol" aria-hidden="true">
                  <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="1.5" strokeLinejoin="round">
                    <path d="m12 3 9 5-9 5-9-5 9-5Z" /><path d="m3 12 9 5 9-5M3 16l9 5 9-5" />
                  </svg>
                </span>
                <span className="item-name" title={stack.name}>{stack.name}</span>
              </span>
              <span className="item-meta">{String(stack.configurations.runtime ?? "Configuration required")}</span>
            </button>
            <button type="button" className="stack-card-settings" title="Stack settings"
              aria-label={`Settings for ${stack.name}`} onClick={() => onSettings(stack)}>
              <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="1.5" strokeLinecap="round" aria-hidden="true">
                <path d="M4 7h16M4 17h16" /><circle cx="9" cy="7" r="3" fill="var(--bg-surface)" />
                <circle cx="15" cy="17" r="3" fill="var(--bg-surface)" />
              </svg>
            </button>
          </div>
        ))}

        {stacks.length === 0 && (
          <div className="empty-state">
            <p className="empty-state-title">No stacks yet</p>
            <button type="button" className="btn-ghost" onClick={onAddStack}>Add your first stack</button>
          </div>
        )}
      </div>
    </section>
  );
}
