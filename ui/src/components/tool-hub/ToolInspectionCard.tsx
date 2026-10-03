import React from 'react';
import { MessageSquare, X, Check, Terminal, Shield, RefreshCw, Zap } from 'lucide-react';
import { InspectedToolInfo } from '../../types';

export interface ToolInspectionCardProps {
  inspectedTool: InspectedToolInfo;
  isInstalling: boolean;
  onDismiss: () => void;
  onInstall: () => void;
}

export const ToolInspectionCard: React.FC<ToolInspectionCardProps> = ({
  inspectedTool,
  isInstalling,
  onDismiss,
  onInstall,
}) => {
  return (
    <div className="bg-syndae-900 border-2 border-emerald-600/90 rounded-xl p-4 font-mono space-y-4 shadow-2xl animate-in fade-in slide-in-from-top-2">
      <div className="flex items-start justify-between border-b border-syndae-800 pb-3">
        <div className="flex items-center space-x-3">
          <div className="p-2.5 rounded-lg bg-emerald-950 border border-emerald-700 text-emerald-400 shadow">
            <MessageSquare className="w-6 h-6" />
          </div>
          <div>
            <div className="flex items-center space-x-2">
              <h3 className="text-sm font-bold text-slate-100">
                {inspectedTool.display_name}
              </h3>
              <span className="px-2 py-0.5 rounded bg-emerald-950 text-emerald-300 border border-emerald-800 text-[10px] font-bold">
                v{inspectedTool.version}
              </span>
              {inspectedTool.is_registered && (
                <span className="px-2 py-0.5 rounded bg-cyan-950 text-cyan-300 border border-cyan-800 text-[10px] font-bold">
                  Connected & Active
                </span>
              )}
            </div>
            <p className="text-xs text-slate-400 mt-0.5">{inspectedTool.description}</p>
          </div>
        </div>

        <button
          type="button"
          onClick={onDismiss}
          className="text-slate-400 hover:text-slate-200 p-1"
        >
          <X className="w-4 h-4" />
        </button>
      </div>

      {/* Metadata Grid */}
      <div className="grid grid-cols-1 md:grid-cols-3 gap-3 text-xs">
        <div className="p-2.5 rounded-lg bg-syndae-950 border border-syndae-800 space-y-1">
          <span className="text-[10px] uppercase text-slate-500 font-bold">Location & Repo</span>
          <div className="text-slate-200 font-mono truncate text-[11px]">{inspectedTool.path}</div>
          <div className="text-[10px] text-emerald-400 flex items-center space-x-1">
            <Check className="w-3 h-3" />
            <span>package.json & compose valid</span>
          </div>
        </div>

        <div className="p-2.5 rounded-lg bg-syndae-950 border border-syndae-800 space-y-1">
          <span className="text-[10px] uppercase text-slate-500 font-bold">Container Micro-Daemon</span>
          <div className="text-slate-200 font-mono text-[11px] truncate">
            {inspectedTool.container_name}
          </div>
          <div className="text-[10px] text-cyan-300 flex items-center space-x-1">
            <span
              className={`w-2 h-2 rounded-full ${
                inspectedTool.container_running ? 'bg-emerald-400' : 'bg-amber-400'
              }`}
            />
            <span>
              {inspectedTool.container_running ? 'Running (Port 8765)' : 'Ready to start'}
            </span>
          </div>
        </div>

        <div className="p-2.5 rounded-lg bg-syndae-950 border border-syndae-800 space-y-1">
          <span className="text-[10px] uppercase text-slate-500 font-bold">Tool Instructions & Skills</span>
          <div className="text-slate-200 font-mono text-[11px] truncate">
            {inspectedTool.instructions ? 'Provided by Tool Package' : 'Standard MCP Protocol'}
          </div>
          <div className="text-[10px] text-purple-300 truncate">
            Auto-installs to {inspectedTool.skill_file || `skills/${inspectedTool.name}.md`}
          </div>
        </div>
      </div>

      {/* Exposed Tools Pills */}
      <div className="space-y-2">
        <div className="text-xs font-bold text-slate-300 flex items-center space-x-2">
          <Terminal className="w-3.5 h-3.5 text-cyan-400" />
          <span>EXPOSED MCP TOOLS ({inspectedTool.tools?.length || 0})</span>
        </div>
        <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-2">
          {inspectedTool.tools?.map(t => (
            <div
              key={t.name}
              className="p-2 rounded-lg bg-syndae-950 border border-syndae-800 flex items-start justify-between space-x-2"
            >
              <div>
                <code className="text-emerald-400 font-bold text-[11px]">{t.name}</code>
                <p className="text-[10px] text-slate-400 line-clamp-1">{t.description}</p>
              </div>
              <span
                className={`px-1.5 py-0.5 rounded text-[9px] font-bold uppercase shrink-0 ${
                  t.risk_level === 'CRITICAL'
                    ? 'bg-rose-950 text-rose-300 border border-rose-900'
                    : t.risk_level === 'HIGH'
                    ? 'bg-amber-950 text-amber-300 border border-amber-900'
                    : 'bg-emerald-950 text-emerald-300 border border-emerald-900'
                }`}
              >
                {t.policy}
              </span>
            </div>
          ))}
        </div>
      </div>

      {/* Instructions Preview if supplied by tool */}
      {inspectedTool.instructions && (
        <details className="text-xs bg-syndae-950/70 border border-syndae-800 rounded-lg p-2.5">
          <summary className="cursor-pointer text-slate-300 font-semibold hover:text-emerald-400 select-none flex items-center space-x-1.5">
            <span>📖 View Tool Instructions & Syndae Integration Guide</span>
          </summary>
          <div className="mt-2 pt-2 text-slate-400 font-mono text-[11px] whitespace-pre-wrap max-h-48 overflow-y-auto leading-relaxed border-t border-syndae-800/80 bg-syndae-950 p-2 rounded">
            {inspectedTool.instructions}
          </div>
        </details>
      )}

      {/* Deploy Action Bar */}
      <div className="pt-2 border-t border-syndae-800 flex items-center justify-between">
        <div className="text-[11px] text-slate-400 flex items-center space-x-1.5">
          <Shield className="w-3.5 h-3.5 text-amber-400" />
          <span>Deterministic HITL gates will safeguard mutating actions (e.g. sending real messages).</span>
        </div>
        <div className="flex items-center space-x-2">
          <button
            type="button"
            onClick={onDismiss}
            className="px-3 py-1.5 rounded-lg text-slate-400 hover:text-slate-200 text-xs"
          >
            Dismiss
          </button>
          <button
            type="button"
            onClick={onInstall}
            disabled={isInstalling}
            className="px-4 py-1.5 rounded-lg bg-emerald-600 hover:bg-emerald-500 text-white font-bold text-xs flex items-center space-x-2 shadow-lg disabled:opacity-50"
          >
            {isInstalling ? (
              <>
                <RefreshCw className="w-3.5 h-3.5 animate-spin" />
                <span>Deploying & Handshaking...</span>
              </>
            ) : (
              <>
                <Zap className="w-3.5 h-3.5" />
                <span>
                  {inspectedTool.is_registered
                    ? 'Re-Deploy & Sync Tool'
                    : 'Deploy & Connect MCP Daemon'}
                </span>
              </>
            )}
          </button>
        </div>
      </div>
    </div>
  );
};
