import React from 'react';
import { Terminal, Play } from 'lucide-react';
import { McpTool } from '../../types';

export interface TestToolDrawerProps {
  testingTool: McpTool;
  testArgs: string;
  testOutput: any | null;
  isCalling: boolean;
  onChangeArgs: (val: string) => void;
  onRunTest: () => void;
  onClose: () => void;
}

export const TestToolDrawer: React.FC<TestToolDrawerProps> = ({
  testingTool,
  testArgs,
  testOutput,
  isCalling,
  onChangeArgs,
  onRunTest,
  onClose,
}) => {
  return (
    <div className="bg-styx-900 border border-cyan-800/80 rounded-lg p-4 font-mono space-y-3">
      <div className="flex items-center justify-between border-b border-styx-800 pb-2">
        <div className="flex items-center space-x-2 text-cyan-300 font-bold">
          <Terminal className="w-4 h-4" />
          <span>TEST EXECUTION: {testingTool.name}</span>
        </div>
        <button
          type="button"
          onClick={onClose}
          className="text-slate-400 hover:text-slate-200"
        >
          Close
        </button>
      </div>

      <div>
        <label className="text-[11px] text-slate-400">Arguments Payload (JSON):</label>
        <textarea
          rows={4}
          value={testArgs}
          onChange={e => onChangeArgs(e.target.value)}
          className="w-full bg-styx-950 border border-styx-700 rounded p-2 text-slate-200 mt-1 font-mono text-xs focus:outline-none focus:border-cyan-500"
        />
      </div>

      {testOutput && (
        <div>
          <label className="text-[11px] text-slate-400">Execution Result:</label>
          <pre className="bg-styx-950 p-2.5 rounded border border-styx-800 text-[11px] text-emerald-300 overflow-x-auto max-h-48 mt-1">
            {JSON.stringify(testOutput, null, 2)}
          </pre>
        </div>
      )}

      <div className="flex justify-end pt-1">
        <button
          type="button"
          onClick={onRunTest}
          disabled={isCalling}
          className="px-4 py-1.5 rounded bg-cyan-600 hover:bg-cyan-500 text-white font-bold flex items-center space-x-1.5 shadow"
        >
          <Play className="w-3.5 h-3.5" />
          <span>{isCalling ? 'Executing...' : 'Invoke Tool Method'}</span>
        </button>
      </div>
    </div>
  );
};
