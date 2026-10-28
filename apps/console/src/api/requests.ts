import type { SignInOutput } from "../types/auth";
import type { Job } from "../types/job";
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
