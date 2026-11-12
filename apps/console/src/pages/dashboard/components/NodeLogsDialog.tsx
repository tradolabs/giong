import { useEffect, useId, useRef, useState } from "react";
import type { FlowNode } from "../../../types/flow";
import type { NodeRun } from "../../../types/node-run";
import { NodeLogPreview } from "./NodeLogPreview";
import { StatusBadge } from "./StatusBadge";
import "./NodeLogsDialog.css";

interface Props { node: FlowNode; runs: NodeRun[]; onClose: () => void; onSignOut: () => void }
export function NodeLogsDialog({ node, runs, onClose, onSignOut }: Props) {
  const dialog = useRef<HTMLDialogElement>(null);
  const titleId = useId();
  const logs = runs.filter(run => run.node_id === node.id && run.log_timestamp)
    .sort((a, b) => (b.log_timestamp ?? "").localeCompare(a.log_timestamp ?? "")).slice(0, 10);
  const [selected, setSelected] = useState<string | null>(null);
  useEffect(() => {
    const element = dialog.current;
    element?.showModal();
    return () => element?.close();
  }, []);
  const active = logs.find(run => run.id === selected) ?? logs[0] ?? null;
  return <dialog ref={dialog} className="secrets-dialog node-logs-dialog" aria-labelledby={titleId}
    onCancel={event => { event.preventDefault(); onClose(); }}>
    <header className="secrets-header"><div><h2 id={titleId}>{node.name} · Logs</h2><p>Latest 10 logs</p></div>
      <button type="button" className="btn-ghost" onClick={onClose}>Close</button>
    </header>
    <div className="node-logs-grid">
      <aside className="node-logs-list" aria-label="Log files">
        {!logs.length && <p className="secrets-hint">No logs yet. Run this node to create a log.</p>}
        {logs.map(run => <button key={run.id} type="button" className={`node-log-file ${active?.id === run.id ? "selected" : ""}`}
          aria-pressed={active?.id === run.id} onClick={() => setSelected(run.id)}>
          <strong>{new Date((run.started_at ?? run.created_at).replace(" ", "T") + "Z").toLocaleString()}</strong>
          <code>{run.log_timestamp}.log</code><StatusBadge status={run.status} />
        </button>)}
      </aside>
      <NodeLogPreview run={active} onSignOut={onSignOut} />
    </div>
  </dialog>;
}
