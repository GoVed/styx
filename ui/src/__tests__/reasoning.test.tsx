import { describe, it, expect, vi, beforeEach } from 'vitest';
import { render, screen, fireEvent, waitFor } from '@testing-library/react';
import React from 'react';
import { ReasoningSelector } from '../components/model-manager/ReasoningSelector';

describe('ReasoningSelector Component', () => {
  beforeEach(() => {
    vi.clearAllMocks();
    global.fetch = vi.fn().mockImplementation((url, options) => {
      if (typeof url === 'string' && url.includes('/api/models/reasoning')) {
        if (options && options.method === 'POST') {
          const body = JSON.parse(options.body as string);
          return Promise.resolve({
            ok: true,
            json: () => Promise.resolve({ success: true, reasoning_effort: body.reasoning_effort }),
          });
        }
        return Promise.resolve({
          ok: true,
          json: () =>
            Promise.resolve({
              success: true,
              reasoning_effort: 'high',
              available_levels: ['high', 'medium', 'low', 'off'],
            }),
        });
      }
      return Promise.reject(new Error('Unknown endpoint'));
    });
  });

  it('renders reasoning levels with High set as default', async () => {
    render(<ReasoningSelector />);

    expect(screen.getByText('Reasoning Effort')).toBeDefined();
    expect(screen.getByText('Default: High')).toBeDefined();
    expect(screen.getByText('High')).toBeDefined();
    expect(screen.getByText('Medium')).toBeDefined();
    expect(screen.getByText('Low')).toBeDefined();
    expect(screen.getByText('Off')).toBeDefined();
  });

  it('allows switching reasoning effort to Medium and sends POST request', async () => {
    render(<ReasoningSelector />);

    await waitFor(() => {
      expect(screen.getByText('High')).toBeDefined();
    });

    const mediumBtn = screen.getByText('Medium').closest('button');
    expect(mediumBtn).not.toBeNull();
    fireEvent.click(mediumBtn!);

    await waitFor(() => {
      expect(global.fetch).toHaveBeenCalledWith(
        '/api/models/reasoning',
        expect.objectContaining({
          method: 'POST',
          body: JSON.stringify({ reasoning_effort: 'medium' }),
        })
      );
    });
  });

  it('allows disabling reasoning with Off option', async () => {
    render(<ReasoningSelector />);

    const offBtn = screen.getByText('Off').closest('button');
    expect(offBtn).not.toBeNull();
    fireEvent.click(offBtn!);

    await waitFor(() => {
      expect(global.fetch).toHaveBeenCalledWith(
        '/api/models/reasoning',
        expect.objectContaining({
          method: 'POST',
          body: JSON.stringify({ reasoning_effort: 'off' }),
        })
      );
    });
  });
});
