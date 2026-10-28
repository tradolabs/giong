import "./DashboardPage.css";
import { useEffect, useState } from "react";
import { UnauthorizedError } from "../../api/exception";
import { fetchJobs, fetchStacks } from "../../api/requests";
import type { Job } from "../../types/job";
import type { Stack } from "../../types/stack";
import { LogStreamPane } from "./components/LogStreamPane";
import { JobsPane } from "./components/JobsPane";
import { StackPane } from "./components/StackPane";

export function DashboardPage({ onSignOut }: { onSignOut: () => void }) {
  const [stacks, setStacks] = useState<Stack[]>([]);
  const [selectedStackId, setSelectedStackId] = useState<string | null>(null);
  const [jobs, setJobs] = useState<Job[]>([]);
  const [selectedJobId, setSelectedJobId] = useState<string | null>(null);

  useEffect(() => {
    let active = true;

    async function loadStacks() {
      try {
        const data = await fetchStacks();
        if (!active) return;
        setStacks(data);
        setSelectedStackId((current) => {
          if (current && data.some((s) => s.id === current)) return current;
          return data[0]?.id ?? null;
        });
      } catch (err) {
        if (err instanceof UnauthorizedError) onSignOut();
      }
    }

    loadStacks();
    const timer = window.setInterval(loadStacks, 5000);
    return () => {
      active = false;
      window.clearInterval(timer);
    };
  }, [onSignOut]);

  useEffect(() => {
    if (!selectedStackId) {
      setJobs([]);
      setSelectedJobId(null);
      return;
    }

    let active = true;
    const stackId = selectedStackId;

    async function loadJobs() {
      try {
        const data = await fetchJobs(stackId);
        if (!active) return;
        setJobs(data);
        setSelectedJobId((current) => {
          if (current && data.some((j) => j.id === current)) return current;
          return data[0]?.id ?? null;
        });
      } catch (err) {
        if (err instanceof UnauthorizedError) onSignOut();
      }
    }

    loadJobs();
    const timer = window.setInterval(loadJobs, 2500);
    return () => {
      active = false;
      window.clearInterval(timer);
    };
  }, [selectedStackId, onSignOut]);

  const selectedJob = jobs.find((j) => j.id === selectedJobId) ?? null;

  return (
    <div className="app-shell">
      <header className="topbar">
        <div className="topbar-brand">
          <span className="brand-mark" aria-hidden="true">G</span>
          <span className="topbar-logo">Giong</span>
          <span className="topbar-tag">Console</span>
          <span className="topbar-context">CI/CD</span>
        </div>
        <div className="topbar-actions">
          <button className="btn-ghost" type="button" onClick={onSignOut}>
            Sign out
          </button>
        </div>
      </header>

      <main className="workspace">
        <StackPane
          stacks={stacks}
          selectedStackId={selectedStackId}
          onSelectStack={setSelectedStackId}
        />
        <JobsPane
          key={selectedStackId}
          stackId={selectedStackId}
          onSignOut={onSignOut}
          onCreated={(job) => {
            setJobs((current) => [job, ...current.filter((item) => item.id !== job.id)]);
            setSelectedJobId(job.id);
          }}
          jobs={jobs}
          stacks={stacks}
          selectedJobId={selectedJobId}
          onSelectJob={setSelectedJobId}
        />
        <LogStreamPane
          job={selectedJob}
          stack={selectedJob ? stacks.find((stack) => stack.id === selectedJob.stack_id) : undefined}
          onSignOut={onSignOut}
        />
      </main>
    </div>
  );
}
