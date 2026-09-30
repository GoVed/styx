import React from 'react';
import { CheckCircle, Edit2, Eye, Folder, Save } from 'lucide-react';
import { marked } from 'marked';

export interface MemoryEditorProps {
  selectedPath: string;
  fileContent: string;
  saveStatus: string;
  activeView: 'edit' | 'preview';
  onChangeActiveView: (view: 'edit' | 'preview') => void;
  onChangeContent: (val: string) => void;
  onSave: () => void;
  className?: string;
  onBackToList?: () => void;
}

export const MemoryEditor: React.FC<MemoryEditorProps> = ({
  selectedPath,
  fileContent,
  saveStatus,
  activeView,
  onChangeActiveView,
  onChangeContent,
  onSave,
  className,
  onBackToList,
}) => {
  return (
    <main className={`flex flex-col h-full overflow-hidden bg-styx-950 select-none ${className || 'flex-1'}`}>
      {/* Editor Toolbar */}
      <div className="p-2 sm:p-2.5 px-3 sm:px-4 border-b border-styx-800 bg-styx-900/80 flex items-center justify-between gap-2 font-mono flex-shrink-0">
        <div className="flex items-center space-x-1.5 sm:space-x-2 min-w-0">
          {onBackToList && (
            <button
              type="button"
              onClick={onBackToList}
              className="md:hidden flex items-center space-x-1 px-2 py-1 rounded bg-styx-800 hover:bg-styx-750 text-emerald-400 border border-styx-700 text-[11px] font-semibold flex-shrink-0 active:scale-95 transition-all"
              title="Browse memory files"
            >
              <Folder className="w-3.5 h-3.5" />
              <span>Files</span>
            </button>
          )}
          <span className="hidden sm:inline text-slate-500 text-xs">FILE:</span>
          <span className="font-bold text-slate-200 bg-styx-950 px-2 py-0.5 rounded border border-styx-800 truncate text-[11px] sm:text-xs max-w-[130px] sm:max-w-none">
            {selectedPath}
          </span>
          {saveStatus && (
            <span className="text-emerald-400 text-xs flex items-center space-x-1 animate-pulse flex-shrink-0">
              <CheckCircle className="w-3 h-3" />
              <span className="hidden sm:inline">{saveStatus}</span>
            </span>
          )}
        </div>

        <div className="flex items-center space-x-1.5 sm:space-x-2 flex-shrink-0">
          {/* View Switcher */}
          <div className="flex items-center bg-styx-950 p-0.5 rounded border border-styx-800">
            <button
              type="button"
              onClick={() => onChangeActiveView('edit')}
              className={`px-2 sm:px-2.5 py-1 rounded flex items-center space-x-1 text-xs ${
                activeView === 'edit'
                  ? 'bg-styx-800 text-emerald-300 font-semibold'
                  : 'text-slate-400 hover:text-slate-200'
              }`}
              title="Edit Markdown source"
            >
              <Edit2 className="w-3 h-3" />
              <span className="hidden sm:inline">Edit</span>
            </button>
            <button
              type="button"
              onClick={() => onChangeActiveView('preview')}
              className={`px-2 sm:px-2.5 py-1 rounded flex items-center space-x-1 text-xs ${
                activeView === 'preview'
                  ? 'bg-styx-800 text-cyan-300 font-semibold'
                  : 'text-slate-400 hover:text-slate-200'
              }`}
              title="Rendered Markdown view"
            >
              <Eye className="w-3 h-3" />
              <span className="hidden sm:inline">View</span>
            </button>
          </div>

          {/* Save Button */}
          <button
            type="button"
            onClick={onSave}
            className="px-2.5 sm:px-3 py-1 sm:py-1.5 rounded bg-emerald-600 hover:bg-emerald-500 text-white font-bold flex items-center space-x-1 shadow text-xs transition-colors active:scale-95"
            title="Save file and re-index in Tantivy"
          >
            <Save className="w-3.5 h-3.5" />
            <span>Save</span>
            <span className="hidden sm:inline">& Re-Index</span>
          </button>
        </div>
      </div>

      {/* Content Area */}
      <div className="flex-1 overflow-hidden p-2 sm:p-4 select-text">
        {activeView === 'edit' ? (
          <textarea
            value={fileContent}
            onChange={e => onChangeContent(e.target.value)}
            className="w-full h-full bg-styx-900/90 text-slate-200 p-3 sm:p-4 rounded-lg border border-styx-800 font-mono text-xs sm:text-sm leading-relaxed resize-none focus:outline-none focus:border-emerald-500"
            placeholder="Enter markdown content..."
          />
        ) : (
          <div className="w-full h-full bg-styx-900/90 text-slate-200 p-4 sm:p-6 rounded-lg border border-styx-800 overflow-y-auto">
            <div
              className="prose prose-invert prose-sm max-w-none break-words"
              dangerouslySetInnerHTML={{ __html: marked.parse(fileContent) }}
            />
          </div>
        )}
      </div>
    </main>
  );
};
