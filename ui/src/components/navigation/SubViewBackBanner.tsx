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
    <div className="flex-shrink-0 bg-styx-900/95 border-b border-styx-800 px-3 sm:px-4 py-1.5 flex items-center justify-between text-xs font-mono select-none">
      <button
        type="button"
        onClick={onBackToSettings}
        className="flex items-center gap-1.5 px-2 sm:px-2.5 py-1 rounded-md bg-styx-800 hover:bg-styx-750 text-emerald-400 hover:text-emerald-300 font-semibold transition-colors border border-styx-700/60 active:scale-95"
      >
        <ArrowLeft className="w-3.5 h-3.5" />
        <span className="sm:hidden">Settings</span>
        <span className="hidden sm:inline">Back to Settings</span>
      </button>
      <div className="flex items-center space-x-1.5 sm:space-x-2 text-slate-400 text-[11px] min-w-0">
        <span className="hidden sm:inline">Settings</span>
        <span className="hidden sm:inline text-slate-600">/</span>
        <span className="text-slate-200 font-semibold uppercase truncate">
          <span className="sm:hidden">
            {activeTab === 'models'
              ? 'Models'
              : activeTab === 'memory'
              ? 'Memory Hub'
              : 'Tools'}
          </span>
          <span className="hidden sm:inline">
            {activeTab === 'models'
              ? 'AI Voice & Intelligence'
              : activeTab === 'memory'
              ? 'Memory & What I Know About You'
              : 'Connected Apps & Assistants'}
          </span>
        </span>
      </div>
    </div>
  );
};
