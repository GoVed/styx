import React from 'react';
import { Cpu, HardDrive, MemoryStick, ShieldAlert, Sparkles, Activity, Layers, Terminal, Box } from 'lucide-react';
import { SystemTelemetry } from '../types';

interface TelemetryStripProps {
  telemetry: SystemTelemetry | null;
  pendingApprovalsCount: number;
  onOpenApprovals: () => void;
  onToggleAudit: () => void;
  auditOpen: boolean;
}

export const TelemetryStrip: React.FC<TelemetryStripProps> = ({
  telemetry,
  pendingApprovalsCount,
  onOpenApprovals,
  onToggleAudit,
  auditOpen,
}) => {
  const cpu = telemetry?.host_cpu_pct ?? 0;
  const memPct = telemetry?.memory_pct ?? 0;
  const diskPct = telemetry?.disk_pct ?? 0;
  const tokSec = telemetry?.tokens_per_second ?? 0;
  const activeModel = telemetry?.active_model ?? 'Local vLLM (Docker)';
  const gpu = telemetry?.gpu;

  return (
    <header className="bg-styx-900 border-b border-styx-700/80 px-4 py-2 text-xs font-mono select-none flex-shrink-0 shadow-md z-40">
      <div className="flex items-center justify-between gap-3 overflow-x-auto min-w-0">
        {/* Brand / Status */}
        <div className="flex items-center space-x-3 flex-shrink-0">
          <div className="flex items-center space-x-2">
            <span className="relative flex h-2.5 w-2.5">
              <span className="animate-ping absolute inline-flex h-full w-full rounded-full bg-emerald-400 opacity-75"></span>
              <span className="relative inline-flex rounded-full h-2.5 w-2.5 bg-emerald-500"></span>
            </span>
            <span className="font-bold tracking-wider text-slate-100 text-sm">STYX</span>
            <span className="px-1.5 py-0.5 rounded bg-emerald-950 text-emerald-400 text-[10px] font-semibold border border-emerald-800/80">
              RUNTIME ARMED
            </span>
          </div>
          <span className="text-slate-600">|</span>
          <div className="flex items-center space-x-1.5 text-slate-300 min-w-0">
            <span className="text-slate-500 flex-shrink-0">MODEL:</span>
            <span
              className="px-2 py-0.5 rounded bg-styx-800 text-cyan-300 font-semibold border border-cyan-900/50 truncate max-w-[160px] sm:max-w-[220px] md:max-w-[300px]"
              title={activeModel}
            >
              {activeModel}
            </span>
          </div>
        </div>

        {/* System Vitals Strip */}
        <div className="flex items-center space-x-3 flex-shrink-0">
          {/* Host CPU */}
          <div className="flex items-center space-x-1.5 bg-styx-950 px-2.5 py-1 rounded border border-styx-700/60 flex-shrink-0">
            <Cpu className="w-3.5 h-3.5 text-emerald-400 flex-shrink-0" />
            <span className="text-slate-400">CPU</span>
            <span className={`font-semibold ${cpu > 80 ? 'text-rose-400' : 'text-slate-200'}`}>
              {cpu.toFixed(1)}%
            </span>
            <div className="w-12 h-1.5 bg-styx-800 rounded-full overflow-hidden ml-1">
              <div
                className={`h-full ${cpu > 80 ? 'bg-rose-500' : 'bg-emerald-500'}`}
                style={{ width: `${Math.min(100, Math.max(2, cpu))}%` }}
              />
            </div>
          </div>

          {/* Host RAM */}
          <div className="flex items-center space-x-1.5 bg-styx-950 px-2.5 py-1 rounded border border-styx-700/60 flex-shrink-0">
            <MemoryStick className="w-3.5 h-3.5 text-cyan-400 flex-shrink-0" />
            <span className="text-slate-400">RAM</span>
            <span className="font-semibold text-slate-200">
              {telemetry ? `${(telemetry.memory_used_mb / 1024).toFixed(1)}G` : '0G'}
            </span>
            <span className="text-slate-500">({memPct.toFixed(0)}%)</span>
            <div className="w-12 h-1.5 bg-styx-800 rounded-full overflow-hidden ml-1">
              <div
                className="h-full bg-cyan-500"
                style={{ width: `${Math.min(100, Math.max(2, memPct))}%` }}
              />
            </div>
          </div>

          {/* Disk */}
          <div className="hidden lg:flex items-center space-x-1.5 bg-styx-950 px-2.5 py-1 rounded border border-styx-700/60 flex-shrink-0">
            <HardDrive className="w-3.5 h-3.5 text-purple-400 flex-shrink-0" />
            <span className="text-slate-400">DISK</span>
            <span className="font-semibold text-slate-200">{telemetry?.disk_used_gb ?? 0}GB</span>
            <span className="text-slate-500">({diskPct.toFixed(0)}%)</span>
          </div>

          {/* GPU / VRAM */}
          {gpu && (
            <div
              className="flex items-center space-x-1.5 bg-styx-950 px-2.5 py-1 rounded border border-styx-700/60 flex-shrink-0"
              title={`${gpu.name} (${gpu.vendor === 'amd' ? 'AMD ROCm' : 'NVIDIA CUDA'})`}
            >
              <Activity className={`w-3.5 h-3.5 flex-shrink-0 ${gpu.vendor === 'amd' ? 'text-rose-400' : 'text-emerald-400'}`} />
              <span className={`text-[10px] font-bold px-1 rounded ${gpu.vendor === 'amd' ? 'bg-rose-950 text-rose-300 border border-rose-800' : 'bg-emerald-950 text-emerald-300 border border-emerald-800'}`}>
                {gpu.vendor === 'amd' ? 'AMD' : 'NV'}
              </span>
              <span className="text-slate-400">VRAM</span>
              <span className="font-semibold text-slate-200">
                {(gpu.vram_used_mb / 1024).toFixed(1)}G
              </span>
              <span className="text-slate-500">({gpu.vram_pct.toFixed(0)}%)</span>
              {gpu.temperature_c && (
                <span className="text-[10px] text-slate-400">{gpu.temperature_c}°C</span>
              )}
            </div>
          )}

          {/* Tok/s */}
          <div className="flex items-center space-x-1.5 bg-styx-950 px-2.5 py-1 rounded border border-styx-700/60 flex-shrink-0">
            <Sparkles className="w-3.5 h-3.5 text-yellow-400 flex-shrink-0" />
            <span className="text-slate-400">SPEED</span>
            <span className="font-semibold text-amber-300">
              {tokSec > 0 ? `${tokSec.toFixed(1)} t/s` : 'IDLE'}
            </span>
          </div>

          {/* GPU Queue / Parallel Slots */}
          <div
            className={`flex items-center space-x-1.5 px-2.5 py-1 rounded border flex-shrink-0 ${(telemetry?.queued_turns ?? 0) > 0 ? 'bg-amber-950/70 border-amber-700/80 text-amber-300 animate-pulse' : 'bg-styx-950 border-styx-700/60 text-slate-300'}`}
            title={`GPU Inference Concurrency: ${telemetry?.active_turns ?? 0} active, ${telemetry?.queued_turns ?? 0} queued (Max parallel: ${telemetry?.max_concurrent_turns ?? 1})`}
          >
            <Layers className={`w-3.5 h-3.5 flex-shrink-0 ${(telemetry?.queued_turns ?? 0) > 0 ? 'text-amber-400' : 'text-cyan-400'}`} />
            <span className="text-slate-400">QUEUE:</span>
            <span className="font-semibold">
              {(telemetry?.queued_turns ?? 0) > 0
                ? `${telemetry?.queued_turns} WAIT (${telemetry?.active_turns}/${telemetry?.max_concurrent_turns ?? 1})`
                : `${telemetry?.active_turns ?? 0}/${telemetry?.max_concurrent_turns ?? 1} SLOTS`}
            </span>
          </div>

          {/* Docker Containers Count */}
          <div className="hidden xl:flex items-center space-x-1 text-slate-400 flex-shrink-0">
            <Box className="w-3.5 h-3.5 text-blue-400" />
            <span>DOCKER:</span>
            <span className="text-slate-200 font-semibold">{telemetry?.running_containers ?? 0}</span>
          </div>

          {/* Active MCP Tools */}
          <div className="hidden xl:flex items-center space-x-1 text-slate-400 flex-shrink-0">
            <Terminal className="w-3.5 h-3.5 text-emerald-400" />
            <span>MCP:</span>
            <span className="text-slate-200 font-semibold">{telemetry?.active_mcp_servers ?? 0}</span>
          </div>
        </div>

        {/* Action Controls */}
        <div className="flex items-center space-x-2 flex-shrink-0">
          {/* Pending Approvals Button */}
          {pendingApprovalsCount > 0 ? (
            <button
              onClick={onOpenApprovals}
              className="flex items-center space-x-1.5 px-2.5 py-1 rounded bg-rose-950 text-rose-300 border border-rose-600 animate-pulse hover:bg-rose-900 transition-colors font-semibold"
            >
              <ShieldAlert className="w-3.5 h-3.5 text-rose-400" />
              <span>APPROVAL GATES:</span>
              <span className="bg-rose-600 text-white rounded-full px-1.5 py-0.2 text-[10px]">
                {pendingApprovalsCount}
              </span>
            </button>
          ) : (
            <div className="flex items-center space-x-1 px-2 py-1 rounded bg-styx-800 text-slate-400 border border-styx-700/50 text-[11px]">
              <ShieldAlert className="w-3.5 h-3.5 text-emerald-500" />
              <span>GATES: 0</span>
            </div>
          )}

          {/* Audit Drawer Button */}
          <button
            onClick={onToggleAudit}
            className={`flex items-center space-x-1 px-2.5 py-1 rounded border text-[11px] transition-colors ${
              auditOpen
                ? 'bg-styx-700 text-white border-styx-500'
                : 'bg-styx-800 text-slate-300 border-styx-700 hover:bg-styx-700'
            }`}
          >
            <Terminal className="w-3.5 h-3.5 text-cyan-400" />
            <span>AUDIT TRAIL</span>
          </button>
        </div>
      </div>
    </header>
  );
};
