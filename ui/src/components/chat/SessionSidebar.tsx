import React from 'react';
import { X, Plus, Zap, Bot, Trash2, FolderOpen, Settings } from 'lucide-react';
import { ChatSession, MemoryFileNode } from '../../types';

export interface SessionSidebarProps {
  sessions: ChatSession[];
  activeSession: ChatSession | null;
  mobileSidebarOpen: boolean;
  selectedSkill: string;
  memoryFiles: MemoryFileNode[];
  onSelectSession: (s: ChatSession) => void;
  onCreateSession: (mode: 'chat' | 'mission') => void;
  onDeleteSession: (id: string) => void;
  onCloseMobileSidebar: () => void;
  onChangeSkill: (skillPath: string) => void;
  onNavigateToSettings?: () => void;
}

export const SessionSidebar: React.FC<SessionSidebarProps> = ({
  sessions,
  activeSession,
  mobileSidebarOpen,
  selectedSkill,
  memoryFiles,
  onSelectSession,
  onCreateSession,
  onDeleteSession,
  onCloseMobileSidebar,
  onChangeSkill,
  onNavigateToSettings,
}) => {
  return (
    <>
      {/* Mobile Drawer Backdrop */}
      {mobileSidebarOpen && (
        <div
          onClick={onCloseMobileSidebar}
          className="fixed inset-0 bg-black/70 z-40 md:hidden backdrop-blur-sm transition-opacity"
        />
      )}

      {/* Session Drawer / Sidebar */}
      <aside
        className={`flex-shrink-0 fixed top-0 bottom-auto left-0 z-50 w-72 bg-syndae-900 border-r border-syndae-800 shadow-2xl flex flex-col overflow-hidden transition-transform duration-300 md:relative md:w-64 md:h-full md:shadow-none max-md:h-[var(--app-height,100dvh)] max-md:max-h-[var(--app-height,100dvh)] ${
          mobileSidebarOpen ? 'translate-x-0' : '-translate-x-full md:translate-x-0'
        }`}
      >
        <div className="p-3 pt-[max(0.75rem,env(safe-area-inset-top))] md:pt-3 border-b border-syndae-800 flex items-center justify-between flex-shrink-0">
          <div className="flex items-center space-x-2">
            <button
              type="button"
              onClick={onCloseMobileSidebar}
              className="md:hidden p-1 rounded hover:bg-syndae-800 text-slate-400 hover:text-white"
              title="Close drawer"
            >
              <X className="w-4 h-4" />
            </button>
            <span className="text-xs font-mono font-bold text-slate-300 tracking-wider">CHATS</span>
          </div>
          <div className="flex space-x-1">
            <button
              type="button"
              onClick={() => {
                onCreateSession('chat');
                onCloseMobileSidebar();
              }}
              className="p-1.5 rounded bg-syndae-800 hover:bg-syndae-750 text-emerald-400 text-xs flex items-center space-x-1 border border-syndae-700/60"
              title="Start new chat"
            >
              <Plus className="w-3.5 h-3.5" />
              <span className="font-mono text-[10px]">Chat</span>
            </button>
            <button
              type="button"
              onClick={() => {
                onCreateSession('mission');
                onCloseMobileSidebar();
              }}
              className="p-1.5 rounded bg-syndae-800 hover:bg-syndae-750 text-cyan-400 text-xs flex items-center space-x-1 border border-syndae-700/60"
              title="Start new deep task"
            >
              <Zap className="w-3.5 h-3.5" />
              <span className="font-mono text-[10px]">Task</span>
            </button>
          </div>
        </div>

        {/* Sessions List */}
        <div className="flex-1 min-h-0 overflow-y-auto p-2 space-y-1">
          {sessions.map(s => {
            const isSelected = s.id === activeSession?.id;
            return (
              <div
                key={s.id}
                onClick={() => {
                  onSelectSession(s);
                  onCloseMobileSidebar();
                }}
                className={`group flex items-center justify-between p-2 rounded cursor-pointer text-xs transition-colors border ${
                  isSelected
                    ? 'bg-syndae-800 text-slate-100 border-emerald-500/40'
                    : 'text-slate-400 hover:bg-syndae-850 hover:text-slate-200 border-transparent'
                }`}
              >
                <div className="flex items-center space-x-2 truncate">
                  {s.mode === 'mission' ? (
                    <Zap className="w-3.5 h-3.5 text-cyan-400 flex-shrink-0" />
                  ) : (
                    <Bot className="w-3.5 h-3.5 text-emerald-400 flex-shrink-0" />
                  )}
                  <span className="truncate font-mono">{s.title || 'Untitled Session'}</span>
                </div>
                <button
                  type="button"
                  onClick={e => {
                    e.stopPropagation();
                    onDeleteSession(s.id);
                  }}
                  className="opacity-0 group-hover:opacity-100 p-1 hover:text-rose-400 text-slate-500 transition-opacity"
                >
                  <Trash2 className="w-3 h-3" />
                </button>
              </div>
            );
          })}
        </div>

        {/* Active Skills Selector Info */}
        <div className="p-3 border-t border-syndae-800 bg-syndae-950 text-[11px] font-mono flex-shrink-0">
          <div className="text-slate-400 mb-1 flex items-center space-x-1">
            <FolderOpen className="w-3 h-3 text-emerald-400" />
            <span>AVAILABLE SKILLSETS</span>
          </div>
          <select
            value={selectedSkill}
            onChange={e => onChangeSkill(e.target.value)}
            className="w-full bg-syndae-900 border border-syndae-700 text-slate-200 rounded p-1.5 text-xs focus:outline-none focus:border-emerald-500"
          >
            <option value="">Attach assistant skill (optional)...</option>
            {memoryFiles
              .filter(f => f.category === 'skills')
              .map(f => (
                <option key={f.path} value={f.path}>
                  {f.title || f.filename}
                </option>
              ))}
          </select>
        </div>

        {/* Mobile Settings Shortcut at Drawer Bottom */}
        {onNavigateToSettings && (
          <div className="p-2.5 pb-[max(0.75rem,env(safe-area-inset-bottom))] border-t border-syndae-800 bg-syndae-900 md:hidden flex-shrink-0">
            <button
              type="button"
              onClick={() => {
                onCloseMobileSidebar();
                onNavigateToSettings();
              }}
              className="w-full flex items-center justify-center space-x-2 py-2 px-3 rounded-lg bg-syndae-800 hover:bg-syndae-750 text-slate-200 text-xs font-medium border border-syndae-700/60 transition-colors"
            >
              <Settings className="w-3.5 h-3.5 text-emerald-400" />
              <span>Mission Control & Settings</span>
            </button>
          </div>
        )}
      </aside>
    </>
  );
};
