import "./SignInPage.css";
import { type SubmitEvent, useState } from "react";
import { signIn } from "../../api/requests";

export function SignInPage({ onSuccess }: { onSuccess: () => void }) {
  const [username, setUsername] = useState("admin");
  const [password, setPassword] = useState("");
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);

  async function handleSubmit(event: SubmitEvent<HTMLFormElement>) {
    event.preventDefault();
    setError(null);
    setLoading(true);
    try {
      await signIn(username.trim(), password);
      onSuccess();
    } catch (err) {
      setError(err instanceof Error ? err.message : "Sign in failed");
    } finally {
      setLoading(false);
    }
  }

  return (
    <div className="login-shell">
      <div className="login-intro">
        <div className="login-brand">
          <span className="brand-mark" aria-hidden="true">G</span>
          <span>Giong</span>
        </div>
        <div className="login-intro-content">
          <p className="login-kicker">Your deployment workspace</p>
          <h1>Every build, <span>in view.</span></h1>
          <p className="login-description">Keep an eye on your stacks, follow jobs as they run, and find the logs that matter.</p>
          <div className="login-pipeline" aria-hidden="true">
            <div className="pipeline-step"><span>01</span>Commit</div>
            <div className="pipeline-connector" />
            <div className="pipeline-step"><span>02</span>Build</div>
            <div className="pipeline-connector" />
            <div className="pipeline-step"><span>03</span>Deploy</div>
          </div>
        </div>
        <p className="login-intro-footer">Giong Console · CI/CD</p>
      </div>
      <form className="login-card" onSubmit={handleSubmit}>
        <p className="brand-eyebrow">Welcome back</p>
        <h2 className="brand-title">Sign in to your console</h2>
        <p className="brand-subtitle">Enter your credentials to continue.</p>

        {error && <div className="error-banner" role="alert">{error}</div>}

        <div className="field">
          <label htmlFor="username">Username</label>
          <input
            id="username"
            type="text"
            value={username}
            onChange={(e) => setUsername(e.target.value)}
            autoComplete="username"
            required
          />
        </div>

        <div className="field">
          <label htmlFor="password">Password</label>
          <input
            id="password"
            type="password"
            value={password}
            onChange={(e) => setPassword(e.target.value)}
            autoComplete="current-password"
            placeholder="••••••••••••"
            required
          />
        </div>

        <button className="btn-primary" type="submit" disabled={loading}>
          {loading ? "Signing in..." : "Sign in"}
          {!loading && <span aria-hidden="true">↗</span>}
        </button>
      </form>
    </div>
  );
}
