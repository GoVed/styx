import { describe, it, expect, vi, beforeEach } from 'vitest';
import { render, screen, fireEvent, waitFor, act } from '@testing-library/react';
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
  {
    id: 'vllm-qwen3.5-9b',
    label: 'Qwen 3.5 9B (vLLM)',
    engine: 'vllm',
    hf_repo: 'Qwen/Qwen3.5-9B-Instruct',
    default_context: 65536,
    default_port: 8000,
    recommended_gpu: '12GB VRAM (AWQ) or 24GB (FP8)',
    default_quantization: 'fp8',
    default_tp: 1,
    description: 'High-speed 9B multimodal model with native MTP.',
    model_size_b: 9,
    enable_mtp: true,
    enable_vision: true,
  },
  {
    id: 'vllm-gemma4-12b',
    label: 'Gemma 4 12B (vLLM)',
    engine: 'vllm',
    hf_repo: 'google/gemma-4-12b-it',
    default_context: 65536,
    default_port: 8000,
    recommended_gpu: '16GB VRAM (AWQ) or 24GB (FP8)',
    default_quantization: 'awq',
    default_tp: 1,
    description: 'Google Gemma 4 12B with MTP and vision encoding.',
    model_size_b: 12,
    enable_mtp: true,
    enable_vision: true,
  },
  {
    id: 'vllm-gemma4-31b',
    label: 'Gemma 4 31B (vLLM)',
    engine: 'vllm',
    hf_repo: 'google/gemma-4-31b-it',
    default_context: 65536,
    default_port: 8000,
    recommended_gpu: '24GB VRAM (AWQ/GPTQ) or 48GB+ (FP8)',
    default_quantization: 'awq',
    default_tp: 1,
    description: 'Flagship Gemma 4 31B with 64k context and vision support.',
    model_size_b: 31,
    enable_mtp: true,
    enable_vision: true,
  },
  {
    id: 'vllm-muse-glimmer',
    label: 'Muse Glimmer (vLLM)',
    engine: 'vllm',
    hf_repo: 'muse-ai/muse-glimmer-instruct',
    default_context: 65536,
    default_port: 8000,
    recommended_gpu: '16GB VRAM (AWQ) or 24GB (FP8)',
    default_quantization: 'awq',
    default_tp: 1,
    description: 'Next-gen Muse Glimmer with multimodal image capabilities.',
    model_size_b: 14,
    enable_mtp: true,
    enable_vision: true,
  },
];

describe('ModelManager Component', () => {
  beforeEach(() => {
    global.fetch = vi.fn().mockImplementation((url: string) => {
      let body = { success: true };
      if (url === '/api/models/presets') {
        body = { success: true, presets: mockPresets } as any;
      } else if (url === '/api/models/containers') {
        body = { success: true, containers: [] } as any;
      } else if (url === '/api/models/configs') {
        body = { success: true, configs: [] } as any;
      } else if (url === '/api/models/preview-command') {
        body = { success: true, command: 'docker run ...' } as any;
      } else if (url === '/api/models/local-files') {
        body = { success: true, files: ['/models/Qwen3.5-9B-UD-Q4_K_XL.gguf'] } as any;
      }

      return Promise.resolve({
        ok: true,
        status: 200,
        statusText: 'OK',
        text: () => Promise.resolve(JSON.stringify(body)),
        json: () => Promise.resolve(body),
      });
    });
  });

  it('renders all latest preset models with 64k context, MTP, and Vision badges', async () => {
    render(<ModelManager presets={mockPresets} />);

    await waitFor(() => {
      expect(screen.getByText('Qwen 3.8 27B (vLLM)')).toBeInTheDocument();
      expect(screen.getByText('Qwen 3.5 9B (vLLM)')).toBeInTheDocument();
      expect(screen.getByText('Gemma 4 12B (vLLM)')).toBeInTheDocument();
      expect(screen.getByText('Gemma 4 31B (vLLM)')).toBeInTheDocument();
      expect(screen.getByText('Muse Glimmer (vLLM)')).toBeInTheDocument();
    });

    // Check for badges
    const mtpBadges = screen.getAllByText(/MTP ⚡/);
    expect(mtpBadges.length).toBeGreaterThanOrEqual(5);

    const visionBadges = screen.getAllByText(/Image Input 🖼️/);
    expect(visionBadges.length).toBeGreaterThanOrEqual(5);

    const contextBadges = screen.getAllByText(/64k Context/);
    expect(contextBadges.length).toBeGreaterThanOrEqual(5);
  });

  it('calculates quantization based on GPU size selector', async () => {
    render(<ModelManager presets={mockPresets} />);

    await waitFor(() => {
      expect(screen.getByText('Qwen 3.8 27B (vLLM)')).toBeInTheDocument();
    });

    // Verify GPU size selectors are present
    expect(screen.getByText('12 GB')).toBeInTheDocument();
    expect(screen.getByText('16 GB')).toBeInTheDocument();
    expect(screen.getByText('24 GB (Standard)')).toBeInTheDocument();
    expect(screen.getByText('48 GB+ (Datacenter)')).toBeInTheDocument();

    // Click 12 GB GPU pill
    act(() => {
      fireEvent.click(screen.getByText('12 GB'));
    });

    // Check that VRAM budget calculation is updated
    expect(screen.getByText(/Optimal Quantization:/)).toBeInTheDocument();
  });

  it('applies preset with 65536 context, MTP enabled, and Vision enabled to the form', async () => {
    render(<ModelManager presets={mockPresets} />);

    await waitFor(() => {
      expect(screen.getByText('Qwen 3.8 27B (vLLM)')).toBeInTheDocument();
    });

    // Click on Qwen 3.8 27B preset
    const presetCard = screen.getByText('Qwen 3.8 27B (vLLM)').closest('div');
    if (presetCard) {
      act(() => {
        fireEvent.click(presetCard);
      });
    }

    // Context length input should be 65536
    const contextInput = screen.getByDisplayValue('65536');
    expect(contextInput).toBeInTheDocument();

    // HF Repo input should be Qwen/Qwen3.8-27B-Instruct
    expect(screen.getByDisplayValue('Qwen/Qwen3.8-27B-Instruct')).toBeInTheDocument();

    // Verify MTP and Vision switches are checked
    const mtpCheckbox = screen.getByRole('checkbox', { name: /MTP/i }) as HTMLInputElement;
    expect(mtpCheckbox.checked).toBe(true);

    const visionCheckbox = screen.getByRole('checkbox', { name: /Image/i }) as HTMLInputElement;
    expect(visionCheckbox.checked).toBe(true);
  });

  it('supports both AMD ROCm and NVIDIA CUDA GPU platforms out of the box', async () => {
    const mockTelemetry = {
      host_cpu_pct: 10,
      host_cpu_cores: 16,
      memory_used_mb: 8000,
      memory_total_mb: 32000,
      memory_pct: 25,
      disk_used_gb: 100,
      disk_total_gb: 500,
      disk_pct: 20,
      uptime_secs: 1000,
      running_containers: 0,
      active_mcp_servers: 0,
      pending_approvals: 0,
      active_model: 'None',
      tokens_per_second: 0,
      gpu: {
        name: 'AMD Radeon RX 9070 XT',
        vendor: 'amd' as const,
        vram_used_mb: 2500,
        vram_total_mb: 16384,
        vram_pct: 15,
        gpu_util_pct: 5,
        temperature_c: 54,
      },
    };

    render(<ModelManager presets={mockPresets} telemetry={mockTelemetry} />);

    await waitFor(() => {
      expect(screen.getByText('Qwen 3.8 27B (vLLM)')).toBeInTheDocument();
    });

    // Check that AMD ROCm is detected and indicated
    expect(screen.getAllByText(/AMD ROCm/).length).toBeGreaterThanOrEqual(1);

    // Check GPU Platform dropdown
    const gpuSelect = screen.getByRole('combobox', { name: /GPU Hardware Platform/i }) as HTMLSelectElement;
    expect(gpuSelect).toBeInTheDocument();
    expect(gpuSelect.value).toBe('auto');

    // Switch to AMD ROCm
    act(() => {
      fireEvent.change(gpuSelect, { target: { value: 'amd' } });
    });
    expect(gpuSelect.value).toBe('amd');

    // Switch to NVIDIA CUDA
    act(() => {
      fireEvent.change(gpuSelect, { target: { value: 'nvidia' } });
    });
    expect(gpuSelect.value).toBe('nvidia');
  });

  it('renders interactive pill selectors for every field (model, context, quant, kv quant, and other params)', async () => {
    render(<ModelManager presets={mockPresets} />);

    await waitFor(() => {
      expect(screen.getByText('CONTAINER PROVISIONER CONFIGURATION')).toBeInTheDocument();
    });

    // 1. Popular Model pills
    expect(screen.getByText('Qwen 3.5 9B')).toBeInTheDocument();
    expect(screen.getByText('Llama 3.3 70B')).toBeInTheDocument();
    expect(screen.getByText('DeepSeek R1 14B')).toBeInTheDocument();

    // Click Qwen 3.5 9B pill
    act(() => {
      fireEvent.click(screen.getByText('Qwen 3.5 9B'));
    });
    expect(screen.getByDisplayValue('QuantTrio/Qwen3.5-9B-AWQ')).toBeInTheDocument();
    expect(screen.getByDisplayValue('16384')).toBeInTheDocument();

    // 2. Context Window quick pills
    expect(screen.getByText('16k (16,384)')).toBeInTheDocument();
    expect(screen.getByText('32k (32,768)')).toBeInTheDocument();
    expect(screen.getByText('64k (65,536)')).toBeInTheDocument();
    expect(screen.getByText('128k (131,072)')).toBeInTheDocument();

    // Click 32k pill
    act(() => {
      fireEvent.click(screen.getByText('32k (32,768)'));
    });
    expect(screen.getByDisplayValue('32768')).toBeInTheDocument();

    // 3. Model Quant pills
    expect(screen.getByText('AWQ (4-bit)')).toBeInTheDocument();
    expect(screen.getByText('FP8 (8-bit)')).toBeInTheDocument();
    expect(screen.getAllByText('GPTQ (4-bit)').length).toBeGreaterThanOrEqual(1);
    expect(screen.getByText('None (FP16)')).toBeInTheDocument();

    // Click FP8 model quant
    act(() => {
      fireEvent.click(screen.getByText('FP8 (8-bit)'));
    });

    // 4. KV Cache Quant pills & slider for vLLM
    expect(screen.getByText('FP8 (Recommended)')).toBeInTheDocument();
    expect(screen.getByText('Auto / FP16')).toBeInTheDocument();
    expect(screen.getByText('BF16')).toBeInTheDocument();
    expect(screen.getByText('INT4 (4-bit)')).toBeInTheDocument();
    expect(screen.getByText('TurboQuant (3-bit)')).toBeInTheDocument();

    // Click Auto / FP16 KV cache quant
    act(() => {
      fireEvent.click(screen.getByText('Auto / FP16'));
    });

    // 5. Speed & Multimodal pills
    expect(screen.getByText(/ON \(3 tok\/step/)).toBeInTheDocument();
    expect(screen.getByText(/Vision Enabled/)).toBeInTheDocument();

    // 6. Engine and Port pills
    expect(screen.getByText('vLLM (Recommended)')).toBeInTheDocument();
    expect(screen.getByText('8000')).toBeInTheDocument();
  });
});



