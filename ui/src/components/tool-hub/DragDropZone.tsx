import React from 'react';
import { UploadCloud, FolderDown, MessageSquare, Zap, RefreshCw } from 'lucide-react';

export interface DragDropZoneProps {
  isDragging: boolean;
  inputPath: string;
  isInspecting: boolean;
  fileInputRef: React.RefObject<HTMLInputElement>;
  onDragOver: (e: React.DragEvent) => void;
  onDragLeave: (e: React.DragEvent) => void;
  onDrop: (e: React.DragEvent) => void;
  onFileInputChange: (e: React.ChangeEvent<HTMLInputElement>) => void;
  onInspectPath: (path: string) => void;
  onChangeInputPath: (val: string) => void;
}

export const DragDropZone: React.FC<DragDropZoneProps> = ({
  isDragging,
  inputPath,
  isInspecting,
  fileInputRef,
  onDragOver,
  onDragLeave,
  onDrop,
  onFileInputChange,
  onInspectPath,
  onChangeInputPath,
}) => {
  return (
    <div
      onDragOver={onDragOver}
      onDragLeave={onDragLeave}
      onDrop={onDrop}
      className={`relative rounded-xl border-2 border-dashed transition-all p-5 font-mono ${
        isDragging
          ? 'border-emerald-400 bg-emerald-950/30 scale-[1.01] shadow-2xl shadow-emerald-500/10'
          : 'border-syndae-700 bg-syndae-900/60 hover:border-syndae-600'
      }`}
    >
      {/* Hidden Directory/File input for file picker */}
      <input
        type="file"
        ref={fileInputRef}
        onChange={onFileInputChange}
        // @ts-ignore
        webkitdirectory=""
        directory=""
        multiple
        className="hidden"
      />

      <div className="flex flex-col md:flex-row md:items-center justify-between gap-4">
        <div className="space-y-1.5">
          <div className="flex items-center space-x-2">
            <UploadCloud className="w-5 h-5 text-emerald-400" />
            <h3 className="text-sm font-bold text-slate-100">
              DRAG & DROP TOOL PACKAGE OR DIRECTORY
            </h3>
          </div>
          <p className="text-xs text-slate-400 max-w-2xl leading-relaxed">
            Drop any local tool repository folder, <code className="text-cyan-300">package.json</code>, or tool manifest here to auto-inspect and register its MCP tools.
          </p>
        </div>

        {/* Quick-action and manual inspection controls */}
        <div className="flex flex-wrap items-center gap-2">
          <button
            type="button"
            onClick={() => fileInputRef.current?.click()}
            className="px-3 py-1.5 rounded-lg bg-syndae-800 hover:bg-syndae-700 text-slate-200 border border-syndae-600 text-xs font-semibold flex items-center space-x-1.5 transition-colors"
          >
            <FolderDown className="w-3.5 h-3.5 text-cyan-400" />
            <span>Browse Tool Folder</span>
          </button>
        </div>
      </div>

      {/* Manual Path Inspection */}
      <div className="mt-4 pt-3 border-t border-syndae-800/80 flex flex-wrap items-center gap-2">
        <span className="text-[11px] text-slate-500 font-bold uppercase">
          Inspect Tool Directory:
        </span>

        {/* Path text input fallback */}
        <div className="flex items-center space-x-1.5 ml-auto w-full md:w-auto">
          <input
            type="text"
            value={inputPath}
            onChange={e => onChangeInputPath(e.target.value)}
            placeholder="/path/to/tool"
            className="bg-syndae-950 border border-syndae-700 rounded-lg px-2.5 py-1 text-xs text-slate-200 font-mono focus:outline-none focus:border-emerald-500 w-full md:w-64"
          />
          <button
            type="button"
            onClick={() => onInspectPath(inputPath)}
            disabled={isInspecting || !inputPath.trim()}
            className="px-3 py-1 rounded-lg bg-emerald-600 hover:bg-emerald-500 text-white font-bold text-xs flex items-center space-x-1 shrink-0 disabled:opacity-50"
          >
            {isInspecting ? (
              <RefreshCw className="w-3 h-3 animate-spin" />
            ) : (
              <Zap className="w-3 h-3" />
            )}
            <span>Inspect</span>
          </button>
        </div>
      </div>
    </div>
  );
};
