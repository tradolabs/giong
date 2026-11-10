import { SettingsTabs } from "./SettingsTabs";
import { StackSecrets } from "./StackSecrets";
import { useEffect, useRef, useState } from "react";
import { fetchRepositories, updateStack } from "../../../api/requests";
import { UnauthorizedError } from "../../../api/exception";
import type { Repository } from "../../../types/repository";
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
  const [secretsVisited, setSecretsVisited] = useState(initialTab === "secrets");
  const dialog = useRef<HTMLDialogElement>(null);
  const [repoUrl, setRepoUrl] = useState(stack.repo_url);
  const [folder, setFolder] = useState(stack.folder_path);
  const [repositories, setRepositories] = useState<Repository[]>([]);
  const [loading, setLoading] = useState(true);
  const [loadError, setLoadError] = useState<string | null>(null);
  const [pending, setPending] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [saved, setSaved] = useState(false);

  useEffect(() => {
    const element = dialog.current;
    element?.showModal();
    return () => element?.close();
  }, []);

  useEffect(() => {
    let active = true;
    async function load() {
      try {
        const repos = await fetchRepositories();
        if (active) { setRepositories(repos); setLoadError(null); }
      } catch (err) {
        if (!active) return;
        if (err instanceof UnauthorizedError) onSignOut();
        else setLoadError(err instanceof Error ? err.message : "Could not load repositories");
      } finally {
        if (active) setLoading(false);
      }
    }
    void load();
    const timer = window.setInterval(load, 10000);
    return () => { active = false; window.clearInterval(timer); };
  }, [onSignOut]);

  async function save() {
    if (pending || loading || loadError) return;
    setPending(true);
    setError(null);
    setSaved(false);
    try {
      const updated = await updateStack(stack.id, {
        repo_url: repoUrl, folder_path: folder.trim(),
      });
      onSaved(updated);
      setSaved(true);
    } catch (err) {
      if (err instanceof UnauthorizedError) onSignOut();
      else setError(err instanceof Error ? err.message : "Could not save settings");
    } finally {
      setPending(false);
    }
  }

  return (
    <dialog ref={dialog} className="secrets-dialog" aria-labelledby="stack-settings-title"
      onCancel={(event) => { event.preventDefault(); if (!pending) onClose(); }}>
      <header className="secrets-header">
        <div><h2 id="stack-settings-title">Stack settings</h2><p>{stack.name}</p></div>
        <button type="button" className="btn-ghost" disabled={pending} onClick={onClose}>Close</button>
      </header>
      <SettingsTabs id="stack-settings" tabs={[
        { id: "general", label: "General" }, { id: "secrets", label: "Secrets" },
      ]} selected={tab} disabled={pending} onSelect={(next) => {
        setTab(next as "general" | "secrets");
        if (next === "secrets") setSecretsVisited(true);
      }} />
      <div role="tabpanel" id="stack-settings-general-panel" aria-labelledby="stack-settings-general-tab" hidden={tab !== "general"}>
        <form className="secrets-content stack-settings-form" aria-busy={pending} onChange={() => setSaved(false)}
          onSubmit={(event) => { event.preventDefault(); void save(); }}>
          <label>Repository
            <select required value={repoUrl} disabled={pending || loading}
              onChange={(event) => setRepoUrl(event.target.value)}>
              {!repositories.some((repo) => repo.url === repoUrl) && <option value={repoUrl}>{repoUrl} (unavailable)</option>}
              {repositories.map((repo) => <option key={repo.id} value={repo.url}>{repo.name} — {repo.url}</option>)}
            </select>
          </label>
          <label>Stack folder
            <input required value={folder} disabled={pending} placeholder="."
              onChange={(event) => setFolder(event.target.value)} />
          </label>
          <p className="secrets-hint">Define the runtime and runner options in giong.toml inside the stack folder.</p>
          {loading && <p role="status">Loading repositories…</p>}
          {loadError && <p className="secrets-error" role="alert">{loadError}</p>}
          {error && <p className="secrets-error" role="alert">{error}</p>}
          {saved && <p className="secrets-success" role="status">Settings saved.</p>}
          <div className="secrets-actions"><button type="submit" className="btn-ghost" disabled={pending || loading || !!loadError}>{pending ? "Saving…" : "Save settings"}</button></div>
        </form>
      </div>
      <div role="tabpanel" id="stack-settings-secrets-panel" aria-labelledby="stack-settings-secrets-tab" hidden={tab !== "secrets"}>
        {secretsVisited && <StackSecrets stack={stack} onSignOut={onSignOut} onSaved={onSaved} />}
      </div>
    </dialog>
  );
}
