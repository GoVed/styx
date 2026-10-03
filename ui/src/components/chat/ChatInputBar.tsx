import React from 'react';
import { Send, AlertTriangle } from 'lucide-react';

export interface ChatInputBarProps {
  inputPrompt: string;
  isStreaming: boolean;
  mode: 'chat' | 'mission';
  chatError?: string | null;
  textareaRef: React.RefObject<HTMLTextAreaElement>;
  onChangePrompt: (val: string) => void;
  onSend: () => void;
  onClearChatError?: () => void;
}

export const ChatInputBar: React.FC<ChatInputBarProps> = ({
  inputPrompt,
  isStreaming,
  mode,
  chatError,
  textareaRef,
  onChangePrompt,
  onSend,
  onClearChatError,
}) => {
  const handleKeyDown = (e: React.KeyboardEvent<HTMLTextAreaElement>) => {
    if (e.key === 'Enter' && !e.shiftKey) {
      e.preventDefault();
      onSend();
    }
  };

  return (
    <div className="p-2 sm:p-3 pb-[max(0.6rem,env(safe-area-inset-bottom))] sm:pb-3 md:pb-3.5 border-t border-syndae-800 bg-syndae-900 font-mono flex-shrink-0 space-y-1.5 sm:space-y-2">
      {chatError && (
        <div className="p-2.5 sm:p-3 rounded-lg bg-rose-950/80 border border-rose-800 text-rose-200 text-xs font-mono shadow-lg flex items-start space-x-2.5">
          <AlertTriangle className="w-4 h-4 text-rose-400 flex-shrink-0 mt-0.5" />
          <div className="flex-1 space-y-1">
            <div className="font-bold text-rose-300 flex items-center justify-between">
              <span>Inference Engine Notice</span>
              {onClearChatError && (
                <button
                  type="button"
                  onClick={onClearChatError}
                  className="text-slate-400 hover:text-slate-200 text-[10px] uppercase font-bold"
                >
                  Dismiss [✕]
                </button>
              )}
            </div>
            <div className="text-slate-300 text-[11px] leading-relaxed whitespace-pre-wrap">{chatError}</div>
          </div>
        </div>
      )}

      <div className="relative flex items-center bg-syndae-950 border border-syndae-700 rounded-lg focus-within:border-emerald-500 focus-within:ring-1 focus-within:ring-emerald-500/40">
        <textarea
          ref={textareaRef}
          rows={2}
          value={inputPrompt}
          onChange={e => onChangePrompt(e.target.value)}
          onKeyDown={handleKeyDown}
          disabled={isStreaming}
          placeholder={
            mode === 'mission'
              ? 'Define autonomous mission goal or project (Syndae will organize steps & assist)...'
              : 'Ask me anything, describe a task, or type a message...'
          }
          className="w-full bg-transparent px-3 py-2 text-xs text-slate-100 placeholder-slate-500 resize-none focus:outline-none"
        />
        <button
          type="button"
          onClick={onSend}
          disabled={isStreaming || !inputPrompt.trim()}
          className={`mr-2 p-2 rounded-md transition-colors ${
            isStreaming || !inputPrompt.trim()
              ? 'text-slate-600 cursor-not-allowed'
              : 'bg-emerald-600 hover:bg-emerald-500 text-white shadow'
          }`}
        >
          <Send className="w-4 h-4" />
        </button>
      </div>
      <div className="hidden sm:flex items-center justify-between mt-1 text-[10px] text-slate-500">
        <span>Press Enter to send, Shift+Enter for newline</span>
        <span>100% Private • Safe Mode Active</span>
      </div>
    </div>
  );
};
