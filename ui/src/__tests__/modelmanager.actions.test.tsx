import { describe, it, expect, vi, beforeEach } from 'vitest';
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

describe('ModelManager Actions Tests', () => {
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

  it('renders running containers and supports 1-click Set Active model', async () => {
    const mockContainers = [
      {
        id: 'c1234567890ab',
        names: ['/styx-qwen-3-5-9b-vllm'],
        image: 'vllm/vllm-openai-rocm:latest',
        status: 'Up 10 minutes',
        state: 'running',
        ports: ['8000:8000 (TCP)'],
        is_styx_managed: true,
        created: 1790450000,
      },
    ];

    const mockConfigs = [
      {
        id: 'cfg-1',
        name: 'Existing Model',
        provider: 'openai',
        base_url: 'http://localhost:11434/v1',
        api_key: '',
        model_id: 'some-model',
        context_length: 32768,
        is_active: true,
        created_at: '2026-09-26T00:00:00Z',
      },
    ];

    const onRefreshMock = vi.fn();

    render(
      <ModelManager
        presets={mockPresets}
        containers={mockContainers}
        modelConfigs={mockConfigs}
        onRefresh={onRefreshMock}
      />
    );

    expect(screen.getByText('/styx-qwen-3-5-9b-vllm')).toBeInTheDocument();
    const setActiveBtn = screen.getByRole('button', { name: /Set Active/i });
    expect(setActiveBtn).toBeInTheDocument();

    // Click Set Active
    await act(async () => {
      fireEvent.click(setActiveBtn);
    });

    await waitFor(() => {
      expect(global.fetch).toHaveBeenCalledWith('/api/models/containers/c1234567890ab/activate', {
        method: 'POST',
      });
    });
  });

  it('sends engine as llama_cpp when llama.cpp is selected and deploys successfully', async () => {
    let capturedDeployPayload: any = null;
    (global.fetch as any).mockImplementation((url: string, options?: any) => {
      if (url === '/api/models/deploy' && options?.body) {
        capturedDeployPayload = JSON.parse(options.body);
        return Promise.resolve({
          ok: true,
          status: 200,
          statusText: 'OK',
          text: () => Promise.resolve(JSON.stringify({ success: true, container_id: 'llama-cont-123' })),
          json: () => Promise.resolve({ success: true, container_id: 'llama-cont-123' }),
        });
      }
      return Promise.resolve({
        ok: true,
        status: 200,
        statusText: 'OK',
        text: () => Promise.resolve(JSON.stringify({ success: true, command: 'docker run llama.cpp ...' })),
        json: () => Promise.resolve({ success: true, command: 'docker run llama.cpp ...' }),
      });
    });

    render(<ModelManager presets={mockPresets} />);

    // Click llama.cpp engine button
    const llamaBtn = screen.getByRole('button', { name: /llama\.cpp/i });
    fireEvent.click(llamaBtn);

    // Click Deploy Inference Container
    const deployBtn = screen.getByText('Deploy Inference Container');
    fireEvent.click(deployBtn);

    await waitFor(() => {
      expect(capturedDeployPayload).not.toBeNull();
      expect(capturedDeployPayload.engine).toBe('llama_cpp');
    });
  });
});
