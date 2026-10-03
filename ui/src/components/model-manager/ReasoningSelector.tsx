import React, { useState, useEffect } from 'react';
import { Brain, Check, Sparkles } from 'lucide-react';

export type ReasoningLevel = 'high' | 'medium' | 'low' | 'off';

interface ReasoningSelectorProps {
  compact?: boolean;
}

const REASONING_OPTIONS: {
  level: ReasoningLevel;
  label: string;
  badge?: string;
  desc: string;
}[] = [
  {
    level: 'high',
    label: 'High',
    badge: 'Default',
    desc: 'Deep multi-step <think> planning & memory grounding',
  },
  {
    level: 'medium',
    label: 'Medium',
    desc: 'Balanced step-by-step reasoning & verification',
  },
  {
    level: 'low',
    label: 'Low',
    desc: 'Brief 1-2 sentence check before replying',
  },
  {
    level: 'off',
    label: 'Off',
    desc: 'Direct answer without internal thinking tags',
  },
];

export const ReasoningSelector: React.FC<ReasoningSelectorProps> = ({ compact = false }) => {
  const [level, setLevel] = useState<ReasoningLevel>('high');
  const [isUpdating, setIsUpdating] = useState(false);
  const [saveSuccess, setSaveSuccess] = useState(false);

  useEffect(() => {
    fetch('/api/models/reasoning')
      .then(res => (res.ok ? res.json() : null))
      .then(data => {
        if (data && data.reasoning_effort) {
          setLevel(data.reasoning_effort as ReasoningLevel);
        }
      })
      .catch(() => {});
  }, []);

  const handleSelect = async (newLevel: ReasoningLevel) => {
    if (newLevel === level || isUpdating) return;
    setLevel(newLevel);
    setIsUpdating(true);
    setSaveSuccess(false);

    try {
      const res = await fetch('/api/models/reasoning', {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({ reasoning_effort: newLevel }),
      });
      if (res.ok) {
        setSaveSuccess(true);
        setTimeout(() => setSaveSuccess(false), 2000);
      }
    } catch {
      // rollback or keep optimistic
    } finally {
      setIsUpdating(false);
    }
  };

  return (
    <div className={`rounded-xl border border-syndae-800 bg-syndae-950/80 p-4 space-y-3 ${compact ? 'text-xs' : ''}`}>
      <div className="flex items-center justify-between">
        <div className="flex items-center space-x-2">
          <div className="w-7 h-7 rounded-lg bg-purple-500/20 border border-purple-500/40 flex items-center justify-center text-purple-300">
            <Brain className="w-4 h-4" />
          </div>
          <div>
            <div className="flex items-center space-x-2">
              <span className="font-bold text-slate-200 text-xs sm:text-sm font-mono flex items-center gap-1.5">
                <span>Reasoning Effort</span>
                <Sparkles className="w-3 h-3 text-purple-400" />
              </span>
              <span className="px-1.5 py-0.5 rounded text-[10px] font-bold font-mono bg-purple-950 text-purple-300 border border-purple-800">
                Default: High
              </span>
              {saveSuccess && (
                <span className="px-1.5 py-0.5 rounded text-[10px] font-bold font-mono bg-emerald-950 text-emerald-300 border border-emerald-800 flex items-center gap-1">
                  <Check className="w-2.5 h-2.5" />
                  <span>Saved</span>
                </span>
              )}
            </div>
            <p className="text-[11px] text-slate-400 font-sans">
              Adjust depth of internal step-by-step thinking in &lt;think&gt; tags
            </p>
          </div>
        </div>
      </div>

      <div className="grid grid-cols-2 sm:grid-cols-4 gap-2 font-mono">
        {REASONING_OPTIONS.map(opt => {
          const isActive = level === opt.level;
          return (
            <button
              key={opt.level}
              type="button"
              disabled={isUpdating}
              onClick={() => handleSelect(opt.level)}
              className={`p-2.5 rounded-lg border text-left transition-all ${
                isActive
                  ? 'bg-purple-950/60 border-purple-500 text-purple-100 shadow-md ring-1 ring-purple-500/50'
                  : 'bg-syndae-900/60 border-syndae-800 text-slate-300 hover:border-syndae-700 hover:bg-syndae-900'
              }`}
            >
              <div className="flex items-center justify-between">
                <span className="font-bold text-xs">{opt.label}</span>
                {opt.badge && (
                  <span className="px-1 rounded text-[9px] font-semibold bg-purple-500/20 text-purple-300 border border-purple-500/40">
                    {opt.badge}
                  </span>
                )}
                {isActive && !opt.badge && (
                  <Check className="w-3 h-3 text-purple-400" />
                )}
              </div>
              <p className="text-[10px] text-slate-400 font-sans mt-1 line-clamp-2 leading-tight">
                {opt.desc}
              </p>
            </button>
          );
        })}
      </div>
    </div>
  );
};
