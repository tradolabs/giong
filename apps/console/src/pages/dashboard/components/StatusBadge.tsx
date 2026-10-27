import "./StatusBadge.css";
import type { JobStatus } from "../../../types/job";

const STATUS_LABELS: Record<JobStatus, string> = {
  pending: "Pending",
  running: "Running",
  succeeded: "Succeeded",
  failed: "Failed",
  interrupted: "Interrupted",
};

export function StatusBadge({ status }: { status: JobStatus }) {
  return (
    <span className={`status-badge status-${status}`}>
      <span className="status-dot" />
      {STATUS_LABELS[status] ?? status}
    </span>
  );
}
