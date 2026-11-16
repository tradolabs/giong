import { useState } from "react";
import { getApiErrorMessage } from "../../../api/errors";
import { deleteFlow, updateFlow } from "../../../api/requests";
import { Dialog } from "../../../components/ui/Dialog";
import type { Flow } from "../../../types/flow";
import { DeleteFlowDialog } from "./DeleteFlowDialog";
import { FlowSecrets } from "./FlowSecrets";
import { SettingsTabs } from "./SettingsTabs";
import "./FlowSettingsDialog.css";

interface Props {
  flow: Flow;
  onClose: () => void;
  onSignOut: () => void;
  onSaved: (flow: Flow) => void;
  onDeleted: (id: string) => void;
  initialTab?: "general" | "secrets";
}

export function FlowSettingsDialog({
  flow,
  onClose,
  onSignOut,
  onSaved,
  onDeleted,
  initialTab = "general",
}: Props) {
  const [tab, setTab] = useState(initialTab);
  const [name, setName] = useState(flow.name);
  const [confirmDelete, setConfirmDelete] = useState(false);
  const [pending, setPending] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [message, setMessage] = useState<string | null>(null);

  async function remove() {
    if (pending) return;
    setPending(true);
    setError(null);
    try {
      await deleteFlow(flow.id);
      onDeleted(flow.id);
    } catch (err) {
      setError(getApiErrorMessage(err, onSignOut, "Could not delete flow"));
    } finally {
      setPending(false);
    }
  }

  async function save() {
    if (pending) return;
    setPending(true);
    setError(null);
    setMessage(null);
    try {
      const saved = await updateFlow(flow.id, {
        name: name.trim(),
        inherited_vault_ids: flow.inherited_vault_ids,
      });
      onSaved(saved);
      setName(saved.name);
      setMessage("Saved.");
    } catch (err) {
      setError(getApiErrorMessage(err, onSignOut, "Could not save flow"));
    } finally {
      setPending(false);
    }
  }

  return (
    <>
      <Dialog
        title="Flow settings"
        className="tabbed-settings-dialog"
        closeDisabled={pending}
        onClose={onClose}
      >
        <SettingsTabs
          id="flow-settings"
          tabs={[
            { id: "general", label: "General" },
            { id: "secrets", label: "Secrets" },
          ]}
          selected={tab}
          disabled={pending}
          onSelect={next => setTab(next as "general" | "secrets")}
        />
        <div className="flow-settings-general" hidden={tab !== "general"}>
          <form
            className="flow-general-form"
            onSubmit={(event) => { event.preventDefault(); void save(); }}
          >
            <div className="flow-general-content">
              <label>
                Flow name
                <input
                  required
                  value={name}
                  disabled={pending}
                  onChange={(event) => {
                    setName(event.target.value);
                    setError(null);
                    setMessage(null);
                  }}
                />
              </label>
              <dl className="flow-general-overview">
                <div><dt>Nodes</dt><dd>{flow.nodes.length}</dd></div>
                <div><dt>Inherited vaults</dt><dd>{flow.inherited_vault_ids.length}</dd></div>
              </dl>
              {error && !confirmDelete && <p role="alert" className="ui-error">{error}</p>}
              {message && <p role="status" className="ui-success">{message}</p>}
            </div>
            <footer className="flow-general-footer">
              <button
                type="button"
                className="btn-danger"
                disabled={pending}
                onClick={() => {
                  setError(null);
                  setMessage(null);
                  setConfirmDelete(true);
                }}
              >Delete flow</button>
              <div className="ui-form-actions">
                <button type="button" className="btn-ghost" disabled={pending} onClick={onClose}>Cancel</button>
                <button className="btn-primary" disabled={pending || !name.trim() || name.trim() === flow.name} type="submit">
                  {pending ? "Saving…" : "Save changes"}
                </button>
              </div>
            </footer>
          </form>
        </div>
        {tab === "secrets" && (
          <FlowSecrets flow={flow} onSignOut={onSignOut} onSaved={onSaved} />
        )}
      </Dialog>
      {confirmDelete && (
        <DeleteFlowDialog
          name={flow.name}
          pending={pending}
          error={error}
          onCancel={() => {
            setConfirmDelete(false);
            setError(null);
          }}
          onConfirm={() => {
            void remove();
          }}
        />
      )}
    </>
  );
}
