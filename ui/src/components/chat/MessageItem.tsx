import React from 'react';
import { Bot, User, MessageSquare, BrainCircuit, ChevronDown, ChevronRight, CornerDownRight } from 'lucide-react';
import { ChatMessage } from '../../types';
import { extractThoughtAndContent, extractOptionsAndContent, parseToolEvent, renderMarkdown } from './helpers';
import { OptionsPills } from './OptionsPills';

export interface MessageItemProps {
  message: ChatMessage;
  nextMessage?: ChatMessage;
  isStreaming: boolean;
  isThoughtOpen: boolean;
  activeOtherMsgId: string | null;
  otherInputText: string;
  onToggleThought: (id: string) => void;
  onSendOption: (option: string) => void;
  onToggleOther: (msgId: string) => void;
  onChangeOtherText: (text: string) => void;
  onSubmitOther: (text: string) => void;
}

export const MessageItem: React.FC<MessageItemProps> = ({
  message,
  nextMessage,
  isStreaming,
  isThoughtOpen,
  activeOtherMsgId,
  otherInputText,
  onToggleThought,
  onSendOption,
  onToggleOther,
  onChangeOtherText,
  onSubmitOther,
}) => {
  const isUser = message.role === 'user';
  const parsedToolEvent = isUser ? parseToolEvent(message.content) : null;
  const isToolEvent = !!parsedToolEvent;

  const { thought, content: cleanThoughtContent } = isUser
    ? { thought: null, content: message.content }
    : extractThoughtAndContent(message.content, message.thought);

  const { content: cleanContent, options } = isUser
    ? { content: cleanThoughtContent, options: [] }
    : extractOptionsAndContent(cleanThoughtContent);

  const isNextMessageToolEvent = !!(
    nextMessage &&
    nextMessage.role === 'user' &&
    parseToolEvent(nextMessage.content)
  );
  const hasUserReplied = !!nextMessage && nextMessage.role === 'user' && !isNextMessageToolEvent;

  return (
    <div className={`flex flex-col w-full max-w-full min-w-0 ${isUser && !isToolEvent ? 'items-end' : 'items-start'}`}>
      <div className="flex items-center space-x-1.5 mb-1 px-1 text-[11px] font-mono text-slate-500 max-w-full overflow-hidden">
        {isToolEvent ? (
          <>
            <div className="w-2 h-2 rounded-full bg-emerald-500 animate-pulse shrink-0" />
            <span className="text-emerald-400 font-semibold truncate">{parsedToolEvent.senderName}</span>
            <span className="text-slate-500 shrink-0">via</span>
            <span className="px-1.5 py-0.5 rounded bg-emerald-950/80 border border-emerald-800/60 text-emerald-300 font-mono text-[10px] shrink-0">
              {parsedToolEvent.protocol}
            </span>
            {parsedToolEvent.isGroup && (
              <span className="px-1.5 py-0.5 rounded bg-slate-800 border border-slate-700 text-slate-400 font-mono text-[10px] shrink-0">
                Group
              </span>
            )}
            <MessageSquare className="w-3 h-3 text-emerald-400 ml-1 shrink-0" />
          </>
        ) : isUser ? (
          <>
            <span>OPERATOR</span>
            <User className="w-3 h-3 text-slate-400" />
          </>
        ) : (
          <>
            <Bot className="w-3 h-3 text-emerald-400" />
            <span>STYX AGENT</span>
          </>
        )}
      </div>

      {/* Collapsible Thought / Reasoning Trace Block */}
      {thought && (
        <details open className="w-full max-w-3xl mb-2 rounded-lg bg-styx-900/60 border border-purple-900/30 text-xs font-mono p-2 shadow-sm transition-all group min-w-0 overflow-hidden">
          <summary className="flex items-center justify-between w-full text-left text-purple-400/90 text-[11px] font-medium select-none hover:text-purple-300 transition-colors py-0.5 px-1 cursor-pointer list-none">
            <div className="flex items-center space-x-1.5">
              <BrainCircuit className="w-3.5 h-3.5 text-purple-400" />
              <span>Reasoning Trace</span>
              <span className="text-[10px] text-purple-400/60 font-normal">
                (click to toggle)
              </span>
            </div>
            <ChevronDown className="w-3.5 h-3.5 group-open:rotate-180 transition-transform" />
          </summary>
          <div className="whitespace-pre-wrap text-slate-300 leading-relaxed max-h-56 overflow-y-auto pt-2 border-t border-purple-900/30 text-[11px] mt-1 break-words">
            {thought}
          </div>
        </details>
      )}

      {/* Message Content */}
      {isToolEvent ? (
        <div className="w-full max-w-full sm:max-w-3xl rounded-lg p-3.5 text-xs leading-relaxed bg-gradient-to-r from-emerald-950/60 via-styx-900 to-styx-900 border border-emerald-500/40 shadow-sm space-y-2 min-w-0 overflow-hidden">
          <div className="flex items-center justify-between text-[11px] font-mono text-emerald-300/90 border-b border-emerald-900/40 pb-1.5 gap-2">
            <span className="font-semibold flex items-center space-x-1 shrink-0">
              <span>Incoming {parsedToolEvent.protocol} Message</span>
            </span>
            <span className="text-slate-400 text-[10px] truncate max-w-[200px] text-right">{parsedToolEvent.channel}</span>
          </div>
          {parsedToolEvent.replyTo && (
            <div className="flex items-start gap-1.5 px-2.5 py-1.5 rounded bg-emerald-950/40 border-l-2 border-emerald-400/80 text-[11px] text-slate-300 font-sans break-words">
              <CornerDownRight className="w-3.5 h-3.5 text-emerald-400 shrink-0 mt-0.5" />
              <div className="min-w-0">
                <span className="text-emerald-400 font-mono text-[10px] mr-1.5 font-semibold">Replying to:</span>
                <span className="text-slate-200 italic">{parsedToolEvent.replyTo}</span>
              </div>
            </div>
          )}
          <div className="text-slate-100 text-sm font-sans font-medium px-0.5 leading-normal break-words">
            "{parsedToolEvent.messageText}"
          </div>
          {parsedToolEvent.rawContent && (
            <details className="text-[11px] text-slate-500 font-mono pt-1">
              <summary className="cursor-pointer hover:text-slate-300 select-none">
                Technical event details
              </summary>
              <div className="mt-1.5 p-2 rounded bg-styx-950/80 border border-slate-800 text-[10px] text-slate-400 whitespace-pre-wrap max-h-40 overflow-y-auto overflow-x-auto break-all">
                {parsedToolEvent.rawContent}
              </div>
            </details>
          )}
        </div>
      ) : cleanContent && (
        <div
          className={`max-w-[88%] sm:max-w-3xl rounded-lg p-3 text-xs leading-relaxed min-w-0 overflow-hidden break-words ${
            isUser
              ? 'bg-styx-800 text-slate-100 border border-styx-700/60'
              : 'bg-styx-900 text-slate-200 border border-styx-800 shadow-sm'
          }`}
        >
          <div
            className="prose max-w-none break-words min-w-0 overflow-hidden"
            dangerouslySetInnerHTML={{ __html: renderMarkdown(cleanContent) }}
          />
        </div>
      )}

      {/* Interactive Options Pills */}
      {!isUser && options.length > 0 && (
        <OptionsPills
          messageId={message.id}
          options={options}
          hasUserReplied={hasUserReplied}
          isStreaming={isStreaming}
          activeOtherMsgId={activeOtherMsgId}
          otherInputText={otherInputText}
          nextMessageContent={nextMessage?.content}
          onSendOption={onSendOption}
          onToggleOther={onToggleOther}
          onChangeOtherText={onChangeOtherText}
          onSubmitOther={onSubmitOther}
        />
      )}
    </div>
  );
};
