import { useId, useState } from "react";
import "./NodeTypePicker.css";

interface Props {
  kinds: string[];
  value: string;
  loading: boolean;
  error: string | null;
  onChange: (kind: string) => void;
}

export function NodeTypePicker({ kinds, value, loading, error, onChange }: Props) {
  const id = useId();
  const [query, setQuery] = useState("");
  const available = new Set(kinds);
  const options = [...new Set([...kinds, ...(value ? [value] : [])])].sort();
  const terms = query.toLowerCase().trim().split(/\s+/);
  const filteredKinds = options.filter(kind => terms.every(term =>
    kind.toLowerCase().replace(/[-_/]/g, " ").includes(term)
    || kind.toLowerCase().includes(term),
  ));

  return (
    <fieldset className="node-type-picker" aria-busy={loading}>
      <legend>Kind</legend>
      <div className="node-type-toolbar">
        <input
          type="search"
          aria-label="Search node kinds"
          placeholder="Search kinds…"
          value={query}
          onChange={event => setQuery(event.target.value)}
        />
        <span className="node-type-count" role="status">
          {loading ? "Loading kinds…" : `${available.size} available`}
        </span>
      </div>
      {error && <p className="ui-error" role="alert">{error}</p>}
      <div className="node-type-options">
        {filteredKinds.map(kind => (
          <label className="node-type-option" key={kind}>
            <input
              type="radio"
              name={id}
              value={kind}
              checked={value === kind}
              onChange={() => onChange(kind)}
            />
            <span className="node-type-copy">
              <code>{kind}</code>
              {!loading && !error && !available.has(kind) && <span className="node-type-unavailable">Currently unavailable</span>}
            </span>
          </label>
        ))}
      </div>
      {!filteredKinds.length && !loading && !error && (
        <p className="ui-empty" role="status">
          {query.trim() ? "No kinds match your search." : "No node kinds are available."}
        </p>
      )}
      {value && <p className="node-type-selected">Selected: <code>{value}</code></p>}
    </fieldset>
  );
}
