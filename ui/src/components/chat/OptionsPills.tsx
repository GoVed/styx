import React from 'react';
import { Sparkles, Edit3, Send } from 'lucide-react';
import { InteractiveOption } from './types';

export interface OptionsPillsProps {
  messageId: string;
  options: InteractiveOption[];
  hasUserReplied: boolean;
  isStreaming: boolean;
  activeOtherMsgId: string | null;
  otherInputText: string;
  nextMessageContent?: string;
  onSendOption: (optionLabel: string) => void;
  onToggleOther: (msgId: string) => void;
  onChangeOtherText: (text: string) => void;
  onSubmitOther: (text: string) => void;
}

export const OptionsPills: React.FC<OptionsPillsProps> = ({
  messageId,
  options,
  hasUserReplied,
  isStreaming,
  activeOtherMsgId,
  otherInputText,
  nextMessageContent,
  onSendOption,
  onToggleOther,
  onChangeOtherText,
  onSubmitOther,
}) => {
  if (options.length === 0) return null;

  return (
    <div className="w-full max-w-3xl mt-2 rounded-lg bg-syndae-950/80 border border-emerald-500/30 p-2.5 space-y-2 shadow-sm">
      <div className="flex items-center justify-between text-[11px] font-mono">
        <span className="flex items-center space-x-1.5 font-semibold text-emerald-400">
          <Sparkles className="w-3.5 h-3.5 text-emerald-400" />
          <span>
            {hasUserReplied ? 'OPTIONS OFFERED:' : 'CHOOSE AN OPTION OR SPECIFY DETAILS:'}
          </span>
        </span>
        {!hasUserReplied && (
          <span className="text-[10px] text-slate-500 font-mono">1-click reply</span>
        )}
      </div>

      <div className="flex flex-wrap gap-2">
        {options.map(opt => {
          const isSelectedByFollowup =
            hasUserReplied &&
            nextMessageContent
              ?.trim()
              .toLowerCase()
              .includes(opt.label.trim().toLowerCase());

          if (opt.isOther) {
            return (
              <button
                key={opt.id}
                type="button"
                onClick={() => {
                  if (hasUserReplied || isStreaming) return;
                  onToggleOther(messageId);
                }}
                disabled={hasUserReplied || isStreaming}
                className={`px-3 py-1.5 rounded-md text-xs font-mono transition-all flex items-center space-x-1.5 shadow-sm ${
                  hasUserReplied
                    ? 'bg-syndae-900/50 text-slate-500 border border-syndae-800 cursor-default'
                    : activeOtherMsgId === messageId
                    ? 'bg-purple-900/80 text-purple-200 border border-purple-400 ring-1 ring-purple-400'
                    : 'bg-purple-950/40 hover:bg-purple-900/60 text-purple-300 border border-purple-500/40 hover:border-purple-400 cursor-pointer active:scale-95'
                }`}
              >
                <Edit3 className="w-3 h-3 text-purple-400" />
                <span>{opt.label}</span>
              </button>
            );
          }

          return (
            <div
              key={opt.id}
              className={`group inline-flex items-center rounded-md text-xs transition-all shadow-sm ${
                isSelectedByFollowup
                  ? 'bg-emerald-950 text-emerald-300 border border-emerald-500/70 ring-1 ring-emerald-500/50'
                  : hasUserReplied
                  ? 'bg-syndae-900/50 text-slate-500 border border-syndae-800'
                  : 'bg-syndae-850 hover:bg-emerald-950/60 text-slate-200 hover:text-emerald-200 border border-syndae-700/80 hover:border-emerald-500/60'
              }`}
            >
              <button
                type="button"
                onClick={() => {
                  if (hasUserReplied || isStreaming) return;
                  onSendOption(opt.label);
                }}
                disabled={hasUserReplied || isStreaming}
                className={`px-3 py-1.5 font-medium flex items-center space-x-1.5 ${
                  hasUserReplied ? 'cursor-default' : 'cursor-pointer active:scale-95'
                }`}
              >
                <span>{opt.label}</span>
              </button>
            </div>
          );
        })}
      </div>

      {/* Inline custom prompt when "Other..." pill is activated */}
      {activeOtherMsgId === messageId && !hasUserReplied && (
        <div className="pt-2 border-t border-purple-900/40 flex items-center space-x-2">
          <input
            type="text"
            value={otherInputText}
            onChange={e => onChangeOtherText(e.target.value)}
            onKeyDown={e => {
              if (e.key === 'Enter' && otherInputText.trim() && !isStreaming) {
                onSubmitOther(otherInputText.trim());
              }
            }}
            placeholder="Type custom response or details..."
            autoFocus
            className="flex-1 bg-syndae-900 border border-purple-800/80 rounded px-2.5 py-1 text-xs text-slate-100 placeholder-slate-500 focus:outline-none focus:border-purple-500"
          />
          <button
            type="button"
            onClick={() => {
              if (otherInputText.trim() && !isStreaming) {
                onSubmitOther(otherInputText.trim());
              }
            }}
            disabled={!otherInputText.trim() || isStreaming}
            className="px-3 py-1 rounded bg-purple-700 hover:bg-purple-600 disabled:opacity-40 text-white text-xs font-semibold flex items-center space-x-1"
          >
            <span>Send</span>
            <Send className="w-3 h-3" />
          </button>
        </div>
      )}
    </div>
  );
};
