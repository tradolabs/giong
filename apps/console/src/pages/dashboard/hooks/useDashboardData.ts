import { useEffect, useRef, useState } from "react";
import { getApiErrorMessage } from "../../../api/errors";
import { fetchFlows, executeNode as requestNodeExecution } from "../../../api/requests";
import type { Flow } from "../../../types/flow";

export function useDashboardData(
  onSignOut: () => void,
  selectedFlowId: string | null,
  enabled: boolean,
  refreshRevision?: number,
) {
  const flowRevision = useRef(0);
  const currentFlowId = useRef(selectedFlowId);
  currentFlowId.current = selectedFlowId;
  const [flows, setFlows] = useState<Flow[]>([]);
  const [loading, setLoading] = useState(true);
  const [selectedNodeId, setSelectedNodeId] = useState<string | null>(null);
  const [submitting, setSubmitting] = useState<string | null>(null);
  const [error, setError] = useState<string | null>(null);

  const selectedFlow = flows.find(flow => flow.id === selectedFlowId);

  useEffect(() => {
    if (!enabled) return;
    let active = true;
    let timeout: number | undefined;
    const controller = new AbortController();
    setLoading(true);
    setError(null);

    async function load() {
      try {
        const revision = flowRevision.current;
        const data = await fetchFlows(controller.signal);
        if (!active || revision !== flowRevision.current) return;
        setFlows(data);
        setError(null);
      } catch (err) {
        if (active) setError(getApiErrorMessage(err, onSignOut, "Could not load flows"));
      } finally {
        if (active) {
          setLoading(false);
          if (selectedFlowId) timeout = window.setTimeout(() => { void load(); }, 10_000);
        }
      }
    }

    void load();
    return () => {
      active = false;
      controller.abort();
      window.clearTimeout(timeout);
    };
  }, [enabled, selectedFlowId, onSignOut, refreshRevision]);

  useEffect(() => {
    setSelectedNodeId(current =>
      current && selectedFlow?.nodes.some(node => node.id === current)
        ? current
        : (selectedFlow?.nodes[0]?.id ?? null),
    );
  }, [selectedFlowId, selectedFlow?.nodes]);

  async function executeNode(nodeId: string) {
    if (!selectedFlowId || submitting) return;
    setSubmitting(nodeId);
    setSelectedNodeId(nodeId);
    setError(null);
    try {
      const flowId = selectedFlowId;
      const execution = await requestNodeExecution(flowId, nodeId);
      if (currentFlowId.current !== flowId) return;
      flowRevision.current += 1;
      setFlows(current => current.map(flow => flow.id !== flowId ? flow : {
        ...flow,
        nodes: flow.nodes.map(node => {
          if (node.id !== nodeId || (node.last_execution && node.last_execution.id >= execution.id)) return node;
          return { ...node, status: execution.status, last_execution: execution };
        }),
      }));
    } catch (err) {
      const message = getApiErrorMessage(
        err,
        onSignOut,
        "Could not execute node",
      );
      if (message && currentFlowId.current === selectedFlowId) setError(message);
    } finally {
      setSubmitting(null);
    }
  }

  function addFlow(flow: Flow) {
    flowRevision.current += 1;
    setFlows(current => [flow, ...current]);
  }

  function updateFlowInList(updated: Flow) {
    flowRevision.current += 1;
    setFlows(current =>
      current.map(flow => (flow.id === updated.id ? updated : flow)),
    );
  }

  function removeFlow(id: string) {
    flowRevision.current += 1;
    setFlows(current => current.filter(flow => flow.id !== id));
    if (selectedFlowId === id) {
      setSelectedNodeId(null);
    }
  }

  function removeNode(nodeId: string) {
    flowRevision.current += 1;
    setFlows(current => current.map(flow => {
      if (flow.id !== selectedFlowId) return flow;
      const removed = flow.nodes.find(node => node.id === nodeId);
      return { ...flow, nodes: flow.nodes.filter(node => node.id !== nodeId).map(node => ({
        ...node,
        triggers: node.triggers.filter(trigger => trigger.type !== "event" || trigger.node !== removed?.name),
      })) };
    }));
  }

  return {
    flows,
    selectedFlow,
    selectedFlowId,
    loading,
    selectedNodeId,
    setSelectedNodeId,
    submitting,
    error,
    executeNode,
    addFlow,
    updateFlowInList,
    removeFlow,
    removeNode,
  };
}
