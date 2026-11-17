import { useEffect, useId, useState } from "react";
import { fetchNodeDetails } from "../../../api/requests";
import { getApiErrorMessage } from "../../../api/errors";
import type { NodeDetails } from "../../../types/node-details";
import { SettingsTabs } from "./SettingsTabs";
import { Dialog } from "../../../components/ui/Dialog";
import type { FlowNode } from "../../../types/flow";
import { NodeLogPreview } from "./NodeLogPreview";
import { StatusBadge } from "./StatusBadge";
import "./NodeDetailsDialog.css";

interface Props {
  flowId: string;
  node: FlowNode;
  onClose: () => void;
  onSignOut: () => void;
}

function formatValue(value: unknown): string {
  return typeof value === "string" ? value : JSON.stringify(value, null, 2);
}

export function NodeDetailsDialog({ flowId, node, onClose, onSignOut }: Props) {
  const id = useId();
  const [tab, setTab] = useState("general");
  const [details, setDetails] = useState<NodeDetails | null>(null);
  const [error, setError] = useState<string | null>(null);
  useEffect(() => {
    if (!node.id) return;
    let active = true;
    const controller = new AbortController();
    void fetchNodeDetails(flowId, node.id, controller.signal)
      .then((data) => {
        if (active) {
          setDetails(data);
          setError(null);
        }
      })
      .catch((err: unknown) => {
        if (active) setError(getApiErrorMessage(err, onSignOut, "Could not load node details"));
      });
    return () => { active = false; controller.abort(); };
  }, [flowId, node.id, node.last_execution?.id, node.last_execution?.status, onSignOut]);
  const merged = new Map((details?.executions ?? []).map(execution => [execution.id, execution]));
  if (node.last_execution && !merged.has(node.last_execution.id)) {
    merged.set(node.last_execution.id, node.last_execution);
  }
  const logs = [...merged.values()]
    .filter(execution => execution.node_id === node.id)
    .sort((a, b) => b.id.localeCompare(a.id))
    .slice(0, 10);
  const [selected, setSelected] = useState<string | null>(null);
  const active = logs.find(execution => execution.id === selected) ?? logs[0] ?? null;
  const entries = details?.output && typeof details.output === "object"
    ? Object.entries(details.output as Record<string, unknown>)
    : [];

  return (
    <Dialog
      title={node.name}
      subtitle={node.type}
      className="node-details-dialog"
      onClose={onClose}
    >
      <div className="node-details-layout">
        <SettingsTabs id={id} orientation="vertical" tabs={[{ id: "general", label: "General" }, { id: "logs", label: "Logs" }]} selected={tab} onSelect={setTab} />
        <section className="node-details-general" role="tabpanel" id={`${id}-general-panel`} aria-labelledby={`${id}-general-tab`} hidden={tab !== "general"}>
          {!details && !error && <p role="status">Loading node data…</p>}
          {error && <p role="alert" className="ui-error">{error}</p>}
          {details && !entries.length && <p className="ui-empty">No stored data yet.</p>}
          {details && <dl className="node-storage-entries">{entries.map(([key, value]) => (
            <div key={key}><dt><code>{key}</code></dt><dd><pre>{formatValue(value)}</pre></dd></div>
          ))}</dl>}
        </section>
        <div className="node-details-logs" role="tabpanel" id={`${id}-logs-panel`} aria-labelledby={`${id}-logs-tab`} hidden={tab !== "logs"}>
          {error && <p role="alert" className="ui-error">{error}</p>}
          <div className="node-logs-grid">
            <aside className="node-logs-list" aria-label="Log files">
              {!logs.length && (
                <p className="ui-hint">
                  No logs yet. Execute this node to create a log.
                </p>
              )}
              {logs.map(execution => (
                <button
                  key={execution.id}
                  type="button"
                  className={`node-log-file ${active?.id === execution.id ? "selected" : ""}`}
                  aria-pressed={active?.id === execution.id}
                  onClick={() => setSelected(execution.id)}
                >
                  <strong>
                    {new Date(
                      (execution.started_at ?? execution.created_at).replace(" ", "T") + "Z",
                    ).toLocaleString()}
                  </strong>
                  <code>
                    {execution.id}
                    .log
                  </code>
                  <StatusBadge status={execution.status} />
                </button>
              ))}
            </aside>
            {tab === "logs" && <NodeLogPreview execution={active} onSignOut={onSignOut} />}
          </div>
        </div>
      </div>
    </Dialog>
  );
}
