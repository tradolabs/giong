import { useEffect, useId, useRef, type ReactNode } from "react";

interface Props {
  title: ReactNode;
  subtitle?: ReactNode;
  headerActions?: ReactNode;
  role?: "dialog" | "alertdialog";
  describedBy?: string;
  className?: string;
  closeDisabled?: boolean;
  hideClose?: boolean;
  onClose: () => void;
  children: ReactNode;
}

export function Dialog({
  title,
  subtitle,
  headerActions,
  role = "dialog",
  describedBy,
  className,
  closeDisabled = false,
  hideClose = false,
  onClose,
  children,
}: Props) {
  const dialog = useRef<HTMLDialogElement>(null);
  const titleId = useId();

  useEffect(() => {
    const element = dialog.current;
    element?.showModal();
    return () => element?.close();
  }, []);

  return (
    <dialog
      ref={dialog}
      className={className ? `ui-dialog ${className}` : "ui-dialog"}
      role={role}
      aria-labelledby={titleId}
      aria-describedby={describedBy}
      onCancel={(event) => {
        event.preventDefault();
        if (!closeDisabled) onClose();
      }}
    >
      <header className="ui-dialog-header">
        <div>
          <h2 id={titleId}>{title}</h2>
          {subtitle && <p>{subtitle}</p>}
        </div>
        {(headerActions || !hideClose) && (
          <div className="ui-dialog-header-actions">
            {headerActions}
            {!hideClose && (
              <button
                type="button"
                className="btn-ghost"
                disabled={closeDisabled}
                onClick={onClose}
              >
                Close
              </button>
            )}
          </div>
        )}
      </header>
      {children}
    </dialog>
  );
}
