import type { FlowNode } from "../../../types/flow";
import { newTrigger } from "./configuration";
import type { TriggerDraft } from "./configuration";

interface Props {
  triggers: TriggerDraft[];
  nodes: FlowNode[];
  nodeId?: string;
  disabled: boolean;
  onChange: (triggers: TriggerDraft[]) => void;
}

export function NodeTriggersEditor({ triggers, nodes, nodeId, disabled, onChange }: Props) {
  const sources = nodes.filter(node => node.id !== nodeId);
  function update(id: string, changes: Partial<TriggerDraft>) {
    onChange(triggers.map(trigger => trigger.id === id ? { ...trigger, ...changes } : trigger));
  }
  return (
    <section className="node-triggers" aria-label="Triggers">
      <div className="section-heading">
        <h3>Triggers</h3>
        <button type="button" className="btn-ghost" disabled={disabled} onClick={() => onChange([...triggers, newTrigger()])}>Add trigger</button>
      </div>
      {!triggers.length && <p className="ui-empty">Manual execution</p>}
      {triggers.map((trigger, index) => (
        <fieldset key={trigger.id} className="node-trigger" disabled={disabled}>
          <legend>Trigger {index + 1}</legend>
          <div className="node-trigger-fields">
            <label>
              Type
              <select value={trigger.type} onChange={event => update(trigger.id, { type: event.target.value as TriggerDraft["type"] })}>
                <option value="interval">Interval</option>
                <option value="event" disabled={!sources.length}>Node event</option>
              </select>
            </label>
            {trigger.type === "interval" ? (
              <>
                <label>
                  Every
                  <input type="number" min={1} max={Number.MAX_SAFE_INTEGER} step={1} required value={trigger.interval} onChange={event => update(trigger.id, { interval: event.target.value })} />
                </label>
                <label>
                  Unit
                  <select value={trigger.unit} onChange={event => update(trigger.id, { unit: event.target.value as TriggerDraft["unit"] })}>
                    <option value="seconds">Seconds</option>
                    <option value="minutes">Minutes</option>
                    <option value="hours">Hours</option>
                  </select>
                </label>
              </>
            ) : (
              <label>
                Source node
                <select required value={trigger.source} onChange={event => update(trigger.id, { source: event.target.value })}>
                  <option value="" disabled>Select node</option>
                  {sources.map(node => <option key={node.id ?? node.name} value={node.name}>{node.name}</option>)}
                </select>
              </label>
            )}
          </div>
          <button type="button" className="btn-ghost node-trigger-remove" aria-label={`Remove trigger ${index + 1}`} onClick={() => onChange(triggers.filter(item => item.id !== trigger.id))}>Remove</button>
        </fieldset>
      ))}
    </section>
  );
}
