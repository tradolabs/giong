export interface Stack {
  id: string;
  name: string;
  repo_url: string;
  branch: string;
  folder_path: string;
  last_commit: string | null;
  created_at: string;
  updated_at: string;
}
