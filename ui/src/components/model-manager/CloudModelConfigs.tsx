import React from 'react';
import { Layers, CheckCircle, XCircle } from 'lucide-react';

export interface CloudModelConfigsProps {
  extName: string;
  extProvider: string;
  extBaseUrl: string;
  extApiKey: string;
  extModelId: string;
  extContextLength: number;
  testResult: { success: boolean; message: string } | null;
  isTesting: boolean;
  onChangeName: (val: string) => void;
  onChangeProvider: (val: string) => void;
  onChangeBaseUrl: (val: string) => void;
  onChangeApiKey: (val: string) => void;
  onChangeModelId: (val: string) => void;
  onChangeContextLength: (val: number) => void;
  onTestConnection: () => void;
  onSaveExternalConfig: () => void;
}

export const CloudModelConfigs: React.FC<CloudModelConfigsProps> = ({
  extName,
  extProvider,
  extBaseUrl,
  extApiKey,
  extModelId,
  extContextLength,
  testResult,
  isTesting,
  onChangeName,
  onChangeProvider,
  onChangeBaseUrl,
  onChangeApiKey,
  onChangeModelId,
  onChangeContextLength,
  onTestConnection,
  onSaveExternalConfig,
}) => {
  return (
    <div className="max-w-2xl bg-styx-900 border border-styx-800 rounded-lg p-4 font-mono space-y-3">
      <div className="text-xs font-bold text-slate-200 flex items-center space-x-1.5">
        <Layers className="w-3.5 h-3.5 text-cyan-400" />
        <span>CONNECT EXTERNAL OR CLOUD MODEL PROVIDER</span>
      </div>

      <div className="grid grid-cols-2 gap-3">
        <div>
          <label className="text-[11px] text-slate-400">Configuration Name:</label>
          <input
            type="text"
            value={extName}
            onChange={e => onChangeName(e.target.value)}
            className="w-full bg-styx-950 border border-styx-700 rounded p-1.5 text-slate-200 mt-1"
          />
        </div>
        <div>
          <label className="text-[11px] text-slate-400">Provider Family:</label>
          <select
            value={extProvider}
            onChange={e => onChangeProvider(e.target.value)}
            className="w-full bg-styx-950 border border-styx-700 rounded p-1.5 text-slate-200 mt-1"
          >
            <option value="anthropic">Anthropic (Claude)</option>
            <option value="gemini">Google Gemini</option>
            <option value="openai">OpenAI / Compatible Endpoint</option>
            <option value="docker_vllm">vLLM OpenAI Server</option>
            <option value="docker_ollama">Ollama Server</option>
          </select>
        </div>
      </div>

      <div>
        <label className="text-[11px] text-slate-400">Base URL (optional for cloud APIs):</label>
        <input
          type="text"
          value={extBaseUrl}
          onChange={e => onChangeBaseUrl(e.target.value)}
          placeholder="e.g. http://localhost:8000/v1 or https://api.anthropic.com/v1"
          className="w-full bg-styx-950 border border-styx-700 rounded p-1.5 text-slate-200 mt-1"
        />
      </div>

      <div>
        <label className="text-[11px] text-slate-400">API Key / Secret Token:</label>
        <input
          type="password"
          value={extApiKey}
          onChange={e => onChangeApiKey(e.target.value)}
          placeholder="sk-ant-... or sk-..."
          className="w-full bg-styx-950 border border-styx-700 rounded p-1.5 text-slate-200 mt-1"
        />
      </div>

      <div className="grid grid-cols-2 gap-3">
        <div>
          <label className="text-[11px] text-slate-400">Target Model ID:</label>
          <input
            type="text"
            value={extModelId}
            onChange={e => onChangeModelId(e.target.value)}
            placeholder="claude-3-7-sonnet-20250219"
            className="w-full bg-styx-950 border border-styx-700 rounded p-1.5 text-slate-200 mt-1"
          />
        </div>
        <div>
          <label className="text-[11px] text-slate-400">Context Length (Tokens):</label>
          <input
            type="number"
            value={extContextLength}
            onChange={e => onChangeContextLength(parseInt(e.target.value) || 4096)}
            className="w-full bg-styx-950 border border-styx-700 rounded p-1.5 text-slate-200 mt-1"
          />
        </div>
      </div>

      {testResult && (
        <div
          className={`p-3 rounded border text-[11px] space-y-1 ${
            testResult.success
              ? 'bg-emerald-950/60 border-emerald-800 text-emerald-200'
              : 'bg-rose-950/60 border-rose-800 text-rose-200'
          }`}
        >
          <div className="flex items-center space-x-1.5 font-bold">
            {testResult.success ? (
              <CheckCircle className="w-3.5 h-3.5 text-emerald-400" />
            ) : (
              <XCircle className="w-3.5 h-3.5 text-rose-400" />
            )}
            <span>{testResult.success ? 'Handshake Succeeded' : 'Handshake Failed'}</span>
          </div>
          <div>{testResult.message}</div>
        </div>
      )}

      <div className="flex items-center justify-between pt-2">
        <button
          type="button"
          onClick={onTestConnection}
          disabled={isTesting}
          className="px-3 py-1.5 rounded bg-styx-800 hover:bg-styx-700 text-cyan-300 border border-styx-600 font-semibold"
        >
          {isTesting ? 'Testing Handshake...' : 'Test Connection'}
        </button>
        <button
          type="button"
          onClick={onSaveExternalConfig}
          className="px-4 py-1.5 rounded bg-emerald-600 hover:bg-emerald-500 text-white font-bold"
        >
          Register Model Provider
        </button>
      </div>
    </div>
  );
};
