import React from 'react';
import { Sparkles, X } from 'lucide-react';

export interface OnboardingBannerProps {
  onStartOnboarding: () => void;
  onDismissOnboarding: () => void;
}

export const OnboardingBanner: React.FC<OnboardingBannerProps> = ({
  onStartOnboarding,
  onDismissOnboarding,
}) => {
  return (
    <div className="flex-shrink-0 bg-emerald-950/70 border-b border-emerald-500/40 px-4 py-2 flex items-center justify-between text-xs font-sans text-emerald-200 z-40">
      <div className="flex items-center gap-2">
        <Sparkles className="w-4 h-4 text-emerald-400 animate-pulse" />
        <span>
          <strong className="text-emerald-300">Welcome to Styx!</strong> Take 30 seconds to set up your preferences so your assistant adapts to you.
        </span>
      </div>
      <div className="flex items-center space-x-2">
        <button
          type="button"
          onClick={onStartOnboarding}
          className="bg-emerald-500 hover:bg-emerald-400 text-zinc-950 px-3 py-1 rounded-md font-semibold text-xs transition-colors shadow-sm"
        >
          Personal Setup
        </button>
        <button
          type="button"
          onClick={onDismissOnboarding}
          className="p-1 rounded hover:bg-emerald-900/60 text-emerald-400 hover:text-emerald-200 transition-colors"
          title="Dismiss banner (mark setup complete)"
          aria-label="Dismiss onboarding banner"
        >
          <X className="w-4 h-4" />
        </button>
      </div>
    </div>
  );
};
