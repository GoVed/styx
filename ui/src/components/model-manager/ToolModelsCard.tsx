import React, { useState, useEffect } from 'react';
import { Sparkles, RefreshCw } from 'lucide-react';
import { ModelConfigRecord } from '../../types';
import { VisionModelSection } from './VisionModelSection';
import { TranslationModelSection } from './TranslationModelSection';

interface ToolModelsCardProps {
  onRefresh?: () => void;
}

interface RoleConfigsResponse {
  success: boolean;
  main?: ModelConfigRecord | null;
  vision?: ModelConfigRecord | null;
  translation?: ModelConfigRecord | null;
}

export const ToolModelsCard: React.FC<ToolModelsCardProps> = ({ onRefresh }) => {
  const [roleConfigs, setRoleConfigs] = useState<RoleConfigsResponse | null>(null);
  const [loading, setLoading] = useState(false);

  const fetchRoles = async () => {
    setLoading(true);
    try {
      const res = await fetch('/api/models/roles', { credentials: 'omit' });
      if (res.ok) {
        const data = await res.json();
        setRoleConfigs(data);
      }
    } catch {
      // ignore
    } finally {
      setLoading(false);
    }
  };

  useEffect(() => {
    fetchRoles();
  }, []);

  const handleSaved = () => {
    fetchRoles();
    onRefresh?.();
  };

  return (
    <div className="w-full space-y-6">
      {/* Banner */}
      <div className="bg-styx-900 border border-styx-700/80 rounded-lg p-4 flex flex-col sm:flex-row sm:items-center justify-between gap-3">
        <div className="flex items-center space-x-3">
          <div className="p-2 bg-emerald-500/10 border border-emerald-500/30 rounded-md">
            <Sparkles className="w-5 h-5 text-emerald-400" />
          </div>
          <div>
            <h3 className="text-sm font-bold text-slate-100 font-mono">
              SPECIALIZED TOOL MODELS (VISION & TRANSLATION)
            </h3>
            <p className="text-xs text-slate-400 mt-0.5">
              Host and swap dedicated sidecar models for specific tools without interrupting your primary reasoning LLM.
            </p>
          </div>
        </div>
        <button
          onClick={fetchRoles}
          disabled={loading}
          className="px-3 py-1.5 rounded bg-styx-800 hover:bg-styx-700 border border-styx-700 text-xs font-mono text-slate-300 flex items-center space-x-1.5 self-start sm:self-auto"
        >
          <RefreshCw className={`w-3.5 h-3.5 ${loading ? 'animate-spin' : ''}`} />
          <span>Refresh Status</span>
        </button>
      </div>

      <div className="grid grid-cols-1 lg:grid-cols-2 gap-6">
        <VisionModelSection
          currentModelId={roleConfigs?.vision?.model_id || 'moondream (Default)'}
          onSaved={handleSaved}
        />
        <TranslationModelSection
          currentModelId={roleConfigs?.translation?.model_id || 'Universal Translator'}
          onSaved={handleSaved}
        />
      </div>
    </div>
  );
};
