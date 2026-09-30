import { useState, useEffect } from 'react';
import { ContainerSummaryInfo, EnginePreset, ModelConfigRecord, SystemTelemetry } from '../../types';
import { calculateTargetQuantization, safeFetchJson } from './vram';
import { LocalGgufFile } from './types';
import { useExternalModels } from './useExternalModels';
import { useContainerLogs } from './useContainerLogs';

export interface UseModelManagerProps {
  containers?: ContainerSummaryInfo[];
  presets?: EnginePreset[];
  modelConfigs?: ModelConfigRecord[];
  telemetry?: SystemTelemetry | null;
  onRefresh?: () => void;
}

export function useModelManager({
  containers = [],
  modelConfigs = [],
  telemetry = null,
  onRefresh = () => {},
}: UseModelManagerProps) {
  const runningContainers = containers.filter(c => c.state === 'running');
  const activeConfig = modelConfigs.find(cfg => cfg.is_active);

  const isContainerActive = (c: ContainerSummaryInfo) => {
    if (activeConfig) {
      const cfg = activeConfig;
      if (c.id === cfg.model_id || c.model_id === cfg.model_id) return true;
      const portMatch = c.ports?.some(p => cfg.base_url && cfg.base_url.includes(p.split(':')[0]));
      const nameMatch = c.names?.some(n => {
        const clean = n.replace('/', '').toLowerCase();
        return (cfg.model_id && clean.includes(cfg.model_id.toLowerCase())) ||
               (cfg.name && clean.includes(cfg.name.toLowerCase()));
      });
      return portMatch || nameMatch;
    }
    return false;
  };

  const activeContainer =
    runningContainers.find(c => isContainerActive(c)) ||
    runningContainers[0] ||
    containers[0];

  const hasActiveModel = Boolean(activeConfig || (activeContainer && activeContainer.state === 'running'));

  const [activeSubTab, setActiveSubTab] = useState<'docker' | 'external'>('docker');
  const [showAdvancedActive, setShowAdvancedActive] = useState<boolean>(false);
  const [showTechnicalLogs, setShowTechnicalLogs] = useState<boolean>(false);

  const detectedVram = telemetry?.gpu ? Math.round(telemetry.gpu.vram_total_mb / 1024) : 24;
  const [targetGpuVram, setTargetGpuVram] = useState<number>(detectedVram > 0 ? detectedVram : 24);
  const [selectedModelSize, setSelectedModelSize] = useState<number>(27);

  const [selectedEngine, setSelectedEngine] = useState<string>('vllm');
  const [containerName, setContainerName] = useState('qwen-3-5-9b');
  const [hfRepo, setHfRepo] = useState('QuantTrio/Qwen3.5-9B-AWQ');
  const [hfToken, setHfToken] = useState('');
  const [contextWindow, setContextWindow] = useState<number>(65536);
  const [gpuDevices, setGpuDevices] = useState('all');
  const [tensorParallel, setTensorParallel] = useState<number>(1);
  const [gpuMemoryUtil, setGpuMemoryUtil] = useState<number>(0.92);
  const [quantization, setQuantization] = useState('awq');
  const [kvCacheDtype, setKvCacheDtype] = useState<string>('fp8');
  const [maxNumSeqs, setMaxNumSeqs] = useState<number>(2);
  const [gpuVendor, setGpuVendor] = useState<'auto' | 'amd' | 'nvidia' | 'none'>('auto');
  const [customImage, setCustomImage] = useState<string>('');
  const [enableMtp, setEnableMtp] = useState<boolean>(true);
  const [speculativeModel, setSpeculativeModel] = useState('[mtp]');
  const [speculativeTokens, setSpeculativeTokens] = useState<number>(3);
  const [enableVision, setEnableVision] = useState<boolean>(true);
  const [port, setPort] = useState<number>(8000);
  const [previewCmd, setPreviewCmd] = useState<string>('');
  const [isDeploying, setIsDeploying] = useState<boolean>(false);
  const [localGgufFiles, setLocalGgufFiles] = useState<LocalGgufFile[]>([]);

  // Logs & Loading Progress
  const logsState = useContainerLogs(containers, activeContainer, isDeploying);

  // External Endpoints Form State
  const externalState = useExternalModels(onRefresh);

  useEffect(() => {
    fetchLocalFiles();
  }, []);

  const fetchLocalFiles = async () => {
    try {
      const res = await fetch('/api/models/local-files');
      const data = await safeFetchJson(res);
      if (data.success && data.files) setLocalGgufFiles(data.files);
    } catch (e) {
      console.warn('Failed to fetch local files:', e);
    }
  };

  useEffect(() => {
    updatePreview();
  }, [
    containerName, selectedEngine, hfRepo, contextWindow, gpuDevices, gpuVendor,
    customImage, tensorParallel, gpuMemoryUtil, quantization, kvCacheDtype,
    maxNumSeqs, speculativeModel, speculativeTokens, enableMtp, enableVision, port,
  ]);

  const updatePreview = async () => {
    try {
      const payload = {
        name: containerName,
        engine: selectedEngine === 'llamacpp' ? 'llama_cpp' : selectedEngine,
        hf_repo: hfRepo,
        hf_token: hfToken || undefined,
        context_window: contextWindow,
        gpu_devices: gpuDevices,
        gpu_vendor: gpuVendor,
        custom_image: customImage || undefined,
        tensor_parallel_size: tensorParallel,
        gpu_memory_utilization: gpuMemoryUtil,
        quantization: quantization === 'none' ? undefined : quantization,
        kv_cache_dtype: kvCacheDtype,
        max_num_seqs: maxNumSeqs,
        speculative_model: enableMtp ? (speculativeModel || '[mtp]') : undefined,
        num_speculative_tokens: enableMtp ? speculativeTokens : undefined,
        enable_mtp: enableMtp,
        enable_vision: enableVision,
        port,
      };

      const res = await fetch('/api/models/preview-command', {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify(payload),
      });
      const data = await safeFetchJson(res);
      if (data.success && data.command) setPreviewCmd(data.command);
    } catch (e) {
      console.error(e);
    }
  };

  const applyPreset = (preset: EnginePreset) => {
    setSelectedEngine(preset.engine);
    setHfRepo(preset.hf_repo);
    const ctx = preset.default_context || 65536;
    setContextWindow(ctx);
    setPort(preset.default_port);
    setTensorParallel(preset.default_tp);
    setContainerName(preset.id.replace(/[^a-zA-Z0-9]/g, '-'));
    setEnableMtp(preset.enable_mtp ?? true);
    setEnableVision(preset.enable_vision ?? true);
    setSpeculativeModel(preset.enable_mtp !== false ? '[mtp]' : '');
    setSpeculativeTokens(3);

    const size =
      preset.model_size_b ||
      (preset.id.includes('27b') ? 27 : preset.id.includes('31b') ? 31 : preset.id.includes('12b') ? 12 : preset.id.includes('9b') ? 9 : 14);
    setSelectedModelSize(size);

    const rec = calculateTargetQuantization(size, targetGpuVram, ctx);
    setQuantization(rec.recommendedQuant);
  };

  const handleSelectGpuSize = (vramGb: number) => {
    setTargetGpuVram(vramGb);
    const rec = calculateTargetQuantization(selectedModelSize, vramGb, contextWindow);
    setQuantization(rec.recommendedQuant);
  };

  const triggerDeployApi = async (payload: any, logContainerName: string) => {
    setIsDeploying(true);
    try {
      const res = await fetch('/api/models/deploy', {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify(payload),
      });
      const data = await safeFetchJson(res);
      if (data.success) {
        onRefresh();
        logsState.setSelectedLogContainer(data.container_id || logContainerName);
        logsState.fetchLogs(data.container_id || logContainerName);
      } else {
        alert('Deployment failed: ' + (data.error || 'Server error'));
      }
    } catch (e) {
      alert('Error triggering deployment: ' + (e instanceof Error ? e.message : String(e)));
    } finally {
      setIsDeploying(false);
    }
  };

  const handleDeploy = () => {
    const payload = {
      name: containerName,
      engine: selectedEngine === 'llamacpp' ? 'llama_cpp' : selectedEngine,
      hf_repo: hfRepo,
      hf_token: hfToken || undefined,
      context_window: contextWindow,
      gpu_devices: gpuDevices,
      gpu_vendor: gpuVendor,
      custom_image: customImage || undefined,
      tensor_parallel_size: tensorParallel,
      gpu_memory_utilization: gpuMemoryUtil,
      quantization: quantization === 'none' ? undefined : quantization,
      kv_cache_dtype: kvCacheDtype,
      max_num_seqs: maxNumSeqs,
      speculative_model: enableMtp ? (speculativeModel || '[mtp]') : undefined,
      num_speculative_tokens: enableMtp ? speculativeTokens : undefined,
      enable_mtp: enableMtp,
      enable_vision: enableVision,
      port,
    };
    triggerDeployApi(payload, containerName);
  };

  const handleFixDeploy16k = () => {
    setContextWindow(16384);
    setQuantization('awq');
    setSelectedEngine('vllm');
    setHfRepo('QuantTrio/Qwen3.5-9B-AWQ');
    setContainerName('qwen-3-5-9b');
    setPort(8000);
    const payload = {
      name: 'qwen-3-5-9b',
      engine: 'vllm',
      hf_repo: 'QuantTrio/Qwen3.5-9B-AWQ',
      context_window: 16384,
      gpu_devices: gpuDevices,
      gpu_vendor: gpuVendor,
      tensor_parallel_size: 1,
      gpu_memory_utilization: 0.92,
      quantization: 'awq',
      kv_cache_dtype: 'fp8',
      max_num_seqs: maxNumSeqs,
      enable_mtp: false,
      enable_vision: true,
      port: 8000,
    };
    triggerDeployApi(payload, 'qwen-3-5-9b');
  };

  const handleFixDeployLlama128k = () => {
    setContextWindow(131072);
    setKvCacheDtype('q4_0');
    setSelectedEngine('llamacpp');
    setHfRepo('/models/Qwen3.5-9B-UD-Q4_K_XL.gguf');
    setContainerName('qwen-3-5-9b-128k');
    setPort(8080);
    const payload = {
      name: 'qwen-3-5-9b-128k',
      engine: 'llama_cpp',
      hf_repo: '/models/Qwen3.5-9B-UD-Q4_K_XL.gguf',
      context_window: 131072,
      gpu_devices: gpuDevices,
      gpu_vendor: gpuVendor,
      tensor_parallel_size: 1,
      gpu_memory_utilization: 0.95,
      kv_cache_dtype: 'q4_0',
      enable_mtp: false,
      enable_vision: true,
      port: 8080,
    };
    triggerDeployApi(payload, 'qwen-3-5-9b-128k');
  };

  const handleActivateContainer = async (containerId: string) => {
    try {
      const res = await fetch(`/api/models/containers/${containerId}/activate`, { method: 'POST' });
      const data = await safeFetchJson(res);
      if (data.success) {
        onRefresh();
      } else {
        alert('Failed to activate container: ' + (data.error || 'Unknown error'));
      }
    } catch (e) {
      console.error(e);
    }
  };

  const handleContainerAction = async (id: string, action: 'start' | 'stop' | 'restart' | 'delete') => {
    try {
      const method = action === 'delete' ? 'DELETE' : 'POST';
      const endpoint = action === 'delete' ? `/api/models/containers/${id}` : `/api/models/containers/${id}/${action}`;
      await fetch(endpoint, { method });
      onRefresh();
    } catch (e) {
      console.error(e);
    }
  };

  const activeModelTitle =
    activeConfig?.name ||
    activeContainer?.names[0]?.replace('/', '') ||
    activeContainer?.model_id ||
    'Active Local Model';
  const activeModelPort =
    activeContainer?.ports[0]?.split('->')[0] || (activeContainer?.ports?.length ? activeContainer.ports[0] : '8080');

  return {
    runningContainers,
    activeConfig,
    activeContainer,
    hasActiveModel,
    activeSubTab,
    setActiveSubTab,
    showAdvancedActive,
    setShowAdvancedActive,
    showTechnicalLogs,
    setShowTechnicalLogs,
    targetGpuVram,
    selectedModelSize,
    selectedEngine,
    setSelectedEngine,
    containerName,
    setContainerName,
    hfRepo,
    setHfRepo,
    contextWindow,
    setContextWindow,
    port,
    setPort,
    quantization,
    setQuantization,
    kvCacheDtype,
    setKvCacheDtype,
    enableMtp,
    setEnableMtp,
    enableVision,
    setEnableVision,
    gpuVendor,
    setGpuVendor,
    previewCmd,
    isDeploying,
    localGgufFiles,
    applyPreset,
    handleSelectGpuSize,
    handleDeploy,
    handleFixDeploy16k,
    handleFixDeployLlama128k,
    handleActivateContainer,
    handleContainerAction,
    isContainerActive,
    activeModelTitle,
    activeModelPort,
    ...logsState,
    ...externalState,
  };
}
