import { useId, useState } from "react";
import "./NodeTypePicker.css";

interface Props {
  kinds: string[];
  value: string;
  loading: boolean;
  error: string | null;
  onChange: (kind: string) => void;
}

function displayName(value: string) {
  return value.replace(/[-_]/g, " ").replace(/\b\w/g, character => character.toUpperCase());
}

export function NodeTypePicker({ kinds, value, loading, error, onChange }: Props) {
  const id = useId();
  const [query, setQuery] = useState("");
  const available = new Set(kinds);
  const options = [...new Set([...kinds, ...(value ? [value] : [])])].sort();
  const terms = query.toLowerCase().trim().split(/\s+/);
  const groups = new Map<string, string[]>();

  for (const kind of options) {
    if (!terms.every(term => kind.toLowerCase().replace(/[-_/]/g, " ").includes(term)
      || kind.toLowerCase().includes(term))) continue;
    const segments = kind.split("/");
    const group = segments.length > 1 ? segments.slice(0, -1).map(displayName).join(" / ") : "Other";
    groups.set(group, [...(groups.get(group) ?? []), kind]);
  }

  return (
    <fieldset className="node-type-picker" aria-busy={loading}>
      <legend>Type</legend>
      <div className="node-type-toolbar">
        <input
          type="search"
          aria-label="Search node types"
          placeholder="Search types…"
          value={query}
          onChange={event => setQuery(event.target.value)}
        />
        <span className="node-type-count" role="status">
          {loading ? "Loading types…" : `${available.size} available`}
        </span>
      </div>
      {error && <p className="ui-error" role="alert">{error}</p>}
      {Array.from(groups, ([group, groupKinds], index) => (
        <div className="node-type-group" role="group" aria-labelledby={`${id}-group-${index}`} key={group}>
          <h3 id={`${id}-group-${index}`}>{group}</h3>
          <div className="node-type-options">
            {groupKinds.map(kind => (
              <label className="node-type-option" key={kind}>
                <input
                  type="radio"
                  name={id}
                  value={kind}
                  checked={value === kind}
                  onChange={() => onChange(kind)}
                />
                <span className="node-type-copy">
                  <span className="node-type-name">{displayName(kind.split("/").at(-1) ?? kind)}</span>
                  <code>{kind}</code>
                  {!loading && !error && !available.has(kind) && <span className="node-type-unavailable">Currently unavailable</span>}
                </span>
              </label>
            ))}
          </div>
        </div>
      ))}
      {!groups.size && !loading && !error && (
        <p className="ui-empty" role="status">
          {query.trim() ? "No types match your search." : "No node types are available."}
        </p>
      )}
      {value && <p className="node-type-selected">Selected: <code>{value}</code></p>}
    </fieldset>
  );
}
