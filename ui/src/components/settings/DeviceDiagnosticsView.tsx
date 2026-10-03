import React from 'react';
import { Cpu, MemoryStick, Zap, Activity, Sliders } from 'lucide-react';
import { SystemTelemetry } from '../../types';

export interface DeviceDiagnosticsViewProps {
  telemetry: SystemTelemetry | null;
  onBack: () => void;
  onOpenAudit: () => void;
}

export const DeviceDiagnosticsView: React.FC<DeviceDiagnosticsViewProps> = ({
  telemetry,
  onBack,
  onOpenAudit,
}) => {
  const cpu = telemetry?.host_cpu_pct ?? 0;
  const memMb = telemetry?.memory_used_mb ?? 0;
  const memTotal = telemetry?.memory_total_mb ?? 1;
  const memPct = telemetry?.memory_pct ?? 0;
  const tokSec = telemetry?.tokens_per_second ?? 0;
  const gpu = telemetry?.gpu;

  return (
    <div className="space-y-4">
      <div className="flex items-center justify-between pb-2 border-b border-syndae-800">
        <div>
          <h2 className="text-sm font-bold text-white">Device Performance & Health</h2>
          <p className="text-xs text-slate-400 font-mono">Real-time device responsiveness and memory</p>
        </div>
        <span className="px-2 py-0.5 rounded-full text-[10px] font-mono bg-emerald-950 text-emerald-400 border border-emerald-800">
          LIVE
        </span>
      </div>

      <div className="grid grid-cols-1 sm:grid-cols-2 gap-3">
        {/* CPU Card */}
        <div className="p-4 rounded-2xl bg-syndae-900 border border-syndae-800 space-y-2">
          <div className="flex items-center justify-between text-xs font-mono text-slate-400">
            <span className="flex items-center space-x-1.5 text-emerald-400 font-bold">
              <Cpu className="w-4 h-4" />
              <span>PROCESSOR (CPU)</span>
            </span>
            <span>{telemetry?.host_cpu_cores ?? 0} Cores</span>
          </div>
          <div className="flex items-baseline justify-between">
            <span className="text-2xl font-bold text-white font-mono">{cpu.toFixed(1)}%</span>
            <span className="text-xs text-slate-400 font-mono">Usage</span>
          </div>
          <div className="w-full h-2 bg-syndae-950 rounded-full overflow-hidden">
            <div
              className={`h-full transition-all duration-500 ${cpu > 80 ? 'bg-rose-500' : 'bg-emerald-500'}`}
              style={{ width: `${Math.min(100, Math.max(2, cpu))}%` }}
            />
          </div>
        </div>

        {/* Memory Card */}
        <div className="p-4 rounded-2xl bg-syndae-900 border border-syndae-800 space-y-2">
          <div className="flex items-center justify-between text-xs font-mono text-slate-400">
            <span className="flex items-center space-x-1.5 text-cyan-400 font-bold">
              <MemoryStick className="w-4 h-4" />
              <span>DEVICE MEMORY (RAM)</span>
            </span>
            <span>{(memTotal / 1024).toFixed(0)} GB Total</span>
          </div>
          <div className="flex items-baseline justify-between">
            <span className="text-2xl font-bold text-white font-mono">
              {(memMb / 1024).toFixed(1)} GB
            </span>
            <span className="text-xs text-slate-400 font-mono">{memPct.toFixed(0)}% used</span>
          </div>
          <div className="w-full h-2 bg-syndae-950 rounded-full overflow-hidden">
            <div
              className="h-full bg-cyan-500 transition-all duration-500"
              style={{ width: `${Math.min(100, Math.max(2, memPct))}%` }}
            />
          </div>
        </div>

        {/* GPU / VRAM Card */}
        {gpu && (
          <div className="p-4 rounded-2xl bg-syndae-900 border border-syndae-800 space-y-2 sm:col-span-2">
            <div className="flex items-center justify-between text-xs font-mono text-slate-400">
              <span className="flex items-center space-x-1.5 text-purple-400 font-bold">
                <Zap className="w-4 h-4" />
                <span>{gpu.name}</span>
              </span>
              <span className="px-1.5 py-0.5 rounded bg-purple-950/80 text-purple-300 border border-purple-800 text-[10px]">
                Graphics & AI Core
              </span>
            </div>
            <div className="flex items-baseline justify-between">
              <span className="text-2xl font-bold text-white font-mono">
                {(gpu.vram_used_mb / 1024).toFixed(1)} / {(gpu.vram_total_mb / 1024).toFixed(1)} GB
              </span>
              <span className="text-xs text-slate-400 font-mono">
                {gpu.vram_pct.toFixed(0)}% Memory • AI Core {gpu.gpu_util_pct}%
              </span>
            </div>
            <div className="w-full h-2 bg-syndae-950 rounded-full overflow-hidden">
              <div
                className="h-full bg-purple-500 transition-all duration-500"
                style={{ width: `${Math.min(100, Math.max(2, gpu.vram_pct))}%` }}
              />
            </div>
          </div>
        )}

        {/* Tokens & Queue */}
        <div className="p-4 rounded-2xl bg-syndae-900 border border-syndae-800 space-y-2">
          <div className="flex items-center justify-between text-xs font-mono text-slate-400">
            <span className="flex items-center space-x-1.5 text-amber-400 font-bold">
              <Activity className="w-4 h-4" />
              <span>TYPING SPEED</span>
            </span>
            <span>Responsiveness</span>
          </div>
          <div className="flex items-baseline justify-between">
            <span className="text-2xl font-bold text-white font-mono">{tokSec.toFixed(1)}</span>
            <span className="text-xs text-slate-400 font-mono">words/sec</span>
          </div>
        </div>

        {/* Turn Queue Status */}
        <div className="p-4 rounded-2xl bg-syndae-900 border border-syndae-800 space-y-2">
          <div className="flex items-center justify-between text-xs font-mono text-slate-400">
            <span className="flex items-center space-x-1.5 text-emerald-400 font-bold">
              <Sliders className="w-4 h-4" />
              <span>ASSISTANT STATUS</span>
            </span>
            <span>Local</span>
          </div>
          <div className="flex items-baseline justify-between">
            <span className="text-2xl font-bold text-white font-mono">
              {telemetry?.active_turns ? 'Active' : 'Ready'}
            </span>
            <span className="text-xs text-slate-400 font-mono">
              {telemetry?.queued_turns ?? 0} waiting
            </span>
          </div>
        </div>
      </div>

      <div className="flex flex-col sm:flex-row gap-2 pt-2">
        <button
          type="button"
          onClick={onBack}
          className="flex-1 py-2.5 rounded-xl bg-syndae-850 hover:bg-syndae-800 text-slate-200 text-xs font-medium border border-syndae-700 transition-all"
        >
          ← Back to Settings Menu
        </button>
        <button
          type="button"
          onClick={onOpenAudit}
          className="py-2.5 px-4 rounded-xl bg-syndae-900 hover:bg-syndae-850 text-slate-300 text-xs font-medium border border-syndae-800 transition-all"
        >
          Security Audit Log
        </button>
      </div>
    </div>
  );
};
