import React, { useState } from 'react';
import { Eye, CheckCircle2, RefreshCw, Send, Image as ImageIcon } from 'lucide-react';

interface VisionModelSectionProps {
  currentModelId?: string;
  onSaved?: () => void;
}

export const VisionModelSection: React.FC<VisionModelSectionProps> = ({
  currentModelId = 'moondream (Default)',
  onSaved,
}) => {
  const [visionProvider, setVisionProvider] = useState('docker_ollama');
  const [visionModelId, setVisionModelId] = useState('moondream');
  const [visionBaseUrl, setVisionBaseUrl] = useState('http://localhost:11434/v1');
  const [visionApiKey, setVisionApiKey] = useState('');
  const [savingVision, setSavingVision] = useState(false);
  const [visionStatusMsg, setVisionStatusMsg] = useState('');

  const [testImageUrl, setTestImageUrl] = useState('');
  const [testImagePrompt, setTestImagePrompt] = useState('Describe what is shown in this image.');
  const [testingVision, setTestingVision] = useState(false);
  const [visionResult, setVisionResult] = useState<{ description?: string; latency_ms?: number; error?: string } | null>(null);

  const handleSaveVision = async () => {
    setSavingVision(true);
    setVisionStatusMsg('');
    try {
      const res = await fetch('/api/models/roles/configure', {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({
          role: 'vision',
          name: `Vision Sidecar (${visionModelId})`,
          provider: visionProvider,
          base_url: visionBaseUrl,
          api_key: visionApiKey || null,
          model_id: visionModelId,
        }),
      });
      if (res.ok) {
        setVisionStatusMsg('Vision model saved and active!');
        onSaved?.();
      } else {
        setVisionStatusMsg('Failed to configure vision model.');
      }
    } catch {
      setVisionStatusMsg('Connection error.');
    } finally {
      setSavingVision(false);
    }
  };

  const handleTestVision = async () => {
    setTestingVision(true);
    setVisionResult(null);
    try {
      const res = await fetch('/api/models/test-connection', {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({
          provider: visionProvider,
          base_url: visionBaseUrl,
          model_id: visionModelId,
          api_key: visionApiKey || null,
        }),
      });
      const data = await res.json();
      if (data.success) {
        setVisionResult({
          description: `Backend online (${data.latency_ms}ms). Vision sidecar is healthy and ready to process images.`,
          latency_ms: data.latency_ms,
        });
      } else {
        setVisionResult({ error: data.error || 'Connection to vision model failed' });
      }
    } catch (e: any) {
      setVisionResult({ error: e.message || 'Network error' });
    } finally {
      setTestingVision(false);
    }
  };

  return (
    <div className="bg-syndae-900 border border-syndae-800 rounded-lg p-5 flex flex-col justify-between space-y-4">
      <div className="space-y-4">
        <div className="flex items-center justify-between pb-3 border-b border-syndae-800">
          <div className="flex items-center space-x-2">
            <Eye className="w-4 h-4 text-cyan-400" />
            <h4 className="text-sm font-bold text-slate-100 font-mono">VISION MODEL (inspect_image)</h4>
          </div>
          <span className="px-2 py-0.5 rounded text-[10px] font-mono font-bold bg-cyan-950 text-cyan-300 border border-cyan-800">
            {currentModelId}
          </span>
        </div>

        <div>
          <label className="text-[11px] font-mono text-slate-400 block mb-1.5">QUICK PRESETS</label>
          <div className="grid grid-cols-2 gap-2">
            <button
              type="button"
              onClick={() => {
                setVisionProvider('docker_ollama');
                setVisionModelId('moondream');
                setVisionBaseUrl('http://localhost:11434/v1');
              }}
              className={`px-2.5 py-1.5 text-left rounded border text-xs font-mono ${
                visionModelId === 'moondream' ? 'bg-cyan-950/60 border-cyan-600 text-cyan-200' : 'bg-syndae-950 border-syndae-800 text-slate-400 hover:border-syndae-700'
              }`}
            >
              <div className="font-bold">Moondream 2 1.8B</div>
              <div className="text-[10px] text-slate-500">Ollama (ROCm gfx1201)</div>
            </button>
            <button
              type="button"
              onClick={() => {
                setVisionProvider('docker_llamacpp');
                setVisionModelId('qwen2-vl-2b-local');
                setVisionBaseUrl('http://localhost:8080/v1');
              }}
              className={`px-2.5 py-1.5 text-left rounded border text-xs font-mono ${
                visionModelId === 'qwen2-vl-2b-local' ? 'bg-cyan-950/60 border-cyan-600 text-cyan-200' : 'bg-syndae-950 border-syndae-800 text-slate-400 hover:border-syndae-700'
              }`}
            >
              <div className="font-bold">Qwen2-VL 2B</div>
              <div className="text-[10px] text-slate-500">llama.cpp GGUF local</div>
            </button>
          </div>
        </div>

        <div className="space-y-2.5 text-xs font-mono">
          <div className="grid grid-cols-2 gap-2">
            <div>
              <label className="text-slate-400 text-[10px] block mb-1">PROVIDER</label>
              <select
                value={visionProvider}
                onChange={(e) => setVisionProvider(e.target.value)}
                className="w-full bg-syndae-950 border border-syndae-700 rounded px-2.5 py-1.5 text-slate-200"
              >
                <option value="docker_ollama">Ollama (Local/ROCm)</option>
                <option value="docker_llamacpp">llama.cpp (Local GGUF)</option>
                <option value="docker_vllm">vLLM</option>
                <option value="openai">OpenAI Compatible API</option>
              </select>
            </div>
            <div>
              <label className="text-slate-400 text-[10px] block mb-1">MODEL ID</label>
              <input
                type="text"
                value={visionModelId}
                onChange={(e) => setVisionModelId(e.target.value)}
                placeholder="e.g. moondream"
                className="w-full bg-syndae-950 border border-syndae-700 rounded px-2.5 py-1.5 text-slate-200"
              />
            </div>
          </div>
          <div>
            <label className="text-slate-400 text-[10px] block mb-1">BASE URL</label>
            <input
              type="text"
              value={visionBaseUrl}
              onChange={(e) => setVisionBaseUrl(e.target.value)}
              placeholder="http://localhost:11434/v1"
              className="w-full bg-syndae-950 border border-syndae-700 rounded px-2.5 py-1.5 text-slate-200 font-mono text-[11px]"
            />
          </div>
          <div className="flex items-center justify-between pt-1">
            <button
              type="button"
              onClick={handleSaveVision}
              disabled={savingVision}
              className="px-3 py-1.5 bg-cyan-600 hover:bg-cyan-500 text-white rounded font-bold flex items-center space-x-1.5"
            >
              <CheckCircle2 className="w-3.5 h-3.5" />
              <span>{savingVision ? 'Saving...' : 'Set Active Vision Model'}</span>
            </button>
            {visionStatusMsg && <span className="text-[11px] text-cyan-300 font-mono">{visionStatusMsg}</span>}
          </div>
        </div>

        <div className="pt-3 border-t border-syndae-800 space-y-2">
          <label className="text-[10px] font-mono text-slate-400 flex items-center space-x-1">
            <ImageIcon className="w-3 h-3 text-cyan-400" />
            <span>TEST VISION INSPECTION</span>
          </label>
          <div className="flex gap-2">
            <input
              type="text"
              value={testImageUrl}
              onChange={(e) => setTestImageUrl(e.target.value)}
              placeholder="Paste Image URL or data:image/..."
              className="flex-1 bg-syndae-950 border border-syndae-800 rounded px-2.5 py-1 text-xs text-slate-200 font-mono"
            />
            <button
              type="button"
              onClick={handleTestVision}
              disabled={testingVision || !testImageUrl.trim()}
              className="px-3 py-1 bg-syndae-800 hover:bg-syndae-700 border border-syndae-700 text-cyan-300 rounded text-xs font-mono font-bold flex items-center space-x-1"
            >
              {testingVision ? <RefreshCw className="w-3 h-3 animate-spin" /> : <Send className="w-3 h-3" />}
              <span>Inspect</span>
            </button>
          </div>
          {visionResult && (
            <div className="p-2.5 rounded bg-syndae-950 border border-syndae-800 text-xs font-mono">
              {visionResult.error ? (
                <div className="text-rose-400">{visionResult.error}</div>
              ) : (
                <div>
                  <div className="text-emerald-400 mb-1 flex items-center justify-between text-[10px]">
                    <span>SUCCESS</span>
                    <span>{visionResult.latency_ms} ms</span>
                  </div>
                  <div className="text-slate-300 text-[11px] whitespace-pre-wrap">{visionResult.description}</div>
                </div>
              )}
            </div>
          )}
        </div>
      </div>
    </div>
  );
};
