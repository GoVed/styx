import React from 'react';
import { CheckCircle, Edit2, Eye, Save } from 'lucide-react';
import { marked } from 'marked';

export interface MemoryEditorProps {
  selectedPath: string;
  fileContent: string;
  saveStatus: string;
  activeView: 'edit' | 'preview';
  onChangeActiveView: (view: 'edit' | 'preview') => void;
  onChangeContent: (val: string) => void;
  onSave: () => void;
}

export const MemoryEditor: React.FC<MemoryEditorProps> = ({
  selectedPath,
  fileContent,
  saveStatus,
  activeView,
  onChangeActiveView,
  onChangeContent,
  onSave,
}) => {
  return (
    <main className="flex-1 flex flex-col h-full overflow-hidden bg-styx-950">
      {/* Editor Toolbar */}
      <div className="p-2.5 px-4 border-b border-styx-800 bg-styx-900/60 flex items-center justify-between font-mono">
        <div className="flex items-center space-x-2">
          <span className="text-slate-500">FILE:</span>
          <span className="font-bold text-slate-200 bg-styx-950 px-2 py-0.5 rounded border border-styx-800">
            {selectedPath}
          </span>
          {saveStatus && (
            <span className="text-emerald-400 text-xs flex items-center space-x-1 animate-pulse">
              <CheckCircle className="w-3 h-3" />
              <span>{saveStatus}</span>
            </span>
          )}
        </div>

        <div className="flex items-center space-x-2">
          {/* View Switcher */}
          <div className="flex items-center bg-styx-950 p-0.5 rounded border border-styx-800">
            <button
              type="button"
              onClick={() => onChangeActiveView('edit')}
              className={`px-2.5 py-1 rounded flex items-center space-x-1 ${
                activeView === 'edit'
                  ? 'bg-styx-800 text-emerald-300 font-semibold'
                  : 'text-slate-400 hover:text-slate-200'
              }`}
            >
              <Edit2 className="w-3 h-3" />
              <span>Edit Source</span>
            </button>
            <button
              type="button"
              onClick={() => onChangeActiveView('preview')}
              className={`px-2.5 py-1 rounded flex items-center space-x-1 ${
                activeView === 'preview'
                  ? 'bg-styx-800 text-cyan-300 font-semibold'
                  : 'text-slate-400 hover:text-slate-200'
              }`}
            >
              <Eye className="w-3 h-3" />
              <span>Rendered View</span>
            </button>
          </div>

          {/* Save Button */}
          <button
            type="button"
            onClick={onSave}
            className="px-3 py-1.5 rounded bg-emerald-600 hover:bg-emerald-500 text-white font-bold flex items-center space-x-1 shadow"
          >
            <Save className="w-3.5 h-3.5" />
            <span>Save & Re-Index</span>
          </button>
        </div>
      </div>

      {/* Content Area */}
      <div className="flex-1 overflow-hidden p-4">
        {activeView === 'edit' ? (
          <textarea
            value={fileContent}
            onChange={e => onChangeContent(e.target.value)}
            className="w-full h-full bg-styx-900/90 text-slate-200 p-4 rounded-lg border border-styx-800 font-mono text-xs leading-relaxed resize-none focus:outline-none focus:border-emerald-500"
            placeholder="Enter markdown content..."
          />
        ) : (
          <div className="w-full h-full bg-styx-900/90 text-slate-200 p-6 rounded-lg border border-styx-800 overflow-y-auto">
            <div
              className="prose prose-invert prose-sm max-w-none"
              dangerouslySetInnerHTML={{ __html: marked.parse(fileContent) }}
            />
          </div>
        )}
      </div>
    </main>
  );
};
