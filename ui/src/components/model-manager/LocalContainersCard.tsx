import React from 'react';
import { Box, Play, Square, RotateCcw, Terminal, Trash2 } from 'lucide-react';
import { ContainerSummaryInfo } from '../../types';

export interface LocalContainersCardProps {
  containers: ContainerSummaryInfo[];
  selectedLogContainer: string | null;
  containerLogs: string[];
  isContainerActive: (c: ContainerSummaryInfo) => boolean;
  onRefresh: () => void;
  onSelectLogContainer: (id: string) => void;
  onActivateContainer: (id: string) => void;
  onContainerAction: (id: string, action: 'start' | 'stop' | 'restart' | 'delete') => void;
}

export const LocalContainersCard: React.FC<LocalContainersCardProps> = ({
  containers,
  selectedLogContainer,
  containerLogs,
  isContainerActive,
  onRefresh,
  onSelectLogContainer,
  onActivateContainer,
  onContainerAction,
}) => {
  return (
    <div className="space-y-4 font-mono">
      {/* Running Containers */}
      <div className="bg-styx-900 border border-styx-800 rounded-lg p-3 space-y-2">
        <div className="text-xs font-bold text-slate-200 flex items-center justify-between">
          <div className="flex items-center space-x-1.5">
            <Box className="w-3.5 h-3.5 text-blue-400" />
            <span>LOCAL CONTAINERS ({containers.length})</span>
          </div>
          <button
            type="button"
            onClick={onRefresh}
            className="text-slate-400 hover:text-slate-200 text-[10px] underline"
          >
            Refresh
          </button>
        </div>

        <div className="space-y-2 max-h-64 overflow-y-auto">
          {containers.length === 0 ? (
            <div className="p-4 text-center text-slate-500 text-xs">
              No Docker containers currently active.
            </div>
          ) : (
            containers.map(c => {
              const active = isContainerActive(c);
              return (
                <div
                  key={c.id}
                  className="p-2.5 rounded bg-styx-950 border border-styx-800 space-y-1.5"
                >
                  <div className="flex items-center justify-between">
                    <span className="font-bold text-slate-200 truncate">
                      {c.names[0] || c.id.substring(0, 12)}
                    </span>
                    <div className="flex items-center space-x-1.5">
                      {active ? (
                        <span className="px-1.5 py-0.5 rounded text-[10px] font-bold bg-emerald-950 text-emerald-400 border border-emerald-800">
                          ACTIVE
                        </span>
                      ) : (
                        c.state === 'running' && (
                          <button
                            type="button"
                            onClick={() => onActivateContainer(c.id)}
                            className="px-2 py-0.5 rounded bg-styx-850 hover:bg-emerald-900 text-emerald-300 border border-styx-700 text-[10px] font-bold"
                          >
                            Set Active
                          </button>
                        )
                      )}
                      <span
                        className={`px-1.5 py-0.5 rounded text-[10px] font-bold ${
                          c.state === 'running'
                            ? 'bg-emerald-950 text-emerald-400 border border-emerald-800'
                            : 'bg-rose-950 text-rose-400 border border-rose-800'
                        }`}
                      >
                        {c.state.toUpperCase()}
                      </span>
                    </div>
                  </div>
                  <div className="text-[10px] text-slate-400 truncate">Image: {c.image}</div>
                  <div className="text-[10px] text-cyan-400">Ports: {c.ports?.join(', ') || 'none'}</div>

                  {/* Controls */}
                  <div className="flex items-center space-x-1 pt-1">
                    {c.state !== 'running' ? (
                      <button
                        type="button"
                        onClick={() => onContainerAction(c.id, 'start')}
                        className="p-1 rounded bg-styx-850 hover:bg-styx-800 text-emerald-400"
                        title="Start"
                      >
                        <Play className="w-3 h-3" />
                      </button>
                    ) : (
                      <button
                        type="button"
                        onClick={() => onContainerAction(c.id, 'stop')}
                        className="p-1 rounded bg-styx-850 hover:bg-styx-800 text-amber-400"
                        title="Stop"
                      >
                        <Square className="w-3 h-3" />
                      </button>
                    )}
                    <button
                      type="button"
                      onClick={() => onContainerAction(c.id, 'restart')}
                      className="p-1 rounded bg-styx-850 hover:bg-styx-800 text-cyan-400"
                      title="Restart"
                    >
                      <RotateCcw className="w-3 h-3" />
                    </button>
                    <button
                      type="button"
                      onClick={() => onSelectLogContainer(c.id)}
                      className="px-2 py-0.5 rounded bg-styx-850 hover:bg-styx-800 text-slate-300 text-[10px] flex items-center space-x-1"
                    >
                      <Terminal className="w-3 h-3 text-purple-400" />
                      <span>Logs</span>
                    </button>
                    <button
                      type="button"
                      onClick={() => onContainerAction(c.id, 'delete')}
                      className="p-1 rounded bg-styx-850 hover:bg-rose-900 text-rose-400 ml-auto"
                      title="Remove"
                    >
                      <Trash2 className="w-3 h-3" />
                    </button>
                  </div>
                </div>
              );
            })
          )}
        </div>
      </div>

      {/* Live Logs Terminal */}
      <div className="bg-styx-900 border border-styx-800 rounded-lg p-3 space-y-2">
        <div className="flex items-center justify-between">
          <div className="text-xs font-bold text-slate-200 flex items-center space-x-1.5">
            <Terminal className="w-3.5 h-3.5 text-purple-400" />
            <span>DOCKER LOG STREAM {selectedLogContainer ? `(${selectedLogContainer.substring(0, 10)})` : ''}</span>
          </div>
          {selectedLogContainer && (
            <button
              type="button"
              onClick={() => onSelectLogContainer(selectedLogContainer)}
              className="text-cyan-400 hover:underline text-[10px]"
            >
              Refresh Logs
            </button>
          )}
        </div>

        <div className="h-64 bg-styx-950 p-3 rounded border border-styx-800 text-[11px] text-slate-300 overflow-y-auto space-y-0.5">
          {containerLogs.length === 0 ? (
            <div className="text-slate-600 italic">Select a container to stream stdout/stderr logs...</div>
          ) : (
            containerLogs.map((line, idx) => (
              <div key={idx} className="whitespace-pre-wrap leading-snug">
                {line}
              </div>
            ))
          )}
        </div>
      </div>
    </div>
  );
};
