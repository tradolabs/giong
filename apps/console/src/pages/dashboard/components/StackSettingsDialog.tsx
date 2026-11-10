import { SettingsTabs } from "./SettingsTabs";
import { updateStack } from "../../../api/requests";
import { parseConfiguration } from "./RunnerConfigurations";
import { stringify } from "smol-toml";
import { StackSecrets } from "./StackSecrets";
import { useEffect, useRef, useState } from "react";
import { UnauthorizedError } from "../../../api/exception";
import type { Stack } from "../../../types/stack";
import "./SecretsDialog.css";
import "./StackSettingsDialog.css";

interface Props {
  stack: Stack;
  onClose: () => void;
  onSignOut: () => void;
  onSaved: (stack: Stack) => void;
  initialTab?: "general" | "secrets";
}

export function StackSettingsDialog({ stack, onClose, onSignOut, onSaved, initialTab = "general" }: Props) {
 const [tab, setTab] = useState(initialTab);
 const dialog = useRef<HTMLDialogElement>(null);
 const [name, setName] = useState(stack.name);
 const [configuration, setConfiguration] = useState(stringify(stack.configurations));
 const [pending, setPending] = useState(false);
 const [error, setError] = useState<string | null>(null);
 useEffect(() => { dialog.current?.showModal(); }, []);
 async function save() {
  if (pending) return;
  setPending(true); setError(null);
  try { const saved = await updateStack(stack.id, {name, configurations: parseConfiguration(configuration)}); onSaved(saved); } catch (error) {
   if (error instanceof UnauthorizedError) onSignOut();
   else setError(error instanceof Error ? error.message : "Could not save stack");
  } finally { setPending(false); }
 }
 return <dialog ref={dialog} className="secrets-dialog" onCancel={event => {event.preventDefault(); if (!pending) onClose();}}>
 <header className="secrets-header"><h2>Stack configuration</h2><button disabled={pending} onClick={onClose}>Close</button></header>
 <SettingsTabs id="stack-settings" tabs={[{id: "general", label: "General"}, {id: "secrets", label: "Secrets"}]} selected={tab} disabled={pending} onSelect={next => setTab(next as "general" | "secrets")} />
 <div hidden={tab !== "general"}><form className="secrets-content stack-settings-form" onSubmit={event => {event.preventDefault(); void save();}}>
 <label>Name<input required value={name} disabled={pending} onChange={event => setName(event.target.value)} /></label>
 <label>Configuration (TOML)<textarea rows={16} value={configuration} disabled={pending} onChange={event => setConfiguration(event.target.value)} /></label>
 {error && <p role="alert" className="secrets-error">{error}</p>}
 <button disabled={pending} type="submit">{pending ? "Saving…" : "Save stack"}</button>
 </form></div>{tab === "secrets" && <StackSecrets stack={stack} onSignOut={onSignOut} onSaved={onSaved} />}</dialog>;
}
