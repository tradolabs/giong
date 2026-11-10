export interface Stack {
 id: string; name: string; configurations: Record<string, unknown>;
 secret_set_id: string | null; inherited_secret_set_ids: string[];
 created_at: string; updated_at: string;
}
