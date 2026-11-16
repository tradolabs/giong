import { useEffect, useState } from "react";
import { getApiErrorMessage } from "../../../api/errors";
import { fetchVaultSecrets } from "../../../api/requests";
import type { Secret } from "../../../types/secret";

interface Props {
  vaultId: string;
  name: string;
  onSignOut: () => void;
}

export function InheritedVaultSecrets({ vaultId, name, onSignOut }: Props) {
  const [secrets, setSecrets] = useState<Secret[]>([]);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);

  useEffect(() => {
    let active = true;
    setLoading(true);
    setError(null);
    fetchVaultSecrets(vaultId)
      .then(data => {
        if (active) setSecrets(data);
      })
      .catch((err: unknown) => {
        if (active) setError(getApiErrorMessage(err, onSignOut, "Could not load inherited secrets"));
      })
      .finally(() => {
        if (active) setLoading(false);
      });
    return () => { active = false; };
  }, [vaultId, onSignOut]);

  return (
    <section className="inherited-vault-secrets" aria-label={`Secrets inherited from ${name}`} aria-busy={loading}>
      <div className="section-heading">
        <h3>{name}</h3>
        <span className="inherited-vault-readonly">Read only</span>
      </div>
      {loading && <p role="status">Loading keys…</p>}
      {error && <p className="ui-error" role="alert">{error}</p>}
      {!loading && !error && (
        secrets.length ? (
          <ul className="secret-list">
            {secrets.map(secret => <li key={secret.id}><code>{secret.key}</code></li>)}
          </ul>
        ) : <p className="ui-empty">No secrets yet.</p>
      )}
    </section>
  );
}
