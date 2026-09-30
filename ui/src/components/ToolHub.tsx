import React from 'react';
import { Layers, Plus, CheckCircle, AlertTriangle, Sparkles, X } from 'lucide-react';
import { McpServerRecord, McpTool } from '../types';
import { DragDropZone } from './tool-hub/DragDropZone';
import { ToolInspectionCard } from './tool-hub/ToolInspectionCard';
import { ConnectedServers } from './tool-hub/ConnectedServers';
import { PolicyMatrix } from './tool-hub/PolicyMatrix';
import { TestToolDrawer } from './tool-hub/TestToolDrawer';
import { ManualServerModal } from './tool-hub/ManualServerModal';
import { useToolHub } from './tool-hub/useToolHub';

export interface ToolHubProps {
  servers: McpServerRecord[];
  tools: McpTool[];
  onRefresh: () => void;
  onUpdatePolicy: (
    toolName: string,
    policy: 'AUTONOMOUS' | 'REQUIRE_APPROVAL' | 'BLOCKED',
    riskLevel?: string
  ) => void;
}

export const ToolHub: React.FC<ToolHubProps> = ({
  servers,
  tools,
  onRefresh,
  onUpdatePolicy,
}) => {
  const {
    showAddModal,
    setShowAddModal,
    serverName,
    setServerName,
    transportType,
    setTransportType,
    command,
    setCommand,
    argsStr,
    setArgsStr,
    socketPath,
    setSocketPath,
    url,
    setUrl,
    testingTool,
    setTestingTool,
    testArgs,
    setTestArgs,
    testOutput,
    isCalling,
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
    handleTriggerSimulatedInbound,
    handleAddServer,
    handleRemoveServer,
    handleRunToolTest,
  } = useToolHub(servers, onRefresh);

  return (
    <div className="flex h-full w-full overflow-hidden bg-styx-950 font-sans text-xs">
      <div className="flex-1 flex flex-col h-full min-h-0 overflow-y-auto p-4 space-y-4 max-w-7xl mx-auto">
        {/* Header */}
        <div className="flex flex-col sm:flex-row sm:items-center justify-between border-b border-styx-800 pb-3 gap-3 font-mono">
          <div className="flex items-center space-x-3">
            <div className="p-2 rounded-lg bg-emerald-950/80 border border-emerald-800 text-emerald-400 flex-shrink-0">
              <Layers className="w-5 h-5" />
            </div>
            <div>
              <div className="flex flex-wrap items-center gap-1.5 sm:gap-2">
                <h2 className="text-sm font-bold text-slate-100">DYNAMIC TOOL BUS (MCP 2024-11-05)</h2>
                <span className="px-1.5 py-0.5 rounded bg-cyan-950 text-cyan-400 border border-cyan-800 text-[10px] font-bold">
                  Drag & Drop Enabled
                </span>
              </div>
              <p className="text-[11px] text-slate-400">
                Plug in external micro-daemons via stdio, Unix sockets, or HTTP with deterministic policy gating.
              </p>
            </div>
          </div>
          <div className="flex items-center space-x-2 self-start sm:self-auto">
            <button
              type="button"
              onClick={() => setShowAddModal(true)}
              className="px-3 py-1.5 rounded bg-styx-800 hover:bg-styx-700 text-slate-200 border border-styx-700 font-bold flex items-center space-x-1.5 shadow text-xs active:scale-95 transition-all"
            >
              <Plus className="w-3.5 h-3.5" />
              <span>Manual Setup</span>
            </button>
          </div>
        </div>

        {/* Status Notification Banner */}
        {statusMessage && (
          <div
            className={`p-3 rounded-lg border font-mono text-xs flex items-center justify-between shadow-lg ${
              statusMessage.type === 'success'
                ? 'bg-emerald-950/80 border-emerald-700 text-emerald-200'
                : statusMessage.type === 'error'
                ? 'bg-rose-950/80 border-rose-700 text-rose-200'
                : 'bg-cyan-950/80 border-cyan-700 text-cyan-200'
            }`}
          >
            <div className="flex items-center space-x-2">
              {statusMessage.type === 'success' && <CheckCircle className="w-4 h-4 text-emerald-400" />}
              {statusMessage.type === 'error' && <AlertTriangle className="w-4 h-4 text-rose-400" />}
              {statusMessage.type === 'info' && <Sparkles className="w-4 h-4 text-cyan-400" />}
              <span>{statusMessage.text}</span>
            </div>
            <button
              type="button"
              onClick={() => setStatusMessage(null)}
              className="text-slate-400 hover:text-slate-100 p-1"
            >
              <X className="w-3.5 h-3.5" />
            </button>
          </div>
        )}

        {/* DRAG AND DROP TOOL INSTALLER ZONE (HERO DROPZONE) */}
        <DragDropZone
          isDragging={isDragging}
          inputPath={inputPath}
          isInspecting={isInspecting}
          fileInputRef={fileInputRef}
          onDragOver={handleDragOver}
          onDragLeave={handleDragLeave}
          onDrop={handleDrop}
          onFileInputChange={handleFileInputChange}
          onInspectPath={handleInspectPath}
          onChangeInputPath={setInputPath}
        />

        {/* INSPECTED TOOL DEPLOYMENT CARD */}
        {inspectedTool && (
          <ToolInspectionCard
            inspectedTool={inspectedTool}
            isInstalling={isInstalling}
            onDismiss={() => setInspectedTool(null)}
            onInstall={handleInstallTool}
          />
        )}

        {/* CONNECTED MCP SERVERS */}
        <ConnectedServers
          servers={servers}
          simulatingTrigger={simulatingTrigger}
          onTriggerSimulatedInbound={handleTriggerSimulatedInbound}
          onRemoveServer={handleRemoveServer}
        />

        {/* THREE-TIER POLICY MATRIX & TOOL CATALOG */}
        <PolicyMatrix
          tools={tools}
          onUpdatePolicy={onUpdatePolicy}
          onSelectTestingTool={t => {
            setTestingTool(t);
            setTestArgs(
              t.input_schema?.properties
                ? JSON.stringify(
                    Object.keys(t.input_schema.properties).reduce((acc: any, k) => {
                      acc[k] = '';
                      return acc;
                    }, {}),
                    null,
                    2
                  )
                : '{}'
            );
          }}
        />

        {/* TEST CALL DRAWER */}
        {testingTool && (
          <TestToolDrawer
            testingTool={testingTool}
            testArgs={testArgs}
            testOutput={testOutput}
            isCalling={isCalling}
            onChangeArgs={setTestArgs}
            onRunTest={handleRunToolTest}
            onClose={() => setTestingTool(null)}
          />
        )}
      </div>

      {/* ADD INTEGRATION MODAL */}
      {showAddModal && (
        <ManualServerModal
          serverName={serverName}
          transportType={transportType}
          command={command}
          argsStr={argsStr}
          socketPath={socketPath}
          url={url}
          onChangeServerName={setServerName}
          onChangeTransportType={setTransportType}
          onChangeCommand={setCommand}
          onChangeArgsStr={setArgsStr}
          onChangeSocketPath={setSocketPath}
          onChangeUrl={setUrl}
          onSubmit={handleAddServer}
          onClose={() => setShowAddModal(false)}
        />
      )}
    </div>
  );
};
