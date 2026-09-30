import React, { useState, useEffect, useRef } from 'react';
import { Bot, Zap, BrainCircuit, Menu, Sparkles, Settings } from 'lucide-react';
import { ChatMessage } from '../types';
import {
  extractThoughtAndContent,
  extractOptionsAndContent,
  parseToolEvent,
  renderMarkdown,
} from './chat/helpers';
import {
  InteractiveOption,
  ParsedToolEvent,
  ToolExecutionCard,
  ChatViewProps,
} from './chat/types';
import { SessionSidebar } from './chat/SessionSidebar';
import { MessageItem } from './chat/MessageItem';
import { StreamingResponse } from './chat/StreamingResponse';
import { ApprovalTicketCard } from './chat/ApprovalTicketCard';
import { ChatInputBar } from './chat/ChatInputBar';

export {
  extractThoughtAndContent,
  extractOptionsAndContent,
  parseToolEvent,
  renderMarkdown,
};
export type {
  InteractiveOption,
  ParsedToolEvent,
  ToolExecutionCard,
  ChatViewProps,
};

export const ChatView: React.FC<ChatViewProps> = ({
  sessions,
  activeSession,
  onSelectSession,
  onCreateSession,
  onDeleteSession,
  pendingTickets,
  onResolveTicket,
  memoryFiles,
  onSendPrompt,
  liveThought,
  liveStreamingText,
  liveToolExecutions,
  isStreaming,
  chatError,
  onClearChatError,
  messages: propMessages,
  onNavigateToSettings,
}) => {
  const [inputPrompt, setInputPrompt] = useState('');
  const [internalMessages, setInternalMessages] = useState<ChatMessage[]>([]);
  const messages = propMessages ?? internalMessages;
  const [mode, setMode] = useState<'chat' | 'mission'>(activeSession?.mode === 'mission' ? 'mission' : 'chat');
  const [selectedSkill, setSelectedSkill] = useState<string>('');
  const [editingTicket, setEditingTicket] = useState<{ id: string; argsStr: string } | null>(null);
  const [rejectReason, setRejectReason] = useState<{ id: string; text: string } | null>(null);
  const [expandedHistoricalThoughts, setExpandedHistoricalThoughts] = useState<Record<string, boolean>>({});
  const [activeOtherMsgId, setActiveOtherMsgId] = useState<string | null>(null);
  const [otherInputText, setOtherInputText] = useState('');
  const [mobileSidebarOpen, setMobileSidebarOpen] = useState(false);

  const chatBottomRef = useRef<HTMLDivElement>(null);
  const textareaRef = useRef<HTMLTextAreaElement>(null);

  const handleSendOption = (optionLabel: string) => {
    if (!activeSession || isStreaming) return;
    onSendPrompt(activeSession.id, optionLabel, mode);
  };

  useEffect(() => {
    if (activeSession) {
      setMode(activeSession.mode === 'mission' ? 'mission' : 'chat');
      if (!propMessages) {
        loadInternalMessages(activeSession.id);
      }
    }
  }, [activeSession?.id, propMessages]);

  const loadInternalMessages = async (sessionId: string) => {
    try {
      const res = await fetch(`/api/chat/sessions/${sessionId}/messages`);
      const data = await res.json();
      if (data.success) {
        setInternalMessages(data.messages);
      }
    } catch (e) {
      console.error('Failed to load messages', e);
    }
  };

  useEffect(() => {
    chatBottomRef.current?.scrollIntoView?.({ behavior: 'smooth' });
  }, [messages, liveStreamingText, liveThought, liveToolExecutions]);

  const handleSend = () => {
    if (!inputPrompt.trim() || !activeSession || isStreaming) return;

    let finalPrompt = inputPrompt;
    if (selectedSkill) {
      finalPrompt = `[Context Skillset: ${selectedSkill}]\n${inputPrompt}`;
    }

    onSendPrompt(activeSession.id, finalPrompt, mode);
    setInputPrompt('');
    setSelectedSkill('');
  };

  const sessionTickets = pendingTickets.filter(
    t => !activeSession || t.session_id === activeSession.id
  );

  return (
    <div className="relative flex h-full w-full overflow-hidden bg-styx-950 font-sans">
      {/* Session Drawer / Sidebar */}
      <SessionSidebar
        sessions={sessions}
        activeSession={activeSession}
        mobileSidebarOpen={mobileSidebarOpen}
        selectedSkill={selectedSkill}
        memoryFiles={memoryFiles}
        onSelectSession={onSelectSession}
        onCreateSession={onCreateSession}
        onDeleteSession={onDeleteSession}
        onCloseMobileSidebar={() => setMobileSidebarOpen(false)}
        onChangeSkill={setSelectedSkill}
        onNavigateToSettings={onNavigateToSettings}
      />

      {/* Central Chat Stream */}
      <main className="flex-1 flex flex-col h-full min-h-0 overflow-hidden bg-styx-950">
        {/* Chat Header / Mode Bar with Safe Area Top Support */}
        <div className="p-2 sm:p-2.5 px-3 sm:px-4 pt-[max(0.6rem,env(safe-area-inset-top))] md:pt-2.5 border-b border-styx-800 bg-styx-900 flex items-center justify-between font-mono text-xs flex-shrink-0 z-20">
          <div className="flex items-center space-x-2 sm:space-x-3 min-w-0">
            <button
              type="button"
              onClick={() => setMobileSidebarOpen(true)}
              className="md:hidden p-1.5 rounded-lg bg-styx-850 hover:bg-styx-800 text-slate-300 border border-styx-700/80 flex-shrink-0 transition-colors"
              title="Open workspaces drawer"
            >
              <Menu className="w-4 h-4" />
            </button>

            <span className="font-semibold text-slate-200 truncate max-w-[100px] sm:max-w-xs text-xs">
              {activeSession?.title || 'No active session'}
            </span>

            {/* Mode Switcher */}
            <div className="flex bg-styx-950 p-0.5 rounded border border-styx-800">
              <button
                type="button"
                onClick={() => setMode('chat')}
                className={`px-2 sm:px-2.5 py-1 rounded text-[11px] transition-colors flex items-center space-x-1 ${
                  mode === 'chat'
                    ? 'bg-styx-800 text-emerald-300 font-semibold shadow'
                    : 'text-slate-400 hover:text-slate-200'
                }`}
                title="Conversational Chat Mode"
              >
                <Bot className="w-3 h-3" />
                <span className="hidden xs:inline">Chat</span>
              </button>
              <button
                type="button"
                onClick={() => setMode('mission')}
                className={`px-2 sm:px-2.5 py-1 rounded text-[11px] transition-colors flex items-center space-x-1 ${
                  mode === 'mission'
                    ? 'bg-cyan-950 text-cyan-300 font-semibold shadow border border-cyan-800/60'
                    : 'text-slate-400 hover:text-slate-200'
                }`}
                title="Deep Task Mode"
              >
                <Zap className="w-3 h-3 text-cyan-400" />
                <span className="hidden xs:inline">Task</span>
              </button>
            </div>
          </div>

          <div className="flex items-center space-x-1.5 sm:space-x-2">
            {isStreaming && (
              <div className="flex items-center space-x-1 text-amber-400 text-xs animate-pulse mr-1">
                <Sparkles className="w-3.5 h-3.5" />
                <span className="hidden sm:inline">THINKING...</span>
              </div>
            )}

            {/* Mobile Settings Shortcut */}
            {onNavigateToSettings && (
              <button
                type="button"
                onClick={onNavigateToSettings}
                className="md:hidden p-1.5 rounded-lg bg-styx-850 hover:bg-styx-800 text-slate-300 hover:text-emerald-400 border border-styx-700/80 transition-colors"
                title="Open Settings"
              >
                <Settings className="w-4 h-4" />
              </button>
            )}
          </div>
        </div>

        {/* Messages Feed */}
        <div className="flex-1 overflow-y-auto overflow-x-hidden p-4 space-y-4 min-w-0">
          {messages.length === 0 && !isStreaming && (
            <div className="flex flex-col items-center justify-center h-full text-center max-w-md mx-auto space-y-3">
              <div className="p-3 rounded-full bg-styx-900 border border-styx-700/60 text-emerald-400">
                <BrainCircuit className="w-8 h-8" />
              </div>
              <h3 className="font-mono text-sm font-bold text-slate-200">STYX AGENT READY</h3>
              <p className="text-xs text-slate-400 leading-relaxed">
                Connect directly to your local containerized models (vLLM, llama.cpp, Ollama) or external APIs.
                Autonomous reads &amp; searches operate with zero overhead. Mutating tool actions halt automatically for human approval.
              </p>
            </div>
          )}

          {/* Historical Messages */}
          {messages.map((msg, index) => (
            <MessageItem
              key={msg.id}
              message={msg}
              nextMessage={messages[index + 1]}
              isStreaming={isStreaming}
              isThoughtOpen={expandedHistoricalThoughts[msg.id] ?? false}
              activeOtherMsgId={activeOtherMsgId}
              otherInputText={otherInputText}
              onToggleThought={id =>
                setExpandedHistoricalThoughts(prev => ({
                  ...prev,
                  [id]: !prev[id],
                }))
              }
              onSendOption={handleSendOption}
              onToggleOther={msgId => {
                setActiveOtherMsgId(activeOtherMsgId === msgId ? null : msgId);
                setOtherInputText('');
              }}
              onChangeOtherText={setOtherInputText}
              onSubmitOther={text => {
                handleSendOption(text);
                setActiveOtherMsgId(null);
                setOtherInputText('');
              }}
            />
          ))}

          {/* Real-time Streaming Response Block */}
          {isStreaming && (
            <StreamingResponse
              liveStreamingText={liveStreamingText}
              liveThought={liveThought}
              liveToolExecutions={liveToolExecutions}
            />
          )}

          {/* INLINE HUMAN-IN-THE-LOOP APPROVAL BANNERS */}
          {sessionTickets.map(ticket => (
            <ApprovalTicketCard
              key={ticket.ticket_id}
              ticket={ticket}
              editingTicket={editingTicket}
              rejectReason={rejectReason}
              onSetEditingTicket={setEditingTicket}
              onSetRejectReason={setRejectReason}
              onResolveTicket={onResolveTicket}
            />
          ))}

          <div ref={chatBottomRef} />
        </div>

        {/* Input Bar */}
        <ChatInputBar
          inputPrompt={inputPrompt}
          isStreaming={isStreaming}
          mode={mode}
          chatError={chatError}
          textareaRef={textareaRef}
          onChangePrompt={setInputPrompt}
          onSend={handleSend}
          onClearChatError={onClearChatError}
        />
      </main>
    </div>
  );
};
