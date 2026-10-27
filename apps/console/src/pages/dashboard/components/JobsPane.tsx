import type { Job } from "../../../types/job";
import type { Stack } from "../../../types/stack";
import { shortSha } from "../../../shared/format";
import { StatusBadge } from "./StatusBadge";

interface JobsPaneProps {
  jobs: Job[];
  stacks: Stack[];
  selectedJobId: string | null;
  onSelectJob: (id: string) => void;
}

export function JobsPane({ jobs, stacks, selectedJobId, onSelectJob }: JobsPaneProps) {
  const stackMap = new Map(stacks.map((stack) => [stack.id, stack]));

  return (
    <section className="pane" aria-label="Jobs">
      <div className="pane-header">
        <h2 className="pane-title"><span className="pane-index" aria-hidden="true">02</span>Jobs</h2>
        <span className="pane-count">{jobs.length}</span>
      </div>
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
