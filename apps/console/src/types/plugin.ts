export type PluginStatus = "installed" | "missing" | "unknown";

export interface Plugin {
  kind: string;
  image: string;
  name: string;
  version: string;
  description: string;
  status: PluginStatus;
}

export interface PluginInventory {
  docker_available: boolean;
  plugins: Plugin[];
}
