import React from 'react';

export interface NewMemoryFileModalProps {
  newCategory: string;
  newFilename: string;
  newContent: string;
  onChangeCategory: (cat: string) => void;
  onChangeFilename: (val: string) => void;
  onChangeContent: (val: string) => void;
  onSubmit: () => void;
  onClose: () => void;
}

export const NewMemoryFileModal: React.FC<NewMemoryFileModalProps> = ({
  newCategory,
  newFilename,
  newContent,
  onChangeCategory,
  onChangeFilename,
  onChangeContent,
  onSubmit,
  onClose,
}) => {
  return (
    <div className="fixed inset-0 bg-black/70 flex items-center justify-center z-50 p-4 font-mono">
      <div className="bg-syndae-900 border border-syndae-700 rounded-lg p-4 w-full max-w-md space-y-3 shadow-2xl">
        <h3 className="text-sm font-bold text-emerald-400">CREATE NEW MEMORY FILE</h3>

        <div>
          <label className="text-[11px] text-slate-400">Target Category:</label>
          <select
            value={newCategory}
            onChange={e => onChangeCategory(e.target.value)}
            className="w-full bg-syndae-950 border border-syndae-700 rounded p-1.5 text-slate-200 text-xs mt-1"
          >
            <option value="dictionary">User Dictionary & Vernacular (/memory/dictionary/)</option>
            <option value="people">Contact Profile & Style (/memory/people/)</option>
            <option value="groups">Group Chat Dynamics (/memory/groups/)</option>
            <option value="skills">Modular Skillset (/memory/skills/)</option>
            <option value="scratchpad">Daily Scratchpad (/memory/scratchpad/)</option>
            <option value="core">Core Directives (/memory/core/)</option>
          </select>
        </div>

        <div>
          <label className="text-[11px] text-slate-400">Filename (e.g. `docker_deploy.md`):</label>
          <input
            type="text"
            placeholder="name_of_skill.md"
            value={newFilename}
            onChange={e => onChangeFilename(e.target.value)}
            className="w-full bg-syndae-950 border border-syndae-700 rounded p-1.5 text-slate-200 text-xs mt-1"
          />
        </div>

        <div>
          <label className="text-[11px] text-slate-400">Initial Template:</label>
          <textarea
            rows={6}
            value={newContent}
            onChange={e => onChangeContent(e.target.value)}
            className="w-full bg-syndae-950 border border-syndae-700 rounded p-2 text-slate-200 text-xs mt-1 font-mono"
          />
        </div>

        <div className="flex justify-end space-x-2 pt-2">
          <button
            type="button"
            onClick={onClose}
            className="px-3 py-1.5 rounded text-slate-400 hover:text-slate-200"
          >
            Cancel
          </button>
          <button
            type="button"
            onClick={onSubmit}
            className="px-3 py-1.5 rounded bg-emerald-600 hover:bg-emerald-500 text-white font-bold"
          >
            Create File
          </button>
        </div>
      </div>
    </div>
  );
};
