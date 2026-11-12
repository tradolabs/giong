import { useMemo, useState } from "react";
import type { Flow } from "../../../types/flow";
import type { NodeRun } from "../../../types/node-run";
import { StatusBadge } from "./StatusBadge";
import "./FlowCanvas.css";

interface Props {
  flow: Flow | undefined;
  runs: NodeRun[];
  selected: string | null;
  submitting: string | null;
  error: string | null;
  onSelect: (id: string) => void;
  onRun: (id: string) => void;
  onSettings: () => void;
}
export function FlowCanvas({ flow, runs, selected, submitting, error, onSelect, onRun, onSettings }: Props) {
  const [zoom, setZoom] = useState(1);
  const layout = useMemo(() => {
    const nodes = flow?.nodes ?? [];
    const ranks = new Map<string, number>();
    for (let pass = 0; pass < nodes.length; pass++) {
      for (const node of nodes) {
        const sources = node.triggers.filter(trigger => trigger.type === "event");
        ranks.set(node.name, Math.min(nodes.length, sources.reduce((rank, trigger) =>
          trigger.type === "event" ? Math.max(rank, (ranks.get(trigger.node) ?? -1) + 1) : rank, 0)));
      }
    }
    const rows = new Map<number, number>();
    const placed = nodes.map(node => {
      const column = ranks.get(node.name) ?? 0;
      const row = rows.get(column) ?? 0; rows.set(column, row + 1);
      return { node, x: 40 + column * 400, y: 50 + row * 230 };
    });
    return { placed, width: Math.max(700, ...placed.map(item => item.x + 370)), height: Math.max(380, ...placed.map(item => item.y + 230)) };
  }, [flow?.nodes]);
  return <section className="pane flow-canvas-pane" aria-label="Flow canvas">
    <header className="pane-header">
      <h2 className="pane-title">{flow?.name ?? "Flow canvas"}</h2>
      {flow && <div className="flow-canvas-tools">
        <button type="button" className="btn-ghost" aria-label="Zoom out" onClick={() => setZoom(value => Math.max(.5, value - .1))}>−</button>
        <button type="button" className="btn-ghost" aria-label="Reset zoom" onClick={() => setZoom(1)}>{Math.round(zoom * 100)}%</button>
        <button type="button" className="btn-ghost" aria-label="Zoom in" onClick={() => setZoom(value => Math.min(1.5, value + .1))}>+</button>
        <button type="button" className="btn-ghost" onClick={onSettings}>Edit nodes</button>
      </div>}
    </header>
    {error && <p role="alert" className="canvas-error">{error}</p>}
    {!flow ? <div className="empty-state">Select a flow to view its nodes.</div> : !flow.nodes.length ?
      <div className="empty-state"><p>No nodes yet.</p><button type="button" className="btn-ghost" onClick={onSettings}>Add nodes</button></div> :
      <div className="flow-canvas-scroll"><div style={{ width: layout.width * zoom, height: layout.height * zoom }}>
        <div className="flow-canvas-board" style={{ width: layout.width, height: layout.height, transform: `scale(${zoom})` }}>
          <svg className="flow-canvas-edges" width={layout.width} height={layout.height} aria-hidden="true">
            <defs><marker id="canvas-arrow" viewBox="0 0 10 10" refX="9" refY="5" markerWidth="7" markerHeight="7" orient="auto-start-reverse"><path d="M 0 0 L 10 5 L 0 10 z" /></marker></defs>
            {layout.placed.flatMap(target => target.node.triggers.flatMap((trigger, index) => {
              if (trigger.type !== "event") return [];
              const source = layout.placed.find(item => item.node.name === trigger.node);
              if (!source) return [];
              const x = source.x + 320, y = source.y + 105, endX = target.x, endY = target.y + 105;
              return <g key={`${target.node.id}-${index}`}><path d={`M${x},${y} C${x + 45},${y} ${endX - 45},${endY} ${endX},${endY}`} markerEnd="url(#canvas-arrow)" /><text x={(x + endX) / 2} y={(y + endY) / 2 - 10} textAnchor="middle">{trigger.event}</text></g>;
            }))}
          </svg>
          {layout.placed.map(({ node, x, y }) => {
            const latest = runs.find(run => run.node_id === node.id);
            const active = latest?.status === "pending" || latest?.status === "running";
            return <article key={node.id} className={`canvas-node ${selected === node.id ? "selected" : ""}`} style={{ left: x, top: y }} onClick={() => node.id && onSelect(node.id)}>
              <button type="button" className="canvas-node-name" title={node.name} onClick={() => node.id && onSelect(node.id)}>{node.name}</button>
              <code title={node.type}>{node.type}</code>
              <small className="canvas-last-run">Last run: {latest ? new Date((latest.started_at ?? latest.created_at).replace(" ", "T") + "Z").toLocaleString() : "Never"}</small>
              <footer>{latest ? <StatusBadge status={latest.status} /> : <span className="canvas-idle">Not run</span>}
                <button type="button" className="btn-ghost" disabled={!node.id || active || submitting === node.id} onClick={event => { event.stopPropagation(); if (node.id) onRun(node.id); }}>{submitting === node.id ? "Starting…" : "Run"}</button>
              </footer>
              <small className="canvas-schedule">{node.triggers.filter(trigger => trigger.type === "interval").map(trigger => trigger.type === "interval" ? `Every ${trigger.seconds}s` : "").join(", ") || "\u00a0"}</small>
            </article>;
          })}
        </div>
      </div></div>}
  </section>;
}
