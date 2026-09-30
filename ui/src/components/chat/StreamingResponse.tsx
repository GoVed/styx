import React, { useState } from 'react';
import { Bot, BrainCircuit, ChevronDown, ChevronRight, Terminal, CheckCircle, XCircle } from 'lucide-react';
import { ToolExecutionCard } from './types';
import { extractThoughtAndContent, extractOptionsAndContent, renderMarkdown } from './helpers';

export interface StreamingResponseProps {
  liveStreamingText: string;
  liveThought: string;
  liveToolExecutions: ToolExecutionCard[];
}

export const StreamingResponse: React.FC<StreamingResponseProps> = ({
  liveStreamingText,
  liveThought,
  liveToolExecutions,
}) => {
  const [thoughtExpanded, setThoughtExpanded] = useState(false);
  const [expandedTools, setExpandedTools] = useState<Record<string, boolean>>({});

  const toggleToolExpand = (id: string) => {
    setExpandedTools(prev => ({ ...prev, [id]: !prev[id] }));
  };

  const liveParsed = extractThoughtAndContent(liveStreamingText, liveThought);
  const activeThought = liveParsed.thought || liveThought;
  const { content: activeStreamingText } = extractOptionsAndContent(liveParsed.content);

  return (
    <div className="flex flex-col items-start space-y-2">
      <div className="flex items-center space-x-1.5 text-[11px] font-mono text-emerald-400">
        <Bot className="w-3 h-3" />
        <span>STYX AGENT (STREAMING)</span>
      </div>

      {/* Collapsible Live Thought Bubble */}
      {activeThought && (
        <div className="w-full max-w-3xl rounded-lg bg-styx-900/60 border border-purple-900/30 text-xs font-mono p-2 shadow-sm transition-all">
          <button
            type="button"
            onClick={() => setThoughtExpanded(!thoughtExpanded)}
            className="flex items-center justify-between w-full text-left text-purple-400/90 text-[11px] font-medium select-none hover:text-purple-300 transition-colors py-0.5 px-1"
          >
            <div className="flex items-center space-x-1.5">
              <BrainCircuit className="w-3.5 h-3.5 text-purple-400 animate-spin" />
              <span>Thinking...</span>
              <span className="text-[10px] text-purple-400/60 font-normal">
                {thoughtExpanded ? '(click to collapse)' : '(click to view details)'}
              </span>
            </div>
            {thoughtExpanded ? <ChevronDown className="w-3.5 h-3.5" /> : <ChevronRight className="w-3.5 h-3.5" />}
          </button>
          {thoughtExpanded && (
            <div className="whitespace-pre-wrap text-slate-300 leading-relaxed max-h-48 overflow-y-auto pt-2 border-t border-purple-900/30 text-[11px] font-mono">
              {activeThought}
            </div>
          )}
        </div>
      )}

      {/* Live Tool Execution Cards */}
      {liveToolExecutions.map(exec => {
        const isExpanded = expandedTools[exec.id] ?? true;
        return (
          <div
            key={exec.id}
            className="w-full max-w-3xl rounded border border-styx-700 bg-styx-900 text-xs font-mono overflow-hidden shadow"
          >
            <div
              onClick={() => toggleToolExpand(exec.id)}
              className="p-2 px-3 bg-styx-850 flex items-center justify-between cursor-pointer border-b border-styx-800"
            >
              <div className="flex items-center space-x-2">
                <Terminal className="w-3.5 h-3.5 text-cyan-400" />
                <span className="font-bold text-slate-200">TOOL:</span>
                <span className="text-cyan-300 font-semibold">{exec.tool_name}</span>
                {exec.status === 'running' && (
                  <span className="text-amber-400 animate-pulse text-[10px]">[EXECUTING...]</span>
                )}
                {exec.status === 'completed' && (
                  <span className="text-emerald-400 text-[10px] flex items-center space-x-1">
                    <CheckCircle className="w-3 h-3" />
                    <span>COMPLETED</span>
                  </span>
                )}
                {exec.status === 'rejected' && (
                  <span className="text-rose-400 text-[10px] flex items-center space-x-1">
                    <XCircle className="w-3 h-3" />
                    <span>REJECTED</span>
                  </span>
                )}
              </div>
              {isExpanded ? (
                <ChevronDown className="w-3.5 h-3.5 text-slate-400" />
              ) : (
                <ChevronRight className="w-3.5 h-3.5 text-slate-400" />
              )}
            </div>

            {isExpanded && (
              <div className="p-3 space-y-2 bg-styx-950/80">
                <div>
                  <div className="text-[10px] uppercase text-slate-500 font-semibold mb-0.5">Parameters:</div>
                  <pre className="bg-styx-900 p-2 rounded text-slate-300 text-[11px] overflow-x-auto border border-styx-800">
                    {JSON.stringify(exec.arguments, null, 2)}
                  </pre>
                </div>
                {exec.output && (
                  <div>
                    <div className="text-[10px] uppercase text-slate-500 font-semibold mb-0.5">Output:</div>
                    <pre className="bg-styx-900 p-2 rounded text-emerald-300 text-[11px] overflow-x-auto max-h-40 border border-styx-800">
                      {exec.output}
                    </pre>
                  </div>
                )}
              </div>
            )}
          </div>
        );
      })}

      {/* Streaming Content */}
      {activeStreamingText && (
        <div className="max-w-3xl rounded-lg p-3 bg-styx-900 text-slate-200 border border-styx-800 text-xs leading-relaxed shadow-sm">
          <div
            className="prose max-w-none break-words"
            dangerouslySetInnerHTML={{ __html: renderMarkdown(activeStreamingText) }}
          />
        </div>
      )}
    </div>
  );
};
