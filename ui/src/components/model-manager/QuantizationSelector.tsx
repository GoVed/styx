import React from 'react';
import { Loader2, Sparkles } from 'lucide-react';
import { HfQuantVariant } from './types';
import { calculateModelWeightsGb } from './vram';

export interface QuantizationSelectorProps {
  quantization: string;
  availableQuants: HfQuantVariant[];
  isInspectingHf: boolean;
  isGgufRepo: boolean;
  selectedModelSize: number;
  selectedGgufSizeGb?: number;
  onChangeQuantization: (quant: string, sizeGb?: number) => void;
}

export const QuantizationSelector: React.FC<QuantizationSelectorProps> = ({
  quantization,
  availableQuants,
  isInspectingHf,
  isGgufRepo,
  selectedModelSize,
  selectedGgufSizeGb,
  onChangeQuantization,
}) => {
  const hasDynamicQuants = availableQuants && availableQuants.length > 0;

  const weightsText = selectedGgufSizeGb !== undefined && selectedGgufSizeGb > 0
    ? `${selectedGgufSizeGb.toFixed(1)} GB (Exact GGUF Size)`
    : `${calculateModelWeightsGb(selectedModelSize, quantization)} GB`;

  return (
    <div className="space-y-1.5 pt-1 border-t border-styx-800">
      <div className="flex items-center justify-between">
        <div className="flex items-center space-x-1.5">
          <label className="text-[11px] text-slate-400">Model Quantization:</label>
          {hasDynamicQuants && (
            <span className="text-[9px] px-1.5 py-0.2 rounded bg-cyan-950 border border-cyan-800 text-cyan-300 font-bold flex items-center space-x-1">
              <Sparkles className="w-2.5 h-2.5 text-cyan-400" />
              <span>Hugging Face GGUF ({availableQuants.length} variants)</span>
            </span>
          )}
          {isInspectingHf && (
            <span className="text-[9px] px-1.5 py-0.2 rounded bg-amber-950 border border-amber-800 text-amber-300 flex items-center space-x-1">
              <Loader2 className="w-2.5 h-2.5 animate-spin" />
              <span>Inspecting HF...</span>
            </span>
          )}
        </div>
        <span className="text-[10px] text-emerald-400 font-bold">
          Weights VRAM: ~{weightsText}
        </span>
      </div>

      <div className="flex flex-wrap items-center gap-1.5">
        {hasDynamicQuants ? (
          availableQuants.map(q => {
            const isSelected = quantization.toUpperCase() === q.quant.toUpperCase();
            return (
              <button
                key={q.quant}
                type="button"
                onClick={() => onChangeQuantization(q.quant, q.size_gb)}
                className={`px-2.5 py-1 rounded text-xs border transition-colors ${
                  isSelected
                    ? 'bg-emerald-950 border-emerald-500 text-emerald-200 font-bold shadow-sm'
                    : 'bg-styx-950 border-styx-800 text-slate-400 hover:text-slate-200'
                }`}
              >
                {q.quant} ({q.size_gb.toFixed(1)} GB)
              </button>
            );
          })
        ) : (
          [
            { id: 'awq', label: 'AWQ (4-bit)' },
            { id: 'fp8', label: 'FP8 (8-bit)' },
            { id: 'gptq', label: 'GPTQ (4-bit)' },
            { id: 'none', label: 'None (FP16)' },
          ].map(q => (
            <button
              key={q.id}
              type="button"
              onClick={() => onChangeQuantization(q.id)}
              className={`px-2.5 py-1 rounded text-xs border transition-colors ${
                quantization === q.id
                  ? 'bg-emerald-950 border-emerald-500 text-emerald-200 font-bold shadow-sm'
                  : 'bg-styx-950 border-styx-800 text-slate-400 hover:text-slate-200'
              }`}
            >
              {q.label}
            </button>
          ))
        )}
      </div>
    </div>
  );
};
