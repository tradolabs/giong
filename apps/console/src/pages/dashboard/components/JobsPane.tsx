import { useEffect, useRef, useState } from "react";
import { createJob } from "../../../api/requests";
import { UnauthorizedError } from "../../../api/exception";
import type { Job } from "../../../types/job";
import type { Stack } from "../../../types/stack";
import { shortSha } from "../../../shared/format";
import { StatusBadge } from "./StatusBadge";

interface JobsPaneProps {
  stackId: string | null;
  onCreated: (job: Job) => void;
  onSignOut: () => void;
  jobs: Job[];
  stacks: Stack[];
  selectedJobId: string | null;
  onSelectJob: (id: string) => void;
}

export function JobsPane({ stackId, onCreated, onSignOut, jobs, stacks, selectedJobId, onSelectJob }: JobsPaneProps) {
  const [creating, setCreating] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const active = useRef(true);
  useEffect(() => {
    active.current = true;
    return () => { active.current = false; };
  }, []);
  const stackMap = new Map(stacks.map((stack) => [stack.id, stack]));
  const hasActiveJob = jobs.some((job) => job.stack_id === stackId && (job.status === "pending" || job.status === "running"));

  async function handleCreate() {
    if (!stackId || creating || hasActiveJob) return;
    setCreating(true);
    setError(null);
    try {
      const job = await createJob(stackId);
      if (active.current) onCreated(job);
    } catch (err) {
      if (err instanceof UnauthorizedError) onSignOut();
      else if (active.current) setError(err instanceof Error ? err.message : "Could not create job");
    } finally {
      if (active.current) setCreating(false);
    }
  }

  return (
    <section className="pane" aria-label="Jobs">
      <div className="pane-header">
        <h2 className="pane-title"><span className="pane-index" aria-hidden="true">02</span>Jobs</h2>
        <div className="topbar-actions">
          <span className="pane-count">{jobs.length}</span>
          <button type="button" className="btn-ghost" disabled={!stackId || creating || hasActiveJob} onClick={handleCreate}>
            {creating ? "Creating…" : "Create job"}
          </button>
        </div>
      </div>
      {error && <p className="job-error" role="alert">{error}</p>}
      <div className="pane-body">
        {jobs.map((job) => {
          const stack = stackMap.get(job.stack_id);
          return (
            <button
              key={job.id}
              type="button"
              className={`list-item ${selectedJobId === job.id ? "active" : ""}`}
              aria-pressed={selectedJobId === job.id}
              onClick={() => onSelectJob(job.id)}
            >
              <div className="item-row">
                <span className="item-name">
                  {stack?.name ?? job.stack_id.slice(0, 8)}
                </span>
                <StatusBadge status={job.status} />
              </div>
              <div className="item-row item-meta">
                <span className="mono">{shortSha(job.commit_sha)}</span>
                <span>{job.started_at || job.created_at}</span>
              </div>
            </button>
          );
        })}

        {jobs.length === 0 && (
          <div className="empty-state">
            <p className="empty-state-title">No jobs yet</p>
            <span>Jobs will appear when new commits are detected.</span>
          </div>
        )}
      </div>
    </section>
  );
}
