import React from 'react';
import { Plus } from 'lucide-react';

export interface ManualServerModalProps {
  serverName: string;
  transportType: 'stdio' | 'unix_socket' | 'http_sse';
  command: string;
  argsStr: string;
  socketPath: string;
  url: string;
  onChangeServerName: (val: string) => void;
  onChangeTransportType: (val: 'stdio' | 'unix_socket' | 'http_sse') => void;
  onChangeCommand: (val: string) => void;
  onChangeArgsStr: (val: string) => void;
  onChangeSocketPath: (val: string) => void;
  onChangeUrl: (val: string) => void;
  onSubmit: () => void;
  onClose: () => void;
}

export const ManualServerModal: React.FC<ManualServerModalProps> = ({
  serverName,
  transportType,
  command,
  argsStr,
  socketPath,
  url,
  onChangeServerName,
  onChangeTransportType,
  onChangeCommand,
  onChangeArgsStr,
  onChangeSocketPath,
  onChangeUrl,
  onSubmit,
  onClose,
}) => {
  return (
    <div className="fixed inset-0 bg-black/75 flex items-center justify-center z-50 p-4 font-mono">
      <div className="bg-syndae-900 border border-syndae-700 rounded-lg p-4 w-full max-w-lg space-y-3 shadow-2xl">
        <h3 className="text-sm font-bold text-emerald-400 flex items-center space-x-2">
          <Plus className="w-4 h-4" />
          <span>REGISTER MODEL CONTEXT PROTOCOL (MCP) INTEGRATION</span>
        </h3>

        <div>
          <label className="text-[11px] text-slate-400">Integration Name:</label>
          <input
            type="text"
            value={serverName}
            onChange={e => onChangeServerName(e.target.value)}
            className="w-full bg-syndae-950 border border-syndae-700 rounded p-1.5 text-slate-200 mt-1"
          />
        </div>

        <div>
          <label className="text-[11px] text-slate-400">Transport Type:</label>
          <select
            value={transportType}
            onChange={e => onChangeTransportType(e.target.value as any)}
            className="w-full bg-syndae-950 border border-syndae-700 rounded p-1.5 text-slate-200 mt-1"
          >
            <option value="stdio">stdio (Child process stdin/stdout JSON-RPC)</option>
            <option value="unix_socket">Unix Domain Socket</option>
            <option value="http_sse">HTTP / SSE Endpoint</option>
          </select>
        </div>

        {transportType === 'stdio' && (
          <>
            <div>
              <label className="text-[11px] text-slate-400">Command Executable:</label>
              <input
                type="text"
                value={command}
                onChange={e => onChangeCommand(e.target.value)}
                placeholder="e.g. python3, node, bash"
                className="w-full bg-syndae-950 border border-syndae-700 rounded p-1.5 text-slate-200 mt-1"
              />
            </div>
            <div>
              <label className="text-[11px] text-slate-400">Arguments (JSON array):</label>
              <input
                type="text"
                value={argsStr}
                onChange={e => onChangeArgsStr(e.target.value)}
                placeholder='["./examples/reference_mcp_daemon.py"]'
                className="w-full bg-syndae-950 border border-syndae-700 rounded p-1.5 text-slate-200 mt-1"
              />
            </div>
          </>
        )}

        {transportType === 'unix_socket' && (
          <div>
            <label className="text-[11px] text-slate-400">Unix Socket Path:</label>
            <input
              type="text"
              value={socketPath}
              onChange={e => onChangeSocketPath(e.target.value)}
              placeholder="/var/run/syndae/daemon.sock"
              className="w-full bg-syndae-950 border border-syndae-700 rounded p-1.5 text-slate-200 mt-1"
            />
          </div>
        )}

        {transportType === 'http_sse' && (
          <div>
            <label className="text-[11px] text-slate-400">HTTP/SSE URL:</label>
            <input
              type="text"
              value={url}
              onChange={e => onChangeUrl(e.target.value)}
              placeholder="http://localhost:8080/mcp"
              className="w-full bg-syndae-950 border border-syndae-700 rounded p-1.5 text-slate-200 mt-1"
            />
          </div>
        )}

        <div className="flex justify-end space-x-2 pt-2">
          <button
            type="button"
            onClick={onClose}
            className="px-3 py-1.5 rounded text-slate-400 hover:text-slate-200 text-xs"
          >
            Cancel
          </button>
          <button
            type="button"
            onClick={onSubmit}
            className="px-4 py-1.5 rounded bg-emerald-600 hover:bg-emerald-500 text-white font-bold text-xs"
          >
            Perform Handshake & Register
          </button>
        </div>
      </div>
    </div>
  );
};
