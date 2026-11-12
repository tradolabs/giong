import { createFlow } from "../../../api/requests";
import { buildFlowNodes } from "./configuration";
import type { NodeConfiguration } from "./configuration";
import { FlowNodesEditor } from "./FlowNodesEditor";
import { useEffect, useRef, useState } from "react";
import { UnauthorizedError } from "../../../api/exception";
import type { Flow } from "../../../types/flow";
import "./SecretsDialog.css";
import "./FlowSettingsDialog.css";
import "./RepositorySettingsDialog.css";

interface Props {
  onClose: () => void;
  onSignOut: () => void;
  onCreated: (flow: Flow) => void;
}

export function AddFlowDialog({ onClose, onSignOut, onCreated }: Props) {
 const dialog = useRef<HTMLDialogElement>(null);
 const [name, setName] = useState("");
 const [nodes, setNodes] = useState<NodeConfiguration[]>([]);
 const [pending, setPending] = useState(false);
 const [error, setError] = useState<string | null>(null);
 useEffect(() => { dialog.current?.showModal(); }, []);
 async function save() {
  if (pending) return;
  setPending(true); setError(null);
  try { const saved = await createFlow({name, nodes: buildFlowNodes(nodes)}); onCreated(saved); onClose(); } catch (error) {
   if (error instanceof UnauthorizedError) onSignOut();
   else setError(error instanceof Error ? error.message : "Could not save flow");
  } finally { setPending(false); }
 }
 return <dialog ref={dialog} className="secrets-dialog" onCancel={event => {event.preventDefault(); if (!pending) onClose();}}>
 <header className="secrets-header"><h2>Flow configuration</h2><button disabled={pending} onClick={onClose}>Close</button></header>
 <form className="secrets-content flow-settings-form" onSubmit={event => {event.preventDefault(); void save();}}>
 <label>Name<input required value={name} disabled={pending} onChange={event => setName(event.target.value)} /></label>
 <FlowNodesEditor nodes={nodes} disabled={pending} onChange={setNodes} />
 {error && <p role="alert" className="secrets-error">{error}</p>}
 <button disabled={pending} type="submit">{pending ? "Saving…" : "Save flow"}</button>
 </form></dialog>;
}
