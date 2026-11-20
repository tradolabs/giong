import type { SignInOutput } from "../types/auth";
import type { NodeExecution } from "../types/node-execution";
import type { NodeDetails } from "../types/node-details";
import type { Vault, Secret } from "../types/secret";
import type { Flow, FlowNode, NodeDefinition } from "../types/flow";
import type { Plugin, PluginInventory } from "../types/plugin";
import { UnauthorizedError } from "./exception";
import { getStoredToken, setStoredToken } from "./local-storage";

const API_URL = import.meta.env.API_URL?.replace(/\/+$/, "") || "/api";

async function request<T>(path: string, options: RequestInit = {}): Promise<T> {
  const token = getStoredToken();
  const headers = new Headers(options.headers);
  if (token) {
    headers.set("Authorization", `Bearer ${token}`);
  }
  if (options.body && !headers.has("Content-Type")) {
    headers.set("Content-Type", "application/json");
  }

  const response = await fetch(`${API_URL}${path}`, { ...options, headers });

  if (response.status === 401) {
    setStoredToken(null);
    throw new UnauthorizedError();
  }

  if (!response.ok) {
    let message = `HTTP ${response.status}`;
    try {
      const data = await response.json();
      if (data?.message) message = data.message;
      else if (data?.code === "modules.secret.already_exists")
        message = "A secret with this key already exists.";
      else if (data?.code === "modules.secret.not_found")
        message
          = "This secret no longer exists. Reopen secrets to refresh the list.";
      else if (data?.code === "modules.flow.active_run_exists")
        message = "This flow has an active execution. Wait for it to finish.";
      else if (data?.code === "modules.vault.in_use")
        message
          = "This vault is used by a flow.";
      else if (data?.code === "modules.flow.already_exists")
        message = "A flow with this name already exists.";
      else if (data?.code === "modules.vault.already_exists")
        message = "A vault with this name already exists.";
      else if (data?.code === "modules.vault.not_found")
        message = "This vault no longer exists.";
      else if (data?.code === "modules.node.not_found")
        message = "This node no longer exists in the flow.";
      else if (data?.code === "modules.node.already_exists")
        message = "A node with this name already exists.";
      else if (data?.code === "modules.repository.already_exists")
        message = "This repository URL has already been added.";
      else if (data?.code === "modules.repository.not_found")
        message
          = "This repository no longer exists. Refresh and select another repository.";
      else if (data?.code === "modules.flow.not_found")
        message = "This flow no longer exists.";
    } catch {
      const text = await response.text().catch(() => "");
      if (text) message = text;
    }
    throw new Error(message);
  }

  if (response.status === 204) return undefined as T;

  const contentType = response.headers.get("content-type") ?? "";
  if (contentType.includes("application/json")) {
    return (await response.json()) as T;
  }
  return (await response.text()) as unknown as T;
}

export async function signIn(
  username: string,
  password: string,
): Promise<SignInOutput> {
  const output = await request<SignInOutput>("/auth/sign-in", {
    method: "POST",
    body: JSON.stringify({ username, password }),
  });
  setStoredToken(output.access_token);
  return output;
}

export function fetchFlows(signal?: AbortSignal): Promise<Flow[]> {
  return request<Flow[]>("/flows", { signal });
}

export function fetchNodeKinds(signal?: AbortSignal): Promise<string[]> {
  return request<string[]>("/nodes/kinds", { signal });
}

export function fetchPlugins(signal?: AbortSignal): Promise<PluginInventory> {
  return request<PluginInventory>("/plugins", { signal });
}

export function installPlugin(image: string, pull: boolean): Promise<Plugin> {
  return request<Plugin>("/plugins", {
    method: "POST",
    body: JSON.stringify({ image, pull }),
  });
}

export function removePlugin(kind: string): Promise<void> {
  return request<void>(`/plugins?kind=${encodeURIComponent(kind)}`, { method: "DELETE" });
}

export function executeNode(flowId: string, nodeId: string): Promise<NodeExecution> {
  return request<NodeExecution>(
    `/flows/${encodeURIComponent(flowId)}/nodes/${encodeURIComponent(nodeId)}/execute`,
    { method: "POST" },
  );
}

export function fetchNodeExecutionLogs(
  logId: string,
  fromLines = 0,
): Promise<string> {
  return request<string>(
    `/logs/${encodeURIComponent(logId)}?from_lines=${fromLines}`,
  );
}

export function fetchNodeDetails(flowId: string, nodeId: string, signal?: AbortSignal): Promise<NodeDetails> {
  return request<NodeDetails>(`/flows/${encodeURIComponent(flowId)}/nodes/${encodeURIComponent(nodeId)}`, { signal });
}

export function updateFlow(
  flowId: string,
  settings: Pick<Flow, "name" | "vault_ids">,
): Promise<Flow> {
  return request<Flow>(`/flows/${encodeURIComponent(flowId)}`, {
    method: "PUT",
    body: JSON.stringify(settings),
  });
}

export function createFlow(input: Pick<Flow, "name" | "vault_ids">): Promise<Flow> {
  return request<Flow>("/flows", {
    method: "POST",
    body: JSON.stringify(input),
  });
}

export function createFlowNode(flowId: string, node: NodeDefinition): Promise<FlowNode> {
  return request<FlowNode>(`/flows/${encodeURIComponent(flowId)}/nodes`, {
    method: "POST", body: JSON.stringify(node),
  });
}

export function updateFlowNode(flowId: string, nodeId: string, node: NodeDefinition): Promise<FlowNode> {
  return request<FlowNode>(`/flows/${encodeURIComponent(flowId)}/nodes/${encodeURIComponent(nodeId)}`, {
    method: "PUT",
    body: JSON.stringify(node),
  });
}

export function deleteFlowNode(flowId: string, nodeId: string): Promise<void> {
  return request<void>(`/flows/${encodeURIComponent(flowId)}/nodes/${encodeURIComponent(nodeId)}`, { method: "DELETE" });
}

export function fetchVaults(): Promise<Vault[]> {
  return request<Vault[]>("/vaults");
}

export function createVault(name: string): Promise<Vault> {
  return request<Vault>("/vaults", {
    method: "POST",
    body: JSON.stringify({ name }),
  });
}

export function updateVault(id: string, name: string): Promise<Vault> {
  return request<Vault>(`/vaults/${encodeURIComponent(id)}`, {
    method: "PUT",
    body: JSON.stringify({ name }),
  });
}

export function deleteVault(id: string): Promise<void> {
  return request<void>(`/vaults/${encodeURIComponent(id)}`, {
    method: "DELETE",
  });
}

export function fetchVaultSecrets(id: string): Promise<Secret[]> {
  return request<Secret[]>(`/vaults/${encodeURIComponent(id)}/secrets`);
}

export async function createVaultSecret(
  id: string,
  key: string,
  value: string,
): Promise<Secret[]> {
  await request<void>(
    `/vaults/${encodeURIComponent(id)}/secrets`,
    {
      method: "POST",
      body: JSON.stringify({ key, value }),
    },
  );
  return fetchVaultSecrets(id);
}

export async function updateVaultSecret(
  id: string,
  secretId: string,
  value: string,
): Promise<Secret[]> {
  await request<void>(`/vaults/${encodeURIComponent(id)}/secrets/${encodeURIComponent(secretId)}`, {
    method: "PUT",
    body: JSON.stringify({ value }),
  });
  return fetchVaultSecrets(id);
}

export async function deleteVaultSecret(
  id: string,
  secretId: string,
): Promise<Secret[]> {
  await request<void>(
    `/vaults/${encodeURIComponent(id)}/secrets/${encodeURIComponent(secretId)}`,
    { method: "DELETE" },
  );
  return fetchVaultSecrets(id);
}

export function deleteFlow(id: string): Promise<void> {
  return request<void>(`/flows/${encodeURIComponent(id)}`, {
    method: "DELETE",
  });
}
