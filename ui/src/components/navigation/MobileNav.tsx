import React from 'react';
import { MessageSquare, Settings } from 'lucide-react';
import { ApprovalTicket } from '../../types';

export interface MobileNavProps {
  activeTab: 'chat' | 'settings' | 'models' | 'memory' | 'tools';
  pendingTickets: ApprovalTicket[];
  onSelectTab: (tab: 'chat' | 'settings' | 'models' | 'memory' | 'tools') => void;
}

export const MobileNav: React.FC<MobileNavProps> = ({
  activeTab,
  pendingTickets,
  onSelectTab,
}) => {
  return (
    <div className="md:hidden flex-shrink-0 bg-styx-900 border-t border-styx-800 px-6 py-2 pb-[max(0.75rem,env(safe-area-inset-bottom))] flex items-center justify-around z-30">
      <button
        type="button"
        onClick={() => onSelectTab('chat')}
        className={`flex flex-col items-center justify-center py-1 px-4 rounded-lg text-xs font-medium transition-colors ${
          activeTab === 'chat' ? 'text-emerald-400 font-semibold' : 'text-slate-400 hover:text-slate-200'
        }`}
      >
        <div className="relative">
          <MessageSquare className="w-5 h-5" />
          {pendingTickets.length > 0 && (
            <span className="absolute -top-1 -right-2 bg-rose-600 text-white rounded-full px-1 text-[9px] font-bold animate-pulse">
              {pendingTickets.length}
            </span>
          )}
        </div>
        <span className="text-[10px] mt-1 font-mono">Chat</span>
      </button>

      <button
        type="button"
        onClick={() => onSelectTab('settings')}
        className={`flex flex-col items-center justify-center py-1 px-4 rounded-lg text-xs font-medium transition-colors ${
          activeTab === 'settings' || activeTab === 'models' || activeTab === 'memory' || activeTab === 'tools'
            ? 'text-emerald-400 font-semibold'
            : 'text-slate-400 hover:text-slate-200'
        }`}
      >
        <Settings className="w-5 h-5" />
        <span className="text-[10px] mt-1 font-mono">Settings</span>
      </button>
    </div>
  );
};
