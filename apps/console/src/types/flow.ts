export type NodeTrigger = { type: "interval"; seconds: number } | { type: "event"; node: string; event: string };
export interface FlowNode {
  id?: string;
  name: string;
  type: string;
  properties: Record<string, unknown>;
  triggers: NodeTrigger[];
}
export interface Flow {
  id: string;
  name: string;
  nodes: FlowNode[];
  vault_id: string | null;
  inherited_vault_ids: string[];
  created_at: string;
  updated_at: string;
}
