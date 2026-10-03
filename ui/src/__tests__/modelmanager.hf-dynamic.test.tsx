import { describe, it, expect, vi, beforeEach, afterEach } from 'vitest';
import { render, screen, fireEvent, waitFor, act } from '@testing-library/react';
import { ModelManager } from '../components/ModelManager';

describe('ModelManager - Dynamic Hugging Face Quantization', () => {
  const originalFetch = global.fetch;

  beforeEach(() => {
    vi.useFakeTimers({ shouldAdvanceTime: true });
  });

  afterEach(() => {
    vi.useRealTimers();
    global.fetch = originalFetch;
  });

  it('inspects Hugging Face GGUF repository and renders dynamic quantization variants', async () => {
    global.fetch = vi.fn().mockImplementation((url: string) => {
      if (url.includes('/api/models/hf-inspect')) {
        return Promise.resolve({
          ok: true,
          status: 200,
          text: () =>
            Promise.resolve(
              JSON.stringify({
                success: true,
                repo: 'IFM/K2-Horizon-7B-GGUF',
                is_gguf: true,
                recommended_engine: 'llama.cpp',
                recommended_quant: 'Q4_K_M',
                total_files: 6,
                quants: [
                  { quant: 'Q4_K_M', filename: 'K2-Horizon-7B-Q4_K_M.gguf', size_gb: 5.21 },
                  { quant: 'Q5_0', filename: 'K2-Horizon-7B-Q5_0.gguf', size_gb: 5.90 },
                  { quant: 'Q5_K_M', filename: 'K2-Horizon-7B-Q5_K_M.gguf', size_gb: 6.02 },
                  { quant: 'Q6_K', filename: 'K2-Horizon-7B-Q6_K.gguf', size_gb: 6.89 },
                  { quant: 'Q8_0', filename: 'K2-Horizon-7B-Q8_0.gguf', size_gb: 8.92 },
                  { quant: 'BF16', filename: 'K2-Horizon-7B-BF16.gguf', size_gb: 16.77 },
                ],
              })
            ),
        });
      }
      if (url.includes('/api/models/preview-command')) {
        return Promise.resolve({
          ok: true,
          status: 200,
          text: () => Promise.resolve(JSON.stringify({ success: true, command: 'docker run ... -hf IFM/K2-Horizon-7B-GGUF:Q4_K_M' })),
        });
      }
      return Promise.resolve({
        ok: true,
        status: 200,
        text: () => Promise.resolve(JSON.stringify({ success: true, files: [] })),
      });
    });

    render(<ModelManager />);

    const repoInput = screen.getByPlaceholderText(/cyankiwi\/Qwen3.8-27B-AWQ-INT4/i);
    expect(repoInput).toBeInTheDocument();

    // User pastes full Hugging Face tree URL
    act(() => {
      fireEvent.change(repoInput, {
        target: { value: 'https://huggingface.co/IFM/K2-Horizon-7B-GGUF/tree/main' },
      });
    });

    // Advance debounced inspection timer (450ms)
    await act(async () => {
      vi.advanceTimersByTime(500);
    });

    // Verify dynamic variants are rendered
    await waitFor(() => {
      expect(screen.getByText(/Hugging Face GGUF \(6 variants\)/i)).toBeInTheDocument();
    });

    expect(screen.getByText('Q4_K_M (5.2 GB)')).toBeInTheDocument();
    expect(screen.getByText('Q5_0 (5.9 GB)')).toBeInTheDocument();
    expect(screen.getByText('Q5_K_M (6.0 GB)')).toBeInTheDocument();
    expect(screen.getByText('Q6_K (6.9 GB)')).toBeInTheDocument();
    expect(screen.getByText('Q8_0 (8.9 GB)')).toBeInTheDocument();
    expect(screen.getByText('BF16 (16.8 GB)')).toBeInTheDocument();

    // Verify GGUF auto-switched badge
    expect(screen.getByText(/GGUF Auto-Switched/i)).toBeInTheDocument();

    // Verify exact GGUF weight is displayed
    expect(screen.getByText(/5.2 GB \(Exact GGUF Size\)/i)).toBeInTheDocument();

    // Click on Q8_0 variant
    act(() => {
      fireEvent.click(screen.getByText('Q8_0 (8.9 GB)'));
    });

    // Verify weight updates to Q8_0 exact size
    expect(screen.getByText(/8.9 GB \(Exact GGUF Size\)/i)).toBeInTheDocument();
  });

  it('falls back to standard static quantization pills for non-GGUF repositories', async () => {
    global.fetch = vi.fn().mockImplementation((url: string) => {
      if (url.includes('/api/models/hf-inspect')) {
        return Promise.resolve({
          ok: true,
          status: 200,
          text: () =>
            Promise.resolve(
              JSON.stringify({
                success: true,
                repo: 'QuantTrio/Qwen3.5-9B-AWQ',
                is_gguf: false,
                recommended_engine: 'vllm',
                recommended_quant: 'awq',
                total_files: 8,
                quants: [],
              })
            ),
        });
      }
      return Promise.resolve({
        ok: true,
        status: 200,
        text: () => Promise.resolve(JSON.stringify({ success: true, command: 'docker run ...', files: [] })),
      });
    });

    render(<ModelManager />);

    const repoInput = screen.getByPlaceholderText(/cyankiwi\/Qwen3.8-27B-AWQ-INT4/i);
    act(() => {
      fireEvent.change(repoInput, {
        target: { value: 'QuantTrio/Qwen3.5-9B-AWQ' },
      });
    });

    await act(async () => {
      vi.advanceTimersByTime(500);
    });

    await waitFor(() => {
      expect(screen.getByText('AWQ (4-bit)')).toBeInTheDocument();
      expect(screen.getByText('FP8 (8-bit)')).toBeInTheDocument();
      expect(screen.getByText('GPTQ (4-bit)')).toBeInTheDocument();
      expect(screen.getByText('None (FP16)')).toBeInTheDocument();
    });
  });
});

