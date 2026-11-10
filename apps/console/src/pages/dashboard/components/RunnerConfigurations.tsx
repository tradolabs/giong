import { useId } from "react";
import { parse, stringify } from "smol-toml";
import type { Runner } from "../../../types/runner";

interface Props {
  runner: Runner;
  value: string;
  disabled: boolean;
  onChange: (value: string) => void;
}

export function runnerDefaults(runner?: Runner) {
  return stringify(runner?.defaults ?? {});
}

export function parseConfiguration(value: string): Record<string, unknown> {
  // Round trip to the JSON representation accepted by the runner API.
  return JSON.parse(JSON.stringify(parse(value), (_key, entry: unknown) => {
    if (typeof entry === "number" && !Number.isFinite(entry)) {
      throw new Error("Configuration numbers must be finite.");
    }
    return entry;
  })) as Record<string, unknown>;
}

export function RunnerConfigurations({ runner, value, disabled, onChange }: Props) {
  const id = useId();
  let error = "";
  try { parseConfiguration(value); }
  catch (err) { error = err instanceof Error ? err.message : "Invalid TOML"; }
  const properties = (runner.configuration_schema.properties ?? {}) as Record<string, { title?: string; description?: string }>;
  return <div className="configuration-editor">
    <label htmlFor={id}>TOML</label>
    <textarea id={id} rows={10} value={value} disabled={disabled}
      spellCheck={false} autoCapitalize="none" autoComplete="off"
      aria-invalid={!!error} aria-describedby={error ? `${id}-error` : undefined}
      placeholder={'# Enter runner configuration\n# key = "value"'}
      onChange={(event) => onChange(event.target.value)} />
    {error && <p id={`${id}-error`} className="secrets-error" role="alert">{error}</p>}
    {Object.keys(properties).length > 0 && <details className="configuration-reference">
      <summary>Available options</summary>
      <dl>{Object.entries(properties).map(([name, property]) => <div key={name}>
        <dt><code>{name}</code></dt><dd>{property.description ?? property.title ?? name}</dd>
      </div>)}</dl>
    </details>}
  </div>;
}
