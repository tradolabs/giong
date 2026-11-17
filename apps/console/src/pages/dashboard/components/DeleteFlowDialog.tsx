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

export function DeleteFlowDialog({
  name,
  pending,
  error,
  onCancel,
  onConfirm,
}: Props) {
  const descriptionId = useId();

  return (
    <Dialog
      title="Delete flow?"
      role="alertdialog"
      describedBy={descriptionId}
      className="delete-flow-dialog"
      closeDisabled={pending}
      hideClose
      onClose={onCancel}
    >
      <div className="ui-dialog-content">
        <p id={descriptionId}>
          Delete
          {" "}
          <strong>{name}</strong>
          , its nodes, execution history, and its own
          secrets?
        </p>
        {error && (
          <p className="ui-error" role="alert">
            {error}
          </p>
        )}
        <div className="delete-flow-actions">
          <button
            type="button"
            className="btn-ghost"
            autoFocus
            disabled={pending}
            onClick={onCancel}
          >
            Cancel
          </button>
          <button
            type="button"
            className="btn-danger"
            disabled={pending}
            onClick={onConfirm}
          >
            {pending ? "Deleting…" : "Delete flow"}
          </button>
        </div>
      </div>
    </Dialog>
  );
}
