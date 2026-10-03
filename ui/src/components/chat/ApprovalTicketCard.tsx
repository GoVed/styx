import React from 'react';
import { ShieldAlert, CheckCircle, XCircle, Edit3 } from 'lucide-react';
import { ApprovalTicket } from '../../types';

export interface ApprovalTicketCardProps {
  ticket: ApprovalTicket;
  editingTicket: { id: string; argsStr: string } | null;
  rejectReason: { id: string; text: string } | null;
  onSetEditingTicket: (editing: { id: string; argsStr: string } | null) => void;
  onSetRejectReason: (reason: { id: string; text: string } | null) => void;
  onResolveTicket: (
    ticketId: string,
    action: 'APPROVE' | 'REJECT' | 'MODIFY',
    modifiedArgs?: any,
    reason?: string
  ) => void;
}

export const ApprovalTicketCard: React.FC<ApprovalTicketCardProps> = ({
  ticket,
  editingTicket,
  rejectReason,
  onSetEditingTicket,
  onSetRejectReason,
  onResolveTicket,
}) => {
  return (
    <div
      key={ticket.ticket_id}
      className="w-full max-w-3xl rounded-lg border-2 border-amber-500/80 bg-amber-950/30 p-3.5 shadow-lg space-y-3 font-mono text-xs animate-pulse-glow"
    >
      <div className="flex items-center justify-between border-b border-amber-600/40 pb-2">
        <div className="flex items-center space-x-2 text-amber-300 font-bold">
          <ShieldAlert className="w-4 h-4 text-amber-400" />
          <span>DETERMINISTIC GATE: MUTATING ACTION HALTED</span>
        </div>
        <span className="px-2 py-0.5 rounded bg-amber-900/80 text-amber-200 text-[10px] border border-amber-600 font-bold">
          RISK: {ticket.risk_level}
        </span>
      </div>

      <div>
        <p className="text-slate-300 text-xs mb-1.5 font-sans">
          The agent requires authorization to execute mutating tool{' '}
          <code className="bg-syndae-900 px-1.5 py-0.5 rounded text-amber-300 font-mono font-bold">
            {ticket.tool_name}
          </code>
          .
        </p>
        <div className="text-[10px] text-slate-400 uppercase font-semibold mb-1">
          Proposed Arguments:
        </div>
        <pre className="bg-syndae-900/90 p-2.5 rounded border border-syndae-700 text-slate-200 text-[11px] overflow-x-auto max-h-48">
          {JSON.stringify(ticket.arguments, null, 2)}
        </pre>
      </div>

      {/* Parameter Editor Drawer if Edit Arguments is clicked */}
      {editingTicket?.id === ticket.ticket_id && (
        <div className="p-2.5 rounded bg-syndae-900 border border-cyan-800 space-y-2">
          <div className="text-[11px] text-cyan-300 font-bold">Edit Tool Arguments (JSON):</div>
          <textarea
            rows={5}
            value={editingTicket.argsStr}
            onChange={e =>
              onSetEditingTicket({ id: ticket.ticket_id, argsStr: e.target.value })
            }
            className="w-full bg-syndae-950 border border-syndae-700 rounded p-2 text-slate-200 font-mono text-xs focus:outline-none focus:border-cyan-500"
          />
          <div className="flex justify-end space-x-2">
            <button
              type="button"
              onClick={() => onSetEditingTicket(null)}
              className="px-2 py-1 rounded text-slate-400 hover:text-slate-200 text-xs"
            >
              Cancel
            </button>
            <button
              type="button"
              onClick={() => {
                try {
                  const parsed = JSON.parse(editingTicket.argsStr);
                  onResolveTicket(ticket.ticket_id, 'MODIFY', parsed);
                  onSetEditingTicket(null);
                } catch (err) {
                  alert('Invalid JSON: ' + err);
                }
              }}
              className="px-3 py-1 rounded bg-cyan-700 hover:bg-cyan-600 text-white font-semibold text-xs"
            >
              Apply & Approve
            </button>
          </div>
        </div>
      )}

      {/* Rejection reason prompt */}
      {rejectReason?.id === ticket.ticket_id && (
        <div className="p-2 rounded bg-syndae-900 border border-rose-800 space-y-2">
          <div className="text-[11px] text-rose-300 font-bold">Rejection Reason for Agent:</div>
          <input
            type="text"
            value={rejectReason.text}
            onChange={e =>
              onSetRejectReason({ id: ticket.ticket_id, text: e.target.value })
            }
            placeholder="e.g. Unsafe target path, execute alternative plan"
            className="w-full bg-syndae-950 border border-syndae-700 rounded p-1.5 text-slate-200 text-xs focus:outline-none focus:border-rose-500"
          />
          <div className="flex justify-end space-x-2">
            <button
              type="button"
              onClick={() => onSetRejectReason(null)}
              className="px-2 py-1 rounded text-slate-400 text-xs"
            >
              Cancel
            </button>
            <button
              type="button"
              onClick={() => {
                onResolveTicket(ticket.ticket_id, 'REJECT', undefined, rejectReason.text);
                onSetRejectReason(null);
              }}
              className="px-3 py-1 rounded bg-rose-700 hover:bg-rose-600 text-white font-semibold text-xs"
            >
              Confirm Reject
            </button>
          </div>
        </div>
      )}

      {/* Action Buttons */}
      <div className="flex flex-wrap items-center gap-2 pt-1">
        <button
          type="button"
          onClick={() => onResolveTicket(ticket.ticket_id, 'APPROVE')}
          className="px-3 py-1.5 rounded bg-emerald-600 hover:bg-emerald-500 text-white font-bold text-xs flex items-center space-x-1.5 shadow"
        >
          <CheckCircle className="w-3.5 h-3.5" />
          <span>Approve & Continue</span>
        </button>
        <button
          type="button"
          onClick={() =>
            onSetRejectReason({
              id: ticket.ticket_id,
              text: 'Operator rejected execution.',
            })
          }
          className="px-3 py-1.5 rounded bg-rose-950 hover:bg-rose-900 text-rose-300 border border-rose-800/80 font-bold text-xs flex items-center space-x-1.5"
        >
          <XCircle className="w-3.5 h-3.5" />
          <span>Reject Execution</span>
        </button>
        <button
          type="button"
          onClick={() =>
            onSetEditingTicket({
              id: ticket.ticket_id,
              argsStr: JSON.stringify(ticket.arguments, null, 2),
            })
          }
          className="px-3 py-1.5 rounded bg-syndae-800 hover:bg-syndae-700 text-cyan-300 border border-cyan-800/80 font-bold text-xs flex items-center space-x-1.5"
        >
          <Edit3 className="w-3.5 h-3.5" />
          <span>Edit Arguments</span>
        </button>
      </div>
    </div>
  );
};
