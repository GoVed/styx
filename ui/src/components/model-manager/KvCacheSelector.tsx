import React from 'react';
import { calculateKvCacheGb } from './vram';

export interface KvCacheSelectorProps {
  selectedEngine: string;
  contextWindow: number;
  kvCacheDtype: string;
  onChangeKvCacheDtype: (dtype: string) => void;
}

export const ENGINE_KV_OPTIONS: Record<string, any[]> = {
  vllm: [
    { id: 'int4_per_token_head', aliases: ['int4', 'int4_per_token_head', 'q4'], label: 'INT4 (4-bit)', shortLabel: 'INT4', savings: '~50% VRAM saved' },
    { id: 'turboquant_3bit_nc', aliases: ['turboquant', 'turboquant_3bit_nc', 'q3'], label: 'TurboQuant (3-bit)', shortLabel: 'TurboQuant', savings: '~65% VRAM saved' },
    { id: 'fp8_e4m3', aliases: ['fp8_e4m3'], label: 'FP8 E4M3 (8-bit)', shortLabel: 'FP8 E4M3', savings: '~50% vs FP16' },
    { id: 'fp8', aliases: ['fp8', 'fp8_e5m2'], label: 'FP8 (Recommended)', shortLabel: 'FP8 ★', savings: '~50% vs FP16' },
    { id: 'auto', aliases: ['auto', 'fp16', 'f16'], label: 'Auto / FP16', shortLabel: 'FP16', savings: 'Uncompressed' },
  ],
  llamacpp: [
    { id: 'q2_k', aliases: ['q2', 'q2_k'], label: 'Q2_K (2-bit)', shortLabel: 'Q2_K', savings: '~75% VRAM saved' },
    { id: 'q3_k_m', aliases: ['q3', 'q3_k', 'q3_k_m'], label: 'Q3_K_M (3-bit)', shortLabel: 'Q3_K', savings: '~65% VRAM saved' },
    { id: 'q4_0', aliases: ['q4', 'q4_0', 'int4'], label: 'Q4_0 (4-bit)', shortLabel: 'Q4_0', savings: '~50% VRAM saved' },
    { id: 'q8_0', aliases: ['q8', 'q8_0', 'fp8'], label: 'Q8_0 (8-bit)', shortLabel: 'Q8_0 ★', savings: '~50% vs FP16' },
    { id: 'f16', aliases: ['auto', 'fp16', 'f16'], label: 'Auto / F16', shortLabel: 'F16', savings: 'Uncompressed' },
  ],
};

export const KvCacheSelector: React.FC<KvCacheSelectorProps> = ({
  selectedEngine,
  contextWindow,
  kvCacheDtype,
  onChangeKvCacheDtype,
}) => {
  const availableKvLevels = ENGINE_KV_OPTIONS[selectedEngine] || ENGINE_KV_OPTIONS.vllm;
  const kvSliderIndex = (() => {
    const idx = availableKvLevels.findIndex(
      item => item.id === kvCacheDtype || item.aliases?.includes(kvCacheDtype)
    );
    return idx !== -1 ? idx : Math.min(3, availableKvLevels.length - 1);
  })();
  const currentKv = availableKvLevels[kvSliderIndex] || availableKvLevels[3];

  return (
    <div className="space-y-1.5 pt-1 border-t border-syndae-800">
      <div className="flex items-center justify-between">
        <label className="text-[11px] text-slate-400">
          KV Cache Quantization ({selectedEngine === 'vllm' ? 'vLLM' : 'llama.cpp'}):
        </label>
        <div className="flex items-center space-x-1.5">
          <span className="text-[10px] px-1.5 py-0.5 rounded bg-amber-950 text-amber-300 border border-amber-800 font-bold">
            ~{calculateKvCacheGb(contextWindow, kvCacheDtype)} GB
          </span>
          <span className="text-[10px] text-amber-300">
            {currentKv.savings}
          </span>
        </div>
      </div>

      {/* Snapping Slider */}
      <input
        type="range"
        min="0"
        max={availableKvLevels.length - 1}
        step="1"
        aria-label="KV Cache Quantization Slider"
        value={kvSliderIndex}
        onChange={e => {
          const idx = parseInt(e.target.value, 10);
          if (availableKvLevels[idx]) {
            onChangeKvCacheDtype(availableKvLevels[idx].id);
          }
        }}
        className="w-full h-2 bg-syndae-850 rounded appearance-none cursor-pointer accent-amber-500"
      />

      {/* Quick Snap Pills */}
      <div className="flex flex-wrap items-center gap-1">
        {availableKvLevels.map((kv, idx) => (
          <button
            key={kv.id}
            type="button"
            onClick={() => onChangeKvCacheDtype(kv.id)}
            className={`px-2 py-0.5 rounded text-xs border ${
              kvSliderIndex === idx
                ? 'bg-amber-950 border-amber-500 text-amber-200 font-bold'
                : 'bg-syndae-950 border-syndae-800 text-slate-400'
            }`}
          >
            {kv.label}
          </button>
        ))}
        <button
          type="button"
          onClick={() => onChangeKvCacheDtype('bf16')}
          className={`px-2 py-0.5 rounded text-xs border ${
            kvCacheDtype === 'bf16'
              ? 'bg-amber-950 border-amber-500 text-amber-200 font-bold'
              : 'bg-syndae-950 border-syndae-800 text-slate-400'
          }`}
        >
          BF16
        </button>
      </div>
    </div>
  );
};
