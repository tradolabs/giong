import type { FlowNode, NodeDefinition, NodeTrigger } from "../../../types/flow";
import { parseDocument, stringify } from "yaml";

export type NodeEditorTab = "general" | "configuration" | "trigger";

export class NodeValidationError extends Error {
  constructor(public section: NodeEditorTab, message: string) {
    super(message);
    this.name = "NodeValidationError";
  }
}

export interface TriggerDraft {
  id: string;
  type: "interval" | "event";
  interval: string;
  unit: "seconds" | "minutes" | "hours";
  source: string;
}

export interface NodeDraft {
  name: string;
  type: string;
  configuration: string;
  triggers: TriggerDraft[];
}

export function newTrigger(trigger?: NodeTrigger): TriggerDraft {
  const seconds = trigger?.type === "interval" ? trigger.seconds : 60;
  const unit = seconds % 3600 === 0 ? "hours" : seconds % 60 === 0 ? "minutes" : "seconds";
  const multiplier = unit === "hours" ? 3600 : unit === "minutes" ? 60 : 1;
  return {
    id: crypto.randomUUID(),
    type: trigger?.type ?? "interval",
    interval: String(seconds / multiplier),
    unit,
    source: trigger?.type === "event" ? trigger.node : "",
  };
}

export function readNodeDraft(node: FlowNode | undefined, nodes: FlowNode[]): NodeDraft {
  let number = nodes.length + 1;
  while (nodes.some(item => item.name === `node-${number}`)) number++;
  return {
    name: node?.name ?? `node-${number}`,
    type: node?.type ?? "",
    configuration: stringify(node?.properties ?? {}),
    triggers: (node?.triggers ?? []).map(newTrigger),
  };
}

export function parseConfiguration(value: string): Record<string, unknown> {
  const document = parseDocument(value, { uniqueKeys: true });
  if (document.errors.length) throw new Error(document.errors[0].message);
  if (document.contents === null) return {};
  const parsed: unknown = JSON.parse(JSON.stringify(
    document.toJS({ maxAliasCount: 100 }),
    (_key, entry: unknown) => {
      if (typeof entry === "number" && !Number.isFinite(entry))
        throw new Error("Configuration numbers must be finite.");
      return entry;
    },
  ));
  if (!parsed || typeof parsed !== "object" || Array.isArray(parsed))
    throw new Error("Properties must be a YAML mapping of keys and values.");
  return parsed as Record<string, unknown>;
}

export function buildNodeDefinition(draft: NodeDraft, nodes: FlowNode[], nodeId?: string): NodeDefinition {
  const name = draft.name.trim();
  if (!name || Array.from(name).some(character => /\p{Cc}/u.test(character)))
    throw new NodeValidationError("general", "Node name is required and cannot contain control characters.");
  if (nodes.some(node => node.id !== nodeId && node.name === name))
    throw new NodeValidationError("general", `Node name "${name}" is already used.`);
  const type = draft.type.trim();
  if (!/^[a-zA-Z0-9_-]+(?:\/[a-zA-Z0-9_-]+)*$/.test(type))
    throw new NodeValidationError("general", "Choose a valid node type.");
  const triggers: NodeTrigger[] = draft.triggers.map((trigger) => {
    if (trigger.type === "interval") {
      const interval = Number(trigger.interval);
      const multiplier = trigger.unit === "hours" ? 3600 : trigger.unit === "minutes" ? 60 : 1;
      const seconds = interval * multiplier;
      if (!trigger.interval.trim() || !Number.isSafeInteger(interval) || interval <= 0)
        throw new NodeValidationError("trigger", "Interval must be a positive integer.");
      if (!Number.isSafeInteger(seconds))
        throw new NodeValidationError("trigger", "Interval is too large.");
      return { type: "interval", seconds };
    }
    const source = nodes.find(node => node.name === trigger.source && node.id !== nodeId);
    if (!source || source.name === name)
      throw new NodeValidationError("trigger", "Choose another node in this flow for the event trigger.");
    return { type: "event", node: source.name };
  });
  try {
    return { name, type, properties: parseConfiguration(draft.configuration), triggers };
  } catch (error) {
    throw new NodeValidationError("configuration", error instanceof Error ? error.message : "Invalid YAML.");
  }
}
