import React, { useState } from 'react';
import { Languages, CheckCircle2, RefreshCw, Send } from 'lucide-react';

interface TranslationModelSectionProps {
  currentModelId?: string;
  onSaved?: () => void;
}

export const TranslationModelSection: React.FC<TranslationModelSectionProps> = ({
  currentModelId = 'sarvam-1 (Indic Specialist)',
  onSaved,
}) => {
  const [transProvider, setTransProvider] = useState('docker_ollama');
  const [transModelId, setTransModelId] = useState('sarvam-1');
  const [transBaseUrl, setTransBaseUrl] = useState('http://localhost:11434/v1');
  const [transApiKey, setTransApiKey] = useState('');
  const [savingTrans, setSavingTrans] = useState(false);
  const [transStatusMsg, setTransStatusMsg] = useState('');

  const [testText, setTestText] = useState('How are you doing today? Let us meet in the evening.');
  const [testTargetLang, setTestTargetLang] = useState('gujlish');
  const [testingTrans, setTestingTrans] = useState(false);
  const [transResult, setTransResult] = useState<{
    translation?: string;
    from?: string;
    to?: string;
    method?: string;
    error?: string;
  } | null>(null);

  const handleSaveTranslation = async () => {
    setSavingTrans(true);
    setTransStatusMsg('');
    try {
      const res = await fetch('/api/models/roles/configure', {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({
          role: 'translation',
          name: `Translation Sidecar (${transModelId})`,
          provider: transProvider,
          base_url: transBaseUrl,
          api_key: transApiKey || null,
          model_id: transModelId,
        }),
      });
      if (res.ok) {
        setTransStatusMsg('Translation model saved and active!');
        onSaved?.();
      } else {
        setTransStatusMsg('Failed to configure translation model.');
      }
    } catch {
      setTransStatusMsg('Connection error.');
    } finally {
      setSavingTrans(false);
    }
  };

  const handleTestTranslation = async () => {
    if (!testText.trim()) return;
    setTestingTrans(true);
    setTransResult(null);
    try {
      const res = await fetch('/api/tools/call', {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({
          tool_name: 'translate',
          arguments: {
            text: testText.trim(),
            target_lang: testTargetLang,
          },
        }),
      });
      const data = await res.json();
      if (data.success && !data.is_error) {
        let parsed: any = null;
        try {
          parsed = JSON.parse(data.content);
        } catch {
          parsed = { translated: data.content };
        }
        setTransResult({
          translation: parsed.translated || data.content,
          from: parsed.source_lang || 'auto',
          to: parsed.target_lang || testTargetLang,
          method: parsed.provider || 'mcp',
        });
      } else {
        setTransResult({ error: data.error || data.content || 'Translation failed' });
      }
    } catch (e: any) {
      setTransResult({ error: e.message || 'Network error' });
    } finally {
      setTestingTrans(false);
    }
  };

  return (
    <div className="bg-styx-900 border border-styx-800 rounded-lg p-5 flex flex-col justify-between space-y-4">
      <div className="space-y-4">
        <div className="flex items-center justify-between pb-3 border-b border-styx-800">
          <div className="flex items-center space-x-2">
            <Languages className="w-4 h-4 text-emerald-400" />
            <h4 className="text-sm font-bold text-slate-100 font-mono">TRANSLATION MODEL (translate)</h4>
          </div>
          <span className="px-2 py-0.5 rounded text-[10px] font-mono font-bold bg-emerald-950 text-emerald-300 border border-emerald-800">
            {currentModelId}
          </span>
        </div>

        <div>
          <label className="text-[11px] font-mono text-slate-400 block mb-1.5">GLOBAL & SPECIALIST PRESETS</label>
          <div className="grid grid-cols-2 gap-2">
            <button
              type="button"
              onClick={() => {
                setTransProvider('docker_ollama');
                setTransModelId('sarvam-1');
                setTransBaseUrl('http://localhost:11434/v1');
              }}
              className={`px-2.5 py-1.5 text-left rounded border text-xs font-mono ${
                transModelId === 'sarvam-1' || transModelId.includes('sarvam')
                  ? 'bg-emerald-950/60 border-emerald-600 text-emerald-200'
                  : 'bg-styx-950 border-styx-800 text-slate-400 hover:border-styx-700'
              }`}
            >
              <div className="font-bold">Sarvam 1 (2B)</div>
              <div className="text-[10px] text-slate-500">Indic-First (Gujarati, Hindi, Tamil)</div>
            </button>
            <button
              type="button"
              onClick={() => {
                setTransProvider('docker_ollama');
                setTransModelId('qwen2.5:3b');
                setTransBaseUrl('http://localhost:11434/v1');
              }}
              className={`px-2.5 py-1.5 text-left rounded border text-xs font-mono ${
                transModelId === 'qwen2.5:3b'
                  ? 'bg-emerald-950/60 border-emerald-600 text-emerald-200'
                  : 'bg-styx-950 border-styx-800 text-slate-400 hover:border-styx-700'
              }`}
            >
              <div className="font-bold">Qwen 2.5 3B</div>
              <div className="text-[10px] text-slate-500">Universal Multilingual (50+ Langs)</div>
            </button>
            <button
              type="button"
              onClick={() => {
                setTransProvider('docker_ollama');
                setTransModelId('llama3.2:3b');
                setTransBaseUrl('http://localhost:11434/v1');
              }}
              className={`px-2.5 py-1.5 text-left rounded border text-xs font-mono ${
                transModelId === 'llama3.2:3b'
                  ? 'bg-emerald-950/60 border-emerald-600 text-emerald-200'
                  : 'bg-styx-950 border-styx-800 text-slate-400 hover:border-styx-700'
              }`}
            >
              <div className="font-bold">Llama 3.2 3B</div>
              <div className="text-[10px] text-slate-500">Fast Global Multilingual</div>
            </button>
            <button
              type="button"
              onClick={() => {
                setTransProvider('openai');
                setTransModelId('sarvam-translate');
                setTransBaseUrl('https://api.sarvam.ai/v1');
              }}
              className={`px-2.5 py-1.5 text-left rounded border text-xs font-mono ${
                transProvider === 'openai'
                  ? 'bg-emerald-950/60 border-emerald-600 text-emerald-200'
                  : 'bg-styx-950 border-styx-800 text-slate-400 hover:border-styx-700'
              }`}
            >
              <div className="font-bold">Cloud API</div>
              <div className="text-[10px] text-slate-500">Sarvam / OpenAI / Groq</div>
            </button>
          </div>
        </div>

        <div className="space-y-2.5 text-xs font-mono">
          <div className="grid grid-cols-2 gap-2">
            <div>
              <label className="text-slate-400 text-[10px] block mb-1">PROVIDER</label>
              <select
                value={transProvider}
                onChange={(e) => setTransProvider(e.target.value)}
                className="w-full bg-styx-950 border border-styx-700 rounded px-2.5 py-1.5 text-slate-200"
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
                value={transModelId}
                onChange={(e) => setTransModelId(e.target.value)}
                placeholder="e.g. gemma2:2b"
                className="w-full bg-styx-950 border border-styx-700 rounded px-2.5 py-1.5 text-slate-200"
              />
            </div>
          </div>
          <div>
            <label className="text-slate-400 text-[10px] block mb-1">BASE URL</label>
            <input
              type="text"
              value={transBaseUrl}
              onChange={(e) => setTransBaseUrl(e.target.value)}
              placeholder="http://localhost:11434/v1"
              className="w-full bg-styx-950 border border-styx-700 rounded px-2.5 py-1.5 text-slate-200 font-mono text-[11px]"
            />
          </div>
          <div className="flex items-center justify-between pt-1">
            <button
              type="button"
              onClick={handleSaveTranslation}
              disabled={savingTrans}
              className="px-3 py-1.5 bg-emerald-600 hover:bg-emerald-500 text-white rounded font-bold flex items-center space-x-1.5"
            >
              <CheckCircle2 className="w-3.5 h-3.5" />
              <span>{savingTrans ? 'Saving...' : 'Set Active Translation Model'}</span>
            </button>
            {transStatusMsg && <span className="text-[11px] text-emerald-300 font-mono">{transStatusMsg}</span>}
          </div>
        </div>

        <div className="pt-3 border-t border-styx-800 space-y-2">
          <div className="flex items-center justify-between text-[10px] font-mono text-slate-400">
            <span className="flex items-center space-x-1">
              <Languages className="w-3 h-3 text-emerald-400" />
              <span>TEST TRANSLATION SANDBOX</span>
            </span>
            <div className="flex items-center space-x-1">
              <span>Target:</span>
              <select
                value={testTargetLang}
                onChange={(e) => setTestTargetLang(e.target.value)}
                className="bg-styx-950 border border-styx-700 rounded px-1.5 py-0.5 text-slate-200 text-[10px]"
              >
                <option value="gujlish">Gujlish (Chat)</option>
                <option value="gujarati">Gujarati Script</option>
                <option value="spanish">Spanish</option>
                <option value="french">French</option>
                <option value="german">German</option>
                <option value="hindi">Hindi</option>
                <option value="hinglish">Hinglish</option>
                <option value="japanese">Japanese</option>
                <option value="english">English</option>
              </select>
            </div>
          </div>
          <div className="flex gap-2">
            <input
              type="text"
              value={testText}
              onChange={(e) => setTestText(e.target.value)}
              placeholder="Enter text in any language..."
              className="flex-1 bg-styx-950 border border-styx-800 rounded px-2.5 py-1 text-xs text-slate-200 font-mono"
            />
            <button
              type="button"
              onClick={handleTestTranslation}
              disabled={testingTrans || !testText.trim()}
              className="px-3 py-1 bg-styx-800 hover:bg-styx-700 border border-styx-700 text-emerald-300 rounded text-xs font-mono font-bold flex items-center space-x-1"
            >
              {testingTrans ? <RefreshCw className="w-3 h-3 animate-spin" /> : <Send className="w-3 h-3" />}
              <span>Translate</span>
            </button>
          </div>
          {transResult && (
            <div className="p-2.5 rounded bg-styx-950 border border-styx-800 text-xs font-mono">
              {transResult.error ? (
                <div className="text-rose-400">{transResult.error}</div>
              ) : (
                <div>
                  <div className="text-emerald-400 mb-1 flex items-center justify-between text-[10px]">
                    <span>{transResult.from} &rarr; {transResult.to}</span>
                    <span className="uppercase text-[9px] px-1.5 py-0.5 rounded bg-emerald-950 border border-emerald-800">
                      {transResult.method}
                    </span>
                  </div>
                  <div className="text-slate-100 font-bold text-sm">{transResult.translation}</div>
                </div>
              )}
            </div>
          )}
        </div>
      </div>
    </div>
  );
};
