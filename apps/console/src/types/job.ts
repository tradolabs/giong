export type JobStatus = "pending" | "running" | "succeeded" | "failed" | "interrupted";

export interface Job {
  id: string;
  stack_id: string;
  commit_sha: string;
  status: JobStatus;
  started_at: string;
  finished_at: string | null;
  exit_code: number | null;
  created_at: string;
  updated_at: string;
}
