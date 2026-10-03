import React from 'react';
import {
  Layers,
  Plus,
  Shield,
  FileText,
  FileCode,
  Folder,
  Trash2,
  Search,
  BookOpen,
  Users,
  MessageSquare,
  ChevronRight,
} from 'lucide-react';
import { MemoryFileNode } from '../../types';

export interface FileTreeSidebarProps {
  files: MemoryFileNode[];
  selectedPath: string;
  searchQuery: string;
  searchResults: any[];
  onSelectPath: (path: string) => void;
  onOpenNewModal: () => void;
  onDeleteFile: (path: string) => void;
  onChangeSearchQuery: (val: string) => void;
  onSearch: (e: React.FormEvent) => void;
  className?: string;
  onViewEditor?: () => void;
}

const getCategoryMeta = (cat: string) => {
  switch (cat) {
    case 'core':
      return { icon: Shield, color: 'text-purple-400', activeBorder: 'border-purple-500/30 text-purple-300' };
    case 'dictionary':
      return { icon: BookOpen, color: 'text-emerald-400', activeBorder: 'border-emerald-500/30 text-emerald-300' };
    case 'people':
      return { icon: Users, color: 'text-pink-400', activeBorder: 'border-pink-500/30 text-pink-300' };
    case 'groups':
      return { icon: MessageSquare, color: 'text-blue-400', activeBorder: 'border-blue-500/30 text-blue-300' };
    case 'skills':
      return { icon: FileCode, color: 'text-cyan-400', activeBorder: 'border-cyan-500/30 text-cyan-300' };
    case 'scratchpad':
      return { icon: Folder, color: 'text-amber-400', activeBorder: 'border-amber-500/30 text-amber-300' };
    default:
      return { icon: Folder, color: 'text-slate-400', activeBorder: 'border-slate-500/30 text-slate-300' };
  }
};

export const FileTreeSidebar: React.FC<FileTreeSidebarProps> = ({
  files,
  selectedPath,
  searchQuery,
  searchResults,
  onSelectPath,
  onOpenNewModal,
  onDeleteFile,
  onChangeSearchQuery,
  onSearch,
  className,
  onViewEditor,
}) => {
  const categories = Array.from(new Set(files.map(f => f.category)));
  const priority = ['core', 'dictionary', 'people', 'groups', 'skills', 'scratchpad'];
  categories.sort((a, b) => {
    const idxA = priority.indexOf(a);
    const idxB = priority.indexOf(b);
    if (idxA !== -1 && idxB !== -1) return idxA - idxB;
    if (idxA !== -1) return -1;
    if (idxB !== -1) return 1;
    return a.localeCompare(b);
  });

  return (
    <aside className={`border-r border-syndae-800 bg-syndae-900 flex flex-col flex-shrink-0 font-mono h-full overflow-hidden select-none ${className || 'w-full md:w-72'}`}>
      <div className="p-3 border-b border-syndae-800 flex items-center justify-between flex-shrink-0">
        <div className="flex items-center space-x-2">
          <Layers className="w-4 h-4 text-emerald-400" />
          <span className="font-bold text-slate-200">MEMORY HUB</span>
        </div>
        <div className="flex items-center space-x-2">
          {onViewEditor && selectedPath && (
            <button
              type="button"
              onClick={onViewEditor}
              className="md:hidden px-2.5 py-1 rounded bg-syndae-800 hover:bg-syndae-750 text-cyan-400 border border-syndae-700 text-xs font-semibold flex items-center space-x-1 active:scale-95 transition-all"
              title="Switch to file editor"
            >
              <span>Editor</span>
              <ChevronRight className="w-3.5 h-3.5" />
            </button>
          )}
          <button
            type="button"
            onClick={onOpenNewModal}
            className="px-2.5 py-1 rounded bg-syndae-800 hover:bg-syndae-700 text-emerald-400 border border-syndae-700 text-xs flex items-center space-x-1 font-semibold active:scale-95 transition-all"
          >
            <Plus className="w-3.5 h-3.5" />
            <span>New File</span>
          </button>
        </div>
      </div>

      {/* Dynamic Category Tree List */}
      <div className="flex-1 overflow-y-auto p-3 space-y-4">
        {categories.map(cat => {
          const catFiles = files.filter(f => f.category === cat);
          const meta = getCategoryMeta(cat);
          const IconComp = meta.icon;
          return (
            <div key={cat}>
              <div className="flex items-center space-x-1 text-slate-500 font-bold uppercase tracking-wider text-[10px] mb-1.5">
                <IconComp className={`w-3 h-3 ${meta.color}`} />
                <span>/memory/{cat}/</span>
              </div>
              <div className="space-y-1">
                {catFiles.map(f => (
                  <div
                    key={f.path}
                    onClick={() => onSelectPath(f.path)}
                    className={`group flex items-center justify-between p-2 sm:p-1.5 rounded cursor-pointer transition-colors active:scale-[0.99] ${
                      selectedPath === f.path
                        ? `bg-syndae-800 ${meta.activeBorder} font-semibold border`
                        : 'text-slate-400 hover:bg-syndae-850 hover:text-slate-200'
                    }`}
                  >
                    <div className="flex items-center space-x-2 truncate">
                      <FileText className="w-3.5 h-3.5 flex-shrink-0" />
                      <span className="truncate text-xs">{f.filename}</span>
                    </div>
                    <div className="flex items-center space-x-1">
                      <span className="text-[10px] text-slate-500 group-hover:hidden">
                        {(f.size_bytes / 1024).toFixed(1)}k
                      </span>
                      {cat !== 'core' && (
                        <button
                          type="button"
                          onClick={e => {
                            e.stopPropagation();
                            onDeleteFile(f.path);
                          }}
                          className="hidden group-hover:block text-slate-500 hover:text-rose-400 p-0.5"
                        >
                          <Trash2 className="w-3 h-3" />
                        </button>
                      )}
                    </div>
                  </div>
                ))}
              </div>
            </div>
          );
        })}
      </div>

      {/* Tantivy Fast Search Box */}
      <div className="p-3 border-t border-syndae-800 bg-syndae-950/80">
        <form onSubmit={onSearch} className="relative">
          <input
            type="text"
            placeholder="Tantivy BM25 search..."
            value={searchQuery}
            onChange={e => onChangeSearchQuery(e.target.value)}
            className="w-full bg-syndae-900 border border-syndae-700 rounded pl-7 pr-3 py-1.5 text-xs text-slate-200 placeholder-slate-500 focus:outline-none focus:border-emerald-500"
          />
          <Search className="w-3.5 h-3.5 text-slate-500 absolute left-2 top-2.5" />
        </form>

        {/* Search Result Snippets */}
        {searchResults.length > 0 && (
          <div className="mt-2 max-h-48 overflow-y-auto space-y-1 bg-syndae-900 p-1.5 rounded border border-syndae-800">
            <div className="text-[10px] text-emerald-400 font-bold mb-1">
              {searchResults.length} Match(es) found:
            </div>
            {searchResults.map((r, i) => (
              <div
                key={i}
                onClick={() => onSelectPath(r.path)}
                className="p-1.5 rounded bg-syndae-950 hover:bg-syndae-850 cursor-pointer border border-syndae-800"
              >
                <div className="font-bold text-slate-200 truncate">{r.title || r.path}</div>
                <div className="text-[10px] text-slate-400 truncate">{r.snippet}</div>
              </div>
            ))}
          </div>
        )}
      </div>
    </aside>
  );
};
