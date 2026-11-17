import "./StatusBadge.css";
import type { ExecutionStatus } from "../../../types/node-execution";

const STATUS_LABELS: Record<ExecutionStatus, string> = {
  pending: "Pending",
  executing: "Executing",
  succeeded: "Succeeded",
  failed: "Failed",
  interrupted: "Interrupted",
};

export function StatusBadge({ status }: { status: ExecutionStatus }) {
  return (
    <span className={`status-badge status-${status}`}>
      <span className="status-dot" />
      {STATUS_LABELS[status] ?? status}
    </span>
  );
}
