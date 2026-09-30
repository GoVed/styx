import React from 'react';
import { Box, Layers, AlertTriangle } from 'lucide-react';
import { ContainerSummaryInfo, EnginePreset, ModelConfigRecord, SystemTelemetry } from '../types';
import {
  calculateModelWeightsGb,
  calculateKvCacheGb,
  calculateTargetQuantization,
  parseContainerLoadingStage,
  safeFetchJson,
  LLAMA_CPP_KV_OPTIONS,
  VLLM_KV_OPTIONS,
} from './model-manager/vram';
import { ModelManagerProps, LoadingStage, LoadingProgressInfo, LocalGgufFile } from './model-manager/types';
import { useModelManager } from './model-manager/useModelManager';
import { ActiveModelHero } from './model-manager/ActiveModelHero';
import { PresetsCard } from './model-manager/PresetsCard';
import { ProvisionerCard } from './model-manager/ProvisionerCard';
import { LocalContainersCard } from './model-manager/LocalContainersCard';
import { DeploymentStepper } from './model-manager/DeploymentStepper';
import { CloudModelConfigs } from './model-manager/CloudModelConfigs';

export {
  calculateModelWeightsGb,
  calculateKvCacheGb,
  calculateTargetQuantization,
  parseContainerLoadingStage,
  safeFetchJson,
  LLAMA_CPP_KV_OPTIONS,
  VLLM_KV_OPTIONS,
};
export type { ModelManagerProps, LoadingStage, LoadingProgressInfo, LocalGgufFile };

export const ModelManager: React.FC<ModelManagerProps> = ({
  containers = [],
  presets = [],
  modelConfigs = [],
  telemetry = null,
  onRefresh = () => {},
  onSelectActiveModel = () => {},
  onNavigateToChat = () => {},
}) => {
  const m = useModelManager({
    containers,
    presets,
    modelConfigs,
    telemetry,
    onRefresh,
  });

  return (
    <div className="flex-1 overflow-y-auto p-4 sm:p-6 bg-styx-950 text-slate-100 flex flex-col items-center">
      <div className="w-full max-w-6xl space-y-4">
        {/* Navigation Bar & Header */}
        <div className="flex flex-col sm:flex-row sm:items-center justify-between pb-3 border-b border-styx-800 gap-3">
          <div>
            <div className="flex items-center space-x-2">
              <h2 className="text-xl font-bold tracking-tight text-slate-100 font-mono">
                MODEL INFERENCE HUB
              </h2>
              {m.runningContainers.length > 0 ? (
                <span className="px-2 py-0.5 rounded text-[10px] font-bold bg-emerald-950 text-emerald-300 border border-emerald-800 font-mono">
                  {m.runningContainers.length} LOCAL ENGINE RUNNING
                </span>
              ) : (
                <span className="px-2 py-0.5 rounded text-[10px] font-bold bg-amber-950 text-amber-300 border border-amber-800 font-mono flex items-center space-x-1">
                  <AlertTriangle className="w-3 h-3 text-amber-400" />
                  <span>LOCAL ENGINES OFFLINE</span>
                </span>
              )}
            </div>
            <p className="text-xs text-slate-400 font-sans mt-0.5">
              Deploy and tune local LLMs or connect cloud models.
            </p>
          </div>

          <div className="flex items-center bg-styx-900 p-0.5 rounded border border-styx-700">
            <button
              onClick={() => m.setActiveSubTab('docker')}
              className={`px-3 py-1.5 rounded flex items-center space-x-1.5 ${
                m.activeSubTab === 'docker'
                  ? 'bg-styx-800 text-emerald-400 font-bold'
                  : 'text-slate-400 hover:text-slate-200'
              }`}
            >
              <Box className="w-3.5 h-3.5" />
              <span>Local Docker Provisioner</span>
            </button>
            <button
              onClick={() => m.setActiveSubTab('external')}
              className={`px-3 py-1.5 rounded flex items-center space-x-1.5 ${
                m.activeSubTab === 'external'
                  ? 'bg-styx-800 text-cyan-400 font-bold'
                  : 'text-slate-400 hover:text-slate-200'
              }`}
            >
              <Layers className="w-3.5 h-3.5" />
              <span>External & Cloud Providers</span>
            </button>
          </div>
        </div>

        {m.activeSubTab === 'docker' ? (
          <div className="space-y-4">
            {/* Scenario 2: Active Model Hero */}
            {m.hasActiveModel && (
              <ActiveModelHero
                activeModelTitle={m.activeModelTitle}
                activeModelPort={m.activeModelPort}
                activeConfig={m.activeConfig}
                activeContainer={m.activeContainer}
                telemetry={telemetry}
                showAdvancedActive={m.showAdvancedActive}
                onToggleAdvanced={() => m.setShowAdvancedActive(!m.showAdvancedActive)}
                onNavigateToChat={onNavigateToChat}
                onLoadDifferentModel={() => {
                  const el = document.getElementById('provisioner-section');
                  el?.scrollIntoView({ behavior: 'smooth' });
                }}
              />
            )}

            {/* Stepper if deploying or starting up */}
            {(m.isDeploying || (m.progressInfo.stage !== 'idle' && m.progressInfo.stage !== 'ready')) && (
              <DeploymentStepper
                progressInfo={m.progressInfo}
                containerLogs={m.containerLogs}
                activeContainerObj={m.activeContainerObj}
                showTechnicalLogs={m.showTechnicalLogs}
                isDeploying={m.isDeploying}
                onToggleTechnicalLogs={() => m.setShowTechnicalLogs(!m.showTechnicalLogs)}
                onFixDeploy16k={m.handleFixDeploy16k}
                onFixDeployLlama128k={m.handleFixDeployLlama128k}
                onNavigateToChat={onNavigateToChat}
                onPrev={() => {}}
              />
            )}

            {/* 2-Column Grid: Left (Presets & Provisioner), Right (Containers & Logs) */}
            <div id="provisioner-section" className="grid grid-cols-1 lg:grid-cols-12 gap-4">
              <div className="lg:col-span-7 space-y-4">
                <PresetsCard
                  presets={presets}
                  telemetry={telemetry}
                  targetGpuVram={m.targetGpuVram}
                  isDeploying={m.isDeploying}
                  onApplyPreset={m.applyPreset}
                  onSelectGpuSize={m.handleSelectGpuSize}
                  onFixDeployLlama128k={m.handleFixDeployLlama128k}
                  onFixDeploy16k={m.handleFixDeploy16k}
                />

                <ProvisionerCard
                  containerName={m.containerName}
                  selectedEngine={m.selectedEngine}
                  hfRepo={m.hfRepo}
                  contextWindow={m.contextWindow}
                  port={m.port}
                  quantization={m.quantization}
                  kvCacheDtype={m.kvCacheDtype}
                  enableMtp={m.enableMtp}
                  enableVision={m.enableVision}
                  gpuVendor={m.gpuVendor}
                  previewCmd={m.previewCmd}
                  isDeploying={m.isDeploying}
                  selectedModelSize={m.selectedModelSize}
                  onChangeContainerName={m.setContainerName}
                  onChangeEngine={m.setSelectedEngine}
                  onChangeHfRepo={m.setHfRepo}
                  onChangeContextWindow={m.setContextWindow}
                  onChangePort={m.setPort}
                  onChangeQuantization={m.setQuantization}
                  onChangeKvCacheDtype={m.setKvCacheDtype}
                  onChangeEnableMtp={m.setEnableMtp}
                  onChangeEnableVision={m.setEnableVision}
                  onChangeGpuVendor={m.setGpuVendor}
                  onDeploy={m.handleDeploy}
                  onSelectQuickModel={model => {
                    m.setHfRepo(model.repo);
                    m.setContextWindow(model.context);
                    m.setContainerName(model.label.toLowerCase().replace(/[^a-z0-9]/g, '-'));
                  }}
                />
              </div>

              <div className="lg:col-span-5">
                <LocalContainersCard
                  containers={containers}
                  selectedLogContainer={m.selectedLogContainer}
                  containerLogs={m.containerLogs}
                  isContainerActive={m.isContainerActive}
                  onRefresh={onRefresh}
                  onSelectLogContainer={id => {
                    m.setSelectedLogContainer(id);
                    m.fetchLogs(id);
                  }}
                  onActivateContainer={m.handleActivateContainer}
                  onContainerAction={m.handleContainerAction}
                />
              </div>
            </div>
          </div>
        ) : (
          <CloudModelConfigs
            extName={m.extName}
            extProvider={m.extProvider}
            extBaseUrl={m.extBaseUrl}
            extApiKey={m.extApiKey}
            extModelId={m.extModelId}
            extContextLength={m.extContextLength}
            testResult={m.testResult}
            isTesting={m.isTesting}
            onChangeName={m.setExtName}
            onChangeProvider={m.setExtProvider}
            onChangeBaseUrl={m.setExtBaseUrl}
            onChangeApiKey={m.setExtApiKey}
            onChangeModelId={m.setExtModelId}
            onChangeContextLength={m.setExtContextLength}
            onTestConnection={m.handleTestConnection}
            onSaveExternalConfig={m.handleSaveExternalConfig}
          />
        )}
      </div>
    </div>
  );
};
