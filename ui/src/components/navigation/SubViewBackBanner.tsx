import React from 'react';
import { ArrowLeft } from 'lucide-react';

export interface SubViewBackBannerProps {
  activeTab: 'models' | 'memory' | 'tools';
  onBackToSettings: () => void;
}

export const SubViewBackBanner: React.FC<SubViewBackBannerProps> = ({
  activeTab,
  onBackToSettings,
}) => {
  return (
    <div className="flex-shrink-0 bg-styx-900/90 border-b border-styx-800 px-3 sm:px-4 py-1.5 flex items-center justify-between text-xs font-mono">
      <button
        type="button"
        onClick={onBackToSettings}
        className="flex items-center gap-1.5 px-2.5 py-1 rounded-md bg-styx-800 hover:bg-styx-750 text-emerald-400 hover:text-emerald-300 font-semibold transition-colors border border-styx-700/60"
      >
        <ArrowLeft className="w-3.5 h-3.5" />
        <span>Back to Settings</span>
      </button>
      <div className="flex items-center space-x-2 text-slate-400 text-[11px]">
        <span className="hidden sm:inline">Settings</span>
        <span className="hidden sm:inline text-slate-600">/</span>
        <span className="text-slate-200 font-semibold uppercase">
          {activeTab === 'models'
            ? 'AI Voice & Intelligence'
            : activeTab === 'memory'
            ? 'Memory & What I Know About You'
            : 'Connected Apps & Assistants'}
        </span>
      </div>
    </div>
  );
};
