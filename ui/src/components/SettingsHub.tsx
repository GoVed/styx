import React, { useState } from 'react';
import {
  Cpu,
  BookOpen,
  Layers,
  Activity,
  Shield,
  Sparkles,
  Lock,
  KeyRound,
  ChevronRight,
  ArrowLeft,
  Search,
} from 'lucide-react';
import { DeviceDiagnosticsView } from './settings/DeviceDiagnosticsView';
import { ChangePasswordView } from './settings/ChangePasswordView';
import {
  SystemTelemetry,
  AuthStatus,
  ModelConfigRecord,
  ContainerSummaryInfo,
  McpTool,
} from '../types';

interface SettingsHubProps {
  telemetry: SystemTelemetry | null;
  authStatus: AuthStatus | null;
  modelConfigs?: ModelConfigRecord[];
  containers?: ContainerSummaryInfo[];
  mcpTools?: McpTool[];
  pendingTicketsCount?: number;
  onNavigate?: (view: 'chat' | 'models' | 'memory' | 'tools' | 'diagnostics') => void;
  onNavigateSection?: (section: string) => void;
  onLock?: () => void;
  onStartOnboarding?: () => void;
  onOpenAudit?: () => void;
  onBackToChat?: () => void;
  onPasswordChanged?: (newToken: string) => void;
}

export const SettingsHub: React.FC<SettingsHubProps> = ({
  telemetry,
  authStatus,
  modelConfigs = [],
  containers = [],
  mcpTools = [],
  pendingTicketsCount = 0,
  onNavigate,
  onNavigateSection,
  onLock,
  onStartOnboarding,
  onOpenAudit,
  onBackToChat,
  onPasswordChanged,
}) => {
  const [searchQuery, setSearchQuery] = useState('');
  const [subView, setSubView] = useState<'main' | 'diagnostics' | 'password'>('main');

  const handleNav = (dest: 'chat' | 'models' | 'memory' | 'tools' | 'diagnostics') => {
    if (dest === 'diagnostics') {
      setSubView('diagnostics');
      return;
    }
    if (onNavigate) onNavigate(dest);
    else if (onNavigateSection) onNavigateSection(dest);
  };

  const activeModel = telemetry?.active_model ?? 'Local vLLM';
  const runningContainers = containers.filter(c => c.state === 'running').length;
  const operatorName = authStatus?.operator_name || 'Operator';
  const initial = operatorName.charAt(0).toUpperCase();

  const cpu = telemetry?.host_cpu_pct ?? 0;
  const memMb = telemetry?.memory_used_mb ?? 0;
  const memTotal = telemetry?.memory_total_mb ?? 1;
  const memPct = telemetry?.memory_pct ?? 0;
  const diskPct = telemetry?.disk_pct ?? 0;
  const tokSec = telemetry?.tokens_per_second ?? 0;
  const gpu = telemetry?.gpu;

  const menuItems = [
    {
      id: 'models',
      title: 'AI Voice & Intelligence',
      subtitle: `Active: ${activeModel.split(':')[0]?.replace('QuantTrio/', '') || 'Smart Assistant'} • AI engine is running smoothly`,
      icon: Cpu,
      color: 'bg-emerald-500/20 text-emerald-400 border-emerald-500/30',
      badge: 'Engine',
      action: () => handleNav('models'),
    },
    {
      id: 'memory',
      title: 'Memory & What I Know About You',
      subtitle: 'Your personal preferences, learned habits, and notes',
      icon: BookOpen,
      color: 'bg-purple-500/20 text-purple-400 border-purple-500/30',
      badge: 'Memory',
      action: () => handleNav('memory'),
    },
    {
      id: 'tools',
      title: 'Connected Apps & Assistants',
      subtitle: `${mcpTools.length} connected daemon${mcpTools.length === 1 ? '' : 's'} registered in MCP tool catalog`,
      icon: Layers,
      color: 'bg-cyan-500/20 text-cyan-400 border-cyan-500/30',
      badge: 'Apps',
      action: () => handleNav('tools'),
    },
    {
      id: 'diagnostics',
      title: 'Device Health & Performance',
      subtitle: `Processor ${cpu.toFixed(0)}% • Memory ${memPct.toFixed(0)}% • Speed: ${tokSec.toFixed(0)} words/sec`,
      icon: Activity,
      color: 'bg-amber-500/20 text-amber-400 border-amber-500/30',
      badge: 'Vitals',
      action: () => setSubView('diagnostics'),
    },
    {
      id: 'security',
      title: 'Privacy & Safety Approvals',
      subtitle: `Safe Mode Active: Styx always asks your permission before sending anything`,
      icon: Shield,
      color: 'bg-blue-500/20 text-blue-400 border-blue-500/30',
      badge: 'Safe',
      action: () => (onOpenAudit ? onOpenAudit() : handleNav('chat')),
    },
    {
      id: 'password',
      title: 'Master Access Key & Password',
      subtitle: 'Change your device unlock password and manage enclave access credentials',
      icon: KeyRound,
      color: 'bg-amber-500/20 text-amber-400 border-amber-500/30',
      badge: 'Security',
      action: () => setSubView('password'),
    },
    {
      id: 'onboarding',
      title: 'Personal Setup & Preferences',
      subtitle: authStatus?.onboarded
        ? 'Update how Styx talks, your daily routine, and what to remember'
        : 'Get started: Tell Styx how you would like to be helped',
      icon: Sparkles,
      color: 'bg-rose-500/20 text-rose-400 border-rose-500/30',
      badge: authStatus?.onboarded ? 'Profile' : 'Setup',
      action: () => (onStartOnboarding ? onStartOnboarding() : handleNav('chat')),
    },
  ];

  const filteredItems = searchQuery.trim()
    ? menuItems.filter(
        i =>
          i.title.toLowerCase().includes(searchQuery.toLowerCase()) ||
          i.subtitle.toLowerCase().includes(searchQuery.toLowerCase())
      )
    : menuItems;

  return (
    <div className="h-full w-full overflow-y-auto bg-styx-950 text-slate-100 flex flex-col font-sans select-none">
      {/* Android-style Header Bar */}
      <div className="sticky top-0 z-30 bg-styx-900/95 backdrop-blur-md border-b border-styx-800 px-4 py-3 flex items-center justify-between">
        <div className="flex items-center space-x-3">
          <button
            onClick={() => {
              if (subView !== 'main') {
                setSubView('main');
              } else if (onBackToChat) {
                onBackToChat();
              } else {
                handleNav('chat');
              }
            }}
            className="p-1.5 rounded-full hover:bg-styx-800 text-slate-300 hover:text-white transition-colors flex items-center justify-center"
            title="Back"
          >
            <ArrowLeft className="w-5 h-5" />
          </button>
          <div>
            <h1 className="text-base sm:text-lg font-bold tracking-tight text-white flex items-center gap-2">
              <span>
                {subView === 'diagnostics'
                  ? 'Device Health & Speed'
                  : subView === 'password'
                  ? 'Master Password & Security'
                  : 'Assistant Settings'}
              </span>
            </h1>
            <p className="text-[11px] text-slate-400 font-mono">
              {subView === 'diagnostics'
                ? 'Live device performance and responsiveness'
                : subView === 'password'
                ? 'Update your device enclave unlock credentials'
                : 'Manage your preferences, memory, and apps'}
            </p>
          </div>
        </div>

        <div className="flex items-center space-x-2">
          <button
            onClick={() => onLock?.()}
            className="px-2.5 py-1.5 rounded-lg bg-styx-850 hover:bg-amber-950/40 text-slate-300 hover:text-amber-300 border border-styx-700/60 hover:border-amber-500/40 text-xs font-medium flex items-center space-x-1.5 transition-all"
            title="Lock Styx device"
          >
            <Lock className="w-3.5 h-3.5" />
            <span className="hidden sm:inline">Lock Device</span>
          </button>
        </div>
      </div>

      <div className="flex-1 max-w-3xl w-full mx-auto p-4 sm:p-6 space-y-5">
        {subView === 'main' ? (
          <>
            {/* Operator Profile Card (Android style) */}
            <div className="rounded-2xl bg-gradient-to-br from-styx-900 to-styx-850 border border-styx-800 p-4 sm:p-5 shadow-lg flex items-center justify-between gap-4">
              <div className="flex items-center space-x-3.5 min-w-0">
                <div className="w-12 h-12 rounded-full bg-emerald-500/20 border-2 border-emerald-500/50 flex items-center justify-center text-emerald-300 font-bold text-lg flex-shrink-0 shadow-inner">
                  {initial}
                </div>
                <div className="min-w-0">
                  <div className="flex items-center gap-2">
                    <h2 className="text-base font-bold text-white truncate">{operatorName}</h2>
                    <span className="px-2 py-0.5 rounded-full text-[10px] font-semibold bg-emerald-950/80 text-emerald-400 border border-emerald-800 flex-shrink-0">
                      You
                    </span>
                  </div>
                  <p className="text-xs text-slate-400 truncate mt-0.5">
                    Private Personal Assistant • 100% on your device
                  </p>
                </div>
              </div>

              <div className="flex items-center space-x-2 flex-shrink-0">
                <button
                  onClick={() => setSubView('password')}
                  className="px-2.5 py-1.5 rounded-xl bg-styx-800 hover:bg-amber-950/40 text-slate-300 hover:text-amber-300 text-xs font-medium border border-styx-700/60 hover:border-amber-500/40 transition-all active:scale-95 shadow-sm flex items-center space-x-1.5"
                  title="Change master password"
                >
                  <KeyRound className="w-3.5 h-3.5 text-amber-400" />
                  <span className="hidden sm:inline">Change Password</span>
                </button>
                <button
                  onClick={() => (onStartOnboarding ? onStartOnboarding() : handleNav('chat'))}
                  className="px-3 py-1.5 rounded-xl bg-styx-800 hover:bg-styx-700 text-slate-200 text-xs font-medium border border-styx-700 transition-all active:scale-95 shadow-sm"
                >
                  Edit Profile
                </button>
              </div>
            </div>

            {/* Search Box */}
            <div className="relative">
              <Search className="w-4 h-4 text-slate-500 absolute left-3.5 top-1/2 -translate-y-1/2" />
              <input
                type="text"
                value={searchQuery}
                onChange={e => setSearchQuery(e.target.value)}
                placeholder="Search settings..."
                className="w-full bg-styx-900 border border-styx-800 focus:border-emerald-500 rounded-xl pl-10 pr-4 py-2.5 text-xs text-slate-200 placeholder-slate-500 focus:outline-none focus:ring-1 focus:ring-emerald-500/30 transition-all font-mono"
              />
            </div>

            {/* Menu Items Group (Android Settings style) */}
            <div className="space-y-2.5">
              <h3 className="text-[11px] font-mono font-semibold uppercase tracking-wider text-slate-400 px-1">
                SYSTEM PREFERENCES & MANAGEMENT
              </h3>

              <div className="rounded-2xl bg-styx-900/90 border border-styx-800 overflow-hidden divide-y divide-styx-800/80 shadow-md">
                {filteredItems.map(item => {
                  const Icon = item.icon;
                  return (
                    <div
                      key={item.id}
                      onClick={item.action}
                      className="group p-3.5 sm:p-4 flex items-center justify-between hover:bg-styx-850/80 cursor-pointer transition-colors active:bg-styx-800"
                    >
                      <div className="flex items-center space-x-3.5 min-w-0 pr-2">
                        <div
                          className={`w-10 h-10 rounded-xl flex items-center justify-center flex-shrink-0 border ${item.color}`}
                        >
                          <Icon className="w-5 h-5" />
                        </div>
                        <div className="min-w-0">
                          <div className="flex items-center gap-2">
                            <span className="text-xs sm:text-sm font-semibold text-slate-100 group-hover:text-emerald-300 transition-colors truncate">
                              {item.title}
                            </span>
                            <span className="hidden sm:inline-block px-1.5 py-0.5 rounded text-[9px] font-mono bg-styx-800 text-slate-400 border border-styx-700">
                              {item.badge}
                            </span>
                          </div>
                          <p className="text-[11px] text-slate-400 truncate mt-0.5">
                            {item.subtitle}
                          </p>
                        </div>
                      </div>

                      <div className="flex items-center space-x-1.5 text-slate-500 group-hover:text-slate-300 flex-shrink-0 transition-colors">
                        <ChevronRight className="w-4 h-4" />
                      </div>
                    </div>
                  );
                })}
              </div>
            </div>

            {/* Quick Actions Footer */}
            <div className="pt-2 flex flex-col sm:flex-row gap-2.5">
              <button
                onClick={() => {
                  if (onBackToChat) onBackToChat();
                  else handleNav('chat');
                }}
                className="flex-1 py-2.5 px-4 rounded-xl bg-emerald-600 hover:bg-emerald-500 text-white font-medium text-xs flex items-center justify-center space-x-2 shadow-sm transition-all active:scale-95"
              >
                <span>Return to Assistant Chat</span>
              </button>
              <button
                onClick={() => {
                  if (onOpenAudit) onOpenAudit();
                  else handleNav('chat');
                }}
                className="py-2.5 px-4 rounded-xl bg-styx-900 hover:bg-styx-850 border border-styx-800 text-slate-300 text-xs font-medium flex items-center justify-center space-x-2 transition-all"
              >
                <Shield className="w-3.5 h-3.5 text-blue-400" />
                <span>Security Audit Log</span>
              </button>
            </div>
          </>
        ) : subView === 'diagnostics' ? (
          <DeviceDiagnosticsView
            telemetry={telemetry}
            onBack={() => setSubView('main')}
            onOpenAudit={() => (onOpenAudit ? onOpenAudit() : handleNav('chat'))}
          />
        ) : (
          <ChangePasswordView
            onBack={() => setSubView('main')}
            onPasswordChanged={onPasswordChanged}
          />
        )}
      </div>
    </div>
  );
};
