import type { Stack } from "../../../types/stack";
import { formatRepo, shortSha } from "../../../shared/format";

interface StackPaneProps {
  stacks: Stack[];
  selectedStackId: string | null;
  onSelectStack: (id: string) => void;
  onManageSecrets: (stack: Stack) => void;
}

export function StackPane({ stacks, selectedStackId, onSelectStack, onManageSecrets }: StackPaneProps) {
  const selected = stacks.find((stack) => stack.id === selectedStackId);
  return (
    <section className="pane" aria-label="Stacks">
      <div className="pane-header">
        <h2 className="pane-title"><span className="pane-index" aria-hidden="true">01</span>Stacks</h2>
        <div className="topbar-actions">
          <span className="pane-count">{stacks.length}</span>
          <button type="button" className="btn-ghost" disabled={!selected}
            onClick={() => { if (selected) onManageSecrets(selected); }}>Credentials</button>
        </div>
      </div>
      <div className="pane-body">
        {stacks.map((stack) => (
          <button
            key={stack.id}
            type="button"
            className={`list-item ${selectedStackId === stack.id ? "active" : ""}`}
            aria-pressed={selectedStackId === stack.id}
            onClick={() => onSelectStack(stack.id)}
          >
            <div className="item-row">
              <span className="item-name">{stack.name}</span>
              <span className="branch-tag mono">
                {stack.branch}
              </span>
            </div>
            <div className="item-meta">
              <span>{formatRepo(stack.repo_url)}</span>
              {stack.folder_path && stack.folder_path !== "." && (
                <span>· {stack.folder_path}</span>
              )}
            </div>
            {stack.last_commit && (
              <div className="item-meta mono">
                HEAD {shortSha(stack.last_commit)}
              </div>
            )}
          </button>
        ))}

        {stacks.length === 0 && (
          <div className="empty-state">
            <p className="empty-state-title">No stacks yet</p>
            <span>Use the CLI to add a repository and stack.</span>
          </div>
        )}
      </div>
    </section>
  );
}
