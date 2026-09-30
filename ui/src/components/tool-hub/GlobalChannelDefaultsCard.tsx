import React, { useState, useEffect } from 'react';
import { Users, User, AtSign, SlidersHorizontal } from 'lucide-react';
import { ChannelDefaultsRecord, ChannelTriggerPolicy } from '../../types';

export interface GlobalChannelDefaultsCardProps {
  defaults: ChannelDefaultsRecord;
  saving: boolean;
  onUpdateDefaults: (defaults: Partial<ChannelDefaultsRecord>) => Promise<void>;
}

export const GlobalChannelDefaultsCard: React.FC<GlobalChannelDefaultsCardProps> = ({
  defaults,
  saving,
  onUpdateDefaults,
}) => {
  const [keywordsInput, setKeywordsInput] = useState<string>(defaults?.mention_keywords || '');
  const [isKeywordsDirty, setIsKeywordsDirty] = useState<boolean>(false);

  useEffect(() => {
    if (!isKeywordsDirty && defaults) {
      setKeywordsInput(defaults.mention_keywords);
    }
  }, [defaults, isKeywordsDirty]);

  const handleSaveKeywords = async () => {
    await onUpdateDefaults({ mention_keywords: keywordsInput.trim() });
    setIsKeywordsDirty(false);
  };

  return (
    <div className="p-2.5 rounded bg-styx-950 border border-styx-800 space-y-2">
      <div className="flex items-center justify-between text-[11px] text-slate-300">
        <div className="flex items-center space-x-1.5 font-bold">
          <SlidersHorizontal className="w-3.5 h-3.5 text-amber-400" />
          <span>GLOBAL TRIGGER DEFAULTS</span>
        </div>
        <span className="text-[10px] text-slate-500">
          Applied to newly discovered channels unless customized
        </span>
      </div>

      <div className="grid grid-cols-1 md:grid-cols-3 gap-2.5 text-[11px]">
        {/* Group Chats Default */}
        <div className="p-2 rounded bg-styx-900 border border-styx-800/80 space-y-1">
          <div className="text-slate-400 text-[10px] flex items-center space-x-1">
            <Users className="w-3 h-3 text-cyan-400" />
            <span>Group Chats Default:</span>
          </div>
          <div className="flex items-center space-x-1">
            {(['mentions', 'all', 'muted'] as ChannelTriggerPolicy[]).map(p => (
              <button
                key={p}
                type="button"
                onClick={() => onUpdateDefaults({ default_group_policy: p })}
                className={`flex-1 py-1 px-1 rounded text-[10px] font-bold transition-all text-center ${
                  defaults.default_group_policy === p
                    ? p === 'mentions'
                      ? 'bg-amber-950 text-amber-300 border border-amber-700'
                      : p === 'all'
                      ? 'bg-emerald-950 text-emerald-300 border border-emerald-700'
                      : 'bg-rose-950 text-rose-300 border border-rose-700'
                    : 'text-slate-500 hover:text-slate-300 bg-styx-950 border border-transparent'
                }`}
              >
                {p === 'mentions' ? 'Mentions Only' : p === 'all' ? 'Always' : 'Muted'}
              </button>
            ))}
          </div>
        </div>

        {/* Direct Chats Default */}
        <div className="p-2 rounded bg-styx-900 border border-styx-800/80 space-y-1">
          <div className="text-slate-400 text-[10px] flex items-center space-x-1">
            <User className="w-3 h-3 text-emerald-400" />
            <span>Direct Chats Default:</span>
          </div>
          <div className="flex items-center space-x-1">
            {(['all', 'mentions', 'muted'] as ChannelTriggerPolicy[]).map(p => (
              <button
                key={p}
                type="button"
                onClick={() => onUpdateDefaults({ default_direct_policy: p })}
                className={`flex-1 py-1 px-1 rounded text-[10px] font-bold transition-all text-center ${
                  defaults.default_direct_policy === p
                    ? p === 'all'
                      ? 'bg-emerald-950 text-emerald-300 border border-emerald-700'
                      : p === 'mentions'
                      ? 'bg-amber-950 text-amber-300 border border-amber-700'
                      : 'bg-rose-950 text-rose-300 border border-rose-700'
                    : 'text-slate-500 hover:text-slate-300 bg-styx-950 border border-transparent'
                }`}
              >
                {p === 'all' ? 'Always' : p === 'mentions' ? 'Mentions Only' : 'Muted'}
              </button>
            ))}
          </div>
        </div>

        {/* Mention Keywords */}
        <div className="p-2 rounded bg-styx-900 border border-styx-800/80 space-y-1">
          <div className="text-slate-400 text-[10px] flex items-center justify-between">
            <div className="flex items-center space-x-1">
              <AtSign className="w-3 h-3 text-amber-400" />
              <span>Mention Keywords:</span>
            </div>
            {isKeywordsDirty && (
              <span className="text-[9px] text-amber-400 font-bold">Unsaved</span>
            )}
          </div>
          <div className="flex items-center space-x-1">
            <input
              type="text"
              value={keywordsInput}
              onChange={e => {
                setKeywordsInput(e.target.value);
                setIsKeywordsDirty(true);
              }}
              onKeyDown={e => {
                if (e.key === 'Enter') handleSaveKeywords();
              }}
              placeholder="styx, assistant, ai, bot"
              className="flex-1 bg-styx-950 border border-styx-800 rounded px-2 py-0.5 text-[10px] text-slate-200 focus:outline-none focus:border-amber-600"
            />
            <button
              type="button"
              onClick={handleSaveKeywords}
              disabled={saving || !isKeywordsDirty}
              className={`px-2 py-0.5 rounded text-[10px] font-bold transition-colors ${
                isKeywordsDirty
                  ? 'bg-amber-700 hover:bg-amber-600 text-slate-900'
                  : 'bg-styx-800 text-slate-500 cursor-not-allowed'
              }`}
            >
              Save
            </button>
          </div>
        </div>
      </div>
    </div>
  );
};
