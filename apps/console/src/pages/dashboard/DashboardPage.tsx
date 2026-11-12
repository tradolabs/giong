import "./DashboardPage.css";
import { useEffect, useRef, useState } from "react";
import { UnauthorizedError } from "../../api/exception";
import { fetchNodeRuns, fetchFlows, runNode } from "../../api/requests";
import type { NodeRun } from "../../types/node-run";
import type { Flow } from "../../types/flow";
import { NodeLogsDialog } from "./components/NodeLogsDialog";
import { FlowCanvas } from "./components/FlowCanvas";
import { FlowPane } from "./components/FlowPane";
import { FlowSettingsDialog } from "./components/FlowSettingsDialog";
import { AddFlowDialog } from "./components/AddFlowDialog";
import { GlobalSettingsDialog } from "./components/GlobalSettingsDialog";

export function DashboardPage({ onSignOut }: { onSignOut: () => void }) {
  const flowRevision = useRef(0);
  const runRevision = useRef(0);
  const [addingFlow, setAddingFlow] = useState(false);
  const [showSettings, setShowSettings] = useState(false);
  const [flows, setFlows] = useState<Flow[]>([]);
  const [selectedFlowId, setSelectedFlowId] = useState<string | null>(null);
  const [runs, setRuns] = useState<NodeRun[]>([]);
  const [logsNodeId, setLogsNodeId] = useState<string | null>(null);
  const [selectedNodeId, setSelectedNodeId] = useState<string | null>(null);
  const [submitting, setSubmitting] = useState<string | null>(null);
  const [error, setError] = useState<string | null>(null);
  const [settingsFlow, setSettingsFlow] = useState<Flow | null>(null);
  const [settingsTab, setSettingsTab] = useState<"general" | "secrets">("general");
  const selectedFlow = flows.find(flow => flow.id === selectedFlowId);

  useEffect(() => {
    let active = true;
    async function load() {
      try {
        const revision = flowRevision.current;
        const data = await fetchFlows();
        if (!active || revision !== flowRevision.current) return;
        setFlows(data);
        setSelectedFlowId(current => current && data.some(flow => flow.id === current) ? current : data[0]?.id ?? null);
      } catch (err) { if (err instanceof UnauthorizedError) onSignOut(); }
    }
    void load();
    const timer = window.setInterval(load, 5000);
    return () => { active = false; window.clearInterval(timer); };
  }, [onSignOut]);

  useEffect(() => {
    setSelectedNodeId(current => current && selectedFlow?.nodes.some(node => node.id === current) ? current : selectedFlow?.nodes[0]?.id ?? null);
  }, [selectedFlowId, selectedFlow?.nodes]);

  useEffect(() => {
    setRuns([]); setError(null); setLogsNodeId(null);
    if (!selectedFlowId) return;
    let active = true;
    const id = selectedFlowId;
    async function load() {
      try {
        const revision = runRevision.current;
        const data = await fetchNodeRuns(id);
        if (active && revision === runRevision.current) setRuns(data);
      } catch (err) {
        if (!active) return;
        if (err instanceof UnauthorizedError) onSignOut();
        else setError(err instanceof Error ? err.message : "Could not load node status");
      }
    }
    void load();
    const timer = window.setInterval(load, 2000);
    return () => { active = false; window.clearInterval(timer); };
  }, [selectedFlowId, onSignOut]);

  async function executeNode(nodeId: string) {
    if (!selectedFlowId || submitting) return;
    setSubmitting(nodeId); setSelectedNodeId(nodeId); setError(null);
    try {
      const run = await runNode(selectedFlowId, nodeId);
      runRevision.current += 1;
      setRuns(current => [run, ...current.filter(item => item.id !== run.id)]);
    } catch (err) {
      if (err instanceof UnauthorizedError) onSignOut();
      else setError(err instanceof Error ? err.message : "Could not run node");
    } finally { setSubmitting(null); }
  }
  const logsNode = selectedFlow?.nodes.find(node => node.id === logsNodeId);

  return <div className="app-shell">
    <header className="topbar">
      <div className="topbar-brand"><span className="brand-mark" aria-hidden="true">G</span><span className="topbar-logo">Giong</span><span className="topbar-tag">Console</span><span className="topbar-context">CI/CD</span></div>
      <div className="topbar-actions"><button className="btn-ghost" type="button" onClick={() => setShowSettings(true)}>Settings</button><button className="btn-ghost" type="button" onClick={onSignOut}>Sign out</button></div>
    </header>
    <main className="workspace canvas-workspace">
      <FlowPane flows={flows} selectedFlowId={selectedFlowId} onSelectFlow={setSelectedFlowId}
        onSettings={flow => { setSettingsTab("general"); setSettingsFlow(flow); }} onAddFlow={() => setAddingFlow(true)} />
      <div className="flow-canvas-content">
        <FlowCanvas key={selectedFlowId} flow={selectedFlow} runs={runs} selected={selectedNodeId} submitting={submitting} error={error}
          onSelect={id => { setSelectedNodeId(id); setLogsNodeId(id); }} onRun={nodeId => { void executeNode(nodeId); }}
          onSettings={() => { if (selectedFlow) { setSettingsTab("general"); setSettingsFlow(selectedFlow); } }} />
      </div>
    </main>
    {logsNode && <NodeLogsDialog node={logsNode} runs={runs} onClose={() => setLogsNodeId(null)} onSignOut={onSignOut} />}
    {addingFlow && <AddFlowDialog onClose={() => setAddingFlow(false)} onSignOut={onSignOut} onCreated={flow => {
      flowRevision.current += 1; setFlows(current => [flow, ...current]); setSelectedFlowId(flow.id); setSettingsTab("secrets"); setSettingsFlow(flow);
    }} />}
    {showSettings && <GlobalSettingsDialog onClose={() => setShowSettings(false)} onSignOut={onSignOut} />}
    {settingsFlow && <FlowSettingsDialog key={settingsFlow.id} flow={settingsFlow} initialTab={settingsTab} onClose={() => setSettingsFlow(null)} onSignOut={onSignOut}
      onDeleted={id => {
        flowRevision.current += 1; setFlows(current => current.filter(flow => flow.id !== id));
        if (selectedFlowId === id) { setSelectedFlowId(flows.find(flow => flow.id !== id)?.id ?? null); setRuns([]); setSelectedNodeId(null); }
        setSettingsFlow(null);
      }} onSaved={updated => {
        flowRevision.current += 1; setFlows(current => current.map(flow => flow.id === updated.id ? updated : flow)); setSettingsFlow(updated);
      }} />}
  </div>;
}
