import { describe, it, expect, beforeEach } from 'vitest';
import { render, screen, fireEvent, waitFor, act } from '@testing-library/react';
import React from 'react';
import { ModelManager } from '../components/ModelManager';
import { EnginePreset } from '../types';

const mockPresets: EnginePreset[] = [
  {
    id: 'vllm-qwen3.8-27b',
    label: 'Qwen 3.8 27B (vLLM)',
    engine: 'vllm',
    hf_repo: 'Qwen/Qwen3.8-27B-Instruct',
    default_context: 65536,
    default_port: 8000,
    recommended_gpu: '24GB VRAM (AWQ/GPTQ) or 48GB+ (FP8)',
    default_quantization: 'awq',
    default_tp: 1,
    description: 'Latest 27B dense reasoning model with MTP and 64k context.',
    model_size_b: 27,
    enable_mtp: true,
    enable_vision: true,
  },
];

describe('ModelManager VRAM & KV Cache Slider Tests', () => {
  beforeEach(() => {
    (global.fetch as any) = vi.fn().mockImplementation(() =>
      Promise.resolve({
        ok: true,
        status: 200,
        text: () => Promise.resolve(JSON.stringify({ success: true, command: 'docker run ...' })),
        json: () => Promise.resolve({ success: true, command: 'docker run ...' }),
      })
    );
  });

  it('snaps KV cache quantization slider between engine-available options', async () => {
    render(<ModelManager presets={mockPresets} />);

    await waitFor(() => {
      expect(screen.getByLabelText('KV Cache Quantization Slider')).toBeInTheDocument();
    });

    const slider = screen.getByLabelText('KV Cache Quantization Slider') as HTMLInputElement;
    // Default is FP8 (index 3)
    expect(slider.value).toBe('3');

    // Move slider to 0 (INT4 for vLLM)
    act(() => {
      fireEvent.change(slider, { target: { value: '0' } });
    });
    expect(slider.value).toBe('0');
    expect(screen.getAllByText(/~50% VRAM saved/).length).toBeGreaterThanOrEqual(1);

    // Move slider to 1 (TurboQuant for vLLM)
    act(() => {
      fireEvent.change(slider, { target: { value: '1' } });
    });
    expect(slider.value).toBe('1');
    expect(screen.getAllByText(/~65% VRAM saved/).length).toBeGreaterThanOrEqual(1);

    // Click INT4 pill directly and verify slider snaps to 0
    act(() => {
      fireEvent.click(screen.getByText('INT4 (4-bit)'));
    });
    expect(slider.value).toBe('0');
  });

  it('updates VRAM requirement dynamically when model quant or KV quant changes', async () => {
    render(<ModelManager presets={mockPresets} />);

    await waitFor(() => {
      expect(screen.getByText('CONTAINER PROVISIONER CONFIGURATION')).toBeInTheDocument();
    });

    // Initial state: AWQ (4-bit) for 27B model: 27 * 0.55 = ~14.9 GB
    expect(screen.getAllByText(/~14.9 GB/).length).toBeGreaterThanOrEqual(1);

    // Initial KV cache: 64k tokens FP8: ~5.5 GB
    expect(screen.getAllByText(/~5.5 GB/).length).toBeGreaterThanOrEqual(1);

    // Click 'FP8 (8-bit)' model quant: 27 * 1.05 = ~28.4 GB
    act(() => {
      fireEvent.click(screen.getByText('FP8 (8-bit)'));
    });
    expect(screen.getAllByText(/~28.4 GB/).length).toBeGreaterThanOrEqual(1);

    // Click 'INT4 (4-bit)' KV cache: 5.5 * 0.5 = ~2.8 GB
    act(() => {
      fireEvent.click(screen.getByText('INT4 (4-bit)'));
    });
    expect(screen.getAllByText(/~2.8 GB/).length).toBeGreaterThanOrEqual(1);

    // Click 'TurboQuant (3-bit)' KV cache: 5.5 * 0.38 = ~2.1 GB
    act(() => {
      fireEvent.click(screen.getByText('TurboQuant (3-bit)'));
    });
    expect(screen.getAllByText(/~2.1 GB/).length).toBeGreaterThanOrEqual(1);
  });
});
