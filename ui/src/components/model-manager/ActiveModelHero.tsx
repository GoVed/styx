import React from 'react';
import { Zap, MessageSquare, Terminal, ChevronDown, ChevronUp, RotateCcw } from 'lucide-react';
import { ContainerSummaryInfo, ModelConfigRecord, SystemTelemetry } from '../../types';
import { ReasoningSelector } from './ReasoningSelector';

interface ActiveModelHeroProps {
  activeModelTitle: string;
  activeModelPort: number | string;
  activeConfig?: ModelConfigRecord;
  activeContainer?: ContainerSummaryInfo | null;
  telemetry?: SystemTelemetry | null;
  showAdvancedActive: boolean;
  onToggleAdvanced: () => void;
  onNavigateToChat: () => void;
  onLoadDifferentModel: () => void;
}

export const ActiveModelHero: React.FC<ActiveModelHeroProps> = ({
  activeModelTitle,
  activeModelPort,
  activeConfig,
  activeContainer,
  telemetry,
  showAdvancedActive,
  onToggleAdvanced,
  onNavigateToChat,
  onLoadDifferentModel,
}) => {
  return (
    <div className="bg-gradient-to-r from-emerald-950/80 via-syndae-900 to-cyan-950/80 border-2 border-emerald-500/70 rounded-xl p-5 shadow-2xl space-y-4">
      <div className="flex flex-col sm:flex-row sm:items-center justify-between gap-3">
        <div className="flex items-center space-x-3">
          <div className="w-10 h-10 rounded-xl bg-emerald-500/20 border border-emerald-500/40 flex items-center justify-center text-emerald-400">
            <Zap className="w-6 h-6 text-emerald-300 animate-pulse" />
          </div>
          <div>
            <div className="flex items-center space-x-2">
              <h3 className="text-base font-bold text-slate-100">{activeModelTitle}</h3>
              <span className="px-2 py-0.5 rounded-full text-[10px] font-bold font-mono bg-emerald-500/20 text-emerald-300 border border-emerald-500/50 flex items-center space-x-1">
                <span className="w-1.5 h-1.5 rounded-full bg-emerald-400 animate-ping mr-1" />
                <span>ACTIVE • RUNNING</span>
              </span>
            </div>
            <p className="text-xs text-slate-300 font-mono mt-0.5">
              Port {activeModelPort} • OpenAI-Compatible API Endpoint
            </p>
          </div>
        </div>

        {/* Guide User to Chat Screen */}
        <button
          type="button"
          onClick={onNavigateToChat}
          className="px-5 py-2.5 rounded-lg bg-emerald-600 hover:bg-emerald-500 text-white font-bold text-xs flex items-center justify-center space-x-2 shadow-lg transition-transform active:scale-95"
        >
          <MessageSquare className="w-4 h-4 text-emerald-100" />
          <span>💬 Go to Chat Screen →</span>
        </button>
      </div>

      {/* Key Specifications Grid */}
      <div className="grid grid-cols-2 sm:grid-cols-4 gap-2.5 font-mono text-xs">
        <div className="p-3 rounded-lg bg-syndae-950/90 border border-syndae-800 space-y-1">
          <div className="text-[10px] text-slate-400 uppercase">Context Length</div>
          <div className="font-bold text-slate-100">
            {activeConfig?.context_length
              ? `${(activeConfig.context_length / 1024).toFixed(0)}k (${activeConfig.context_length.toLocaleString()})`
              : '128k (131,072)'}
          </div>
        </div>
        <div className="p-3 rounded-lg bg-syndae-950/90 border border-syndae-800 space-y-1">
          <div className="text-[10px] text-slate-400 uppercase">Inference Engine</div>
          <div className="font-bold text-cyan-300">
            {activeContainer?.image.includes('llama') ? 'llama.cpp ROCm' : 'vLLM OpenAI'}
          </div>
        </div>
        <div className="p-3 rounded-lg bg-syndae-950/90 border border-syndae-800 space-y-1">
          <div className="text-[10px] text-slate-400 uppercase">GPU VRAM Usage</div>
          <div className="font-bold text-emerald-300">
            {telemetry?.gpu
              ? `~${Math.round(telemetry.gpu.vram_used_mb / 1024)} GB / ${Math.round(telemetry.gpu.vram_total_mb / 1024)} GB`
              : '~9.3 GB / 16 GB'}
          </div>
        </div>
        <div className="p-3 rounded-lg bg-syndae-950/90 border border-syndae-800 space-y-1">
          <div className="text-[10px] text-slate-400 uppercase">Vision & MTP</div>
          <div className="font-bold text-purple-300">
            Image Input 🖼️ • Flash Attention
          </div>
        </div>
      </div>

      {/* Model Reasoning Level Selector */}
      <ReasoningSelector />

      {/* Two User-Requested Options: Advance Info & Logs / Load a Different Model */}
      <div className="flex flex-col sm:flex-row items-center justify-between gap-2 pt-2 border-t border-syndae-800/80 font-mono">
        <button
          type="button"
          onClick={onToggleAdvanced}
          className="w-full sm:w-auto px-4 py-2 rounded-lg bg-syndae-850 hover:bg-syndae-800 text-slate-200 border border-syndae-700 font-semibold text-xs flex items-center justify-center space-x-2 transition-colors"
        >
          <Terminal className="w-3.5 h-3.5 text-purple-400" />
          <span>{showAdvancedActive ? 'Hide Advance Info & Logs' : 'Advance Info & Logs'}</span>
          {showAdvancedActive ? <ChevronUp className="w-3.5 h-3.5" /> : <ChevronDown className="w-3.5 h-3.5" />}
        </button>

        <button
          type="button"
          onClick={onLoadDifferentModel}
          className="w-full sm:w-auto px-4 py-2 rounded-lg bg-cyan-950/80 hover:bg-cyan-900 text-cyan-200 border border-cyan-700 font-bold text-xs flex items-center justify-center space-x-2 transition-all"
        >
          <RotateCcw className="w-3.5 h-3.5 text-cyan-300" />
          <span>Load a Different Model</span>
        </button>
      </div>
    </div>
  );
};
