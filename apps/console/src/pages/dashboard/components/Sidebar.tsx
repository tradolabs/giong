interface Props {
  expanded: boolean;
  active: "flow" | "setting";
  onToggle: () => void;
  onSignOut: () => void;
  onNavigate: () => void;
}

export function Sidebar({ expanded, active, onToggle, onSignOut, onNavigate }: Props) {
  return (
    <aside className={`sidebar${expanded ? " sidebar-expanded" : ""}`} aria-label="Sidebar">
      <div className="sidebar-header">
        {expanded ? (
          <a className="sidebar-brand" href="#/flows" onClick={onNavigate} aria-label="Giong home" title="Giong">
            <span className="brand-mark" aria-hidden="true">G</span>
            <span>Giong</span>
          </a>
        ) : (
          <button type="button" className="sidebar-brand" onClick={onToggle} aria-label="Expand sidebar" title="Expand sidebar" aria-expanded={false} aria-controls="console-navigation">
            <span className="brand-mark" aria-hidden="true">G</span>
          </button>
        )}
        {expanded && <button
          type="button"
          className="sidebar-toggle"
          aria-label="Collapse sidebar"
          title="Collapse sidebar"
          aria-expanded={true}
          aria-controls="console-navigation"
          onClick={onToggle}
        >
          <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="1.5" aria-hidden="true">
            <rect x="3" y="4" width="18" height="16" rx="2" />
            <path d="M9 4v16" />
            <path d="m16 9-3 3 3 3" />
          </svg>
        </button>}
      </div>
      <nav id="console-navigation" className="sidebar-navigation" aria-label="Main navigation">
        <a href="#/flows" onClick={onNavigate} aria-current={active === "flow" ? "page" : undefined} aria-label="Flow" title="Flow">
          <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="1.5" aria-hidden="true">
            <rect x="3" y="3" width="6" height="6" rx="1.5" />
            <rect x="15" y="15" width="6" height="6" rx="1.5" />
            <path d="M6 9v9h9M9 6h9v9" />
          </svg>
          {expanded && <span>Flow</span>}
        </a>
        <a href="#/settings" onClick={onNavigate} aria-current={active === "setting" ? "page" : undefined} aria-label="Setting" title="Setting">
          <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="1.5" strokeLinecap="round" aria-hidden="true">
            <path d="M4 7h16M4 17h16" />
            <circle cx="9" cy="7" r="3" fill="var(--bg-surface)" />
            <circle cx="15" cy="17" r="3" fill="var(--bg-surface)" />
          </svg>
          {expanded && <span>Setting</span>}
        </a>
      </nav>
      <button type="button" className="sidebar-sign-out" aria-label="Sign out" title="Sign out" onClick={onSignOut}>
        <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="1.5" strokeLinecap="round" aria-hidden="true">
          <path d="M9 4H5v16h4M10 12h11m-4-4 4 4-4 4" />
        </svg>
        {expanded && <span>Sign out</span>}
      </button>
    </aside>
  );
}
