import React, { useState, useEffect } from 'react';
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

  return (
    <div className="flex h-full w-full overflow-hidden bg-styx-950 font-sans text-xs">
      <FileTreeSidebar
        files={files}
        selectedPath={selectedPath}
        searchQuery={searchQuery}
        searchResults={searchResults}
        onSelectPath={setSelectedPath}
        onOpenNewModal={() => setShowNewModal(true)}
        onDeleteFile={handleDelete}
        onChangeSearchQuery={setSearchQuery}
        onSearch={handleSearch}
      />

      <MemoryEditor
        selectedPath={selectedPath}
        fileContent={fileContent}
        saveStatus={saveStatus}
        activeView={activeView}
        onChangeActiveView={setActiveView}
        onChangeContent={setFileContent}
        onSave={handleSave}
      />

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
