import "./DashboardPage.css";
import { useEffect, useState } from "react";
import type { Flow, FlowNode } from "../../types/flow";
import { AddFlowDialog } from "./components/AddFlowDialog";
import { Flow as FlowView } from "./components/Flow";
import { FlowList } from "./components/FlowList";
import { FlowSettingsDialog } from "./components/FlowSettingsDialog";
import { GlobalSettings } from "./components/GlobalSettings";
import { Sidebar } from "./components/Sidebar";
import { NodeDialog } from "./components/NodeDialog";
import { NodeDetailsDialog } from "./components/NodeDetailsDialog";
import { useDashboardData } from "./hooks/useDashboardData";

function selectedFlowFromUrl(hash: string): string | null {
  const match = hash.match(/^#\/flows\/([^/]+)$/);
  if (!match) return null;
  try { return decodeURIComponent(match[1]); } catch { return null; }
}

export function DashboardPage({ onSignOut }: { onSignOut: () => void }) {
  const [route, setRoute] = useState(() => window.location.hash);
  const [sidebarExpanded, setSidebarExpanded] = useState(() => {
    try {
      const saved = localStorage.getItem("giong_sidebar_expanded");
      if (saved !== null) return saved === "true";
    } catch { /* Use the viewport default when storage is unavailable. */ }
    return window.matchMedia("(min-width: 760px)").matches;
  });
  const selectedFlowId = selectedFlowFromUrl(route);
  const isSetting = route === "#/settings";
  const {
    flows,
    selectedFlow,
    loading,
    executions,
    selectedNodeId,
    setSelectedNodeId,
    submitting,
    error,
    executeNode,
    addFlow,
    updateFlowInList,
    removeFlow,
    removeNode,
  } = useDashboardData(onSignOut, selectedFlowId, !isSetting);

  const [nodeEditor, setNodeEditor] = useState<{ flowId: string; node?: FlowNode } | null>(null);
  const [addingFlow, setAddingFlow] = useState(false);
  const [logsNodeId, setLogsNodeId] = useState<string | null>(null);
  const [settingsFlow, setSettingsFlow] = useState<Flow | null>(null);
  const [settingsTab, setSettingsTab] = useState<"general" | "secrets">(
    "general",
  );

  const logsNode = selectedFlow?.nodes.find(node => node.id === logsNodeId);

  useEffect(() => {
    function navigate() {
      setRoute(window.location.hash);
      setLogsNodeId(null);
      setSettingsFlow(null);
      setNodeEditor(null);
    }
    window.addEventListener("hashchange", navigate);
    return () => window.removeEventListener("hashchange", navigate);
  }, []);

  function showFlows() {
    window.location.hash = "/flows";
    setRoute("#/flows");
    setLogsNodeId(null);
  }

  function openFlowSettings(flow: Flow, tab: "general" | "secrets" = "general") {
    setSettingsTab(tab);
    setSettingsFlow(flow);
  }

  return (
    <div className={`app-shell${sidebarExpanded ? " sidebar-open" : ""}`}>
      <Sidebar
        expanded={sidebarExpanded}
        active={isSetting ? "setting" : "flow"}
        onToggle={() => {
          const expanded = !sidebarExpanded;
          setSidebarExpanded(expanded);
          try { localStorage.setItem("giong_sidebar_expanded", String(expanded)); } catch { /* Keep the preference for this session. */ }
        }}
        onNavigate={() => {
          if (window.matchMedia("(max-width: 759px)").matches) setSidebarExpanded(false);
        }}
        onSignOut={onSignOut}
      />
      {sidebarExpanded && <button className="sidebar-backdrop" type="button" aria-label="Close navigation" onClick={() => setSidebarExpanded(false)} />}
      <div className="app-main">
        <main className={isSetting ? "settings-screen" : selectedFlowId ? "flow-details-screen" : "flows-screen"}>
          {isSetting ? <GlobalSettings onSignOut={onSignOut} /> : !selectedFlowId ? (
            <FlowList flows={flows} loading={loading} error={error} onSettings={flow => openFlowSettings(flow)} onAddFlow={() => setAddingFlow(true)} />
          ) : (
            <>
              <nav className="flow-breadcrumb" aria-label="Breadcrumb">
                <a href="#/flows">Flows</a><span aria-hidden="true">/</span><span>{selectedFlow?.name ?? "Flow"}</span>
              </nav>
              {loading ? <p role="status">Loading flow…</p> : selectedFlow ? (
                <FlowView
                  key={selectedFlowId}
                  flow={selectedFlow}
                  executions={executions}
                  selected={selectedNodeId}
                  submitting={submitting}
                  error={error}
                  onSelect={(id) => { setSelectedNodeId(id); setLogsNodeId(id); }}
                  onExecute={(nodeId) => { void executeNode(nodeId); }}
                  onSettings={() => openFlowSettings(selectedFlow)}
                  onAddNode={() => setNodeEditor({ flowId: selectedFlow.id })}
                  onEditNode={node => setNodeEditor({ flowId: selectedFlow.id, node })}
                />
              ) : <div className="empty-state"><p className="empty-state-title">Flow not found</p><button className="btn-ghost" type="button" onClick={showFlows}>Back to flows</button></div>}
            </>
          )}
        </main>
        {nodeEditor && selectedFlow && nodeEditor.flowId === selectedFlow.id && (
          <NodeDialog
            flow={selectedFlow}
            node={nodeEditor.node}
            onClose={() => setNodeEditor(null)}
            onSignOut={onSignOut}
            onDeleted={nodeId => {
              removeNode(nodeId);
              if (logsNodeId === nodeId) setLogsNodeId(null);
            }}
            onSaved={(saved) => {
              const exists = selectedFlow.nodes.some(node => node.id === saved.id);
              updateFlowInList({ ...selectedFlow, nodes: exists
                ? selectedFlow.nodes.map(node => node.id === saved.id ? saved : node)
                : [...selectedFlow.nodes, saved] });
            }}
          />
        )}
        {logsNode && selectedFlow && (
          <NodeDetailsDialog
            key={logsNode.id}
            flowId={selectedFlow.id}
            node={logsNode}
            executions={executions}
            onClose={() => setLogsNodeId(null)}
            onSignOut={onSignOut}
          />
        )}
        {addingFlow && (
          <AddFlowDialog
            onClose={() => setAddingFlow(false)}
            onSignOut={onSignOut}
            onCreated={(flow) => {
              addFlow(flow);
              window.location.hash = `/flows/${encodeURIComponent(flow.id)}`;
              setRoute(window.location.hash);
            }}
          />
        )}
        {settingsFlow && (
          <FlowSettingsDialog
            key={settingsFlow.id}
            flow={settingsFlow}
            initialTab={settingsTab}
            onClose={() => setSettingsFlow(null)}
            onSignOut={onSignOut}
            onDeleted={(id) => {
              if (selectedFlowId === id) setLogsNodeId(null);
              removeFlow(id);
              setSettingsFlow(null);
              if (selectedFlowId === id) showFlows();
            }}
            onSaved={(updated) => {
              updateFlowInList(updated);
              setSettingsFlow(updated);
            }}
          />
        )}
      </div>
    </div>
  );
}
