import { useId } from "react";
import { newNodeConfiguration } from "./configuration";
import type { NodeConfiguration } from "./configuration";
interface Props { nodes: NodeConfiguration[]; disabled: boolean; onChange: (nodes: NodeConfiguration[]) => void }
export function FlowNodesEditor({ nodes, disabled, onChange }: Props) {
  const typesId = useId();
  function update(id: string, changes: Partial<NodeConfiguration>) {
    onChange(nodes.map(node => node.id === id ? { ...node, ...changes } : node));
  }
  function add() {
    let number = nodes.length + 1;
    while (nodes.some(node => node.name.trim() === `node-${number}`)) number++;
    onChange([...nodes, newNodeConfiguration(`node-${number}`)]);
  }
  return <section className="flow-nodes-editor" aria-label="Flow nodes">
    <div className="flow-nodes-heading"><h3>Nodes</h3>
      <button type="button" className="btn-ghost" disabled={disabled} onClick={add}>Add node</button>
    </div>
    <p className="secrets-hint">Each node runs independently. Use event triggers to run a node when another node changes.</p>
    <datalist id={typesId}><option value="sources/repository" /><option value="runners/kubectl" /></datalist>
    {!nodes.length && <p className="secrets-hint">No nodes yet. You can save this flow and add nodes later.</p>}
    {nodes.map((node, index) => {
      const title = node.name.trim() || `Node ${index + 1}`;
      return <fieldset key={node.id} className="flow-node-configuration" disabled={disabled}>
        <legend>{index + 1}. {title}</legend>
        <div className="flow-node-fields">
          <label>Name<input required value={node.name} onChange={event => update(node.id, { name: event.target.value })} /></label>
          <label>Type<input required list={typesId} value={node.type} placeholder="sources/repository"
            onChange={event => update(node.id, { type: event.target.value })} /></label>
        </div>
        <label>Properties (YAML)<textarea rows={6} spellCheck={false} value={node.configuration}
          placeholder="key: value" onChange={event => update(node.id, { configuration: event.target.value })} /></label>
        <label>Triggers (YAML)<textarea rows={5} spellCheck={false} value={node.triggerConfiguration}
          placeholder={'- type: interval\n  seconds: 60'} onChange={event => update(node.id, { triggerConfiguration: event.target.value })} /></label>
        <details className="configuration-reference"><summary>Run when another node changes</summary>
          <pre>{'- type: event\n  node: repository\n  event: changed'}</pre>
        </details>
        <div className="flow-node-actions">
          <button type="button" className="btn-ghost" aria-label={`Remove ${title}`} onClick={() => onChange(nodes.filter(item => item.id !== node.id))}>Remove</button>
        </div>
      </fieldset>;
    })}
  </section>;
}
