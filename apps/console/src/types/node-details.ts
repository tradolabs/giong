import type { NodeExecution } from "./node-execution";

export interface NodeDetails {
  output: Record<string, unknown> | null;
  executions: NodeExecution[];
}
