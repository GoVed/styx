import React from 'react';
import { Shield, Play } from 'lucide-react';
import { McpTool } from '../../types';

export interface PolicyMatrixProps {
  tools: McpTool[];
  onUpdatePolicy: (
    toolName: string,
    policy: 'AUTONOMOUS' | 'REQUIRE_APPROVAL' | 'BLOCKED',
    riskLevel?: string
  ) => void;
  onSelectTestingTool: (tool: McpTool) => void;
}

export const PolicyMatrix: React.FC<PolicyMatrixProps> = ({
  tools,
  onUpdatePolicy,
  onSelectTestingTool,
}) => {
  return (
    <div className="bg-syndae-900 border border-syndae-800 rounded-lg p-3 font-mono space-y-3">
      <div className="flex items-center justify-between">
        <div className="text-xs font-bold text-slate-200 flex items-center space-x-1.5">
          <Shield className="w-3.5 h-3.5 text-amber-400" />
          <span>THREE-TIER POLICY MATRIX & TOOL CATALOG ({tools.length} Tools)</span>
        </div>
        <div className="flex items-center space-x-3 text-[10px] text-slate-400">
          <span className="flex items-center space-x-1">
            <span className="w-2 h-2 rounded-full bg-emerald-500 inline-block" />
            <span>Auto (Autonomous)</span>
          </span>
          <span className="flex items-center space-x-1">
            <span className="w-2 h-2 rounded-full bg-amber-500 inline-block" />
            <span>Ask (Require Approval)</span>
          </span>
          <span className="flex items-center space-x-1">
            <span className="w-2 h-2 rounded-full bg-rose-500 inline-block" />
            <span>Block (Excluded from LLM)</span>
          </span>
        </div>
      </div>

      <div className="overflow-x-auto">
        <table className="w-full text-left text-[11px]">
          <thead>
            <tr className="border-b border-syndae-800 text-slate-400">
              <th className="py-2 px-2">TOOL NAME</th>
              <th className="py-2 px-2">DESCRIPTION</th>
              <th className="py-2 px-2">POLICY TIER</th>
              <th className="py-2 px-2">RISK LEVEL</th>
              <th className="py-2 px-2 text-right">TEST</th>
            </tr>
          </thead>
          <tbody className="divide-y divide-syndae-800/60">
            {tools.map(t => (
              <tr key={t.name} className="hover:bg-syndae-850">
                <td className="py-2 px-2 font-bold text-slate-200">
                  <code>{t.name}</code>
                </td>
                <td className="py-2 px-2 text-slate-400 max-w-md truncate">
                  {t.description}
                </td>
                <td className="py-2 px-2">
                  <div className="flex items-center bg-syndae-950 p-0.5 rounded border border-syndae-800 w-fit">
                    <button
                      type="button"
                      onClick={() => onUpdatePolicy(t.name, 'AUTONOMOUS', t.risk_level)}
                      className={`px-2 py-0.5 rounded text-[10px] font-bold ${
                        t.policy === 'AUTONOMOUS'
                          ? 'bg-emerald-950 text-emerald-400 border border-emerald-800'
                          : 'text-slate-500 hover:text-slate-300'
                      }`}
                    >
                      Auto
                    </button>
                    <button
                      type="button"
                      onClick={() => onUpdatePolicy(t.name, 'REQUIRE_APPROVAL', t.risk_level)}
                      className={`px-2 py-0.5 rounded text-[10px] font-bold ${
                        t.policy === 'REQUIRE_APPROVAL'
                          ? 'bg-amber-950 text-amber-300 border border-amber-800'
                          : 'text-slate-500 hover:text-slate-300'
                      }`}
                    >
                      Ask
                    </button>
                    <button
                      type="button"
                      onClick={() => onUpdatePolicy(t.name, 'BLOCKED', t.risk_level)}
                      className={`px-2 py-0.5 rounded text-[10px] font-bold ${
                        t.policy === 'BLOCKED'
                          ? 'bg-rose-950 text-rose-400 border border-rose-800'
                          : 'text-slate-500 hover:text-slate-300'
                      }`}
                    >
                      Block
                    </button>
                  </div>
                </td>
                <td className="py-2 px-2">
                  <span
                    className={`px-1.5 py-0.5 rounded text-[10px] font-bold ${
                      t.risk_level === 'CRITICAL'
                        ? 'bg-rose-950 text-rose-400 border border-rose-900'
                        : t.risk_level === 'HIGH'
                        ? 'bg-amber-950 text-amber-400 border border-amber-900'
                        : 'bg-emerald-950 text-emerald-400 border border-emerald-900'
                    }`}
                  >
                    {t.risk_level}
                  </span>
                </td>
                <td className="py-2 px-2 text-right">
                  <button
                    type="button"
                    onClick={() => onSelectTestingTool(t)}
                    className="px-2 py-1 rounded bg-syndae-800 hover:bg-syndae-700 text-cyan-300 border border-syndae-700 text-[10px] flex items-center space-x-1 ml-auto"
                  >
                    <Play className="w-3 h-3" />
                    <span>Inspect & Call</span>
                  </button>
                </td>
              </tr>
            ))}
          </tbody>
        </table>
      </div>
    </div>
  );
};
