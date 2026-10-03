import { useState, useEffect, useCallback } from 'react';
import { ChannelPolicyRecord, ChannelDefaultsRecord, ChannelTriggerPolicy } from '../../types';

export const useChannelPolicies = () => {
  const [channels, setChannels] = useState<ChannelPolicyRecord[]>([]);
  const [defaults, setDefaults] = useState<ChannelDefaultsRecord>({
    default_group_policy: 'mentions',
    default_direct_policy: 'all',
    mention_keywords: 'syndae,assistant,ai,bot',
  });
  const [loading, setLoading] = useState<boolean>(false);
  const [saving, setSaving] = useState<boolean>(false);
  const [searchQuery, setSearchQuery] = useState<string>('');
  const [typeFilter, setTypeFilter] = useState<'all' | 'groups' | 'direct'>('all');
  const [statusMessage, setStatusMessage] = useState<{
    type: 'success' | 'error' | 'info';
    text: string;
  } | null>(null);

  const fetchChannels = useCallback(async () => {
    try {
      const res = await fetch('/api/tools/channels');
      if (res.ok) {
        const data = await res.json();
        if (data.success && Array.isArray(data.channels)) {
          setChannels(data.channels);
        }
      }
    } catch (e) {
      console.error('Failed to fetch channels:', e);
    }
  }, []);

  const fetchDefaults = useCallback(async () => {
    try {
      const res = await fetch('/api/tools/channels/defaults');
      if (res.ok) {
        const data = await res.json();
        if (data.success && data.defaults) {
          setDefaults(data.defaults);
        }
      }
    } catch (e) {
      console.error('Failed to fetch channel defaults:', e);
    }
  }, []);

  const refreshAll = useCallback(async () => {
    setLoading(true);
    await Promise.all([fetchChannels(), fetchDefaults()]);
    setLoading(false);
  }, [fetchChannels, fetchDefaults]);

  useEffect(() => {
    refreshAll();
  }, [refreshAll]);

  const updatePolicy = async (
    channelId: string,
    policy: ChannelTriggerPolicy,
    options?: {
      channelName?: string;
      isGroup?: boolean;
      mentionKeywords?: string;
    }
  ) => {
    // Optimistic update
    setChannels(prev =>
      prev.map(c =>
        c.channel_id === channelId
          ? {
              ...c,
              policy,
              channel_name: options?.channelName ?? c.channel_name,
              mention_keywords: options?.mentionKeywords ?? c.mention_keywords,
              updated_at: new Date().toISOString(),
            }
          : c
      )
    );

    try {
      const existing = channels.find(c => c.channel_id === channelId);
      const res = await fetch('/api/tools/channels/policy', {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({
          channel_id: channelId,
          protocol: existing?.protocol || 'whatsapp',
          channel_name: options?.channelName ?? existing?.channel_name,
          is_group: options?.isGroup ?? existing?.is_group ?? false,
          policy,
          mention_keywords: options?.mentionKeywords ?? existing?.mention_keywords,
        }),
      });

      if (!res.ok) {
        await fetchChannels();
        setStatusMessage({ type: 'error', text: 'Failed to update channel trigger policy' });
      } else {
        setStatusMessage({
          type: 'success',
          text: `Updated policy for ${existing?.channel_name || channelId} to '${policy}'`,
        });
        setTimeout(() => setStatusMessage(null), 3000);
      }
    } catch (e: any) {
      await fetchChannels();
      setStatusMessage({ type: 'error', text: 'Network error updating policy: ' + (e.message || e) });
    }
  };

  const updateDefaults = async (newDefaults: Partial<ChannelDefaultsRecord>) => {
    const merged: ChannelDefaultsRecord = { ...defaults, ...newDefaults };
    setDefaults(merged);
    setSaving(true);
    try {
      const res = await fetch('/api/tools/channels/defaults', {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify(merged),
      });
      if (res.ok) {
        setStatusMessage({ type: 'success', text: 'Global inbound trigger defaults saved' });
        setTimeout(() => setStatusMessage(null), 3000);
      } else {
        await fetchDefaults();
        setStatusMessage({ type: 'error', text: 'Failed to save defaults' });
      }
    } catch (e: any) {
      await fetchDefaults();
      setStatusMessage({ type: 'error', text: 'Error saving defaults: ' + (e.message || e) });
    } finally {
      setSaving(false);
    }
  };

  const quickSetAllGroups = async (policy: ChannelTriggerPolicy) => {
    const groupChannels = channels.filter(c => c.is_group);
    if (groupChannels.length === 0) return;

    setLoading(true);
    try {
      await Promise.all(
        groupChannels.map(c =>
          fetch('/api/tools/channels/policy', {
            method: 'POST',
            headers: { 'Content-Type': 'application/json' },
            body: JSON.stringify({
              channel_id: c.channel_id,
              protocol: c.protocol,
              channel_name: c.channel_name,
              is_group: true,
              policy,
            }),
          })
        )
      );
      await fetchChannels();
      setStatusMessage({
        type: 'success',
        text: `Set all ${groupChannels.length} group channels to '${policy}'`,
      });
      setTimeout(() => setStatusMessage(null), 3000);
    } catch (e: any) {
      setStatusMessage({ type: 'error', text: 'Failed to bulk update groups: ' + (e.message || e) });
    } finally {
      setLoading(false);
    }
  };

  const filteredChannels = channels.filter(c => {
    if (typeFilter === 'groups' && !c.is_group) return false;
    if (typeFilter === 'direct' && c.is_group) return false;
    if (searchQuery.trim()) {
      const q = searchQuery.toLowerCase();
      const nameMatch = c.channel_name?.toLowerCase().includes(q);
      const idMatch = c.channel_id.toLowerCase().includes(q);
      const protoMatch = c.protocol.toLowerCase().includes(q);
      return nameMatch || idMatch || protoMatch;
    }
    return true;
  });

  return {
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
    setStatusMessage,
    refreshAll,
    updatePolicy,
    updateDefaults,
    quickSetAllGroups,
  };
};
