import "./NodeLogPreview.css";
import { useEffect, useRef, useState } from "react";
import { UnauthorizedError } from "../../../api/exception";
import { fetchNodeRunLogs } from "../../../api/requests";
import type { NodeRun } from "../../../types/node-run";
import { StatusBadge } from "./StatusBadge";

export function NodeLogPreview({
  run,
  onSignOut,
}: {
  run: NodeRun | null;
  onSignOut: () => void;
}) {
  const [lines, setLines] = useState<string[]>([]);
  const [autoScroll, setAutoScroll] = useState(true);
  const containerRef = useRef<HTMLDivElement | null>(null);
  const lineCountRef = useRef<number>(0);

  useEffect(() => {
    setLines([]);
    lineCountRef.current = 0;

    if (!run) return;
    const runId = run.id;
    let cancelled = false;
    let polling = false;

    async function pollLogs() {
      if (polling || cancelled) return;
      polling = true;
      try {
        const chunk = await fetchNodeRunLogs(runId, lineCountRef.current);
        if (cancelled || !chunk) return;

        const newLines = chunk.split("\n");
        if (newLines[newLines.length - 1] === "") {
          newLines.pop();
        }

        if (newLines.length > 0) {
          lineCountRef.current += newLines.length;
          setLines((prev) => [...prev, ...newLines]);
        }
      } catch (err) {
        if (err instanceof UnauthorizedError) onSignOut();
      } finally { polling = false; }
    }

    pollLogs();

    const isStreaming = run.status === "pending" || run.status === "running";
    if (!isStreaming) return () => { cancelled = true; };

    const interval = window.setInterval(pollLogs, 1000);
    return () => {
      cancelled = true;
      window.clearInterval(interval);
    };
  }, [run?.id, run?.status, onSignOut]);

  useEffect(() => {
    if (autoScroll && containerRef.current) {
      containerRef.current.scrollTop = containerRef.current.scrollHeight;
    }
  }, [lines, autoScroll]);

  if (!run) {
    return (
      <section className="log-pane" aria-label="Logs">
        <div className="pane-header">
          <h2 className="pane-title">Log output</h2>
        </div>
        <div className="empty-state">
          <p className="empty-state-title">Select a log to view its contents</p>
          <span>Output appears here while the node runs.</span>
        </div>
      </section>
    );
  }

  const isStreaming = run.status === "pending" || run.status === "running";

  return (
    <section className="log-pane" aria-label="Logs">
      <div className="log-header">
        <div className="log-heading">
          <p className="log-label">Log output</p>
          <div className="log-meta">
            <strong>{run.log_timestamp ? `${run.log_timestamp}.log` : "Log output"}</strong>
            <StatusBadge status={run.status} />
            {run.exit_code !== null && <span className="mono">exit {run.exit_code}</span>}
          </div>
        </div>

        <div className="topbar-actions">
          {isStreaming && (
            <span className="log-stream-indicator">
              <span className="status-dot" />
              Streaming
            </span>
          )}
          <button
            type="button"
            className="btn-ghost"
            aria-pressed={autoScroll}
            onClick={() => setAutoScroll((v) => !v)}
          >
            {autoScroll ? "Auto-scroll: On" : "Auto-scroll: Off"}
          </button>
        </div>
      </div>

      <div ref={containerRef} className="log-content">
        {lines.length === 0 ? (
          <span className="log-placeholder">
            {isStreaming ? "Waiting for log output..." : "No log output available."}
          </span>
        ) : (
          lines.map((line, index) => (
            <div key={index} className="log-line">
              <span className="log-line-num">{index + 1}</span>
              <span>{line}</span>
            </div>
          ))
        )}
      </div>
    </section>
  );
}
