import React from 'react';
import { Box, Code, Play } from 'lucide-react';
import { calculateModelWeightsGb } from './vram';
import { KvCacheSelector } from './KvCacheSelector';

export interface ProvisionerCardProps {
  containerName: string;
  selectedEngine: string;
  hfRepo: string;
  contextWindow: number;
  port: number;
  quantization: string;
  kvCacheDtype: string;
  enableMtp: boolean;
  enableVision: boolean;
  gpuVendor: 'auto' | 'amd' | 'nvidia' | 'none';
  previewCmd: string;
  isDeploying: boolean;
  selectedModelSize: number;
  onChangeContainerName: (name: string) => void;
  onChangeEngine: (engine: string) => void;
  onChangeHfRepo: (repo: string) => void;
  onChangeContextWindow: (ctx: number) => void;
  onChangePort: (port: number) => void;
  onChangeQuantization: (quant: string) => void;
  onChangeKvCacheDtype: (dtype: string) => void;
  onChangeEnableMtp: (val: boolean) => void;
  onChangeEnableVision: (val: boolean) => void;
  onChangeGpuVendor: (vendor: 'auto' | 'amd' | 'nvidia' | 'none') => void;
  onDeploy: () => void;
  onSelectQuickModel: (model: { label: string; repo: string; context: number; size: number }) => void;
}

export const ProvisionerCard: React.FC<ProvisionerCardProps> = ({
  containerName,
  selectedEngine,
  hfRepo,
  contextWindow,
  port,
  quantization,
  kvCacheDtype,
  enableMtp,
  enableVision,
  gpuVendor,
  previewCmd,
  isDeploying,
  selectedModelSize,
  onChangeContainerName,
  onChangeEngine,
  onChangeHfRepo,
  onChangeContextWindow,
  onChangePort,
  onChangeQuantization,
  onChangeKvCacheDtype,
  onChangeEnableMtp,
  onChangeEnableVision,
  onChangeGpuVendor,
  onDeploy,
  onSelectQuickModel,
}) => {
  return (
    <div className="bg-styx-900 border border-styx-800 rounded-lg p-4 space-y-3 font-mono">
      <div className="text-xs font-bold text-slate-200 flex items-center space-x-1.5">
        <Box className="w-3.5 h-3.5 text-cyan-400" />
        <span>CONTAINER PROVISIONER CONFIGURATION</span>
      </div>

      {/* Container Name & Engine Selection */}
      <div className="grid grid-cols-1 sm:grid-cols-2 gap-3">
        <div>
          <label className="text-[11px] text-slate-400">Container Name:</label>
          <input
            type="text"
            value={containerName}
            onChange={e => onChangeContainerName(e.target.value)}
            className="w-full bg-styx-950 border border-styx-700 rounded p-1.5 text-slate-200 mt-1 text-xs"
          />
        </div>
        <div>
          <label className="text-[11px] text-slate-400">Inference Engine:</label>
          <div className="flex flex-wrap items-center gap-1.5 mt-1">
            <button
              type="button"
              onClick={() => onChangeEngine('vllm')}
              className={`px-2.5 py-1 rounded text-xs border ${
                selectedEngine === 'vllm'
                  ? 'bg-emerald-950 border-emerald-500 text-emerald-200 font-bold'
                  : 'bg-styx-950 border-styx-800 text-slate-400'
              }`}
            >
              vLLM (Recommended)
            </button>
            <button
              type="button"
              onClick={() => onChangeEngine('llamacpp')}
              className={`px-2.5 py-1 rounded text-xs border ${
                selectedEngine === 'llamacpp'
                  ? 'bg-emerald-950 border-emerald-500 text-emerald-200 font-bold'
                  : 'bg-styx-950 border-styx-800 text-slate-400'
              }`}
            >
              llama.cpp
            </button>
            <button
              type="button"
              onClick={() => onChangeEngine('ollama')}
              className={`px-2.5 py-1 rounded text-xs border ${
                selectedEngine === 'ollama'
                  ? 'bg-emerald-950 border-emerald-500 text-emerald-200 font-bold'
                  : 'bg-styx-950 border-styx-800 text-slate-400'
              }`}
            >
              Ollama
            </button>
          </div>
        </div>
      </div>

      {/* Popular Models & Repo */}
      <div className="space-y-1.5 pt-1">
        <label className="text-[11px] text-slate-400">Select Popular Model or Enter Repo:</label>
        <div className="flex flex-wrap items-center gap-1.5">
          {[
            { label: 'Qwen 3.5 9B', repo: 'QuantTrio/Qwen3.5-9B-AWQ', context: 16384, size: 9 },
            { label: 'Llama 3.3 70B', repo: 'meta-llama/Llama-3.3-70B-Instruct', context: 32768, size: 70 },
            { label: 'DeepSeek R1 14B', repo: 'deepseek-ai/DeepSeek-R1-Distill-Qwen-14B', context: 32768, size: 14 },
          ].map(m => (
            <button
              key={m.label}
              type="button"
              onClick={() => onSelectQuickModel(m)}
              className={`px-2 py-0.5 rounded text-xs border ${
                hfRepo === m.repo
                  ? 'bg-cyan-950 border-cyan-500 text-cyan-200 font-bold'
                  : 'bg-styx-950 border-styx-800 text-slate-400 hover:text-slate-200'
              }`}
            >
              {m.label}
            </button>
          ))}
        </div>
        <input
          type="text"
          value={hfRepo}
          onChange={e => onChangeHfRepo(e.target.value)}
          placeholder="cyankiwi/Qwen3.8-27B-AWQ-INT4 or /models/Qwen3.5-9B.gguf"
          className="w-full bg-styx-950 border border-styx-700 rounded p-1.5 text-slate-200 text-xs"
        />
      </div>

      {/* Context Window & Host Port */}
      <div className="grid grid-cols-1 sm:grid-cols-2 gap-3 pt-1">
        <div>
          <label className="text-[11px] text-slate-400">Context Window:</label>
          <div className="flex flex-wrap items-center gap-1 mt-1">
            {[
              { label: '16k (16,384)', value: 16384 },
              { label: '32k (32,768)', value: 32768 },
              { label: '64k (65,536)', value: 65536 },
              { label: '128k (131,072)', value: 131072 },
            ].map(c => (
              <button
                key={c.value}
                type="button"
                onClick={() => onChangeContextWindow(c.value)}
                className={`px-2 py-0.5 rounded text-xs border ${
                  contextWindow === c.value
                    ? 'bg-cyan-950 border-cyan-500 text-cyan-200 font-bold'
                    : 'bg-styx-950 border-styx-800 text-slate-400'
                }`}
              >
                {c.label}
              </button>
            ))}
          </div>
          <input
            type="number"
            value={contextWindow}
            onChange={e => onChangeContextWindow(parseInt(e.target.value) || 4096)}
            className="w-full bg-styx-950 border border-styx-700 rounded p-1.5 text-slate-200 text-xs mt-1"
          />
        </div>
        <div>
          <label className="text-[11px] text-slate-400">Host Port:</label>
          <div className="flex flex-wrap items-center gap-1 mt-1">
            {[8000, 8080, 11434].map(p => (
              <button
                key={p}
                type="button"
                onClick={() => onChangePort(p)}
                className={`px-2.5 py-0.5 rounded text-xs border ${
                  port === p
                    ? 'bg-cyan-950 border-cyan-500 text-cyan-200 font-bold'
                    : 'bg-styx-950 border-styx-800 text-slate-400'
                }`}
              >
                {p}
              </button>
            ))}
            <input
              type="number"
              value={port}
              onChange={e => onChangePort(parseInt(e.target.value) || 8000)}
              className="w-20 bg-styx-950 border border-styx-700 rounded p-1 text-slate-200 text-xs"
            />
          </div>
        </div>
      </div>

      {/* Model Quantization */}
      <div className="space-y-1 pt-1 border-t border-styx-800">
        <div className="flex items-center justify-between">
          <label className="text-[11px] text-slate-400">Model Quantization:</label>
          <span className="text-[10px] text-emerald-400 font-bold">
            Weights VRAM: ~{calculateModelWeightsGb(selectedModelSize, quantization)} GB
          </span>
        </div>
        <div className="flex flex-wrap items-center gap-1.5">
          {[
            { id: 'awq', label: 'AWQ (4-bit)' },
            { id: 'fp8', label: 'FP8 (8-bit)' },
            { id: 'gptq', label: 'GPTQ (4-bit)' },
            { id: 'none', label: 'None (FP16)' },
          ].map(q => (
            <button
              key={q.id}
              type="button"
              onClick={() => onChangeQuantization(q.id)}
              className={`px-2.5 py-1 rounded text-xs border ${
                quantization === q.id
                  ? 'bg-emerald-950 border-emerald-500 text-emerald-200 font-bold'
                  : 'bg-styx-950 border-styx-800 text-slate-400'
              }`}
            >
              {q.label}
            </button>
          ))}
        </div>
      </div>

      {/* KV Cache Quantization */}
      <KvCacheSelector
        selectedEngine={selectedEngine}
        contextWindow={contextWindow}
        kvCacheDtype={kvCacheDtype}
        onChangeKvCacheDtype={onChangeKvCacheDtype}
      />

      {/* MTP & Vision Toggles */}
      <div className="grid grid-cols-1 sm:grid-cols-2 gap-2 pt-1 border-t border-styx-800">
        <label className="flex items-center space-x-2 text-xs text-slate-300 cursor-pointer">
          <input
            type="checkbox"
            aria-label="MTP Speculative Decoding"
            checked={enableMtp}
            onChange={e => onChangeEnableMtp(e.target.checked)}
            className="rounded bg-styx-950 border-styx-700 text-cyan-500 w-4 h-4"
          />
          <span>MTP (Speculative Decoding)</span>
          <span className="text-[10px] text-amber-300 bg-amber-950 px-1 rounded">
            ON (3 tok/step)
          </span>
        </label>
        <label className="flex items-center space-x-2 text-xs text-slate-300 cursor-pointer">
          <input
            type="checkbox"
            aria-label="Image / Multimodal Vision Input"
            checked={enableVision}
            onChange={e => onChangeEnableVision(e.target.checked)}
            className="rounded bg-styx-950 border-styx-700 text-purple-500 w-4 h-4"
          />
          <span>Multimodal Vision Input (Image 🖼️)</span>
          <span className="text-[10px] text-purple-300 bg-purple-950 px-1 rounded">
            Vision Enabled
          </span>
        </label>
      </div>

      {/* GPU Platform Select */}
      <div className="space-y-1 pt-1 border-t border-styx-800">
        <label htmlFor="gpuPlatformSelect" className="text-[11px] text-slate-400">
          GPU Hardware Platform:
        </label>
        <select
          id="gpuPlatformSelect"
          aria-label="GPU Hardware Platform"
          value={gpuVendor}
          onChange={e => onChangeGpuVendor(e.target.value as any)}
          className="w-full bg-styx-950 border border-styx-700 rounded p-1.5 text-slate-200 text-xs"
        >
          <option value="auto">Auto-Detect Platform</option>
          <option value="amd">AMD ROCm (--device=/dev/kfd --device=/dev/dri)</option>
          <option value="nvidia">NVIDIA CUDA (--gpus all)</option>
          <option value="none">CPU Only (No GPU acceleration)</option>
        </select>
      </div>

      {/* Command Preview */}
      <div className="space-y-1 pt-1">
        <div className="text-[10px] text-slate-400 uppercase font-bold flex items-center space-x-1">
          <Code className="w-3 h-3 text-emerald-400" />
          <span>Generated Docker CLI Preview:</span>
        </div>
        <pre className="bg-styx-950 p-2 rounded border border-styx-800 text-[10px] text-emerald-300 overflow-x-auto whitespace-pre-wrap">
          {previewCmd || 'docker run ...'}
        </pre>
      </div>

      {/* Deploy Button */}
      <div className="flex justify-end pt-2">
        <button
          type="button"
          onClick={onDeploy}
          disabled={isDeploying}
          className="px-4 py-2 rounded bg-emerald-600 hover:bg-emerald-500 text-white font-bold text-xs flex items-center space-x-1.5 shadow"
        >
          <Play className="w-3.5 h-3.5 fill-white" />
          <span>{isDeploying ? 'Deploying Container...' : 'Deploy Inference Container'}</span>
        </button>
      </div>
    </div>
  );
};
