import React, { useState } from 'react';
import {
  Activity,
  CheckCircle,
  AlertTriangle,
  RefreshCw,
  Download,
  HardDrive,
  Zap,
  MessageSquare,
  Terminal,
  ChevronDown,
  ChevronUp,
} from 'lucide-react';
import { ContainerSummaryInfo } from '../../types';
import { LoadingProgressInfo } from './types';

interface DeploymentStepperProps {
  progressInfo: LoadingProgressInfo;
  activeContainerObj?: ContainerSummaryInfo | null;
  containerLogs: string[];
  selectedLogContainer?: string | null;
  onNavigateToChat: () => void;
  onReturnToDashboard?: () => void;
  onFixDeployLlama128k?: () => void;
  onFixDeploy16k?: () => void;
  onRefreshLogs?: (id: string) => void;
  showTechnicalLogs?: boolean;
  isDeploying?: boolean;
  onToggleTechnicalLogs?: () => void;
  onPrev?: () => void;
}

export const DeploymentStepper: React.FC<DeploymentStepperProps> = ({
  progressInfo,
  activeContainerObj,
  containerLogs,
  selectedLogContainer,
  onNavigateToChat,
  onReturnToDashboard,
  onFixDeployLlama128k,
  onFixDeploy16k,
  onRefreshLogs,
}) => {
  const [showTechnicalLogs, setShowTechnicalLogs] = useState(false);

  return (
    <div className="bg-styx-900 border border-styx-800 rounded-lg p-4 font-mono space-y-4">
      <div className="flex items-center justify-between border-b border-styx-800 pb-2">
        <div className="text-xs font-bold text-slate-200 flex items-center space-x-2">
          <Activity
            className={`w-4 h-4 ${
              progressInfo.stage === 'error'
                ? 'text-rose-400'
                : progressInfo.stage === 'ready'
                ? 'text-emerald-400'
                : 'text-cyan-400 animate-pulse'
            }`}
          />
          <span>STARTUP & INFERENCE PIPELINE</span>
          {activeContainerObj && (
            <span className="text-[10px] text-slate-400 font-normal">
              ({activeContainerObj.names[0]?.replace('/', '') || activeContainerObj.id.substring(0, 10)})
            </span>
          )}
        </div>

        <div className="flex items-center space-x-1.5">
          {progressInfo.stage === 'ready' && (
            <span className="px-2.5 py-0.5 rounded text-[10px] font-bold bg-emerald-950 text-emerald-300 border border-emerald-700 flex items-center space-x-1">
              <CheckCircle className="w-3.5 h-3.5 text-emerald-400" />
              <span>ONLINE & SERVING</span>
            </span>
          )}
          {progressInfo.stage === 'error' && (
            <span className="px-2.5 py-0.5 rounded text-[10px] font-bold bg-rose-950 text-rose-300 border border-rose-700 flex items-center space-x-1">
              <AlertTriangle className="w-3.5 h-3.5 text-rose-400" />
              <span>ATTENTION NEEDED</span>
            </span>
          )}
          {progressInfo.stage !== 'ready' && progressInfo.stage !== 'error' && progressInfo.stage !== 'idle' && (
            <span className="px-2.5 py-0.5 rounded text-[10px] font-bold bg-cyan-950 text-cyan-300 border border-cyan-700 flex items-center space-x-1 animate-pulse">
              <RefreshCw className="w-3.5 h-3.5 text-cyan-400 animate-spin" />
              <span>{progressInfo.progressPct}% LOADING</span>
            </span>
          )}
        </div>
      </div>

      {/* 4-Stage Stepper Tracker */}
      <div className="p-3 bg-styx-950 rounded border border-styx-800 space-y-2.5">
        <div className="grid grid-cols-4 gap-1 text-center relative">
          {[
            { idx: 1, label: 'Download / Cache', icon: Download },
            { idx: 2, label: 'Load to VRAM', icon: HardDrive },
            { idx: 3, label: 'Compile & Warmup', icon: Zap },
            { idx: 4, label: 'Ready to Chat', icon: CheckCircle },
          ].map(step => {
            const isComplete = progressInfo.stageIndex > step.idx || progressInfo.stage === 'ready';
            const isCurrent = progressInfo.stageIndex === step.idx;
            const isErr = isCurrent && progressInfo.stage === 'error';
            const StepIcon = step.icon;

            return (
              <div key={step.idx} className="flex flex-col items-center space-y-1">
                <div
                  className={`w-8 h-8 rounded-full flex items-center justify-center transition-all ${
                    isComplete
                      ? 'bg-emerald-950 border border-emerald-500 text-emerald-400 shadow-sm shadow-emerald-900/50'
                      : isErr
                      ? 'bg-rose-950 border border-rose-500 text-rose-400'
                      : isCurrent
                      ? 'bg-cyan-950 border-2 border-cyan-400 text-cyan-200 animate-pulse shadow-sm shadow-cyan-900/60'
                      : 'bg-styx-900 border border-styx-800 text-slate-500'
                  }`}
                >
                  {isComplete ? (
                    <CheckCircle className="w-4 h-4 text-emerald-400" />
                  ) : isErr ? (
                    <AlertTriangle className="w-4 h-4 text-rose-400" />
                  ) : (
                    <StepIcon className="w-4 h-4" />
                  )}
                </div>
                <span
                  className={`text-[10px] font-semibold leading-tight ${
                    isComplete
                      ? 'text-emerald-400'
                      : isErr
                      ? 'text-rose-400'
                      : isCurrent
                      ? 'text-cyan-300 font-bold'
                      : 'text-slate-500'
                  }`}
                >
                  {step.label}
                </span>
              </div>
            );
          })}
        </div>

        {/* Progress Bar */}
        <div className="w-full bg-styx-900 h-2 rounded-full overflow-hidden">
          <div
            className={`h-full transition-all duration-500 ${
              progressInfo.stage === 'error'
                ? 'bg-rose-500'
                : progressInfo.stage === 'ready'
                ? 'bg-emerald-500'
                : 'bg-gradient-to-r from-cyan-500 to-emerald-400'
            }`}
            style={{ width: `${progressInfo.progressPct}%` }}
          />
        </div>
      </div>

      {/* Plain-English Status Banner */}
      <div
        className={`p-3 rounded border text-xs space-y-1 ${
          progressInfo.stage === 'error'
            ? 'bg-rose-950/40 border-rose-800 text-rose-200'
            : progressInfo.stage === 'ready'
            ? 'bg-emerald-950/40 border-emerald-800 text-emerald-200'
            : 'bg-cyan-950/30 border-cyan-800/80 text-cyan-200'
        }`}
      >
        <div className="font-bold flex items-center space-x-1.5">
          {progressInfo.stage === 'ready' ? (
            <CheckCircle className="w-4 h-4 text-emerald-400" />
          ) : progressInfo.stage === 'error' ? (
            <AlertTriangle className="w-4 h-4 text-rose-400" />
          ) : (
            <Activity className="w-4 h-4 text-cyan-400 animate-pulse" />
          )}
          <span>{progressInfo.title}</span>
        </div>
        <p className="text-[11px] text-slate-300 leading-relaxed font-sans">
          {progressInfo.description}
        </p>
      </div>

      {/* ONCE DONE: GUIDE USER BACK TO THE CHAT SCREEN */}
      {progressInfo.stage === 'ready' && (
        <div className="p-4 rounded-xl bg-gradient-to-r from-emerald-950/90 to-cyan-950/90 border-2 border-emerald-500 shadow-xl space-y-3">
          <div className="flex items-center space-x-2">
            <CheckCircle className="w-5 h-5 text-emerald-400" />
            <div>
              <h4 className="text-sm font-bold text-slate-100">🎉 Model is Live & Serving!</h4>
              <p className="text-xs text-slate-300 font-sans">
                Initialization is complete. Ready to handle user interactions and inference tasks.
              </p>
            </div>
          </div>
          <div className="flex flex-wrap gap-2 pt-1">
            <button
              type="button"
              onClick={onNavigateToChat}
              className="px-5 py-2.5 rounded-lg bg-emerald-600 hover:bg-emerald-500 text-white font-bold text-xs flex items-center space-x-2 shadow-lg transition-transform active:scale-95"
            >
              <MessageSquare className="w-4 h-4 text-emerald-100" />
              <span>💬 Start Chatting (Go to Chat Screen) →</span>
            </button>
            <button
              type="button"
              onClick={onReturnToDashboard}
              className="px-4 py-2.5 rounded-lg bg-styx-800 hover:bg-styx-750 text-slate-200 border border-styx-700 font-semibold text-xs"
            >
              View Active Model Dashboard
            </button>
          </div>
        </div>
      )}

      {/* Actionable Self-Healing Error Banner */}
      {progressInfo.stage === 'error' && (
        <div className="p-3.5 bg-amber-950/70 border border-amber-500 rounded-lg space-y-2.5 text-xs shadow-lg">
          <div className="font-bold text-amber-300 flex items-center space-x-2">
            <AlertTriangle className="w-4 h-4 text-amber-400" />
            <span className="text-sm">{progressInfo.errorHeadline || 'Inference Container Stopped'}</span>
          </div>
          <p className="text-[11px] text-amber-100/90 leading-relaxed font-sans">
            {progressInfo.errorDetail || progressInfo.description}
          </p>
          <div className="pt-1 flex flex-col sm:flex-row gap-2">
            <button
              type="button"
              onClick={onFixDeployLlama128k}
              className="flex-1 py-2 px-3 rounded-lg bg-emerald-600 hover:bg-emerald-500 text-white font-bold text-xs flex items-center justify-center space-x-2 shadow transition-colors"
            >
              <Zap className="w-3.5 h-3.5 text-amber-300" />
              <span>⚡ Deploy with llama.cpp (128k Local GGUF)</span>
            </button>
            <button
              type="button"
              onClick={onFixDeploy16k}
              className="flex-1 py-2 px-3 rounded-lg bg-cyan-700 hover:bg-cyan-600 text-white font-bold text-xs flex items-center justify-center space-x-2 shadow transition-colors"
            >
              <RefreshCw className="w-3.5 h-3.5 text-cyan-200" />
              <span>Re-deploy with 16k Context (Safe for 16GB VRAM)</span>
            </button>
          </div>
        </div>
      )}

      {/* Collapsible Technical Console Logs */}
      <div className="pt-2 border-t border-styx-800">
        <button
          type="button"
          onClick={() => setShowTechnicalLogs(!showTechnicalLogs)}
          className="w-full flex items-center justify-between text-[11px] text-slate-400 hover:text-slate-200 py-1"
        >
          <div className="flex items-center space-x-1.5">
            <Terminal className="w-3.5 h-3.5 text-purple-400" />
            <span>Show/Hide Technical Console Logs ({containerLogs.length} lines)</span>
          </div>
          {showTechnicalLogs ? <ChevronUp className="w-3.5 h-3.5" /> : <ChevronDown className="w-3.5 h-3.5" />}
        </button>

        {showTechnicalLogs && (
          <div className="mt-2 space-y-2">
            <div className="flex items-center justify-between">
              <span className="text-[10px] text-slate-400">
                Container ID: {selectedLogContainer || 'auto'}
              </span>
              {selectedLogContainer && (
                <button
                  type="button"
                  onClick={() => onRefreshLogs(selectedLogContainer)}
                  className="text-[10px] text-cyan-400 hover:underline"
                >
                  Refresh Logs
                </button>
              )}
            </div>
            <div className="h-64 bg-styx-950 p-3 rounded border border-styx-800 text-[10px] text-slate-300 overflow-y-auto space-y-0.5 font-mono">
              {containerLogs.length === 0 ? (
                <div className="text-slate-600 italic">No console logs received yet...</div>
              ) : (
                containerLogs.map((line, idx) => (
                  <div key={idx} className="whitespace-pre-wrap leading-snug">
                    {line}
                  </div>
                ))
              )}
            </div>
          </div>
        )}
      </div>
    </div>
  );
};
