import { useState } from 'react';

export const useManualServerModal = (onRefresh: () => void) => {
  const [showAddModal, setShowAddModal] = useState(false);
  const [serverName, setServerName] = useState('Shell & Filesystem Daemon');
  const [transportType, setTransportType] = useState<'stdio' | 'unix_socket' | 'http_sse'>('stdio');
  const [command, setCommand] = useState('python3');
  const [argsStr, setArgsStr] = useState('["./examples/reference_mcp_daemon.py"]');
  const [socketPath, setSocketPath] = useState('/tmp/syndae-mcp.sock');
  const [url, setUrl] = useState('http://localhost:8080/mcp');

  const handleAddServer = async () => {
    try {
      let argsParsed = [];
      if (argsStr) {
        try {
          argsParsed = JSON.parse(argsStr);
        } catch {
          argsParsed = argsStr.split(' ');
        }
      }

      const payload = {
        name: serverName,
        transport_type: transportType,
        command: transportType === 'stdio' ? command : undefined,
        args: transportType === 'stdio' ? argsParsed : undefined,
        socket_path: transportType === 'unix_socket' ? socketPath : undefined,
        url: transportType === 'http_sse' ? url : undefined,
      };

      const res = await fetch('/api/tools/servers', {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify(payload),
      });
      const data = await res.json();
      if (data.success) {
        setShowAddModal(false);
        onRefresh();
      } else {
        alert('Failed to register MCP server: ' + data.error);
      }
    } catch (e) {
      alert('Error registering MCP server: ' + e);
    }
  };

  return {
    showAddModal,
    setShowAddModal,
    serverName,
    setServerName,
    transportType,
    setTransportType,
    command,
    setCommand,
    argsStr,
    setArgsStr,
    socketPath,
    setSocketPath,
    url,
    setUrl,
    handleAddServer,
  };
};
