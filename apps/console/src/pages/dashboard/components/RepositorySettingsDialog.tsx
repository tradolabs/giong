import { SecretSets } from "./SecretSets";
import { SettingsTabs } from "./SettingsTabs";
import { useEffect, useRef, useState } from "react";
import { createRepository, fetchRepositories } from "../../../api/requests";
import { UnauthorizedError } from "../../../api/exception";
import type { Repository } from "../../../types/repository";
import "./SecretsDialog.css";
import "./StackSettingsDialog.css";
import "./RepositorySettingsDialog.css";

interface Props {
  onClose: () => void;
  onSignOut: () => void;
  onAdded: (repository: Repository) => void;
}

export function RepositorySettingsDialog({ onClose, onSignOut, onAdded }: Props) {
  const [tab, setTab] = useState("repository");
  const [secretsVisited, setSecretsVisited] = useState(false);
  const dialog = useRef<HTMLDialogElement>(null);
  const submitting = useRef(false);
  const [repositories, setRepositories] = useState<Repository[]>([]);
  const [name, setName] = useState("");
  const [url, setUrl] = useState("");
  const [token, setToken] = useState("");
  const [pending, setPending] = useState(false);
  const [loading, setLoading] = useState(true);
  const [revision, setRevision] = useState(0);
  const [loadError, setLoadError] = useState<string | null>(null);
  const [error, setError] = useState<string | null>(null);
  const [message, setMessage] = useState<string | null>(null);

  useEffect(() => {
    const element = dialog.current;
    element?.showModal();
    return () => element?.close();
  }, []);

  useEffect(() => {
    let active = true;
    setLoading(true);
    setLoadError(null);
    fetchRepositories().then((data) => { if (active) setRepositories(data); })
      .catch((err: unknown) => {
        if (!active) return;
        if (err instanceof UnauthorizedError) onSignOut();
        else setLoadError(err instanceof Error ? err.message : "Could not load repositories");
      }).finally(() => { if (active) setLoading(false); });
    return () => { active = false; };
  }, [onSignOut, revision]);

  async function add() {
    if (submitting.current) return;
    submitting.current = true;
    setPending(true);
    setError(null);
    setMessage(null);
    try {
      const repository = await createRepository(name.trim(), url.trim(), token || undefined);
      setRepositories((current) => [...current.filter((item) => item.id !== repository.id), repository]
        .sort((a, b) => a.name.localeCompare(b.name)));
      setName(""); setUrl(""); setToken("");
      setMessage(`Added ${repository.name}.`);
      onAdded(repository);
    } catch (err) {
      if (err instanceof UnauthorizedError) onSignOut();
      else setError(err instanceof Error ? err.message : "Could not add repository");
    } finally {
      submitting.current = false;
      setPending(false);
    }
  }

  return (
    <dialog ref={dialog} className="secrets-dialog" aria-labelledby="repository-settings-title"
      onCancel={(event) => { event.preventDefault(); if (!submitting.current) onClose(); }}>
      <header className="secrets-header">
        <div><h2 id="repository-settings-title">Settings</h2></div>
        <button type="button" className="btn-ghost" disabled={pending} onClick={onClose}>Close</button>
      </header>
      <SettingsTabs id="global-settings" tabs={[
        { id: "repository", label: "Repository" }, { id: "secret-sets", label: "Secret sets" },
      ]} selected={tab} disabled={pending} onSelect={(next) => {
        setTab(next); if (next === "secret-sets") setSecretsVisited(true);
      }} />
      <div className="secrets-content" role="tabpanel" id="global-settings-repository-panel"
        aria-labelledby="global-settings-repository-tab" hidden={tab !== "repository"}>
        <section aria-label="Saved repositories" className="repository-list">
          {loading && <p role="status">Loading repositories…</p>}
          {loadError && <div role="alert"><p className="secrets-error">{loadError}</p>
            <button type="button" className="btn-ghost" onClick={() => setRevision((value) => value + 1)}>Retry</button></div>}
          {!loading && !loadError && repositories.length === 0 && <p className="secrets-hint">Add your first repository to start creating stacks.</p>}
          {repositories.map((repository) => <div key={repository.id} className="repository-item">
            <strong>{repository.name}</strong><span className="mono">{repository.url}</span>
            {repository.has_credentials && <small>Access token configured</small>}
          </div>)}
        </section>
        <form className="stack-settings-form" aria-label="Add repository" aria-busy={pending}
          onSubmit={(event) => { event.preventDefault(); void add(); }}>
          <h3 className="repository-form-title">Add repository</h3>
          <label>Name<input required value={name} disabled={pending}
            onChange={(event) => setName(event.target.value)} placeholder="Infrastructure" /></label>
          <label>Git URL<input type="url" required value={url} disabled={pending} spellCheck={false}
            onChange={(event) => setUrl(event.target.value)} placeholder="https://github.com/organization/repository.git" /></label>
          <label>Access token (optional)<input type="password" value={token} disabled={pending} autoComplete="new-password"
            onChange={(event) => setToken(event.target.value)} placeholder="For private HTTPS repositories" /></label>
          <p className="secrets-hint">Tokens are stored encrypted and never shown again. Leave blank for a public repository.</p>
          {error && <p className="secrets-error" role="alert">{error}</p>}
          {message && <p className="secrets-success" role="status">{message}</p>}
          <div className="secrets-actions"><button type="submit" className="btn-ghost" disabled={pending || loading}>
            {pending ? "Adding…" : "Add repository"}</button></div>
        </form>
      </div>
      <div role="tabpanel" id="global-settings-secret-sets-panel" aria-labelledby="global-settings-secret-sets-tab"
        hidden={tab !== "secret-sets"}>
        {secretsVisited && <SecretSets onSignOut={onSignOut} />}
      </div>
    </dialog>
  );
}
