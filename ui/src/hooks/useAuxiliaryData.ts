import { useState, useCallback } from 'react';
import {
  MemoryFileNode,
  ContainerSummaryInfo,
  EnginePreset,
  ModelConfigRecord,
  McpServerRecord,
  McpTool,
  AuditEventRecord,
} from '../types';

export const useAuxiliaryData = () => {
  const [memoryFiles, setMemoryFiles] = useState<MemoryFileNode[]>([]);
  const [containers, setContainers] = useState<ContainerSummaryInfo[]>([]);
  const [presets, setPresets] = useState<EnginePreset[]>([]);
  const [modelConfigs, setModelConfigs] = useState<ModelConfigRecord[]>([]);
  const [mcpServers, setMcpServers] = useState<McpServerRecord[]>([]);
  const [mcpTools, setMcpTools] = useState<McpTool[]>([]);
  const [auditEvents, setAuditEvents] = useState<AuditEventRecord[]>([]);

  const refreshMemoryFiles = useCallback(async () => {
    try {
      const res = await fetch('/api/memory/tree');
      const data = await res.json();
      if (data.success) setMemoryFiles(data.files);
    } catch (e) {
      console.error(e);
    }
  }, []);

  const refreshModelData = useCallback(async () => {
    try {
      const [cRes, pRes, cfgRes] = await Promise.all([
        fetch('/api/models/containers'),
        fetch('/api/models/presets'),
        fetch('/api/models/configs'),
      ]);
      const cData = await cRes.json();
      const pData = await pRes.json();
      const cfgData = await cfgRes.json();

      if (cData.success) setContainers(cData.containers);
      if (pData.success) setPresets(pData.presets);
      if (cfgData.success) setModelConfigs(cfgData.configs);
    } catch (e) {
      console.error(e);
    }
  }, []);

  const refreshToolsData = useCallback(async () => {
    try {
      const [sRes, tRes] = await Promise.all([
        fetch('/api/tools/servers'),
        fetch('/api/tools/list'),
      ]);
      const sData = await sRes.json();
      const tData = await tRes.json();

      if (sData.success) setMcpServers(sData.servers);
      if (tData.success) setMcpTools(tData.tools);
    } catch (e) {
      console.error(e);
    }
  }, []);

  const refreshAuditData = useCallback(async () => {
    try {
      const res = await fetch('/api/audit/events?limit=100');
      const data = await res.json();
      if (data.success) setAuditEvents(data.events);
    } catch (e) {
      console.error(e);
    }
  }, []);

  const handleSelectActiveModel = useCallback(async (id: string) => {
    try {
      await fetch(`/api/models/configs/${id}/activate`, { method: 'POST' });
      refreshModelData();
    } catch (e) {
      console.error(e);
    }
  }, [refreshModelData]);

  const handleUpdatePolicy = useCallback(
    async (
      toolName: string,
      policy: 'AUTONOMOUS' | 'REQUIRE_APPROVAL' | 'BLOCKED',
      riskLevel?: string
    ) => {
      try {
        await fetch('/api/tools/policy', {
          method: 'POST',
          headers: { 'Content-Type': 'application/json' },
          body: JSON.stringify({
            tool_name: toolName,
            policy,
            risk_level: riskLevel,
          }),
        });
        refreshToolsData();
      } catch (e) {
        console.error(e);
      }
    },
    [refreshToolsData]
  );

  return {
    memoryFiles,
    containers,
    presets,
    modelConfigs,
    mcpServers,
    mcpTools,
    auditEvents,
    refreshMemoryFiles,
    refreshModelData,
    refreshToolsData,
    refreshAuditData,
    handleSelectActiveModel,
    handleUpdatePolicy,
  };
};
