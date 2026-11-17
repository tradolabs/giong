import { useId, useRef, useState } from "react";
import { getApiErrorMessage } from "../../../api/errors";
import { updateFlowNode, createFlowNode, deleteFlowNode } from "../../../api/requests";
import { Dialog } from "../../../components/ui/Dialog";
import type { Flow, FlowNode } from "../../../types/flow";
import { buildNodeDefinition, readNodeDraft, NodeValidationError } from "./configuration";
import type { NodeEditorTab } from "./configuration";
import { SettingsTabs } from "./SettingsTabs";
import { NodeTriggersEditor } from "./NodeTriggersEditor";
import { DeleteNodeDialog } from "./DeleteNodeDialog";
import { NodeTypePicker } from "./NodeTypePicker";
import "./FlowSettingsDialog.css";

interface Props {
  flow: Flow;
  node?: FlowNode;
  nodeKinds: string[];
  nodeKindsLoading: boolean;
  nodeKindsError: string | null;
  onClose: () => void;
  onSaved: (node: FlowNode) => void;
  onDeleted: (nodeId: string) => void;
  onSignOut: () => void;
}

export function NodeDialog({ flow, node, nodeKinds, nodeKindsLoading, nodeKindsError, onClose, onSaved, onDeleted, onSignOut }: Props) {
  const tabsId = useId();
  const [tab, setTab] = useState<NodeEditorTab>("general");
  const saving = useRef(false);
  const [draft, setDraft] = useState(() => readNodeDraft(node, flow.nodes));
  const [pending, setPending] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [confirmDelete, setConfirmDelete] = useState(false);

  async function remove() {
    if (!node?.id || saving.current) return;
    saving.current = true;
    setPending(true);
    setError(null);
    try {
      await deleteFlowNode(flow.id, node.id);
      onDeleted(node.id);
      onClose();
    } catch (err) {
      setError(getApiErrorMessage(err, onSignOut, "Could not delete node"));
    } finally {
      saving.current = false;
      setPending(false);
    }
  }

  async function save() {
    if (saving.current) return;
    saving.current = true;
    setPending(true);
    setError(null);
    try {
      const definition = buildNodeDefinition(draft, flow.nodes, node?.id);
      let saved: FlowNode;
      if (node) {
        if (!node.id) throw new Error("Node ID is missing. Reopen the flow.");
        saved = await updateFlowNode(flow.id, node.id, definition);
      } else {
        saved = await createFlowNode(flow.id, definition);
      }
      onSaved(saved);
      onClose();
    } catch (err) {
      if (err instanceof NodeValidationError) setTab(err.section);
      setError(getApiErrorMessage(err, onSignOut, "Could not save node"));
    } finally {
      saving.current = false;
      setPending(false);
    }
  }

  return (
    <>
    <Dialog className="tabbed-settings-dialog" title={node ? `Edit node: ${node.name}` : "Add node"} closeDisabled={pending} onClose={onClose}>
      <SettingsTabs
        id={tabsId}
        tabs={[{ id: "general", label: "General" }, { id: "configuration", label: "Configuration" }, { id: "trigger", label: "Trigger" }]}
        selected={tab}
        disabled={pending}
        onSelect={value => setTab(value as NodeEditorTab)}
      />
      <form noValidate className="node-dialog-form" onSubmit={(event) => { event.preventDefault(); void save(); }}>
        <fieldset className="node-dialog-fields" disabled={pending}>
          <section hidden={tab !== "general"} role="tabpanel" id={`${tabsId}-general-panel`} aria-labelledby={`${tabsId}-general-tab`}>
          <div className="flow-node-fields">
            <label>
              Name
              <input autoFocus required value={draft.name} onChange={event => setDraft({ ...draft, name: event.target.value })} />
            </label>
            <NodeTypePicker
              kinds={nodeKinds}
              value={draft.type}
              loading={nodeKindsLoading}
              error={nodeKindsError}
              onChange={type => setDraft({ ...draft, type })}
            />
          </div>
          </section>
          <section hidden={tab !== "configuration"} role="tabpanel" id={`${tabsId}-configuration-panel`} aria-labelledby={`${tabsId}-configuration-tab`}>
          <label>
            Configuration (YAML)
            <textarea className="node-properties" rows={8} spellCheck={false} value={draft.configuration} onChange={event => setDraft({ ...draft, configuration: event.target.value })} />
          </label>
          </section>
          <section hidden={tab !== "trigger"} role="tabpanel" id={`${tabsId}-trigger-panel`} aria-labelledby={`${tabsId}-trigger-tab`}>
          <NodeTriggersEditor triggers={draft.triggers} nodes={flow.nodes} nodeId={node?.id} disabled={pending} onChange={triggers => setDraft({ ...draft, triggers })} />
          </section>
        </fieldset>
        {error && !confirmDelete && <p className="ui-error" role="alert">{error}</p>}
        <div className="ui-form-actions">
          {node?.id && tab === "general" && <button type="button" className="btn-danger node-delete-button" disabled={pending} onClick={() => { setError(null); setConfirmDelete(true); }}>Delete node</button>}
          <button type="button" className="btn-ghost" disabled={pending} onClick={onClose}>Cancel</button>
          <button type="submit" className="btn-primary" disabled={pending}>{pending ? "Saving…" : node ? "Save node" : "Add node"}</button>
        </div>
      </form>
    </Dialog>
    {confirmDelete && node && <DeleteNodeDialog name={node.name} pending={pending} error={error} onCancel={() => { setConfirmDelete(false); setError(null); }} onConfirm={() => { void remove(); }} />}
    </>
  );
}
