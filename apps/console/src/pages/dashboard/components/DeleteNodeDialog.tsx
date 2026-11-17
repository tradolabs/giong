import { useId } from "react";
import { Dialog } from "../../../components/ui/Dialog";
import "./DeleteFlowDialog.css";

interface Props {
  name: string;
  pending: boolean;
  error: string | null;
  onCancel: () => void;
  onConfirm: () => void;
}

export function DeleteNodeDialog({ name, pending, error, onCancel, onConfirm }: Props) {
  const descriptionId = useId();
  return (
    <Dialog title="Delete node?" role="alertdialog" describedBy={descriptionId} className="delete-node-dialog" closeDisabled={pending} hideClose onClose={onCancel}>
      <div className="ui-dialog-content">
        <p id={descriptionId}>Delete <strong>{name}</strong>, its stored data, logs, and execution history? Triggers referencing this node will also be removed.</p>
        {error && <p className="ui-error" role="alert">{error}</p>}
        <div className="delete-flow-actions">
          <button type="button" className="btn-ghost" autoFocus disabled={pending} onClick={onCancel}>Cancel</button>
          <button type="button" className="btn-danger" disabled={pending} onClick={onConfirm}>{pending ? "Deleting…" : "Delete node"}</button>
        </div>
      </div>
    </Dialog>
  );
}
