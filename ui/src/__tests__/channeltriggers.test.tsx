import { describe, it, expect, vi, beforeEach } from 'vitest';
import { render, screen, fireEvent, waitFor } from '@testing-library/react';
import React from 'react';
import { ChannelTriggerMatrix } from '../components/tool-hub/ChannelTriggerMatrix';

const mockChannels = [
  {
    channel_id: '120363040000000000@g.us',
    protocol: 'whatsapp',
    channel_name: 'Computer Scientists Group',
    is_group: true,
    policy: 'mentions',
    mention_keywords: null,
    updated_at: '2026-09-30T12:00:00Z',
  },
  {
    channel_id: '261799882539190@lid',
    protocol: 'whatsapp',
    channel_name: 'Direct Contact',
    is_group: false,
    policy: 'all',
    mention_keywords: null,
    updated_at: '2026-09-30T12:00:00Z',
  },
];

const mockDefaults = {
  default_group_policy: 'mentions',
  default_direct_policy: 'all',
  mention_keywords: 'styx,assistant,ai,bot',
};

describe('ChannelTriggerMatrix Component', () => {
  beforeEach(() => {
    vi.clearAllMocks();
    global.fetch = vi.fn((url: any, init?: any) => {
      const urlStr = typeof url === 'string' ? url : url.toString();

      if (urlStr.includes('/api/tools/channels/defaults')) {
        if (init?.method === 'POST') {
          return Promise.resolve({
            ok: true,
            json: async () => ({ success: true, defaults: JSON.parse(init.body) }),
          } as any);
        }
        return Promise.resolve({
          ok: true,
          json: async () => ({ success: true, defaults: mockDefaults }),
        } as any);
      }

      if (urlStr.includes('/api/tools/channels/policy')) {
        return Promise.resolve({
          ok: true,
          json: async () => ({ success: true, message: 'Updated' }),
        } as any);
      }

      if (urlStr.includes('/api/tools/channels')) {
        return Promise.resolve({
          ok: true,
          json: async () => ({ success: true, channels: mockChannels }),
        } as any);
      }

      return Promise.resolve({
        ok: true,
        json: async () => ({ success: true }),
      } as any);
    });
  });

  it('renders header, global trigger defaults, and channels list', async () => {
    render(<ChannelTriggerMatrix />);

    expect(screen.getByText('INBOUND TRIGGER RULES & CHANNELS')).toBeDefined();
    expect(screen.getByText('GLOBAL TRIGGER DEFAULTS')).toBeDefined();

    await waitFor(() => {
      expect(screen.getByText('Computer Scientists Group')).toBeDefined();
      expect(screen.getByText('Direct Contact')).toBeDefined();
      expect(screen.getByText('120363040000000000@g.us')).toBeDefined();
    });
  });

  it('updates channel trigger policy when policy button clicked', async () => {
    render(<ChannelTriggerMatrix />);

    await waitFor(() => {
      expect(screen.getByText('Computer Scientists Group')).toBeDefined();
    });

    const mutedButtons = screen.getAllByRole('button', { name: 'Muted' });
    expect(mutedButtons.length).toBeGreaterThan(0);

    // Click Muted on group channel
    fireEvent.click(mutedButtons[mutedButtons.length - 2]); // target channel row button

    await waitFor(() => {
      expect(global.fetch).toHaveBeenCalledWith(
        '/api/tools/channels/policy',
        expect.objectContaining({
          method: 'POST',
          body: expect.stringContaining('"policy":"muted"'),
        })
      );
    });
  });

  it('filters channels by search query and type filter', async () => {
    render(<ChannelTriggerMatrix />);

    await waitFor(() => {
      expect(screen.getByText('Computer Scientists Group')).toBeDefined();
    });

    const searchInput = screen.getByPlaceholderText('Search group name, JID...');
    fireEvent.change(searchInput, { target: { value: 'Computer' } });

    expect(screen.getByText('Computer Scientists Group')).toBeDefined();
    expect(screen.queryByText('Direct Contact')).toBeNull();

    // Clear search and test Direct filter
    fireEvent.change(searchInput, { target: { value: '' } });
    const directFilterBtn = screen.getByRole('button', { name: /Direct/ });
    fireEvent.click(directFilterBtn);

    expect(screen.queryByText('Computer Scientists Group')).toBeNull();
    expect(screen.getByText('Direct Contact')).toBeDefined();
  });

  it('allows editing and saving mention keywords', async () => {
    render(<ChannelTriggerMatrix />);

    await waitFor(() => {
      expect(screen.getByText('Computer Scientists Group')).toBeDefined();
    });

    const input = screen.getByPlaceholderText('styx, assistant, ai, bot') as HTMLInputElement;
    fireEvent.change(input, { target: { value: 'styx, assistant, operator' } });

    const saveBtn = screen.getByRole('button', { name: 'Save' });
    fireEvent.click(saveBtn);

    await waitFor(() => {
      expect(global.fetch).toHaveBeenCalledWith(
        '/api/tools/channels/defaults',
        expect.objectContaining({
          method: 'POST',
          body: expect.stringContaining('"mention_keywords":"styx, assistant, operator"'),
        })
      );
    });
  });

  it('performs quick batch mute on all group channels', async () => {
    render(<ChannelTriggerMatrix />);

    await waitFor(() => {
      expect(screen.getByText('Mute All Groups')).toBeDefined();
    });

    const muteAllBtn = screen.getByText('Mute All Groups');
    fireEvent.click(muteAllBtn);

    await waitFor(() => {
      expect(global.fetch).toHaveBeenCalledWith(
        '/api/tools/channels/policy',
        expect.objectContaining({
          method: 'POST',
          body: expect.stringContaining('"channel_id":"120363040000000000@g.us"'),
        })
      );
    });
  });
});
