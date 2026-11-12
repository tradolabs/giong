import { useEffect, useId, useRef } from "react";
import "./DeleteFlowDialog.css";

interface Props {
  name: string;
  pending: boolean;
  error: string | null;
  onCancel: () => void;
  onConfirm: () => void;
}
export function DeleteFlowDialog({ name, pending, error, onCancel, onConfirm }: Props) {
  const dialog = useRef<HTMLDialogElement>(null);
  const id = useId();
  useEffect(() => {
    const element = dialog.current;
    element?.showModal();
    return () => element?.close();
  }, []);
  return <dialog ref={dialog} className="secrets-dialog delete-flow-dialog" role="alertdialog"
    aria-labelledby={`${id}-title`} aria-describedby={`${id}-description`}
    onCancel={event => { event.preventDefault(); if (!pending) onCancel(); }}>
    <header className="secrets-header"><h2 id={`${id}-title`}>Delete flow?</h2></header>
    <div className="secrets-content">
      <p id={`${id}-description`}>Delete <strong>{name}</strong>, its nodes, run history, and its own secrets?</p>
      {error && <p className="secrets-error" role="alert">{error}</p>}
      <div className="delete-flow-actions">
        <button type="button" className="btn-ghost" autoFocus disabled={pending} onClick={onCancel}>Cancel</button>
        <button type="button" className="btn-ghost delete-flow-confirm" disabled={pending} onClick={onConfirm}>
          {pending ? "Deleting…" : "Delete flow"}
        </button>
      </div>
    </div>
  </dialog>;
}
