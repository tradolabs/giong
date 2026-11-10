export interface Stack {
  id: string;
  name: string;
  repo_url: string;
  branch: string;
  folder_path: string;
  inherited_secret_set_ids: string[];
  last_commit: string | null;
  created_at: string;
  updated_at: string;
}
