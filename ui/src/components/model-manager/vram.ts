import { LoadingProgressInfo } from './types';

export const calculateModelWeightsGb = (modelSizeB: number, quant?: string): number => {
  const q = quant?.toLowerCase() || 'awq';
  let multiplier = 0.55;
  if (q === 'none' || q === 'fp16' || q === 'bfloat16' || q === 'bf16') {
    multiplier = 2.05;
  } else if (q === 'fp8') {
    multiplier = 1.05;
  } else if (q === 'awq' || q === 'gptq') {
    multiplier = 0.55;
  } else if (q === 'k-quants' || q.startsWith('q4')) {
    multiplier = 0.60;
  } else if (q === 'q3') {
    multiplier = 0.45;
  } else if (q === 'q2') {
    multiplier = 0.35;
  } else {
    multiplier = 0.55;
  }
  return Math.round(modelSizeB * multiplier * 10) / 10;
};

export const calculateKvCacheGb = (contextTokens: number, kvDtype?: string): number => {
  const kv = kvDtype?.toLowerCase() || 'fp8';
  let ratio = 1.0;
  if (kv === 'auto' || kv === 'f16' || kv === 'fp16' || kv === 'bfloat16' || kv === 'bf16') {
    ratio = 2.0; // 16-bit uncompressed
  } else if (kv === 'fp8' || kv === 'fp8_e4m3' || kv === 'fp8_e5m2' || kv === 'q8_0' || kv === 'q8') {
    ratio = 1.0; // 8-bit
  } else if (kv === 'int4_per_token_head' || kv === 'turboquant_4bit_nc' || kv === 'q4_0' || kv === 'q4' || kv === 'int4') {
    ratio = 0.5; // 4-bit (~50% VRAM saved)
  } else if (kv === 'turboquant_3bit_nc' || kv === 'q3_k_m' || kv === 'q3' || kv === 'int3') {
    ratio = 0.38; // 3-bit (~62% VRAM saved)
  } else if (kv === 'q2_k' || kv === 'q2' || kv === 'int2') {
    ratio = 0.25; // 2-bit (~75% VRAM saved)
  } else {
    ratio = 1.0;
  }

  const baseGb = (contextTokens / 65536) * 5.5;
  const total = baseGb * ratio;
  return Math.max(0.1, Math.round(total * 10) / 10);
};

export const calculateTargetQuantization = (
  modelSizeB: number,
  vramGb: number,
  context: number = 65536,
  selectedQuant?: string,
  selectedKvDtype?: string,
  exactWeightsGb?: number,
) => {
  let recommendedQuant = 'awq';
  if (vramGb <= 12) {
    recommendedQuant = modelSizeB <= 9 ? 'awq' : 'k-quants';
  } else if (vramGb <= 16) {
    recommendedQuant = 'awq';
  } else if (vramGb <= 24) {
    recommendedQuant = modelSizeB <= 10 ? 'fp8' : 'awq';
  } else {
    recommendedQuant = modelSizeB <= 32 ? 'fp8' : 'awq';
  }

  const effectiveQuant = selectedQuant || recommendedQuant;
  const effectiveKv = selectedKvDtype || 'fp8';

  const weightsGb = exactWeightsGb && exactWeightsGb > 0
    ? Math.round(exactWeightsGb * 10) / 10
    : calculateModelWeightsGb(modelSizeB, effectiveQuant);
  const kvCacheGb = calculateKvCacheGb(context, effectiveKv);
  const totalGb = Math.round((weightsGb + kvCacheGb) * 10) / 10;
  const fits = totalGb <= vramGb * 0.95;

  let rationale = '';
  if (fits) {
    rationale = `${effectiveQuant.toUpperCase()} model (~${weightsGb} GB) + ${effectiveKv.toUpperCase()} KV cache (~${kvCacheGb} GB) fits comfortably within ${vramGb} GB VRAM.`;
  } else {
    const diff = (totalGb - vramGb).toFixed(1);
    rationale = `${effectiveQuant.toUpperCase()} model (~${weightsGb} GB) + ${effectiveKv.toUpperCase()} KV cache (~${kvCacheGb} GB) exceeds ${vramGb} GB VRAM by ~${diff} GB. Consider Q4/Q2 KV cache, 4-bit weights, or smaller context.`;
  }

  return {
    recommendedQuant,
    weightsGb,
    kvCacheGb,
    totalGb,
    fits,
    rationale,
  };
};

export const safeFetchJson = async (res: Response): Promise<any> => {
  try {
    if (typeof res.text === 'function') {
      const text = await res.text();
      try {
        return JSON.parse(text);
      } catch {
        const cleanError = text.includes('<html') || text.includes('<!DOCTYPE')
          ? (text.match(/<title>([^<]+)<\/title>/i)?.[1] || text.match(/<h1>([^<]+)<\/h1>/i)?.[1] || `Server error (${res.status} ${res.statusText || 'Error'})`).trim()
          : text.trim();
        return {
          success: false,
          error: cleanError || `Server error (${res.status} ${res.statusText || 'Error'})`,
        };
      }
    } else if (typeof (res as any).json === 'function') {
      return await (res as any).json();
    }
  } catch (e) {
    return {
      success: false,
      error: e instanceof Error ? e.message : String(e),
    };
  }
  return { success: false, error: 'Invalid response object' };
};

export const parseContainerLoadingStage = (
  logs: string[] = [],
  containerState: string = 'running',
  containerStatus: string = ''
): LoadingProgressInfo => {
  const logStr = logs.join('\n');
  const lowerLog = logStr.toLowerCase();

  // 1. Detect Out of Memory / KV Cache limit errors
  if (
    logStr.includes('larger than the available KV cache memory') ||
    logStr.includes("To serve at least one request with the model's max seq len") ||
    logStr.includes('CUDA out of memory') ||
    logStr.includes('HIP out of memory') ||
    logStr.includes('torch.cuda.OutOfMemoryError') ||
    logStr.includes('cudaMalloc failed: out of memory') ||
    logStr.includes('failed to allocate ROCm0 buffer')
  ) {
    return {
      stage: 'error',
      stageIndex: 0,
      progressPct: 0,
      title: 'GPU Memory Limit Exceeded',
      description: 'The requested model and context length exceed your GPU VRAM headroom.',
      errorKind: 'oom',
      errorHeadline: 'Out of GPU Memory (128k Context Limit in vLLM)',
      errorDetail:
        'vLLM requires ~11.7 GB for PyTorch weights and vision encoders, leaving insufficient space for a 128,000-token continuous KV cache on a 16 GB GPU. Decrease context length to 16k in vLLM, or switch to llama.cpp to run full 128k using local GGUF quant.',
    };
  }

  // 2. Detect CLI flag / argument errors
  if (
    logStr.includes('error while handling argument') ||
    logStr.includes('the argument has been removed') ||
    logStr.includes('invalid choice:') ||
    logStr.includes('unrecognized arguments:')
  ) {
    return {
      stage: 'error',
      stageIndex: 0,
      progressPct: 0,
      title: 'Incompatible CLI Parameter',
      description: 'The engine received an invalid command line flag.',
      errorKind: 'invalid_arg',
      errorHeadline: 'Engine Argument Flag Incompatible',
      errorDetail:
        'The container failed to start due to an obsolete or unsupported CLI flag. Click below to launch with verified clean parameters.',
    };
  }

  // 3. Detect Repository Not Found errors
  if (
    logStr.includes('RepositoryNotFoundError') ||
    logStr.includes('404 Client Error') ||
    logStr.includes('is not a local folder or a valid model identifier')
  ) {
    return {
      stage: 'error',
      stageIndex: 0,
      progressPct: 0,
      title: 'Model Repository Not Found',
      description: 'The Hugging Face repo or model path does not exist.',
      errorKind: 'repo_not_found',
      errorHeadline: 'Model Not Found on Hugging Face / Disk',
      errorDetail: 'The repository ID or file path is incorrect. Please verify the model identifier.',
    };
  }

  // 4. Detect container exited unexpectedly
  if (
    (containerState === 'exited' || containerState === 'dead') &&
    !logStr.includes('Application startup complete') &&
    !logStr.includes('Uvicorn running on')
  ) {
    return {
      stage: 'error',
      stageIndex: 0,
      progressPct: 0,
      title: 'Container Exited Unexpectedly',
      description: 'The inference container process stopped.',
      errorKind: 'crashed',
      errorHeadline: 'Container Process Stopped',
      errorDetail: containerStatus || 'Container exited during initialization. Check technical logs for details.',
    };
  }

  // 5. Ready stage
  if (
    lowerLog.includes('application startup complete') ||
    lowerLog.includes('uvicorn running on') ||
    lowerLog.includes('http server listening') ||
    lowerLog.includes('all slots are idle') ||
    (lowerLog.includes('main: server is listening on') && lowerLog.includes('8080')) ||
    lowerLog.includes('listening on [::]:8080')
  ) {
    return {
      stage: 'ready',
      stageIndex: 4,
      progressPct: 100,
      title: 'Model Ready & Active',
      description: 'Inference engine is healthy, warmed up, and serving OpenAI API requests.',
    };
  }

  // 6. Optimizing Kernels & Warmup stage
  if (
    lowerLog.includes('multi-modal warmup') ||
    lowerLog.includes('warmup completed') ||
    lowerLog.includes('capturing cuda graph') ||
    lowerLog.includes('capturing cudagraph') ||
    lowerLog.includes('graph capture') ||
    lowerLog.includes('warmup run') ||
    lowerLog.includes('warming up') ||
    lowerLog.includes('init engine') ||
    lowerLog.includes('profiling')
  ) {
    return {
      stage: 'optimizing',
      stageIndex: 3,
      progressPct: 75,
      title: 'Optimizing Attention Kernels & Warming Up',
      description: 'Capturing CUDA graphs and optimizing GPU execution kernels for maximum speed.',
    };
  }

  // 7. Loading to VRAM stage
  if (
    lowerLog.includes('loading safetensors') ||
    lowerLog.includes('loading model weights') ||
    lowerLog.includes('load_weights') ||
    lowerLog.includes('llama_model_load:') ||
    lowerLog.includes('allocating') ||
    lowerLog.includes('offloaded')
  ) {
    return {
      stage: 'loading_weights',
      stageIndex: 2,
      progressPct: 50,
      title: 'Loading Weights to GPU Memory (VRAM)',
      description: 'Transferring model weights and neural network layers into GPU memory.',
    };
  }

  // 8. Downloading stage
  if (
    lowerLog.includes('downloading') ||
    lowerLog.includes('fetching') ||
    lowerLog.includes('%|') ||
    lowerLog.includes('.safetensors:')
  ) {
    return {
      stage: 'downloading',
      stageIndex: 1,
      progressPct: 25,
      title: 'Downloading Model Weights',
      description:
        'Downloading weights from Hugging Face hub (weights are cached locally for future instant startups).',
    };
  }

  // 9. Starting or Idle
  if (containerState === 'running') {
    return {
      stage: 'downloading',
      stageIndex: 1,
      progressPct: 15,
      title: 'Starting Container & Verifying Cache',
      description: 'Container spawned. Verifying cached weights on disk and preparing GPU runtime...',
    };
  }

  return {
    stage: 'idle',
    stageIndex: 0,
    progressPct: 0,
    title: 'Container Offline',
    description: 'Select a container or deploy a new model above to monitor startup progress.',
  };
};

export const HARDWARE_PROFILES = [
  { vram: 12, label: '12 GB VRAM (RTX 3060 / 4070)', badge: 'Budget GPU' },
  { vram: 16, label: '16 GB VRAM (RX 7800 XT / 6800 / RTX 4080)', badge: 'Mainstream' },
  { vram: 24, label: '24 GB VRAM (RTX 3090 / 4090 / RX 7900 XTX)', badge: 'Power User' },
  { vram: 48, label: '48+ GB VRAM (Dual GPU / Mac Studio / Workstation)', badge: 'Enterprise' },
];

export const VLLM_KV_OPTIONS = [
  { label: 'INT4 (4-bit)', value: 'int4_per_token_head', description: 'Maximum context (~50% VRAM saved)' },
  { label: 'TurboQuant (3-bit)', value: 'turboquant_3bit_nc', description: 'Extreme context (~62% VRAM saved)' },
  { label: 'FP8 (E4M3)', value: 'fp8_e4m3', description: 'Low precision FP8' },
  { label: 'FP8 Recommended', value: 'fp8', description: 'Standard 8-bit cache (Recommended)' },
  { label: 'Auto (FP16)', value: 'auto', description: 'Full 16-bit uncompressed' },
];

export const LLAMACPP_KV_OPTIONS = [
  { label: 'Q2_K (2-bit)', value: 'q2_k', description: 'Extreme context (~75% VRAM saved)' },
  { label: 'Q3_K_M (3-bit)', value: 'q3_k_m', description: 'Deep context (~62% VRAM saved)' },
  { label: 'Q4_0 (4-bit)', value: 'q4_0', description: 'Balanced 4-bit (~50% VRAM saved)' },
  { label: 'Q8_0 (8-bit)', value: 'q8_0', description: 'High precision 8-bit' },
  { label: 'F16 (Auto)', value: 'f16', description: 'Full 16-bit precision' },
];

export const LLAMA_CPP_KV_OPTIONS = LLAMACPP_KV_OPTIONS;

