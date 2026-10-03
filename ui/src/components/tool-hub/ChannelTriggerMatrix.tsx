import React from 'react';
import {
  Radio,
  Users,
  User,
  VolumeX,
  AtSign,
  Search,
  RefreshCw,
  Check,
  Sparkles,
} from 'lucide-react';
import { useChannelPolicies } from './useChannelPolicies';
import { GlobalChannelDefaultsCard } from './GlobalChannelDefaultsCard';

export const ChannelTriggerMatrix: React.FC = () => {
  const {
    channels,
    filteredChannels,
    defaults,
    loading,
    saving,
    searchQuery,
    setSearchQuery,
    typeFilter,
    setTypeFilter,
    statusMessage,
    refreshAll,
    updatePolicy,
    updateDefaults,
    quickSetAllGroups,
  } = useChannelPolicies();

  const groupsCount = channels.filter(c => c.is_group).length;
  const directCount = channels.filter(c => !c.is_group).length;

  return (
    <div className="bg-syndae-900 border border-syndae-800 rounded-lg p-3 font-mono space-y-3">
      {/* Header */}
      <div className="flex flex-col sm:flex-row sm:items-center justify-between pb-2 border-b border-syndae-800/80 gap-2">
        <div className="flex items-center space-x-2">
          <div className="p-1.5 rounded bg-cyan-950/80 border border-cyan-800 text-cyan-400">
            <Radio className="w-4 h-4" />
          </div>
          <div>
            <div className="flex items-center space-x-2">
              <span className="text-xs font-bold text-slate-100">
                INBOUND TRIGGER RULES & CHANNELS
              </span>
              <span className="px-1.5 py-0.2 rounded bg-syndae-800 text-slate-300 text-[10px]">
                {channels.length} Total
              </span>
            </div>
            <p className="text-[10px] text-slate-400">
              Filter ambient group chatter and prevent auto-triggering on noisy groups while staying responsive to direct messages and mentions.
            </p>
          </div>
        </div>
        <div className="flex items-center space-x-2 self-start sm:self-auto">
          <button
            type="button"
            onClick={refreshAll}
            disabled={loading}
            className="px-2 py-1 rounded bg-syndae-800 hover:bg-syndae-700 text-slate-300 border border-syndae-700 text-[10px] flex items-center space-x-1 transition-colors"
            title="Refresh channels and defaults"
          >
            <RefreshCw className={`w-3 h-3 ${loading ? 'animate-spin text-cyan-400' : ''}`} />
            <span>Refresh</span>
          </button>
        </div>
      </div>

      {/* Status Feedback */}
      {statusMessage && (
        <div
          className={`p-2 rounded text-[11px] flex items-center justify-between ${
            statusMessage.type === 'success'
              ? 'bg-emerald-950/80 border border-emerald-800 text-emerald-300'
              : 'bg-rose-950/80 border border-rose-800 text-rose-300'
          }`}
        >
          <div className="flex items-center space-x-1.5">
            {statusMessage.type === 'success' ? (
              <Check className="w-3.5 h-3.5 text-emerald-400" />
            ) : (
              <VolumeX className="w-3.5 h-3.5 text-rose-400" />
            )}
            <span>{statusMessage.text}</span>
          </div>
        </div>
      )}

      {/* Global Defaults Panel */}
      <GlobalChannelDefaultsCard
        defaults={defaults}
        saving={saving}
        onUpdateDefaults={updateDefaults}
      />

      {/* Toolbar: Search, Filter & Quick Batch Actions */}
      <div className="flex flex-col sm:flex-row items-stretch sm:items-center justify-between gap-2 pt-1">
        <div className="flex items-center space-x-2 flex-1">
          <div className="relative flex-1 max-w-xs">
            <Search className="w-3.5 h-3.5 absolute left-2.5 top-2 text-slate-500" />
            <input
              type="text"
              value={searchQuery}
              onChange={e => setSearchQuery(e.target.value)}
              placeholder="Search group name, JID..."
              className="w-full pl-8 pr-2 py-1 rounded bg-syndae-950 border border-syndae-800 text-[11px] text-slate-200 placeholder-slate-600 focus:outline-none focus:border-cyan-600"
            />
          </div>

          <div className="flex items-center bg-syndae-950 p-0.5 rounded border border-syndae-800">
            <button
              type="button"
              onClick={() => setTypeFilter('all')}
              className={`px-2 py-0.5 rounded text-[10px] ${
                typeFilter === 'all'
                  ? 'bg-syndae-800 text-slate-100 font-bold'
                  : 'text-slate-400 hover:text-slate-200'
              }`}
            >
              All ({channels.length})
            </button>
            <button
              type="button"
              onClick={() => setTypeFilter('groups')}
              className={`px-2 py-0.5 rounded text-[10px] flex items-center space-x-1 ${
                typeFilter === 'groups'
                  ? 'bg-syndae-800 text-slate-100 font-bold'
                  : 'text-slate-400 hover:text-slate-200'
              }`}
            >
              <Users className="w-3 h-3 text-cyan-400" />
              <span>Groups ({groupsCount})</span>
            </button>
            <button
              type="button"
              onClick={() => setTypeFilter('direct')}
              className={`px-2 py-0.5 rounded text-[10px] flex items-center space-x-1 ${
                typeFilter === 'direct'
                  ? 'bg-syndae-800 text-slate-100 font-bold'
                  : 'text-slate-400 hover:text-slate-200'
              }`}
            >
              <User className="w-3 h-3 text-emerald-400" />
              <span>Direct ({directCount})</span>
            </button>
          </div>
        </div>

        {/* Quick Batch Actions for Groups */}
        {groupsCount > 0 && (
          <div className="flex items-center space-x-1.5 self-end sm:self-auto">
            <button
              type="button"
              onClick={() => quickSetAllGroups('muted')}
              disabled={loading}
              className="px-2 py-1 rounded bg-rose-950/60 hover:bg-rose-900 text-rose-300 border border-rose-800/80 text-[10px] flex items-center space-x-1 transition-colors"
              title="Mute all group chats to stop AI from triggering on group chatter"
            >
              <VolumeX className="w-3 h-3" />
              <span>Mute All Groups</span>
            </button>
            <button
              type="button"
              onClick={() => quickSetAllGroups('mentions')}
              disabled={loading}
              className="px-2 py-1 rounded bg-amber-950/60 hover:bg-amber-900 text-amber-300 border border-amber-800/80 text-[10px] flex items-center space-x-1 transition-colors"
              title="Set all group chats to trigger only when mentioned"
            >
              <AtSign className="w-3 h-3" />
              <span>All Groups to Mentions</span>
            </button>
          </div>
        )}
      </div>

      {/* Channels Table */}
      <div className="overflow-x-auto">
        <table className="w-full text-left text-[11px]">
          <thead>
            <tr className="border-b border-syndae-800 text-slate-400">
              <th className="py-2 px-2">CHANNEL / GROUP</th>
              <th className="py-2 px-2">TYPE</th>
              <th className="py-2 px-2">PROTOCOL</th>
              <th className="py-2 px-2">TRIGGER POLICY</th>
            </tr>
          </thead>
          <tbody className="divide-y divide-syndae-800/60">
            {filteredChannels.length === 0 ? (
              <tr>
                <td colSpan={4} className="py-8 text-center text-slate-500">
                  <div className="flex flex-col items-center justify-center space-y-1">
                    <Sparkles className="w-5 h-5 text-slate-600" />
                    <span className="text-xs text-slate-400">
                      {channels.length === 0
                        ? 'No external channels registered yet'
                        : 'No channels match the filter criteria'}
                    </span>
                    <span className="text-[10px] text-slate-500 max-w-sm">
                      {channels.length === 0
                        ? 'Incoming group and direct messages from WhatsApp or connectors will auto-appear here for granular control.'
                        : 'Try adjusting your search query or type filter.'}
                    </span>
                  </div>
                </td>
              </tr>
            ) : (
              filteredChannels.map(c => (
                <tr key={c.channel_id} className="hover:bg-syndae-850 transition-colors">
                  {/* Channel Name & ID */}
                  <td className="py-2 px-2 max-w-xs">
                    <div className="flex items-center space-x-2">
                      <div
                        className={`p-1 rounded ${
                          c.is_group
                            ? 'bg-cyan-950 text-cyan-400 border border-cyan-800'
                            : 'bg-emerald-950 text-emerald-400 border border-emerald-800'
                        }`}
                      >
                        {c.is_group ? <Users className="w-3.5 h-3.5" /> : <User className="w-3.5 h-3.5" />}
                      </div>
                      <div className="min-w-0">
                        <div className="font-bold text-slate-200 truncate">
                          {c.channel_name || (c.is_group ? 'Unnamed Group' : 'Direct Contact')}
                        </div>
                        <div className="text-[10px] text-slate-500 font-mono truncate" title={c.channel_id}>
                          {c.channel_id}
                        </div>
                      </div>
                    </div>
                  </td>

                  {/* Type */}
                  <td className="py-2 px-2 whitespace-nowrap">
                    <span
                      className={`px-1.5 py-0.5 rounded text-[10px] font-bold ${
                        c.is_group
                          ? 'bg-cyan-950 text-cyan-300 border border-cyan-800'
                          : 'bg-emerald-950 text-emerald-300 border border-emerald-800'
                      }`}
                    >
                      {c.is_group ? 'Group Chat' : 'Direct Chat'}
                    </span>
                  </td>

                  {/* Protocol */}
                  <td className="py-2 px-2 whitespace-nowrap">
                    <span className="px-1.5 py-0.5 rounded bg-syndae-950 text-slate-300 border border-syndae-800 text-[10px]">
                      {c.protocol}
                    </span>
                  </td>

                  {/* Trigger Policy Buttons */}
                  <td className="py-2 px-2 whitespace-nowrap">
                    <div className="flex items-center bg-syndae-950 p-0.5 rounded border border-syndae-800 w-fit">
                      <button
                        type="button"
                        onClick={() => updatePolicy(c.channel_id, 'all')}
                        className={`px-2 py-0.5 rounded text-[10px] font-bold transition-all ${
                          c.policy === 'all'
                            ? 'bg-emerald-950 text-emerald-400 border border-emerald-800'
                            : 'text-slate-500 hover:text-slate-300'
                        }`}
                        title="Always trigger AI on every message in this channel"
                      >
                        Always
                      </button>
                      <button
                        type="button"
                        onClick={() => updatePolicy(c.channel_id, 'mentions')}
                        className={`px-2 py-0.5 rounded text-[10px] font-bold transition-all ${
                          c.policy === 'mentions'
                            ? 'bg-amber-950 text-amber-300 border border-amber-800'
                            : 'text-slate-500 hover:text-slate-300'
                        }`}
                        title="Trigger AI only when explicitly mentioned (@Syndae, keywords, or direct reply)"
                      >
                        Mentions Only
                      </button>
                      <button
                        type="button"
                        onClick={() => updatePolicy(c.channel_id, 'muted')}
                        className={`px-2 py-0.5 rounded text-[10px] font-bold transition-all ${
                          c.policy === 'muted'
                            ? 'bg-rose-950 text-rose-400 border border-rose-800'
                            : 'text-slate-500 hover:text-slate-300'
                        }`}
                        title="Muted: Suppress all incoming events from this channel. Zero GPU inference."
                      >
                        Muted
                      </button>
                      <button
                        type="button"
                        onClick={() => updatePolicy(c.channel_id, 'manual')}
                        className={`px-2 py-0.5 rounded text-[10px] font-bold transition-all ${
                          c.policy === 'manual'
                            ? 'bg-slate-800 text-slate-300 border border-slate-700'
                            : 'text-slate-500 hover:text-slate-300'
                        }`}
                        title="Manual: Log message but do not auto-queue AI turn"
                      >
                        Manual
                      </button>
                    </div>
                  </td>
                </tr>
              ))
            )}
          </tbody>
        </table>
      </div>
    </div>
  );
};
