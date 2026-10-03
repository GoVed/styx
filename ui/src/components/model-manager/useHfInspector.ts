import { useState, useEffect } from 'react';
import { HfQuantVariant, HfInspectResponse } from './types';
import { safeFetchJson } from './vram';

export interface UseHfInspectorProps {
  hfRepo: string;
  setHfRepo: (repo: string) => void;
  setQuantization: (quant: string) => void;
  setSelectedEngine: (engine: string) => void;
  setSelectedModelSize?: (size: number) => void;
  setContainerName?: (name: string) => void;
}

export function useHfInspector({
  hfRepo,
  setHfRepo,
  setQuantization,
  setSelectedEngine,
  setSelectedModelSize,
  setContainerName,
}: UseHfInspectorProps) {
  const [availableQuants, setAvailableQuants] = useState<HfQuantVariant[]>([]);
  const [isInspectingHf, setIsInspectingHf] = useState<boolean>(false);
  const [isGgufRepo, setIsGgufRepo] = useState<boolean>(false);
  const [selectedGgufSizeGb, setSelectedGgufSizeGb] = useState<number | undefined>(undefined);
  const [hfError, setHfError] = useState<string | null>(null);

  useEffect(() => {
    const trimmed = hfRepo.trim();
    // Only inspect Hugging Face repo if not a local file and contains at least a slash or HF url
    if (!trimmed || trimmed.startsWith('/') || trimmed.endsWith('.gguf') || !trimmed.includes('/')) {
      setAvailableQuants([]);
      setIsGgufRepo(false);
      setSelectedGgufSizeGb(undefined);
      setHfError(null);
      return;
    }

    const timer = setTimeout(() => {
      inspectHfRepo(trimmed);
    }, 450);

    return () => clearTimeout(timer);
  }, [hfRepo]);

  const inspectHfRepo = async (repoInput: string) => {
    setIsInspectingHf(true);
    setHfError(null);

    try {
      const res = await fetch(`/api/models/hf-inspect?repo=${encodeURIComponent(repoInput)}`);
      const data: HfInspectResponse = await safeFetchJson(res);

      if (data.success) {
        if (data.repo && repoInput.includes('huggingface.co')) {
          setHfRepo(data.repo);
        }

        if (data.is_gguf) {
          setIsGgufRepo(true);
          setSelectedEngine('llamacpp');
          const quants = data.quants || [];
          setAvailableQuants(quants);

          const rec = data.recommended_quant || 'Q4_K_M';
          const match = quants.find(q => q.quant.toUpperCase() === rec.toUpperCase()) || quants[0];
          if (match) {
            setQuantization(match.quant);
            setSelectedGgufSizeGb(match.size_gb);
          }

          if (setSelectedModelSize) {
            const sizeMatch = (data.repo || repoInput).match(/(\d+)b/i);
            if (sizeMatch && sizeMatch[1]) {
              setSelectedModelSize(parseInt(sizeMatch[1], 10));
            }
          }

          if (setContainerName && data.repo) {
            const baseName = data.repo.split('/')[1]?.toLowerCase().replace(/[^a-z0-9]/g, '-');
            if (baseName) {
              setContainerName(baseName);
            }
          }
        } else {
          setIsGgufRepo(false);
          setAvailableQuants([]);
          setSelectedGgufSizeGb(undefined);
          if (data.recommended_engine) {
            const eng = data.recommended_engine === 'llama.cpp' ? 'llamacpp' : data.recommended_engine;
            setSelectedEngine(eng);
          }
          if (data.recommended_quant) {
            setQuantization(data.recommended_quant);
          }
        }
      } else {
        setHfError(data.error || 'Failed to inspect Hugging Face repository');
      }
    } catch (e) {
      setHfError(e instanceof Error ? e.message : 'Error inspecting repository');
    } finally {
      setIsInspectingHf(false);
    }
  };

  const handleSelectQuantization = (quant: string, sizeGb?: number) => {
    setQuantization(quant);
    if (sizeGb !== undefined) {
      setSelectedGgufSizeGb(sizeGb);
    } else {
      const match = availableQuants.find(q => q.quant.toUpperCase() === quant.toUpperCase());
      setSelectedGgufSizeGb(match?.size_gb);
    }
  };

  return {
    availableQuants,
    isInspectingHf,
    isGgufRepo,
    selectedGgufSizeGb,
    setSelectedGgufSizeGb,
    hfError,
    inspectHfRepo,
    handleSelectQuantization,
  };
}
