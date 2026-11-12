import { SettingsTabs } from "./SettingsTabs";
import { deleteFlow, updateFlow } from "../../../api/requests";
import { buildFlowNodes, readNodeConfigurations } from "./configuration";
import { FlowNodesEditor } from "./FlowNodesEditor";
import { DeleteFlowDialog } from "./DeleteFlowDialog";
import { FlowSecrets } from "./FlowSecrets";
import { useEffect, useRef, useState } from "react";
import { UnauthorizedError } from "../../../api/exception";
import type { Flow } from "../../../types/flow";
import "./SecretsDialog.css";
import "./FlowSettingsDialog.css";

interface Props {
  flow: Flow;
  onClose: () => void;
  onSignOut: () => void;
  onSaved: (flow: Flow) => void;
  onDeleted: (id: string) => void;
  initialTab?: "general" | "secrets";
}

export function FlowSettingsDialog({ flow, onClose, onSignOut, onSaved, onDeleted, initialTab = "general" }: Props) {
 const [tab, setTab] = useState(initialTab);
 const dialog = useRef<HTMLDialogElement>(null);
 const [name, setName] = useState(flow.name);
 const [nodes, setNodes] = useState(() => readNodeConfigurations(flow.nodes));
 const [confirmDelete, setConfirmDelete] = useState(false);
 const [pending, setPending] = useState(false);
 const [error, setError] = useState<string | null>(null);
 useEffect(() => { dialog.current?.showModal(); }, []);
 async function remove() {
  if (pending) return;
  setPending(true); setError(null);
  try { await deleteFlow(flow.id); onDeleted(flow.id); }
  catch (error) {
   if (error instanceof UnauthorizedError) onSignOut();
   else setError(error instanceof Error ? error.message : "Could not delete flow");
  } finally { setPending(false); }
 }
 async function save() {
  if (pending) return;
  setPending(true); setError(null);
  try { const saved = await updateFlow(flow.id, {name, nodes: buildFlowNodes(nodes), inherited_vault_ids: flow.inherited_vault_ids}); onSaved(saved); } catch (error) {
   if (error instanceof UnauthorizedError) onSignOut();
   else setError(error instanceof Error ? error.message : "Could not save flow");
  } finally { setPending(false); }
 }
 return <><dialog ref={dialog} className="secrets-dialog" onCancel={event => {event.preventDefault(); if (!pending) onClose();}}>
 <header className="secrets-header"><h2>Flow configuration</h2><button disabled={pending} onClick={onClose}>Close</button></header>
 <SettingsTabs id="flow-settings" tabs={[{id: "general", label: "General"}, {id: "secrets", label: "Secrets"}]} selected={tab} disabled={pending} onSelect={next => setTab(next as "general" | "secrets")} />
 <div hidden={tab !== "general"}><form className="secrets-content flow-settings-form" onSubmit={event => {event.preventDefault(); void save();}}>
 <label>Name<input required value={name} disabled={pending} onChange={event => setName(event.target.value)} /></label>
 <FlowNodesEditor nodes={nodes} disabled={pending} onChange={setNodes} />
 {error && !confirmDelete && <p role="alert" className="secrets-error">{error}</p>}
 <button disabled={pending} type="submit">{pending ? "Saving…" : "Save flow"}</button>
 </form>
 <section className="secrets-content flow-delete-section">
  <button type="button" className="btn-ghost" disabled={pending} onClick={() => { setError(null); setConfirmDelete(true); }}>Delete flow</button>
 </section></div>{tab === "secrets" && <FlowSecrets flow={flow} onSignOut={onSignOut} onSaved={onSaved} />}</dialog>
 {confirmDelete && <DeleteFlowDialog name={flow.name} pending={pending} error={error}
   onCancel={() => { setConfirmDelete(false); setError(null); }} onConfirm={() => { void remove(); }} />}
 </>;
}
