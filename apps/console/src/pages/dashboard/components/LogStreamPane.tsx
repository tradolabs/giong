import "./LogStreamPane.css";
import { useEffect, useRef, useState } from "react";
import { UnauthorizedError } from "../../../api/exception";
import { fetchJobLogs } from "../../../api/requests";
import type { Job } from "../../../types/job";
import type { Stack } from "../../../types/stack";
import { shortSha } from "../../../shared/format";
import { StatusBadge } from "./StatusBadge";

export function LogStreamPane({
  job,
  stack,
  onSignOut,
}: {
  job: Job | null;
  stack: Stack | undefined;
  onSignOut: () => void;
}) {
  const [lines, setLines] = useState<string[]>([]);
  const [autoScroll, setAutoScroll] = useState(true);
  const containerRef = useRef<HTMLDivElement | null>(null);
  const lineCountRef = useRef<number>(0);

  useEffect(() => {
    setLines([]);
    lineCountRef.current = 0;

    if (!job) return;
    const jobId = job.id;
    let cancelled = false;

    async function pollLogs() {
      try {
        const chunk = await fetchJobLogs(jobId, lineCountRef.current);
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
      }
    }

    pollLogs();

    const isStreaming = job.status === "pending" || job.status === "running";
    if (!isStreaming) return () => { cancelled = true; };

    const interval = window.setInterval(pollLogs, 1000);
    return () => {
      cancelled = true;
      window.clearInterval(interval);
    };
  }, [job?.id, job?.status, onSignOut]);

  useEffect(() => {
    if (autoScroll && containerRef.current) {
      containerRef.current.scrollTop = containerRef.current.scrollHeight;
    }
  }, [lines, autoScroll]);

  if (!job) {
    return (
      <section className="log-pane" aria-label="Logs">
        <div className="pane-header">
          <h2 className="pane-title"><span className="pane-index" aria-hidden="true">03</span>Execution logs</h2>
        </div>
        <div className="empty-state">
          <p className="empty-state-title">Select a job to view logs</p>
          <span>Execution output will stream here in real time.</span>
        </div>
      </section>
    );
  }

  const isStreaming = job.status === "pending" || job.status === "running";

  return (
    <section className="log-pane" aria-label="Logs">
      <div className="log-header">
        <div className="log-heading">
          <p className="log-label"><span className="pane-index" aria-hidden="true">03</span>Execution logs</p>
          <div className="log-meta">
            <strong>{stack?.name ?? "Job"}</strong>
            <span className="mono">#{shortSha(job.commit_sha)}</span>
            <StatusBadge status={job.status} />
            {job.exit_code !== null && <span className="mono">exit {job.exit_code}</span>}
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
