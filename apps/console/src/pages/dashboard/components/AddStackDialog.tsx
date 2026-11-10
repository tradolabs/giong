import { createStack } from "../../../api/requests";
import { parseConfiguration } from "./RunnerConfigurations";
import { useEffect, useRef, useState } from "react";
import { UnauthorizedError } from "../../../api/exception";
import type { Stack } from "../../../types/stack";
import "./SecretsDialog.css";
import "./StackSettingsDialog.css";
import "./RepositorySettingsDialog.css";

interface Props {
  onClose: () => void;
  onSignOut: () => void;
  onCreated: (stack: Stack) => void;
  onManageRepositories: () => void;
  newRepositoryUrl: string | null;
}

export function AddStackDialog({ onClose, onSignOut, onCreated }: Props) {
 const dialog = useRef<HTMLDialogElement>(null);
 const [name, setName] = useState("");
 const [configuration, setConfiguration] = useState('runtime = "kubectl"\n\n[configuration.source]\nrepo_url = "https://github.com/example/gitops"\nrevision = "main"\npath = "helm"\n');
 const [pending, setPending] = useState(false);
 const [error, setError] = useState<string | null>(null);
 useEffect(() => { dialog.current?.showModal(); }, []);
 async function save() {
  if (pending) return;
  setPending(true); setError(null);
  try { const saved = await createStack({name, configurations: parseConfiguration(configuration)}); onCreated(saved); onClose(); } catch (error) {
   if (error instanceof UnauthorizedError) onSignOut();
   else setError(error instanceof Error ? error.message : "Could not save stack");
  } finally { setPending(false); }
 }
 return <dialog ref={dialog} className="secrets-dialog" onCancel={event => {event.preventDefault(); if (!pending) onClose();}}>
 <header className="secrets-header"><h2>Stack configuration</h2><button disabled={pending} onClick={onClose}>Close</button></header>
 <form className="secrets-content stack-settings-form" onSubmit={event => {event.preventDefault(); void save();}}>
 <label>Name<input required value={name} disabled={pending} onChange={event => setName(event.target.value)} /></label>
 <label>Configuration (TOML)<textarea rows={16} value={configuration} disabled={pending} onChange={event => setConfiguration(event.target.value)} /></label>
 {error && <p role="alert" className="secrets-error">{error}</p>}
 <button disabled={pending} type="submit">{pending ? "Saving…" : "Save stack"}</button>
 </form></dialog>;
}
