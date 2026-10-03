import React from 'react';
import { Cpu, Zap, Eye, CheckCircle, Activity, HardDrive } from 'lucide-react';
import { EnginePreset, SystemTelemetry } from '../../types';
import { calculateTargetQuantization } from './vram';

export interface PresetsCardProps {
  presets: EnginePreset[];
  telemetry?: SystemTelemetry | null;
  targetGpuVram: number;
  isDeploying: boolean;
  onApplyPreset: (p: EnginePreset) => void;
  onSelectGpuSize: (vram: number) => void;
  onFixDeployLlama128k: () => void;
  onFixDeploy16k: () => void;
}

export const PresetsCard: React.FC<PresetsCardProps> = ({
  presets,
  telemetry,
  targetGpuVram,
  isDeploying,
  onApplyPreset,
  onSelectGpuSize,
  onFixDeployLlama128k,
  onFixDeploy16k,
}) => {
  const targetRec = calculateTargetQuantization(27, targetGpuVram, 65536);

  return (
    <div className="space-y-4 font-mono">
      {/* GPU Hardware Card & Quantization Advisor */}
      <div className="bg-syndae-900 border border-syndae-800 rounded-lg p-3.5 space-y-3">
        <div className="flex flex-col sm:flex-row sm:items-center justify-between gap-2">
          <div className="flex items-center space-x-2">
            <Activity className="w-4 h-4 text-cyan-400" />
            <div>
              <div className="text-xs font-bold text-slate-200">
                {telemetry?.gpu
                  ? `Hardware: ${telemetry.gpu.name} (${Math.round(telemetry.gpu.vram_total_mb / 1024)} GB VRAM • ${telemetry.gpu.vendor === 'amd' ? 'AMD ROCm' : 'NVIDIA CUDA'})`
                  : 'Hardware: AMD ROCm Hardware (16 GB VRAM)'}
              </div>
              <div className="text-[10px] text-slate-400 font-sans">
                GPU-Targeted Quantization Advisor adjusts parameters to fit your VRAM budget.
              </div>
            </div>
          </div>

          {/* GPU Size Selector Buttons */}
          <div className="flex flex-wrap items-center gap-1">
            <span className="text-[10px] text-slate-400 mr-1">VRAM:</span>
            {[
              { label: '12 GB', vram: 12 },
              { label: '16 GB', vram: 16 },
              { label: '24 GB (Standard)', vram: 24 },
              { label: '48 GB+ (Datacenter)', vram: 48 },
            ].map(tier => (
              <button
                key={tier.vram}
                type="button"
                onClick={() => onSelectGpuSize(tier.vram)}
                className={`px-2 py-0.5 rounded text-xs transition-colors ${
                  targetGpuVram === tier.vram
                    ? 'bg-cyan-900 text-cyan-200 border border-cyan-500 font-bold'
                    : 'bg-syndae-950 text-slate-400 hover:text-slate-200 border border-syndae-800'
                }`}
              >
                {tier.label}
              </button>
            ))}
          </div>
        </div>

        {/* Quantization Advisor Bar */}
        <div className="p-2 rounded bg-syndae-950 border border-syndae-800 text-[11px] space-y-1">
          <div className="flex items-center justify-between">
            <div className="flex items-center space-x-1.5">
              <span className="text-slate-400 font-bold">GPU-Targeted Quantization Advisor:</span>
              <span className="text-slate-400">Optimal Quantization:</span>
              <span className="px-1.5 py-0.5 rounded bg-emerald-950 text-emerald-300 font-bold border border-emerald-800 text-[10px]">
                {targetRec.recommendedQuant.toUpperCase()}
              </span>
            </div>
            <span className="font-bold text-emerald-400 flex items-center space-x-1 text-[10px]">
              <CheckCircle className="w-3 h-3" />
              <span>Fits in {targetGpuVram}GB</span>
            </span>
          </div>
          <div className="text-[10px] text-slate-400 font-sans">{targetRec.rationale}</div>
        </div>
      </div>

      {/* 1-Click Launchpad */}
      <div className="grid grid-cols-1 sm:grid-cols-2 gap-2">
        <div
          onClick={onFixDeployLlama128k}
          className="p-2.5 rounded bg-syndae-900 border border-emerald-500/40 hover:border-emerald-400 cursor-pointer space-y-1.5"
        >
          <div className="flex items-center justify-between text-xs font-bold text-slate-100">
            <span>Preset A: Qwen 3.5 9B (128k GGUF)</span>
            <span className="text-[9px] px-1 rounded bg-emerald-950 text-emerald-300">LOCAL DISK</span>
          </div>
          <div className="text-[10px] text-slate-400 truncate">/models/Qwen3.5-9B-UD-Q4_K_XL.gguf</div>
          <button
            type="button"
            disabled={isDeploying}
            className="w-full py-1 rounded bg-emerald-600 hover:bg-emerald-500 text-white font-bold text-xs flex items-center justify-center space-x-1"
          >
            <Zap className="w-3 h-3 text-amber-300" />
            <span>⚡ 1-Click Launch (128k Local)</span>
          </button>
        </div>

        <div
          onClick={onFixDeploy16k}
          className="p-2.5 rounded bg-syndae-900 border border-cyan-500/40 hover:border-cyan-400 cursor-pointer space-y-1.5"
        >
          <div className="flex items-center justify-between text-xs font-bold text-slate-100">
            <span>Preset B: Qwen 3.5 9B (16k AWQ)</span>
            <span className="text-[9px] px-1 rounded bg-cyan-950 text-cyan-300">vLLM FAST</span>
          </div>
          <div className="text-[10px] text-slate-400 truncate">QuantTrio/Qwen3.5-9B-AWQ</div>
          <button
            type="button"
            disabled={isDeploying}
            className="w-full py-1 rounded bg-syndae-800 hover:bg-syndae-750 text-slate-100 border border-syndae-700 font-bold text-xs flex items-center justify-center space-x-1"
          >
            <HardDrive className="w-3 h-3 text-cyan-300" />
            <span>1-Click Launch (16k vLLM)</span>
          </button>
        </div>
      </div>

      {/* Preset Inference Templates Grid */}
      <div className="bg-syndae-900 border border-syndae-800 rounded-lg p-3 space-y-2">
        <div className="text-xs font-bold text-slate-300 flex items-center justify-between">
          <div className="flex items-center space-x-1.5">
            <Cpu className="w-3.5 h-3.5 text-emerald-400" />
            <span>PRESET INFERENCE TEMPLATES</span>
          </div>
          <span className="text-[10px] text-emerald-400">64k Context • MTP ON • Vision Ready</span>
        </div>

        <div className="grid grid-cols-1 md:grid-cols-2 gap-2">
          {presets.map(p => (
            <div
              key={p.id}
              onClick={() => onApplyPreset(p)}
              className="p-2 rounded bg-syndae-950 border border-syndae-800 hover:border-emerald-500/50 cursor-pointer space-y-1"
            >
              <div className="flex items-center justify-between">
                <span className="font-bold text-slate-200 text-xs">{p.label}</span>
                <span className="text-[10px] px-1.5 py-0.2 rounded bg-syndae-850 text-cyan-300 uppercase">
                  {p.engine}
                </span>
              </div>
              <p className="text-[10px] text-slate-400 line-clamp-2">{p.description}</p>
              <div className="flex flex-wrap items-center gap-1 pt-1">
                <span className="px-1 py-0.2 rounded bg-emerald-950 text-emerald-300 border border-emerald-800 text-[9px]">
                  64k Context
                </span>
                <span className="px-1 py-0.2 rounded bg-amber-950 text-amber-300 border border-amber-800 text-[9px] flex items-center space-x-0.5">
                  <Zap className="w-2.5 h-2.5 text-amber-400" />
                  <span>MTP ⚡</span>
                </span>
                <span className="px-1 py-0.2 rounded bg-purple-950 text-purple-300 border border-purple-800 text-[9px] flex items-center space-x-0.5">
                  <Eye className="w-2.5 h-2.5 text-purple-400" />
                  <span>Image Input 🖼️</span>
                </span>
              </div>
            </div>
          ))}
        </div>
      </div>
    </div>
  );
};
