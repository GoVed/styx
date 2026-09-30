import { useState, useEffect, useRef } from 'react';
import { McpServerRecord, InspectedToolInfo, DiscoveredLocalTool } from '../../types';
import { useManualServerModal } from './useManualServerModal';
import { useToolTester } from './useToolTester';

export const useToolHub = (servers: McpServerRecord[], onRefresh: () => void) => {
  // Manual Modal subhook
  const manualModal = useManualServerModal(onRefresh);

  // Test Tool Drawer subhook
  const toolTester = useToolTester();

  // Drag and Drop & Auto-Install State
  const [isDragging, setIsDragging] = useState<boolean>(false);
  const [inputPath, setInputPath] = useState<string>('');
  const [isInspecting, setIsInspecting] = useState<boolean>(false);
  const [isInstalling, setIsInstalling] = useState<boolean>(false);
  const [inspectedTool, setInspectedTool] = useState<InspectedToolInfo | null>(null);
  const [, setDiscoveredTools] = useState<DiscoveredLocalTool[]>([]);
  const [statusMessage, setStatusMessage] = useState<{
    type: 'success' | 'error' | 'info';
    text: string;
  } | null>(null);
  const [simulatingTrigger, setSimulatingTrigger] = useState<boolean>(false);

  const fileInputRef = useRef<HTMLInputElement>(null);

  // Fetch local discovered tools on mount
  useEffect(() => {
    fetchDiscoveredTools();
  }, [servers]);

  const fetchDiscoveredTools = async () => {
    try {
      const res = await fetch('/api/tools/discover');
      if (res.ok) {
        const data = await res.json();
        if (data.success && Array.isArray(data.tools)) {
          setDiscoveredTools(data.tools);
        }
      }
    } catch (e) {
      console.error('Failed to discover local tools:', e);
    }
  };

  const handleInspectPath = async (targetPath: string, manifest?: any) => {
    setIsInspecting(true);
    setStatusMessage(null);
    try {
      const res = await fetch('/api/tools/inspect', {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({
          path: targetPath,
          manifest,
        }),
      });
      const data = await res.json();
      if (data.success && data.tool) {
        setInspectedTool(data.tool);
        setStatusMessage({
          type: 'info',
          text: `Tool package detected: ${data.tool.display_name} (${data.tool.tools?.length || 0} tools available)`,
        });
      } else {
        setStatusMessage({
          type: 'error',
          text: data.error || 'Failed to inspect tool package at ' + targetPath,
        });
      }
    } catch (e: any) {
      setStatusMessage({
        type: 'error',
        text: 'Error inspecting tool: ' + (e.message || e),
      });
    } finally {
      setIsInspecting(false);
    }
  };

  const handleDragOver = (e: React.DragEvent) => {
    e.preventDefault();
    e.stopPropagation();
    setIsDragging(true);
  };

  const handleDragLeave = (e: React.DragEvent) => {
    e.preventDefault();
    e.stopPropagation();
    setIsDragging(false);
  };

  const handleDrop = async (e: React.DragEvent) => {
    e.preventDefault();
    e.stopPropagation();
    setIsDragging(false);

    const droppedText = e.dataTransfer.getData('text/plain');
    if (droppedText && (droppedText.includes('/') || droppedText.length > 2)) {
      setInputPath(droppedText.trim());
      await handleInspectPath(droppedText.trim());
      return;
    }

    const items = e.dataTransfer.items;
    let foundPackageJson: any = null;
    let detectedDirName = '';

    if (items && items.length > 0) {
      for (let i = 0; i < items.length; i++) {
        const item = items[i];
        if (item.kind === 'file') {
          const entry = (item as any).webkitGetAsEntry ? (item as any).webkitGetAsEntry() : null;
          if (entry && entry.isDirectory) {
            detectedDirName = entry.name;
          }

          const file = item.getAsFile();
          if (file && file.name === 'package.json') {
            try {
              const text = await file.text();
              foundPackageJson = JSON.parse(text);
            } catch (err) {
              console.warn('Failed to parse dropped package.json:', err);
            }
          }
        }
      }
    }

    let resolvedPath = inputPath;
    if (detectedDirName) {
      resolvedPath = `styx_tools/${detectedDirName}`;
      setInputPath(resolvedPath);
    } else if (foundPackageJson?.name) {
      const toolDir = foundPackageJson.name.replace(/^@?[^/]+\//, '');
      resolvedPath = `styx_tools/${toolDir}`;
      setInputPath(resolvedPath);
    }

    await handleInspectPath(resolvedPath, foundPackageJson);
  };

  const handleFileInputChange = async (e: React.ChangeEvent<HTMLInputElement>) => {
    const files = e.target.files;
    if (!files || files.length === 0) return;

    let foundPackageJson: any = null;
    let dirName = '';

    for (let i = 0; i < files.length; i++) {
      const f = files[i];
      if (f.webkitRelativePath) {
        const parts = f.webkitRelativePath.split('/');
        if (parts.length > 0) dirName = parts[0];
      }
      if (f.name === 'package.json') {
        try {
          const text = await f.text();
          foundPackageJson = JSON.parse(text);
        } catch {}
      }
    }

    let targetPath = inputPath;
    if (dirName) {
      targetPath = `styx_tools/${dirName}`;
      setInputPath(targetPath);
    }

    await handleInspectPath(targetPath, foundPackageJson);
  };

  const handleInstallTool = async () => {
    if (!inspectedTool) return;
    setIsInstalling(true);
    setStatusMessage({ type: 'info', text: 'Connecting to tool container & performing MCP handshake...' });

    try {
      const res = await fetch('/api/tools/install', {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({
          path: inspectedTool.path,
          name: inspectedTool.name,
          transport_type: inspectedTool.recommended_transport || 'http',
          url: inspectedTool.recommended_url,
          auto_start_container: true,
        }),
      });

      const data = await res.json();
      if (data.success) {
        setStatusMessage({
          type: 'success',
          text: `✓ ${inspectedTool.display_name} successfully registered! Discovered ${data.discovered_tools?.length || 0} tools.`,
        });
        setInspectedTool(null);
        onRefresh();
        fetchDiscoveredTools();
      } else {
        setStatusMessage({
          type: 'error',
          text: 'Installation failed: ' + (data.error || 'Unknown error'),
        });
      }
    } catch (e: any) {
      setStatusMessage({
        type: 'error',
        text: 'Error during tool installation: ' + (e.message || e),
      });
    } finally {
      setIsInstalling(false);
    }
  };

  const handleTriggerSimulatedInbound = async (protocolName?: string) => {
    const proto = typeof protocolName === 'string' ? protocolName : 'chat';
    setSimulatingTrigger(true);
    try {
      const res = await fetch('/api/tools/trigger', {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({
          protocol: proto,
          event_type: 'new_message',
          source_id: 'Alex (+15550199882)',
          payload: {
            from: '+15550199882',
            sender_name: 'Alex',
            message: 'Hey, are you free for a quick sync this afternoon?',
            timestamp: Math.floor(Date.now() / 1000),
          },
        }),
      });

      const data = await res.json();
      if (data.success) {
        setStatusMessage({
          type: 'success',
          text: `✓ Inbound event simulation dispatched! Agent session: ${data.session_id}`,
        });
      } else {
        setStatusMessage({
          type: 'error',
          text: 'Failed to simulate trigger: ' + data.error,
        });
      }
    } catch (e: any) {
      setStatusMessage({ type: 'error', text: 'Trigger error: ' + (e.message || e) });
    } finally {
      setSimulatingTrigger(false);
    }
  };

  const handleRemoveServer = async (id: string) => {
    if (!confirm('Disconnect and unregister this MCP daemon?')) return;
    try {
      await fetch(`/api/tools/servers/${id}`, { method: 'DELETE' });
      onRefresh();
      fetchDiscoveredTools();
    } catch (e) {
      console.error(e);
    }
  };

  return {
    isDragging,
    inputPath,
    setInputPath,
    isInspecting,
    isInstalling,
    inspectedTool,
    setInspectedTool,
    statusMessage,
    setStatusMessage,
    simulatingTrigger,
    fileInputRef,
    handleInspectPath,
    handleDragOver,
    handleDragLeave,
    handleDrop,
    handleFileInputChange,
    handleInstallTool,
    handleTriggerSimulatedInbound: () => handleTriggerSimulatedInbound('chat'),
    handleRemoveServer,
    ...manualModal,
    ...toolTester,
  };
};
