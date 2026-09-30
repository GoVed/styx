import React from 'react';
import { Server, Zap, Trash2 } from 'lucide-react';
import { McpServerRecord } from '../../types';

export interface ConnectedServersProps {
  servers: McpServerRecord[];
  simulatingTrigger: boolean;
  onTriggerSimulatedInbound: () => void;
  onRemoveServer: (id: string) => void;
}

export const ConnectedServers: React.FC<ConnectedServersProps> = ({
  servers,
  simulatingTrigger,
  onTriggerSimulatedInbound,
  onRemoveServer,
}) => {
  return (
    <div className="bg-styx-900 border border-styx-800 rounded-lg p-3 font-mono space-y-2">
      <div className="flex items-center justify-between">
        <div className="text-xs font-bold text-slate-200 flex items-center space-x-1.5">
          <Server className="w-3.5 h-3.5 text-cyan-400" />
          <span>CONNECTED MCP DAEMONS ({servers.length})</span>
        </div>
        {servers.length > 0 && (
          <button
            type="button"
            onClick={onTriggerSimulatedInbound}
            disabled={simulatingTrigger}
            className="px-2.5 py-1 rounded bg-emerald-950 text-emerald-300 border border-emerald-800 hover:bg-emerald-900 text-[10px] font-bold flex items-center space-x-1 shadow transition-colors"
            title="Send a simulated test inbound event to trigger reactive turn"
          >
            <Zap className="w-3 h-3 text-emerald-400" />
            <span>{simulatingTrigger ? 'Dispatching...' : 'Simulate Inbound Event'}</span>
          </button>
        )}
      </div>

      <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-3">
        {servers.map(s => {
          return (
            <div
              key={s.id}
              className="p-3 rounded-lg bg-styx-950 border border-styx-800 space-y-2 relative shadow-sm"
            >
              <div className="flex items-center justify-between">
                <div className="flex items-center space-x-2">
                  <Server className="w-4 h-4 text-cyan-400" />
                  <span className="font-bold text-slate-200">{s.name}</span>
                </div>
                <span className="px-1.5 py-0.5 rounded bg-emerald-950 text-emerald-400 border border-emerald-800 text-[10px] font-bold">
                  CONNECTED
                </span>
              </div>

              <div className="text-[11px] text-slate-400 font-mono space-y-0.5">
                <div>
                  Transport: <span className="text-cyan-300">{s.transport_type}</span>
                </div>
                {s.command && <div className="truncate">Cmd: {s.command}</div>}
                {s.socket_path && <div className="truncate">Socket: {s.socket_path}</div>}
                {s.url && <div className="truncate text-slate-300">URL: {s.url}</div>}
              </div>

              <div className="flex items-center justify-between pt-1 border-t border-styx-800/60">
                <span className="text-[10px] text-slate-500">
                  Daemon Active
                </span>
                <button
                  type="button"
                  onClick={() => onRemoveServer(s.id)}
                  className="p-1 text-slate-500 hover:text-rose-400 rounded transition-colors"
                  title="Disconnect daemon"
                >
                  <Trash2 className="w-3.5 h-3.5" />
                </button>
              </div>
            </div>
          );
        })}
      </div>
    </div>
  );
};
