import { useState, useEffect } from 'react';
import { ContainerSummaryInfo } from '../../types';
import { parseContainerLoadingStage, safeFetchJson } from './vram';

export function useContainerLogs(
  containers: ContainerSummaryInfo[],
  activeContainer: ContainerSummaryInfo | undefined,
  isDeploying: boolean
) {
  const [selectedLogContainer, setSelectedLogContainer] = useState<string | null>(null);
  const [containerLogs, setContainerLogs] = useState<string[]>([]);

  useEffect(() => {
    if (!selectedLogContainer && containers.length > 0) {
      setSelectedLogContainer(containers[0].id);
    }
  }, [containers, selectedLogContainer]);

  const fetchLogs = async (id: string) => {
    try {
      const res = await fetch(`/api/models/containers/${id}/logs?tail=200`);
      const data = await safeFetchJson(res);
      if (data.success && data.logs) setContainerLogs(data.logs);
    } catch (e) {
      console.error(e);
    }
  };

  useEffect(() => {
    if (!selectedLogContainer) return;
    fetchLogs(selectedLogContainer);
    const interval = setInterval(() => fetchLogs(selectedLogContainer), 2000);
    return () => clearInterval(interval);
  }, [selectedLogContainer]);

  const activeContainerObj =
    containers.find(
      c => c.id === selectedLogContainer || c.id.startsWith(selectedLogContainer || '')
    ) || activeContainer;

  const progressInfo = parseContainerLoadingStage(
    containerLogs,
    activeContainerObj?.state || (isDeploying ? 'running' : 'offline'),
    activeContainerObj?.status || ''
  );

  return {
    selectedLogContainer,
    setSelectedLogContainer,
    containerLogs,
    fetchLogs,
    activeContainerObj,
    progressInfo,
  };
}
