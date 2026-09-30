import { ContainerSummaryInfo, EnginePreset, ModelConfigRecord, SystemTelemetry } from '../../types';

export interface ModelManagerProps {
  containers?: ContainerSummaryInfo[];
  presets?: EnginePreset[];
  modelConfigs?: ModelConfigRecord[];
  telemetry?: SystemTelemetry | null;
  onRefresh?: () => void;
  onSelectActiveModel?: (id: string) => void;
  onNavigateToChat?: () => void;
}

export type LoadingStage = 'idle' | 'downloading' | 'loading_weights' | 'optimizing' | 'ready' | 'error';

export interface LoadingProgressInfo {
  stage: LoadingStage;
  stageIndex: number; // 0: idle/error, 1: downloading, 2: loading_weights, 3: optimizing, 4: ready
  progressPct: number;
  title: string;
  description: string;
  errorKind?: 'oom' | 'invalid_arg' | 'repo_not_found' | 'crashed' | 'unknown';
  errorHeadline?: string;
  errorDetail?: string;
}

export interface LocalGgufFile {
  filename: string;
  path: string;
  size_gb: number;
  is_mmproj: boolean;
  is_draft: boolean;
}
