import type { ExecutionStatus, NodeExecution } from "./node-execution";

export type NodeTrigger
  = | { type: "interval"; seconds: number }
    | { type: "event"; node: string };

export interface NodeDefinition {
  name: string;
  type: string;
  properties: Record<string, unknown>;
  triggers: NodeTrigger[];
}

export interface FlowNode extends NodeDefinition {
  id?: string;
  status: ExecutionStatus | null;
  last_execution: NodeExecution | null;
}

export interface Flow {
  id: string;
  name: string;
  nodes: FlowNode[];
  vault_ids: string[];
  created_at: string;
  updated_at: string;
}
