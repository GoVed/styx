import React, { useState } from 'react';
import { AuthStatus } from '../types';
import { LockScreen } from './access-gate/LockScreen';
import { FirstBootScreen } from './access-gate/FirstBootScreen';

interface AccessGateProps {
  status: AuthStatus;
  onAuthenticated: (token: string, status: AuthStatus) => void;
}

export const AccessGate: React.FC<AccessGateProps> = ({ status, onAuthenticated }) => {
  const [accessKey, setAccessKey] = useState('');
  const [confirmKey, setConfirmKey] = useState('');
  const [operatorName, setOperatorName] = useState('');
  const [role, setRole] = useState('');
  const [error, setError] = useState<string | null>(null);
  const [loading, setLoading] = useState(false);

  const isFirstBoot = !status.initialized;

  const handleSubmit = async (e: React.FormEvent) => {
    e.preventDefault();
    setError(null);

    const key = accessKey.trim();
    if (!key) {
      setError('Please enter a Device Access Key');
      return;
    }

    if (isFirstBoot) {
      if (key.length < 8) {
        setError('Access Key must be at least 8 characters');
        return;
      }
      if (key !== confirmKey.trim()) {
        setError('Access Keys do not match');
        return;
      }

      setLoading(true);
      try {
        const res = await fetch('/api/auth/setup', {
          method: 'POST',
          headers: { 'Content-Type': 'application/json' },
          body: JSON.stringify({
            access_key: key,
            operator_name: operatorName.trim() || undefined,
            role: role.trim() || undefined,
          }),
        });

        const data = await res.json();
        if (!res.ok || !data.success) {
          setError(data.error || 'Setup failed');
          setLoading(false);
          return;
        }

        localStorage.setItem('styx_access_key', key);
        onAuthenticated(key, {
          initialized: true,
          onboarded: false,
          operator_name: data.operator_name || operatorName.trim() || null,
          requires_auth: true,
        });
      } catch (err: any) {
        setError(err.message || 'Network error during initialization');
      } finally {
        setLoading(false);
      }
    } else {
      // Login / Unlock
      setLoading(true);
      try {
        const res = await fetch('/api/auth/login', {
          method: 'POST',
          headers: { 'Content-Type': 'application/json' },
          body: JSON.stringify({ access_key: key }),
        });

        const data = await res.json();
        if (!res.ok || !data.success) {
          setError(data.error || 'Invalid Device Access Key');
          setLoading(false);
          return;
        }

        localStorage.setItem('styx_access_key', key);
        onAuthenticated(key, {
          initialized: true,
          onboarded: data.onboarded,
          operator_name: data.operator_name,
          requires_auth: true,
        });
      } catch (err: any) {
        setError(err.message || 'Network error during unlock');
      } finally {
        setLoading(false);
      }
    }
  };

  if (!isFirstBoot) {
    return (
      <LockScreen
        operatorName={status.operator_name}
        accessKey={accessKey}
        setAccessKey={setAccessKey}
        error={error}
        loading={loading}
        onSubmit={handleSubmit}
      />
    );
  }

  return (
    <FirstBootScreen
      operatorName={operatorName}
      setOperatorName={setOperatorName}
      role={role}
      setRole={setRole}
      accessKey={accessKey}
      setAccessKey={setAccessKey}
      confirmKey={confirmKey}
      setConfirmKey={setConfirmKey}
      error={error}
      loading={loading}
      onSubmit={handleSubmit}
    />
  );
};
