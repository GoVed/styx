import { useState } from 'react';
import { safeFetchJson } from './vram';

export function useExternalModels(onRefresh: () => void) {
  const [extName, setExtName] = useState('Claude 3.7 Sonnet (Anthropic)');
  const [extProvider, setExtProvider] = useState('anthropic');
  const [extBaseUrl, setExtBaseUrl] = useState('https://api.anthropic.com/v1');
  const [extApiKey, setExtApiKey] = useState('');
  const [extModelId, setExtModelId] = useState('claude-3-7-sonnet-20250219');
  const [extContextLength, setExtContextLength] = useState<number>(200000);
  const [testResult, setTestResult] = useState<any | null>(null);
  const [isTesting, setIsTesting] = useState<boolean>(false);

  const handleTestConnection = async () => {
    setIsTesting(true);
    setTestResult(null);
    try {
      const res = await fetch('/api/models/test-connection', {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({
          provider: extProvider,
          base_url: extBaseUrl || undefined,
          api_key: extApiKey || undefined,
          model_id: extModelId,
        }),
      });
      const data = await safeFetchJson(res);
      setTestResult(data);
    } catch (e) {
      setTestResult({
        success: false,
        message: 'Connection test error: ' + (e instanceof Error ? e.message : String(e)),
      });
    } finally {
      setIsTesting(false);
    }
  };

  const handleSaveExternalConfig = async () => {
    try {
      const res = await fetch('/api/models/configs', {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({
          name: extName,
          provider: extProvider,
          base_url: extBaseUrl || undefined,
          api_key: extApiKey || undefined,
          model_id: extModelId,
          context_length: extContextLength,
        }),
      });
      const data = await safeFetchJson(res);
      if (data.success) {
        onRefresh();
        alert('Model provider registered successfully!');
      } else {
        alert('Failed to register provider: ' + (data.error || 'Server error'));
      }
    } catch (e) {
      alert('Failed to register provider: ' + (e instanceof Error ? e.message : String(e)));
    }
  };

  return {
    extName,
    setExtName,
    extProvider,
    setExtProvider,
    extBaseUrl,
    setExtBaseUrl,
    extApiKey,
    setExtApiKey,
    extModelId,
    setExtModelId,
    extContextLength,
    setExtContextLength,
    testResult,
    isTesting,
    handleTestConnection,
    handleSaveExternalConfig,
  };
}
