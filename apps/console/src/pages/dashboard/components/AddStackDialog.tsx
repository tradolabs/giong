import { useEffect, useRef, useState } from "react";
import { createStack, fetchRepositories } from "../../../api/requests";
import { UnauthorizedError } from "../../../api/exception";
import type { Repository } from "../../../types/repository";
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

export function AddStackDialog({ onClose, onSignOut, onCreated, onManageRepositories, newRepositoryUrl }: Props) {
  const dialog = useRef<HTMLDialogElement>(null);
  const submitting = useRef(false);
  const [repositories, setRepositories] = useState<Repository[]>([]);
  const [name, setName] = useState("");
  const [repoUrl, setRepoUrl] = useState("");
  const [branch, setBranch] = useState("main");
  const [folder, setFolder] = useState(".");
  const [loadingRepos, setLoadingRepos] = useState(true);
  const [repoError, setRepoError] = useState<string | null>(null);
  const [revision, setRevision] = useState(0);
  const [pending, setPending] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const canSave = !pending && !loadingRepos && !repoError && !!repoUrl;

  useEffect(() => {
    const element = dialog.current;
    element?.showModal();
    return () => element?.close();
  }, []);

  useEffect(() => {
    let active = true;
    setLoadingRepos(true); setRepoError(null);
    fetchRepositories().then((data) => {
      if (!active) return;
      setRepositories(data);
      setRepoUrl((current) => newRepositoryUrl && data.some((repo) => repo.url === newRepositoryUrl)
        ? newRepositoryUrl : data.some((repo) => repo.url === current) ? current : data[0]?.url ?? "");
    }).catch((err: unknown) => {
      if (!active) return;
      if (err instanceof UnauthorizedError) onSignOut();
      else setRepoError(err instanceof Error ? err.message : "Could not load repositories");
    }).finally(() => { if (active) setLoadingRepos(false); });
    return () => { active = false; };
  }, [newRepositoryUrl, onSignOut, revision]);

  async function save() {
    if (!canSave || submitting.current) return;
    submitting.current = true; setPending(true); setError(null);
    try {
      const stack = await createStack({ name: name.trim(), repo_url: repoUrl, branch: branch.trim(),
        folder_path: folder.trim() });
      onCreated(stack);
      onClose();
    } catch (err) {
      if (err instanceof UnauthorizedError) onSignOut();
      else setError(err instanceof Error ? err.message : "Could not add stack");
    } finally {
      submitting.current = false; setPending(false);
    }
  }

  return (
    <dialog ref={dialog} className="secrets-dialog" aria-labelledby="add-stack-title"
      onCancel={(event) => { event.preventDefault(); if (!submitting.current) onClose(); }}>
      <header className="secrets-header"><h2 id="add-stack-title">Add stack</h2>
        <button type="button" className="btn-ghost" disabled={pending} onClick={onClose}>Close</button></header>
      <form className="secrets-content stack-settings-form" aria-busy={pending}
        onSubmit={(event) => { event.preventDefault(); void save(); }}>
        <label>Name<input required autoFocus value={name} disabled={pending} placeholder="my-app"
          onChange={(event) => setName(event.target.value)} /></label>
        <label>Repository<select required value={repoUrl} disabled={pending || loadingRepos}
          onChange={(event) => setRepoUrl(event.target.value)}>
          <option value="" disabled>Select a repository</option>
          {repositories.map((repo) => <option key={repo.id} value={repo.url}>{repo.name} — {repo.url}</option>)}
        </select></label>
        {!loadingRepos && !repoError && repositories.length === 0 && <p className="secrets-hint">Add a repository before creating your first stack.</p>}
        <button type="button" className="btn-ghost" disabled={pending} onClick={onManageRepositories}>Add repository in Settings</button>
        <div className="stack-create-fields">
          <label>Branch<input required value={branch} disabled={pending} onChange={(event) => setBranch(event.target.value)} /></label>
          <label>Stack folder<input required value={folder} disabled={pending} onChange={(event) => setFolder(event.target.value)} /></label>
        </div>
        <p className="secrets-hint">Define the runtime and runner options in giong.toml inside the stack folder.</p>
        <p className="secrets-hint">You will configure runner secrets next.</p>
        {loadingRepos && <p role="status">Loading repositories…</p>}
        {repoError && <div role="alert"><p className="secrets-error">{repoError}</p>
          <button type="button" className="btn-ghost" onClick={() => setRevision((value) => value + 1)}>Retry</button></div>}
        {error && <p className="secrets-error" role="alert">{error}</p>}
        <div className="secrets-actions"><button type="submit" className="btn-ghost" disabled={!canSave}>{pending ? "Creating…" : "Create stack"}</button></div>
      </form>
    </dialog>
  );
}
