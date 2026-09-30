import React, { useState, useEffect } from 'react';
import { Layers, FileText } from 'lucide-react';
import { MemoryFileNode } from '../types';
import { FileTreeSidebar } from './memory/FileTreeSidebar';
import { MemoryEditor } from './memory/MemoryEditor';
import { NewMemoryFileModal } from './memory/NewMemoryFileModal';

export interface MemoryHubProps {
  files: MemoryFileNode[];
  onRefreshFiles: () => void;
}

export const MemoryHub: React.FC<MemoryHubProps> = ({ files, onRefreshFiles }) => {
  const [selectedPath, setSelectedPath] = useState<string>(files[0]?.path || 'core/user_profile.md');
  const [mobileTab, setMobileTab] = useState<'files' | 'editor'>('files');
  const [fileContent, setFileContent] = useState<string>('');
  const [searchQuery, setSearchQuery] = useState<string>('');
  const [searchResults, setSearchResults] = useState<any[]>([]);
  const [, setIsSearching] = useState<boolean>(false);
  const [activeView, setActiveView] = useState<'edit' | 'preview'>('edit');
  const [saveStatus, setSaveStatus] = useState<string>('');
  const [showNewModal, setShowNewModal] = useState<boolean>(false);
  const [newCategory, setNewCategory] = useState<string>('dictionary');
  const [newFilename, setNewFilename] = useState<string>('');
  const [newContent, setNewContent] = useState<string>('# Dictionary / Context Notes\n\n## Phrases & Meanings\n\n## Usage Examples\n');

  useEffect(() => {
    if (selectedPath) {
      loadFileContent(selectedPath);
    }
  }, [selectedPath]);

  const loadFileContent = async (path: string) => {
    try {
      const res = await fetch(`/api/memory/file?path=${encodeURIComponent(path)}`);
      const data = await res.json();
      if (data.success) {
        setFileContent(data.content);
      }
    } catch (e) {
      console.error('Failed to load file', e);
    }
  };

  const handleSave = async () => {
    if (!selectedPath) return;
    try {
      const res = await fetch('/api/memory/file', {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({ path: selectedPath, content: fileContent }),
      });
      const data = await res.json();
      if (data.success) {
        setSaveStatus('Saved & Tantivy re-indexed');
        setTimeout(() => setSaveStatus(''), 3000);
        onRefreshFiles();
      }
    } catch {
      setSaveStatus('Error saving file');
    }
  };

  const handleCreate = async () => {
    if (!newFilename.trim()) return;
    try {
      const res = await fetch('/api/memory/create', {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({
          category: newCategory,
          filename: newFilename,
          content: newContent,
        }),
      });
      const data = await res.json();
      if (data.success) {
        setShowNewModal(false);
        setNewFilename('');
        setSelectedPath(data.path);
        onRefreshFiles();
      }
    } catch (e) {
      alert('Failed to create file: ' + e);
    }
  };

  const handleDelete = async (path: string) => {
    if (!confirm(`Permanently delete memory file ${path}?`)) return;
    try {
      const res = await fetch(`/api/memory/file?path=${encodeURIComponent(path)}`, {
        method: 'DELETE',
      });
      const data = await res.json();
      if (data.success) {
        onRefreshFiles();
        if (selectedPath === path) {
          setSelectedPath(files[0]?.path || '');
        }
      }
    } catch (e) {
      alert('Failed to delete file: ' + e);
    }
  };

  const handleSearch = async (e: React.FormEvent) => {
    e.preventDefault();
    if (!searchQuery.trim()) {
      setSearchResults([]);
      return;
    }
    setIsSearching(true);
    try {
      const res = await fetch(`/api/memory/search?q=${encodeURIComponent(searchQuery)}&limit=10`);
      const data = await res.json();
      if (data.success) {
        setSearchResults(data.results);
      }
    } catch (err) {
      console.error(err);
    } finally {
      setIsSearching(false);
    }
  };

  const handleSelectPath = (path: string) => {
    setSelectedPath(path);
    setMobileTab('editor');
  };

  return (
    <div className="flex flex-col h-full w-full overflow-hidden bg-styx-950 font-sans text-xs">
      {/* Mobile Top Segmented Bar */}
      <div className="md:hidden flex border-b border-styx-800 bg-styx-900/90 px-3 py-1.5 gap-2 flex-shrink-0 font-mono select-none">
        <button
          type="button"
          onClick={() => setMobileTab('files')}
          className={`flex-1 py-1.5 px-3 rounded-lg text-xs font-semibold flex items-center justify-center space-x-1.5 transition-all ${
            mobileTab === 'files'
              ? 'bg-styx-800 text-emerald-300 shadow-sm border border-styx-700/80'
              : 'text-slate-400 hover:text-slate-200'
          }`}
        >
          <Layers className="w-3.5 h-3.5 text-emerald-400" />
          <span>Files ({files.length})</span>
        </button>
        <button
          type="button"
          onClick={() => setMobileTab('editor')}
          className={`flex-1 py-1.5 px-3 rounded-lg text-xs font-semibold flex items-center justify-center space-x-1.5 transition-all ${
            mobileTab === 'editor'
              ? 'bg-styx-800 text-cyan-300 shadow-sm border border-styx-700/80'
              : 'text-slate-400 hover:text-slate-200'
          }`}
        >
          <FileText className="w-3.5 h-3.5 text-cyan-400" />
          <span className="truncate max-w-[130px]">{selectedPath ? selectedPath.split('/').pop() : 'Editor'}</span>
        </button>
      </div>

      <div className="flex-1 flex overflow-hidden min-h-0">
        <FileTreeSidebar
          files={files}
          selectedPath={selectedPath}
          searchQuery={searchQuery}
          searchResults={searchResults}
          onSelectPath={handleSelectPath}
          onOpenNewModal={() => setShowNewModal(true)}
          onDeleteFile={handleDelete}
          onChangeSearchQuery={setSearchQuery}
          onSearch={handleSearch}
          onViewEditor={() => setMobileTab('editor')}
          className={mobileTab === 'files' ? 'w-full md:w-72 flex' : 'hidden md:flex md:w-72'}
        />

        <MemoryEditor
          selectedPath={selectedPath}
          fileContent={fileContent}
          saveStatus={saveStatus}
          activeView={activeView}
          onChangeActiveView={setActiveView}
          onChangeContent={setFileContent}
          onSave={handleSave}
          onBackToList={() => setMobileTab('files')}
          className={mobileTab === 'editor' ? 'flex-1 flex w-full' : 'hidden md:flex md:flex-1'}
        />
      </div>

      {showNewModal && (
        <NewMemoryFileModal
          newCategory={newCategory}
          newFilename={newFilename}
          newContent={newContent}
          onChangeCategory={setNewCategory}
          onChangeFilename={setNewFilename}
          onChangeContent={setNewContent}
          onSubmit={handleCreate}
          onClose={() => setShowNewModal(false)}
        />
      )}
    </div>
  );
};
