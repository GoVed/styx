import { useState } from 'react';
import { McpTool } from '../../types';

export const useToolTester = () => {
  const [testingTool, setTestingTool] = useState<McpTool | null>(null);
  const [testArgs, setTestArgs] = useState<string>('{}');
  const [testOutput, setTestOutput] = useState<any | null>(null);
  const [isCalling, setIsCalling] = useState<boolean>(false);

  const handleRunToolTest = async () => {
    if (!testingTool) return;
    setIsCalling(true);
    setTestOutput(null);
    try {
      const parsed = JSON.parse(testArgs);
      const res = await fetch('/api/tools/call', {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({ tool_name: testingTool.name, arguments: parsed }),
      });
      const data = await res.json();
      setTestOutput(data);
    } catch (e) {
      setTestOutput({ success: false, error: 'Call failed: ' + e });
    } finally {
      setIsCalling(false);
    }
  };

  return {
    testingTool,
    setTestingTool,
    testArgs,
    setTestArgs,
    testOutput,
    isCalling,
    handleRunToolTest,
  };
};
