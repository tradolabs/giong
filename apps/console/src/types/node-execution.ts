export type ExecutionStatus
  = "pending" | "executing" | "succeeded" | "failed" | "interrupted";

export interface NodeExecution {
  id: string;
  flow_id: string;
  node_id: string;
  status: ExecutionStatus;
  started_at: string | null;
  finished_at: string | null;
  exit_code: number | null;
  created_at: string;
}
