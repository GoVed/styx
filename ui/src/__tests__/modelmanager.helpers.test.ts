import { describe, it, expect } from 'vitest';
import { parseContainerLoadingStage, safeFetchJson } from '../components/ModelManager';

describe('parseContainerLoadingStage helper', () => {
  it('accurately parses out of memory KV cache errors with plain English explanations', () => {
    const logs = [
      'INFO 09-27 22:33:38 [vllm] Model weights loaded into GPU in 12.4s',
      "ValueError: To serve at least one request with the model's max seq len (131072), 1.45 GiB KV cache is needed, which is larger than the available KV cache memory (0.2 GiB). Try decreasing max_model_len",
    ];
    const res = parseContainerLoadingStage(logs, 'exited', 'Exited (1)');
    expect(res.stage).toBe('error');
    expect(res.errorKind).toBe('oom');
    expect(res.errorHeadline).toContain('Out of GPU Memory (128k Context Limit in vLLM)');
    expect(res.errorDetail).toContain('vLLM requires ~11.7 GB');
  });

  it('accurately parses multi-modal warmup and attention kernel optimization', () => {
    const logs = [
      'INFO 09-27 22:33:28 [vllm] Multi-modal warmup completed in 10.126s',
      'INFO 09-27 22:33:38 [vllm] Readonly multi-modal warmup completed in 0.662s',
    ];
    const res = parseContainerLoadingStage(logs, 'running');
    expect(res.stage).toBe('optimizing');
    expect(res.stageIndex).toBe(3);
    expect(res.progressPct).toBe(75);
    expect(res.title).toBe('Optimizing Attention Kernels & Warming Up');
  });

  it('accurately detects ready serving state', () => {
    const logs = [
      'INFO:     Started server process [1]',
      'INFO:     Application startup complete.',
      'INFO:     Uvicorn running on http://0.0.0.0:8000 (Press CTRL+C to quit)',
    ];
    const res = parseContainerLoadingStage(logs, 'running');
    expect(res.stage).toBe('ready');
    expect(res.stageIndex).toBe(4);
    expect(res.progressPct).toBe(100);
    expect(res.title).toBe('Model Ready & Active');
  });

  it('accurately parses CLI argument errors like removed or invalid options', () => {
    const logs = [
      'error while handling argument "--draft-max": the argument has been removed. use --spec-draft-n-max or --spec-ngram-mod-n-max',
    ];
    const res = parseContainerLoadingStage(logs, 'exited', 'Exited (1)');
    expect(res.stage).toBe('error');
    expect(res.errorKind).toBe('invalid_arg');
    expect(res.errorHeadline).toContain('Engine Argument Flag Incompatible');
  });
});

describe('safeFetchJson helper', () => {
  it('successfully parses valid JSON response', async () => {
    const mockRes = {
      ok: true,
      status: 200,
      text: () => Promise.resolve('{"success":true,"model":"test"}'),
    } as unknown as Response;
    const data = await safeFetchJson(mockRes);
    expect(data).toEqual({ success: true, model: 'test' });
  });

  it('gracefully handles Axum 422 plain text errors without throwing JSON syntax error', async () => {
    const plainText422 = "Failed to deserialize the JSON body into the target type: unknown variant `llamacpp`, expected one of `vllm`, `llama_cpp`, `ollama`";
    const mockRes = {
      ok: false,
      status: 422,
      statusText: 'Unprocessable Entity',
      text: () => Promise.resolve(plainText422),
    } as unknown as Response;
    const data = await safeFetchJson(mockRes);
    expect(data.success).toBe(false);
    expect(data.error).toContain('Failed to deserialize');
  });

  it('gracefully handles 500/502 HTTP errors with status code', async () => {
    const mockRes = {
      ok: false,
      status: 502,
      statusText: 'Bad Gateway',
      text: () => Promise.resolve(''),
    } as unknown as Response;
    const data = await safeFetchJson(mockRes);
    expect(data.success).toBe(false);
    expect(data.error).toContain('502 Bad Gateway');
  });
});
