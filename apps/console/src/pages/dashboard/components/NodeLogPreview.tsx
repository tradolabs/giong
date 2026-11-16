import "./NodeLogPreview.css";
import { useEffect, useRef, useState } from "react";
import { UnauthorizedError } from "../../../api/exception";
import { fetchNodeExecutionLogs } from "../../../api/requests";
import type { NodeExecution } from "../../../types/node-execution";
import { StatusBadge } from "./StatusBadge";

export function NodeLogPreview({
  execution,
  onSignOut,
}: {
  execution: NodeExecution | null;
  onSignOut: () => void;
}) {
  const [lines, setLines] = useState<string[]>([]);
  const [autoScroll, setAutoScroll] = useState(true);
  const containerRef = useRef<HTMLDivElement | null>(null);
  const lineCountRef = useRef<number>(0);

  useEffect(() => {
    setLines([]);
    lineCountRef.current = 0;

    if (!execution) return;
    const executionId = execution.id;
    let cancelled = false;
    let polling = false;

    async function pollLogs() {
      if (polling || cancelled) return;
      polling = true;
      try {
        const chunk = await fetchNodeExecutionLogs(executionId, lineCountRef.current);
        if (cancelled || !chunk) return;

        const newLines = chunk.split("\n");
        if (newLines[newLines.length - 1] === "") {
          newLines.pop();
        }

        if (newLines.length > 0) {
          lineCountRef.current += newLines.length;
          setLines(prev => [...prev, ...newLines]);
        }
      } catch (err) {
        if (err instanceof UnauthorizedError) onSignOut();
      } finally {
        polling = false;
      }
    }

    pollLogs();

    const isStreaming = execution.status === "pending" || execution.status === "executing";
    if (!isStreaming)
      return () => {
        cancelled = true;
      };

    const interval = window.setInterval(pollLogs, 1000);
    return () => {
      cancelled = true;
      window.clearInterval(interval);
    };
  }, [execution?.id, execution?.status, onSignOut]);

  useEffect(() => {
    if (autoScroll && containerRef.current) {
      containerRef.current.scrollTop = containerRef.current.scrollHeight;
    }
  }, [lines, autoScroll]);

  if (!execution) {
    return (
      <section className="log-pane" aria-label="Logs">
        <div className="pane-header">
          <h2 className="pane-title">Log output</h2>
        </div>
        <div className="empty-state">
          <p className="empty-state-title">Select a log to view its contents</p>
          <span>Output appears here while the node executions.</span>
        </div>
      </section>
    );
  }

  const isStreaming = execution.status === "pending" || execution.status === "executing";

  return (
    <section className="log-pane" aria-label="Logs">
      <div className="log-header">
        <div className="log-heading">
          <p className="log-label">Log output</p>
          <div className="log-meta">
            <strong>
              {`${execution.id}.log`}
            </strong>
            <StatusBadge status={execution.status} />
            {execution.exit_code !== null && (
              <span className="mono">
                exit
                {execution.exit_code}
              </span>
            )}
          </div>
        </div>

        <div className="log-actions">
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
            onClick={() => setAutoScroll(v => !v)}
          >
            {autoScroll ? "Auto-scroll: On" : "Auto-scroll: Off"}
          </button>
        </div>
      </div>

      <div ref={containerRef} className="log-content">
        {lines.length === 0
          ? (
              <span className="log-placeholder">
                {isStreaming
                  ? "Waiting for log output..."
                  : "No log output available."}
              </span>
            )
          : (
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
