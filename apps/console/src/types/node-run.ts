export type RunStatus = "pending" | "running" | "succeeded" | "failed" | "interrupted";
export interface NodeRun {
  log_timestamp?: string | null;
  id: string; flow_id: string; node_id: string; status: RunStatus;
  started_at: string | null; finished_at: string | null; exit_code: number | null; created_at: string;
}
