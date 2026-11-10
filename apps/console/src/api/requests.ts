import type { Repository } from "../types/repository";
import type { Runner } from "../types/runner";
import type { SignInOutput } from "../types/auth";
import type { Job } from "../types/job";
import type { SecretSet, Secret } from "../types/secret";
import type { Stack } from "../types/stack";
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
      else if (data?.code === "modules.stack.already_exists") message = "A stack with this name already exists.";
      else if (data?.code === "modules.repository.already_exists") message = "This repository URL has already been added.";
      else if (data?.code === "modules.repository.not_found") message = "This repository no longer exists. Refresh and select another repository.";
      else if (data?.code === "modules.stack.not_found") message = "This stack no longer exists.";
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

export function fetchStacks(): Promise<Stack[]> {
  return request<Stack[]>("/stacks");
}

export function fetchSecrets(stackId: string): Promise<Secret[]> {
  return request<Secret[]>(`/stacks/${encodeURIComponent(stackId)}/secrets`);
}

export function createSecret(stackId: string, key: string, value: string): Promise<Secret[]> {
  return request<Secret[]>(`/stacks/${encodeURIComponent(stackId)}/secrets`, {
    method: "POST",
    body: JSON.stringify({ key, value }),
  });
}

export function updateSecret(stackId: string, key: string, value: string): Promise<Secret[]> {
  return request<Secret[]>(`/stacks/${encodeURIComponent(stackId)}/secrets/${encodeURIComponent(key)}`, {
    method: "PUT",
    body: JSON.stringify({ value }),
  });
}

export function deleteSecret(stackId: string, key: string): Promise<Secret[]> {
  return request<Secret[]>(`/stacks/${encodeURIComponent(stackId)}/secrets/${encodeURIComponent(key)}`, {
    method: "DELETE",
  });
}

export function fetchJobs(stackId: string): Promise<Job[]> {
  return request<Job[]>(`/stacks/${encodeURIComponent(stackId)}/jobs`);
}

export function createJob(stackId: string): Promise<Job> {
  return request<Job>(`/stacks/${encodeURIComponent(stackId)}/jobs`, { method: "POST" });
}

export function fetchJobLogs(jobId: string, fromLines = 0): Promise<string> {
  return request<string>(
    `/jobs/${encodeURIComponent(jobId)}/logs?from_lines=${fromLines}`
  );
}

export function updateStack(stackId: string, settings: Pick<Stack, "name" | "configurations">): Promise<Stack> {
  return request<Stack>(`/stacks/${encodeURIComponent(stackId)}`, {
    method: "PUT",
    body: JSON.stringify(settings),
  });
}

export function fetchRunners(): Promise<Runner[]> {
  return request<Runner[]>("/runners");
}

export function fetchRepositories(): Promise<Repository[]> {
  return request<Repository[]>("/repositories");
}

export function createRepository(name: string, url: string, token?: string): Promise<Repository> {
  return request<Repository>("/repositories", { method: "POST", body: JSON.stringify({ name, url, token }) });
}

export function createStack(input: Pick<Stack, "name" | "configurations">): Promise<Stack> {
  return request<Stack>("/stacks", { method: "POST", body: JSON.stringify(input) });
}

export function fetchSecretSets(): Promise<SecretSet[]> {
  return request<SecretSet[]>("/secret-sets");
}
export function createSecretSet(name: string): Promise<SecretSet> {
  return request<SecretSet>("/secret-sets", { method: "POST", body: JSON.stringify({ name }) });
}
export function deleteSecretSet(id: string): Promise<void> {
  return request<void>(`/secret-sets/${encodeURIComponent(id)}`, { method: "DELETE" });
}
export function fetchSetSecrets(id: string): Promise<Secret[]> {
  return request<Secret[]>(`/secret-sets/${encodeURIComponent(id)}/secrets`);
}
export function createSetSecret(id: string, key: string, value: string): Promise<Secret[]> {
  return request<Secret[]>(`/secret-sets/${encodeURIComponent(id)}/secrets`, {
    method: "POST", body: JSON.stringify({ key, value }),
  });
}
export function updateSetSecret(id: string, key: string, value: string): Promise<Secret[]> {
  return request<Secret[]>(`/secret-sets/${encodeURIComponent(id)}/secrets/${encodeURIComponent(key)}`, {
    method: "PUT", body: JSON.stringify({ value }),
  });
}
export function deleteSetSecret(id: string, key: string): Promise<Secret[]> {
  return request<Secret[]>(`/secret-sets/${encodeURIComponent(id)}/secrets/${encodeURIComponent(key)}`, { method: "DELETE" });
}
export function inheritSecretSets(id: string, inherited_secret_set_ids: string[]): Promise<Secret[]> {
  return request<Secret[]>(`/stacks/${encodeURIComponent(id)}/secret-sets`, {
    method: "PUT", body: JSON.stringify({ inherited_secret_set_ids }),
  });
}
