import { useMemo, useState } from "react";
import type { Flow as FlowDefinition, FlowNode } from "../../../types/flow";
import type { NodeExecution } from "../../../types/node-execution";
import { StatusBadge } from "./StatusBadge";
import "./Flow.css";

interface Props {
  flow: FlowDefinition;
  executions: NodeExecution[];
  selected: string | null;
  submitting: string | null;
  error: string | null;
  onSelect: (id: string) => void;
  onExecute: (id: string) => void;
  onSettings: () => void;
  onAddNode: () => void;
  onEditNode: (node: FlowNode) => void;
}

export function Flow({
  flow,
  executions,
  selected,
  submitting,
  error,
  onSelect,
  onExecute,
  onSettings,
  onAddNode,
  onEditNode,
}: Props) {
  const [zoom, setZoom] = useState(1);
  const layout = useMemo(() => {
    const nodes = flow.nodes;
    const ranks = new Map<string, number>();
    for (let pass = 0; pass < nodes.length; pass++) {
      for (const node of nodes) {
        const sources = node.triggers.filter(
          trigger => trigger.type === "event",
        );
        ranks.set(
          node.name,
          Math.min(
            nodes.length,
            sources.reduce(
              (rank, trigger) =>
                trigger.type === "event"
                  ? Math.max(rank, (ranks.get(trigger.node) ?? -1) + 1)
                  : rank,
              0,
            ),
          ),
        );
      }
    }
    const rows = new Map<number, number>();
    const placed = nodes.map((node) => {
      const column = ranks.get(node.name) ?? 0;
      const row = rows.get(column) ?? 0;
      rows.set(column, row + 1);
      return { node, x: 40 + column * 400, y: 50 + row * 230 };
    });
    return {
      placed,
      width: Math.max(700, ...placed.map(item => item.x + 370)),
      height: Math.max(380, ...placed.map(item => item.y + 230)),
    };
  }, [flow.nodes]);
  return (
    <section className="pane flow-canvas-pane" aria-label="Flow">
      <header className="pane-header">
        <h2 className="pane-title">{flow.name}</h2>
        <div className="flow-canvas-tools">
          <button
            type="button"
            className="btn-ghost"
            aria-label="Zoom out"
            onClick={() => setZoom(value => Math.max(0.5, value - 0.1))}
          >
            −
          </button>
          <button
            type="button"
            className="btn-ghost"
            aria-label="Reset zoom"
            onClick={() => setZoom(1)}
          >
            {Math.round(zoom * 100)}
            %
          </button>
          <button
            type="button"
            className="btn-ghost"
            aria-label="Zoom in"
            onClick={() => setZoom(value => Math.min(1.5, value + 0.1))}
          >
            +
          </button>
          <button type="button" className="btn-primary" onClick={onAddNode}>Add node</button>
          <button type="button" className="btn-ghost" onClick={onSettings}>Settings</button>
        </div>
      </header>
      {error && (
        <p role="alert" className="canvas-error">
          {error}
        </p>
      )}
      {!flow.nodes.length
            ? (
                <div className="empty-state">
                  <p>No nodes yet.</p>

                </div>
              )
            : (
                <div className="flow-canvas-scroll">
                  <div
                    style={{ width: layout.width * zoom, height: layout.height * zoom }}
                  >
                    <div
                      className="flow-canvas-board"
                      style={{
                        width: layout.width,
                        height: layout.height,
                        transform: `scale(${zoom})`,
                      }}
                    >
                      <svg
                        className="flow-canvas-edges"
                        width={layout.width}
                        height={layout.height}
                        aria-hidden="true"
                      >
                        <defs>
                          <marker
                            id="canvas-arrow"
                            viewBox="0 0 10 10"
                            refX="9"
                            refY="5"
                            markerWidth="7"
                            markerHeight="7"
                            orient="auto-start-reverse"
                          >
                            <path d="M 0 0 L 10 5 L 0 10 z" />
                          </marker>
                        </defs>
                        {layout.placed.flatMap(target =>
                          target.node.triggers.flatMap((trigger, index) => {
                            if (trigger.type !== "event") return [];
                            const source = layout.placed.find(
                              item => item.node.name === trigger.node,
                            );
                            if (!source) return [];
                            const x = source.x + 320,
                              y = source.y + 105,
                              endX = target.x,
                              endY = target.y + 105;
                            return (
                              <g key={`${target.node.id}-${index}`}>
                                <path
                                  d={`M${x},${y} C${x + 45},${y} ${endX - 45},${endY} ${endX},${endY}`}
                                  markerEnd="url(#canvas-arrow)"
                                />
                                <text
                                  x={(x + endX) / 2}
                                  y={(y + endY) / 2 - 10}
                                  textAnchor="middle"
                                >
                                  {trigger.event}
                                </text>
                              </g>
                            );
                          }),
                        )}
                      </svg>
                      {layout.placed.map(({ node, x, y }) => {
                        const latest = executions.find(execution => execution.node_id === node.id);
                        return (
                          <article
                            key={node.id}
                            className={`canvas-node ${selected === node.id ? "selected" : ""}`}
                            style={{ left: x, top: y }}
                            onClick={() => node.id && onSelect(node.id)}
                          >
                            <button
                              type="button"
                              className="canvas-node-name"
                              title={node.name}
                              onClick={() => node.id && onSelect(node.id)}
                            >
                              {node.name}
                            </button>
                            <code title={node.type}>{node.type}</code>
                            <small className="canvas-last-execution">
                              Last execution:
                              {" "}
                              {latest
                                ? new Date(
                                    (latest.started_at ?? latest.created_at).replace(
                                      " ",
                                      "T",
                                    ) + "Z",
                                  ).toLocaleString()
                                : "Never"}
                            </small>
                            <footer>
                              {latest
                                ? (
                                    <StatusBadge status={latest.status} />
                                  )
                                : (
                                    <span className="canvas-idle">Not executed</span>
                                  )}
                              <div className="canvas-node-actions">
                                <button type="button" className="btn-ghost" onClick={(event) => { event.stopPropagation(); onEditNode(node); }} aria-label={`Edit ${node.name}`}>Edit</button>
                              <button
                                type="button"
                                className="canvas-execute"
                                title={submitting === node.id ? "Starting…" : `Execute ${node.name}`}
                                aria-label={submitting === node.id ? `Starting ${node.name}` : `Execute ${node.name}`}
                                aria-busy={submitting === node.id}
                                disabled={!node.id || submitting !== null}
                                onClick={(event) => {
                                  event.stopPropagation();
                                  if (node.id) onExecute(node.id);
                                }}
                              >
                                <svg width="16" height="16" viewBox="0 0 24 24" fill="currentColor" aria-hidden="true">
                                  {submitting === node.id
                                    ? <><circle cx="6" cy="12" r="2" /><circle cx="12" cy="12" r="2" /><circle cx="18" cy="12" r="2" /></>
                                    : <path d="M8 5v14l11-7z" />}
                                </svg>
                              </button>
                              </div>
                            </footer>
                            <small className="canvas-schedule">
                              {node.triggers
                                .filter(trigger => trigger.type === "interval")
                                .map(trigger =>
                                  trigger.type === "interval"
                                    ? `Every ${trigger.seconds}s`
                                    : "",
                                )
                                .join(", ") || "\u00a0"}
                            </small>
                          </article>
                        );
                      })}
                    </div>
                  </div>
                </div>
              )}
    </section>
  );
}
