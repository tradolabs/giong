import type { SignInOutput } from "../types/auth";
import type { NodeRun } from "../types/node-run";
import type { Vault, Secret } from "../types/secret";
import type { Flow } from "../types/flow";
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
      else if (data?.code === "modules.secret.already_exists") message = "A secret with this key already exists.";
      else if (data?.code === "modules.secret.not_found") message = "This secret no longer exists. Reopen secrets to refresh the list.";
      else if (data?.code === "modules.flow.active_run_exists") message = "This flow has an active run. Wait for it to finish.";
      else if (data?.code === "modules.flow.vault_in_use") message = "Another flow inherits this flow’s vault. Remove that inheritance before deleting the flow.";
      else if (data?.code === "modules.flow.already_exists") message = "A flow with this name already exists.";
      else if (data?.code === "modules.repository.already_exists") message = "This repository URL has already been added.";
      else if (data?.code === "modules.repository.not_found") message = "This repository no longer exists. Refresh and select another repository.";
      else if (data?.code === "modules.flow.not_found") message = "This flow no longer exists.";
    } catch {
      const text = await response.text().catch(() => "");
      if (text) message = text;
    }
    throw new Error(message);
  }

  const contentType = response.headers.get("content-type") ?? "";
  if (contentType.includes("application/json")) {
    return (await response.json()) as T;
  }
  return (await response.text()) as unknown as T;
}

export async function signIn(username: string, password: string): Promise<SignInOutput> {
  const output = await request<SignInOutput>("/auth/sign-in", {
    method: "POST",
    body: JSON.stringify({ username, password }),
  });
  setStoredToken(output.access_token);
  return output;
}

export function fetchFlows(): Promise<Flow[]> {
  return request<Flow[]>("/flows");
}

export function fetchNodeRuns(flowId: string): Promise<NodeRun[]> {
  return request<NodeRun[]>(`/flows/${encodeURIComponent(flowId)}/runs`);
}
export function runNode(flowId: string, nodeId: string): Promise<NodeRun> {
  return request<NodeRun>(`/flows/${encodeURIComponent(flowId)}/nodes/${encodeURIComponent(nodeId)}/run`, { method: "POST" });
}
export function fetchNodeRunLogs(runId: string, fromLines = 0): Promise<string> {
  return request<string>(`/runs/${encodeURIComponent(runId)}/logs?from_lines=${fromLines}`);
}

export function updateFlow(flowId: string, settings: Pick<Flow, "name" | "nodes" | "inherited_vault_ids">): Promise<Flow> {
  return request<Flow>(`/flows/${encodeURIComponent(flowId)}`, {
    method: "PUT",
    body: JSON.stringify(settings),
  });
}

export function createFlow(input: Pick<Flow, "name" | "nodes">): Promise<Flow> {
  return request<Flow>("/flows", { method: "POST", body: JSON.stringify(input) });
}

export function fetchVaults(): Promise<Vault[]> { return request<Vault[]>("/vaults"); }
export function createVault(name: string): Promise<Vault> {
  return request<Vault>("/vaults", { method: "POST", body: JSON.stringify({ name }) });
}
export function updateVault(id: string, name: string): Promise<Vault> {
  return request<Vault>(`/vaults/${encodeURIComponent(id)}`, { method: "PUT", body: JSON.stringify({ name }) });
}
export function deleteVault(id: string): Promise<void> {
  return request<void>(`/vaults/${encodeURIComponent(id)}`, { method: "DELETE" });
}
export function fetchVaultSecrets(id: string): Promise<Secret[]> {
  return request<Secret[]>(`/vaults/${encodeURIComponent(id)}/secrets`);
}
export async function putVaultSecret(id: string, key: string, value: string): Promise<Secret[]> {
  await request<void>(`/vaults/${encodeURIComponent(id)}/secrets/${encodeURIComponent(key)}`, {
    method: "PUT", body: JSON.stringify({ value }),
  });
  return fetchVaultSecrets(id);
}
export async function deleteVaultSecret(id: string, key: string): Promise<Secret[]> {
  await request<void>(`/vaults/${encodeURIComponent(id)}/secrets/${encodeURIComponent(key)}`, { method: "DELETE" });
  return fetchVaultSecrets(id);
}

export function deleteFlow(id: string): Promise<void> {
  return request<void>(`/flows/${encodeURIComponent(id)}`, { method: "DELETE" });
}

export function updateFlowVaults(flowId: string, inheritedVaultIds: string[]): Promise<Flow> {
  return request<Flow>(`/flows/${encodeURIComponent(flowId)}/vaults`, {
    method: "PUT", body: JSON.stringify({ inherited_vault_ids: inheritedVaultIds }),
  });
}
