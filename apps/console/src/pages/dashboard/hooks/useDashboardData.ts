import { useEffect, useRef, useState } from "react";
import { getApiErrorMessage } from "../../../api/errors";
import { fetchFlows, executeNode as requestNodeExecution } from "../../../api/requests";
import type { Flow } from "../../../types/flow";
import type { NodeExecution } from "../../../types/node-execution";

export function useDashboardData(onSignOut: () => void, selectedFlowId: string | null, enabled: boolean) {
  const flowRevision = useRef(0);
  const currentFlowId = useRef(selectedFlowId);
  currentFlowId.current = selectedFlowId;
  const [flows, setFlows] = useState<Flow[]>([]);
  const [loading, setLoading] = useState(true);
  const [executions, setExecutions] = useState<NodeExecution[]>([]);
  const [selectedNodeId, setSelectedNodeId] = useState<string | null>(null);
  const [submitting, setSubmitting] = useState<string | null>(null);
  const [error, setError] = useState<string | null>(null);

  const selectedFlow = flows.find(flow => flow.id === selectedFlowId);

  useEffect(() => {
    if (!enabled) return;
    let active = true;
    const controller = new AbortController();
    setLoading(true);
    setError(null);

    async function load() {
      try {
        const revision = flowRevision.current;
        const data = await fetchFlows(controller.signal);
        if (!active || revision !== flowRevision.current) return;
        setFlows(data);
      } catch (err) {
        if (active) setError(getApiErrorMessage(err, onSignOut, "Could not load flows"));
      } finally {
        if (active) setLoading(false);
      }
    }

    void load();
    return () => {
      active = false;
      controller.abort();
    };
  }, [enabled, onSignOut]);

  useEffect(() => {
    setSelectedNodeId(current =>
      current && selectedFlow?.nodes.some(node => node.id === current)
        ? current
        : (selectedFlow?.nodes[0]?.id ?? null),
    );
  }, [selectedFlowId, selectedFlow?.nodes]);

  useEffect(() => {
    setExecutions([]);
    setError(null);
  }, [selectedFlowId]);

  async function executeNode(nodeId: string) {
    if (!selectedFlowId || submitting) return;
    setSubmitting(nodeId);
    setSelectedNodeId(nodeId);
    setError(null);
    try {
      const flowId = selectedFlowId;
      const execution = await requestNodeExecution(flowId, nodeId);
      if (currentFlowId.current !== flowId) return;
      setExecutions(current => [
        execution,
        ...current.filter(item => item.id !== execution.id),
      ]);
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
      setExecutions([]);
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
    setExecutions(current => current.filter(execution => execution.node_id !== nodeId));
  }

  return {
    flows,
    selectedFlow,
    selectedFlowId,
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
  };
}
