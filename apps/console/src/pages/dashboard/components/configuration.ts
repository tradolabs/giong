import type { FlowNode, NodeTrigger } from "../../../types/flow";
import { parseDocument, stringify } from "yaml";

function parseYaml(value: string, empty: unknown): unknown {
  const document = parseDocument(value, { uniqueKeys: true });
  if (document.errors.length) throw new Error(document.errors[0].message);
  if (document.contents === null) return empty;
  return JSON.parse(JSON.stringify(document.toJS({ maxAliasCount: 100 }), (_key, entry: unknown) => {
    if (typeof entry === "number" && !Number.isFinite(entry)) {
      throw new Error("Configuration numbers must be finite.");
    }
    return entry;
  })) as unknown;
}
export function parseConfiguration(value: string): Record<string, unknown> {
  const parsed = parseYaml(value, {});
  if (!object(parsed)) throw new Error("Properties must be a YAML mapping of keys and values.");
  return parsed;
}

export interface NodeConfiguration {
  id: string;
  name: string;
  type: string;
  configuration: string;
  nodeId?: string;
  triggerConfiguration: string;
}
let nextNodeId = 0;
export function newNodeConfiguration(name: string): NodeConfiguration {
  return { id: `node-editor-${++nextNodeId}`, name, type: "", configuration: "", triggerConfiguration: "" };
}
function object(value: unknown): value is Record<string, unknown> {
  return !!value && typeof value === "object" && !Array.isArray(value);
}
export function readNodeConfigurations(nodes: FlowNode[]): NodeConfiguration[] {
  return nodes.map((value: unknown) => {
    const node = object(value) ? value : {};
    return {
      ...newNodeConfiguration(typeof node.name === "string" ? node.name : ""),
      type: typeof node.type === "string" ? node.type : "",
      configuration: stringify(object(node.properties) ? node.properties : {}),
      nodeId: typeof node.id === "string" ? node.id : undefined,
      triggerConfiguration: Array.isArray(node.triggers) && node.triggers.length ? stringify(node.triggers) : "",
    };
  });
}
export function buildFlowNodes(drafts: NodeConfiguration[]): FlowNode[] {
  const names = new Set<string>();
  const nodes = drafts.map((draft, index) => {
    const name = draft.name.trim();
    const type = draft.type.trim();
    if (!name || /[\u0000-\u001f\u007f]/.test(name)) throw new Error(`Node ${index + 1} needs a valid name.`);
    if (names.has(name)) throw new Error(`Node name "${name}" is already used.`);
    names.add(name);
    if (!/^[a-zA-Z0-9_-]+(?:\/[a-zA-Z0-9_-]+)*$/.test(type)) throw new Error(`Choose a valid type for "${name}".`);
    try {
      const triggers = parseYaml(draft.triggerConfiguration, []);
      if (!Array.isArray(triggers)) throw new Error("Triggers must be a YAML list.");
      for (const trigger of triggers) {
        if (!object(trigger) || (trigger.type !== "interval" && trigger.type !== "event")) throw new Error("Trigger type must be interval or event.");
        if (trigger.type === "interval" && (!Number.isSafeInteger(trigger.seconds) || Number(trigger.seconds) <= 0)) throw new Error("Interval seconds must be a positive integer.");
        if (trigger.type === "event" && (typeof trigger.node !== "string" || !trigger.node.trim() || typeof trigger.event !== "string" || !trigger.event.trim())) throw new Error("Event triggers need a node name and an event name.");
      }
      return { ...(draft.nodeId ? { id: draft.nodeId } : {}), name, type, properties: parseConfiguration(draft.configuration), triggers: triggers as NodeTrigger[] };
    }
    catch (error) { throw new Error(`Configuration for "${name}": ${error instanceof Error ? error.message : "Invalid YAML"}`); }
  });
  return nodes;
}
