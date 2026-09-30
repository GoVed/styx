import { describe, it, expect, vi, beforeEach } from 'vitest';
import { render, screen, fireEvent, waitFor } from '@testing-library/react';
import React from 'react';
import { MemoryHub } from '../components/MemoryHub';
import { MemoryFileNode } from '../types';

const mockFiles: MemoryFileNode[] = [
  {
    path: 'core/system_instructions.md',
    filename: 'system_instructions.md',
    category: 'core',
    size_bytes: 4600,
    modified_at: '2026-09-30T12:00:00Z',
  },
  {
    path: 'core/user_profile.md',
    filename: 'user_profile.md',
    category: 'core',
    size_bytes: 900,
    modified_at: '2026-09-30T12:00:00Z',
  },
  {
    path: 'dictionary/slang.md',
    filename: 'slang.md',
    category: 'dictionary',
    size_bytes: 500,
    modified_at: '2026-09-30T12:00:00Z',
  },
];

describe('MemoryHub Responsive Mobile UI Tests', () => {
  beforeEach(() => {
    vi.stubGlobal(
      'fetch',
      vi.fn().mockImplementation((url: string) => {
        if (url.includes('/api/memory/file')) {
          return Promise.resolve({
            json: () => Promise.resolve({ success: true, content: '# Test File Content\nHello world' }),
          });
        }
        return Promise.resolve({
          json: () => Promise.resolve({ success: true }),
        });
      })
    );
  });

  it('renders mobile tab switcher with Files count and active editor filename', () => {
    render(<MemoryHub files={mockFiles} onRefreshFiles={() => {}} />);

    expect(screen.getByText('Files (3)')).toBeInTheDocument();
    expect(screen.getAllByText('system_instructions.md').length).toBeGreaterThan(0);
  });

  it('switches between files list and editor when mobile tabs are clicked', async () => {
    render(<MemoryHub files={mockFiles} onRefreshFiles={() => {}} />);

    const editorTabBtn = screen.getByRole('button', { name: /system_instructions\.md/i });
    fireEvent.click(editorTabBtn);

    // Now in editor view, back to files button with title "Browse memory files" is available
    await waitFor(() => {
      const filesBackBtn = screen.getByTitle('Browse memory files');
      expect(filesBackBtn).toBeInTheDocument();
    });

    const filesTabBtn = screen.getByRole('button', { name: /files \(3\)/i });
    fireEvent.click(filesTabBtn);

    expect(screen.getByPlaceholderText(/Tantivy BM25 search/i)).toBeInTheDocument();
  });

  it('automatically transitions to editor when a file row is tapped', async () => {
    render(<MemoryHub files={mockFiles} onRefreshFiles={() => {}} />);

    // Click on user_profile.md row
    const userProfileRow = screen.getByText('user_profile.md');
    fireEvent.click(userProfileRow);

    await waitFor(() => {
      expect(screen.getByText('core/user_profile.md')).toBeInTheDocument();
    });
  });
});
