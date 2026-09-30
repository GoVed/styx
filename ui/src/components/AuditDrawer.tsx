import React, { useState } from 'react';
import {
  X,
  Terminal,
  Clock,
  ShieldCheck,
  CheckCircle,
  XCircle,
  Filter,
  FileText
} from 'lucide-react';
import { AuditEventRecord } from '../types';

interface AuditDrawerProps {
  isOpen: boolean;
  onClose: () => void;
  auditEvents: AuditEventRecord[];
  onRefresh: () => void;
}

export const AuditDrawer: React.FC<AuditDrawerProps> = ({
  isOpen,
  onClose,
  auditEvents,
  onRefresh,
}) => {
  const [filterTool, setFilterTool] = useState<string>('all');
  const [selectedEvent, setSelectedEvent] = useState<AuditEventRecord | null>(null);

  if (!isOpen) return null;

  const toolNames = Array.from(
    new Set(auditEvents.map(e => e.tool_name).filter(Boolean))
  ) as string[];

  const filtered = auditEvents.filter(e => {
    if (filterTool !== 'all' && e.tool_name !== filterTool) return false;
    return true;
  });

  return (
    <div className="fixed inset-y-0 right-0 w-full sm:w-[500px] bg-styx-900 border-l border-styx-700 shadow-2xl z-50 flex flex-col font-mono text-xs">
      {/* Header */}
      <div className="p-3 border-b border-styx-800 flex items-center justify-between bg-styx-950">
        <div className="flex items-center space-x-2">
          <Terminal className="w-4 h-4 text-cyan-400" />
          <span className="font-bold text-slate-100">AUDIT TRAIL & EXECUTION LOG</span>
        </div>
        <div className="flex items-center space-x-2">
          <button
            onClick={onRefresh}
            className="text-[10px] text-cyan-400 hover:underline"
          >
            Refresh
          </button>
          <button onClick={onClose} className="p-1 text-slate-400 hover:text-slate-200">
            <X className="w-4 h-4" />
          </button>
        </div>
      </div>

      {/* Filter Bar */}
      <div className="p-2 border-b border-styx-800 bg-styx-900/60 flex items-center space-x-2 text-[11px]">
        <Filter className="w-3.5 h-3.5 text-slate-500" />
        <span className="text-slate-400">Tool:</span>
        <select
          value={filterTool}
          onChange={e => setFilterTool(e.target.value)}
          className="bg-styx-950 border border-styx-700 rounded px-2 py-0.5 text-slate-200 text-xs"
        >
          <option value="all">All Tools ({auditEvents.length})</option>
          {toolNames.map(t => (
            <option key={t} value={t}>
              {t}
            </option>
          ))}
        </select>
      </div>

      {/* Event Timeline */}
      <div className="flex-1 overflow-y-auto p-3 space-y-2">
        {filtered.length === 0 ? (
          <div className="text-center p-8 text-slate-500 italic">No audit records logged yet.</div>
        ) : (
          filtered.map(evt => {
            const isApproved = evt.decision === 'success' || evt.decision === 'approved';
            const isRejected = evt.decision === 'rejected';

            return (
              <div
                key={evt.id}
                onClick={() => setSelectedEvent(evt)}
                className={`p-2.5 rounded bg-styx-950 border cursor-pointer transition-colors ${
                  selectedEvent?.id === evt.id
                    ? 'border-cyan-500 bg-styx-850'
                    : 'border-styx-800 hover:border-styx-700'
                }`}
              >
                <div className="flex items-center justify-between mb-1">
                  <div className="flex items-center space-x-1.5">
                    {isApproved ? (
                      <CheckCircle className="w-3.5 h-3.5 text-emerald-400" />
                    ) : isRejected ? (
                      <XCircle className="w-3.5 h-3.5 text-rose-400" />
                    ) : (
                      <ShieldCheck className="w-3.5 h-3.5 text-amber-400" />
                    )}
                    <span className="font-bold text-slate-200">{evt.tool_name || evt.event_type}</span>
                  </div>
                  <span
                    className={`px-1.5 py-0.2 rounded text-[10px] font-bold ${
                      isApproved
                        ? 'bg-emerald-950 text-emerald-300'
                        : isRejected
                        ? 'bg-rose-950 text-rose-300'
                        : 'bg-amber-950 text-amber-300'
                    }`}
                  >
                    {evt.decision ? evt.decision.toUpperCase() : 'EXECUTED'}
                  </span>
                </div>

                <div className="flex items-center justify-between text-[10px] text-slate-500">
                  <div className="flex items-center space-x-1">
                    <Clock className="w-3 h-3" />
                    <span>{new Date(evt.created_at).toLocaleTimeString()}</span>
                  </div>
                  {evt.duration_ms !== undefined && (
                    <span>Latency: {evt.duration_ms}ms</span>
                  )}
                </div>

                {evt.payload_json && (
                  <pre className="mt-1.5 p-1.5 rounded bg-styx-900 border border-styx-800 text-[10px] text-slate-400 truncate">
                    {evt.payload_json}
                  </pre>
                )}
              </div>
            );
          })
        )}
      </div>

      {/* Selected Event Detail Modal / Drawer Footer */}
      {selectedEvent && (
        <div className="p-3 border-t border-styx-800 bg-styx-950 space-y-2">
          <div className="text-xs font-bold text-cyan-300 flex items-center justify-between">
            <span>Payload Inspector: {selectedEvent.tool_name}</span>
            <button
              onClick={() => setSelectedEvent(null)}
              className="text-[10px] text-slate-500 hover:text-slate-300"
            >
              Clear
            </button>
          </div>
          <pre className="bg-styx-900 p-2 rounded border border-styx-800 text-[10px] text-slate-300 overflow-x-auto max-h-40">
            {selectedEvent.payload_json}
          </pre>
        </div>
      )}
    </div>
  );
};
