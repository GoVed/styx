import React, { useState } from 'react';
import { Cpu, MessageSquare, Settings, Activity, User, Lock, Bell } from 'lucide-react';
import { AuthStatus, SystemTelemetry, ApprovalTicket } from '../../types';
import {
  isNotificationSupported,
  getNotificationPermission,
  requestNotificationPermission,
  playNotificationChime,
} from '../../utils/notifications';

export interface TopNavBarProps {
  activeTab: 'chat' | 'settings' | 'models' | 'memory' | 'tools';
  authStatus: AuthStatus | null;
  telemetry: SystemTelemetry | null;
  pendingTickets: ApprovalTicket[];
  showTelemetryStrip: boolean;
  onSelectTab: (tab: 'chat' | 'settings' | 'models' | 'memory' | 'tools') => void;
  onToggleTelemetryStrip: () => void;
  onLock: () => void;
}

export const TopNavBar: React.FC<TopNavBarProps> = ({
  activeTab,
  authStatus,
  telemetry,
  pendingTickets,
  showTelemetryStrip,
  onSelectTab,
  onToggleTelemetryStrip,
  onLock,
}) => {
  const [notifPerm, setNotifPerm] = useState<NotificationPermission>(() => getNotificationPermission());

  const handleToggleNotifications = async () => {
    if (notifPerm === 'default') {
      const res = await requestNotificationPermission();
      setNotifPerm(res);
      playNotificationChime();
    } else {
      playNotificationChime();
    }
  };
  return (
    <nav className="flex-shrink-0 bg-styx-900 border-b border-styx-800 px-3 sm:px-4 py-2 pt-[max(0.5rem,env(safe-area-inset-top))] flex items-center justify-between text-xs font-mono z-30">
      <div className="flex items-center space-x-2.5 sm:space-x-3">
        <div
          onClick={() => onSelectTab('chat')}
          className="flex items-center space-x-2 cursor-pointer group"
          title="Return to Chat"
        >
          <div className="w-2.5 h-2.5 rounded-full bg-emerald-500 animate-pulse shadow-[0_0_8px_rgba(16,185,129,0.8)]" />
          <span className="font-bold tracking-wider text-sm sm:text-base text-slate-100 font-sans group-hover:text-emerald-400 transition-colors">
            STYX
          </span>
          <span className="text-[10px] font-mono px-1.5 py-0.5 rounded bg-emerald-950/80 text-emerald-400 border border-emerald-500/40">
            AI
          </span>
        </div>

        {/* Active Model Pill - Clickable to open Model Orchestrator */}
        <button
          type="button"
          onClick={() => onSelectTab('models')}
          title="Click to view AI engine settings"
          className="hidden sm:flex items-center space-x-1.5 px-2.5 py-1 rounded-full bg-styx-800/80 hover:bg-styx-750 border border-styx-700/60 text-slate-300 hover:text-emerald-300 transition-colors text-[11px]"
        >
          <Cpu className="w-3 h-3 text-emerald-400 flex-shrink-0" />
          <span className="truncate max-w-[130px] md:max-w-[200px] font-mono">
            {telemetry?.active_model?.split(':')[0]?.replace('QuantTrio/', '') || 'Smart Assistant'}
          </span>
        </button>
      </div>

      {/* Center Desktop Navigation Tabs */}
      <div className="hidden md:flex items-center space-x-1 bg-styx-950/60 p-1 rounded-lg border border-styx-800/80">
        <button
          type="button"
          onClick={() => onSelectTab('chat')}
          className={`px-3.5 py-1.5 rounded-md flex items-center space-x-2 font-medium transition-all ${
            activeTab === 'chat'
              ? 'bg-styx-800 text-emerald-300 shadow-sm border border-emerald-500/30'
              : 'text-slate-400 hover:text-slate-200 hover:bg-styx-900/60'
          }`}
        >
          <MessageSquare className="w-3.5 h-3.5" />
          <span>Chat & Assistant</span>
          {pendingTickets.length > 0 && (
            <span className="bg-rose-600 text-white rounded-full px-1.5 py-0.2 text-[10px] font-bold animate-pulse">
              {pendingTickets.length}
            </span>
          )}
        </button>

        <button
          type="button"
          onClick={() => onSelectTab('settings')}
          className={`px-3.5 py-1.5 rounded-md flex items-center space-x-2 font-medium transition-all ${
            activeTab === 'settings' || activeTab === 'models' || activeTab === 'memory' || activeTab === 'tools'
              ? 'bg-styx-800 text-emerald-300 shadow-sm border border-emerald-500/30'
              : 'text-slate-400 hover:text-slate-200 hover:bg-styx-900/60'
          }`}
        >
          <Settings className="w-3.5 h-3.5" />
          <span>Assistant Settings</span>
        </button>
      </div>

      {/* Right Action Controls */}
      <div className="flex items-center space-x-2 sm:space-x-2.5 text-[11px] text-slate-400">
        {isNotificationSupported() && (
          <button
            type="button"
            onClick={handleToggleNotifications}
            title={
              notifPerm === 'granted'
                ? 'Alerts active (click to test chime)'
                : 'Click to enable alerts when agent requires input'
            }
            className={`p-1.5 rounded border text-[11px] transition-colors flex items-center justify-center ${
              notifPerm === 'granted'
                ? 'bg-emerald-950/60 border-emerald-500/50 text-emerald-300'
                : 'bg-zinc-800/40 border-zinc-700/50 text-zinc-400 hover:text-amber-300 hover:border-amber-500/40'
            }`}
          >
            <Bell className="w-3.5 h-3.5" />
          </button>
        )}

        <button
          type="button"
          onClick={onToggleTelemetryStrip}
          title="Toggle Device Stats"
          className={`hidden lg:flex items-center gap-1.5 px-2 py-1 rounded border text-[11px] transition-colors ${
            showTelemetryStrip
              ? 'bg-emerald-950/60 border-emerald-500/50 text-emerald-300'
              : 'bg-zinc-800/40 border-zinc-700/50 text-zinc-400 hover:text-zinc-200'
          }`}
        >
          <Activity className="w-3 h-3 text-emerald-400" />
          <span>Stats</span>
        </button>

        <div
          onClick={() => onSelectTab('settings')}
          className="flex items-center gap-1.5 px-2.5 py-1 rounded bg-zinc-800/70 border border-zinc-700/60 text-zinc-300 cursor-pointer hover:border-emerald-500/40 transition-colors"
          title="Open profile in settings"
        >
          <User className="w-3 h-3 text-emerald-400" />
          <span className="font-semibold text-emerald-400 truncate max-w-[90px] sm:max-w-none">
            {authStatus?.operator_name || 'My Assistant'}
          </span>
        </div>

        <button
          type="button"
          onClick={onLock}
          title="Lock Styx device"
          className="flex items-center gap-1 px-2.5 py-1 rounded bg-zinc-800/60 hover:bg-amber-950/40 border border-zinc-700/50 hover:border-amber-500/40 text-zinc-400 hover:text-amber-300 transition-colors"
        >
          <Lock className="w-3 h-3" />
          <span className="hidden sm:inline">LOCK</span>
        </button>
      </div>
    </nav>
  );
};
