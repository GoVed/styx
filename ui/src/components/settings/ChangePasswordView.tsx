import React, { useState } from 'react';
import {
  Lock,
  KeyRound,
  Eye,
  EyeOff,
  CheckCircle,
  AlertTriangle,
  ArrowLeft,
  ShieldCheck,
  RefreshCw,
} from 'lucide-react';

interface ChangePasswordViewProps {
  onBack: () => void;
  onPasswordChanged?: (newToken: string) => void;
}

export const ChangePasswordView: React.FC<ChangePasswordViewProps> = ({
  onBack,
  onPasswordChanged,
}) => {
  const [currentPassword, setCurrentPassword] = useState('');
  const [newPassword, setNewPassword] = useState('');
  const [confirmPassword, setConfirmPassword] = useState('');

  const [showCurrent, setShowCurrent] = useState(false);
  const [showNew, setShowNew] = useState(false);
  const [showConfirm, setShowConfirm] = useState(false);

  const [loading, setLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [success, setSuccess] = useState<string | null>(null);

  const isLengthValid = newPassword.trim().length >= 8;
  const isMatch = newPassword.trim().length > 0 && newPassword === confirmPassword;
  const isDifferent = newPassword.trim().length > 0 && newPassword.trim() !== currentPassword.trim();
  const canSubmit = currentPassword.trim().length > 0 && isLengthValid && isMatch && isDifferent && !loading;

  const handleSubmit = async (e: React.FormEvent) => {
    e.preventDefault();
    setError(null);
    setSuccess(null);

    const current = currentPassword.trim();
    const next = newPassword.trim();
    const confirm = confirmPassword.trim();

    if (!current) {
      setError('Please enter your current master password.');
      return;
    }

    if (next.length < 8) {
      setError('New password must be at least 8 characters long.');
      return;
    }

    if (next === current) {
      setError('New password must be different from your current password.');
      return;
    }

    if (next !== confirm) {
      setError('New passwords do not match. Please verify your entry.');
      return;
    }

    setLoading(true);
    try {
      const activeToken = localStorage.getItem('syndae_access_key') || current;
      const res = await fetch('/api/auth/password', {
        method: 'POST',
        headers: {
          'Content-Type': 'application/json',
          Authorization: `Bearer ${activeToken}`,
        },
        body: JSON.stringify({
          current_password: current,
          new_password: next,
        }),
      });

      const data = await res.json().catch(() => ({}));
      if (!res.ok || !data.success) {
        throw new Error(data.error || 'Failed to update master password');
      }

      const newToken = data.token || next;
      localStorage.setItem('syndae_access_key', newToken);
      setSuccess('Master password successfully updated! Your active enclave session has been renewed.');
      setCurrentPassword('');
      setNewPassword('');
      setConfirmPassword('');

      if (onPasswordChanged) {
        onPasswordChanged(newToken);
      }
    } catch (err: any) {
      setError(err.message || 'Network error while attempting to update password.');
    } finally {
      setLoading(false);
    }
  };

  return (
    <div className="space-y-5 animate-fade-in max-w-2xl mx-auto">
      {/* Back button & view title banner */}
      <div className="flex items-center justify-between">
        <button
          onClick={onBack}
          className="inline-flex items-center space-x-2 text-xs font-mono text-slate-400 hover:text-slate-200 transition-colors py-1.5 px-2.5 rounded-lg hover:bg-syndae-900 border border-transparent hover:border-syndae-800"
        >
          <ArrowLeft className="w-4 h-4" />
          <span>Back to Settings Menu</span>
        </button>
      </div>

      {/* Hero Security Card */}
      <div className="rounded-2xl bg-gradient-to-br from-syndae-900 to-syndae-850 border border-syndae-800 p-5 shadow-lg">
        <div className="flex items-start space-x-4">
          <div className="w-12 h-12 rounded-xl bg-amber-500/20 border border-amber-500/40 flex items-center justify-center text-amber-400 flex-shrink-0 shadow-inner">
            <KeyRound className="w-6 h-6" />
          </div>
          <div>
            <div className="flex items-center gap-2">
              <h2 className="text-base sm:text-lg font-bold text-white tracking-tight">
                Change Master Password
              </h2>
              <span className="px-2 py-0.5 rounded text-[10px] font-mono bg-amber-950/80 text-amber-400 border border-amber-800">
                Enclave Security
              </span>
            </div>
            <p className="text-xs text-slate-400 mt-1 leading-relaxed">
              This master key unlocks Syndae on your machine and authorizes access to your tools, memory, and chat sessions.
              Updating it will store a new Argon2id password hash on your device and renew your active browser session.
            </p>
          </div>
        </div>
      </div>

      {/* Success Notification Banner */}
      {success && (
        <div className="rounded-xl bg-emerald-950/40 border border-emerald-500/40 p-4 text-xs text-emerald-300 flex items-start space-x-3 shadow-sm animate-fade-in">
          <ShieldCheck className="w-5 h-5 text-emerald-400 flex-shrink-0 mt-0.5" />
          <div className="flex-1 space-y-1">
            <p className="font-semibold text-emerald-200">Password Changed</p>
            <p className="text-emerald-300/90 leading-relaxed">{success}</p>
            <div className="pt-2">
              <button
                type="button"
                onClick={onBack}
                className="px-3 py-1.5 rounded-lg bg-emerald-600 hover:bg-emerald-500 text-white font-medium text-xs shadow-sm transition-all"
              >
                Return to Settings
              </button>
            </div>
          </div>
        </div>
      )}

      {/* Error Notification Banner */}
      {error && (
        <div className="rounded-xl bg-rose-950/40 border border-rose-500/40 p-4 text-xs text-rose-300 flex items-start space-x-3 shadow-sm animate-fade-in">
          <AlertTriangle className="w-5 h-5 text-rose-400 flex-shrink-0 mt-0.5" />
          <div className="flex-1">
            <p className="font-semibold text-rose-200">Unable to Update Password</p>
            <p className="text-rose-300/90 mt-0.5">{error}</p>
          </div>
        </div>
      )}

      {/* Password Change Form Card */}
      <div className="rounded-2xl bg-syndae-900/90 border border-syndae-800 p-5 sm:p-6 shadow-md">
        <form onSubmit={handleSubmit} className="space-y-4">
          {/* Current Master Password */}
          <div>
            <label className="block text-xs font-mono font-medium text-slate-300 mb-1.5">
              Current Master Password
            </label>
            <div className="relative">
              <input
                type={showCurrent ? 'text' : 'password'}
                value={currentPassword}
                onChange={e => setCurrentPassword(e.target.value)}
                placeholder="Enter current master password"
                required
                disabled={loading}
                className="w-full bg-syndae-950 border border-syndae-800 focus:border-amber-500 rounded-xl px-3.5 py-2.5 text-xs text-slate-200 placeholder-slate-500 focus:outline-none focus:ring-1 focus:ring-amber-500/30 transition-all font-mono pr-10"
              />
              <button
                type="button"
                onClick={() => setShowCurrent(!showCurrent)}
                className="absolute right-3 top-1/2 -translate-y-1/2 text-slate-500 hover:text-slate-300 transition-colors p-1"
                title={showCurrent ? 'Hide password' : 'Show password'}
              >
                {showCurrent ? <EyeOff className="w-4 h-4" /> : <Eye className="w-4 h-4" />}
              </button>
            </div>
          </div>

          <div className="pt-1 border-t border-syndae-800/60" />

          {/* New Master Password */}
          <div>
            <label className="block text-xs font-mono font-medium text-slate-300 mb-1.5">
              New Master Password
            </label>
            <div className="relative">
              <input
                type={showNew ? 'text' : 'password'}
                value={newPassword}
                onChange={e => setNewPassword(e.target.value)}
                placeholder="Enter new password (min. 8 characters)"
                required
                disabled={loading}
                className="w-full bg-syndae-950 border border-syndae-800 focus:border-amber-500 rounded-xl px-3.5 py-2.5 text-xs text-slate-200 placeholder-slate-500 focus:outline-none focus:ring-1 focus:ring-amber-500/30 transition-all font-mono pr-10"
              />
              <button
                type="button"
                onClick={() => setShowNew(!showNew)}
                className="absolute right-3 top-1/2 -translate-y-1/2 text-slate-500 hover:text-slate-300 transition-colors p-1"
                title={showNew ? 'Hide password' : 'Show password'}
              >
                {showNew ? <EyeOff className="w-4 h-4" /> : <Eye className="w-4 h-4" />}
              </button>
            </div>

            {/* Validation Checklist */}
            <div className="mt-2 space-y-1.5 pl-1">
              <div className="flex items-center space-x-2 text-[11px] font-mono">
                <CheckCircle
                  className={`w-3.5 h-3.5 ${
                    isLengthValid ? 'text-emerald-400' : 'text-slate-600'
                  }`}
                />
                <span className={isLengthValid ? 'text-slate-300' : 'text-slate-500'}>
                  At least 8 characters long
                </span>
              </div>
              {newPassword.length > 0 && currentPassword.length > 0 && (
                <div className="flex items-center space-x-2 text-[11px] font-mono">
                  <CheckCircle
                    className={`w-3.5 h-3.5 ${
                      isDifferent ? 'text-emerald-400' : 'text-rose-400'
                    }`}
                  />
                  <span className={isDifferent ? 'text-slate-300' : 'text-rose-400'}>
                    Different from current password
                  </span>
                </div>
              )}
            </div>
          </div>

          {/* Confirm New Master Password */}
          <div>
            <label className="block text-xs font-mono font-medium text-slate-300 mb-1.5">
              Confirm New Master Password
            </label>
            <div className="relative">
              <input
                type={showConfirm ? 'text' : 'password'}
                value={confirmPassword}
                onChange={e => setConfirmPassword(e.target.value)}
                placeholder="Re-enter new password"
                required
                disabled={loading}
                className="w-full bg-syndae-950 border border-syndae-800 focus:border-amber-500 rounded-xl px-3.5 py-2.5 text-xs text-slate-200 placeholder-slate-500 focus:outline-none focus:ring-1 focus:ring-amber-500/30 transition-all font-mono pr-10"
              />
              <button
                type="button"
                onClick={() => setShowConfirm(!showConfirm)}
                className="absolute right-3 top-1/2 -translate-y-1/2 text-slate-500 hover:text-slate-300 transition-colors p-1"
                title={showConfirm ? 'Hide password' : 'Show password'}
              >
                {showConfirm ? <EyeOff className="w-4 h-4" /> : <Eye className="w-4 h-4" />}
              </button>
            </div>
            {confirmPassword.length > 0 && (
              <p
                className={`text-[11px] font-mono mt-1.5 flex items-center space-x-1.5 ${
                  isMatch ? 'text-emerald-400' : 'text-rose-400'
                }`}
              >
                <CheckCircle className="w-3.5 h-3.5" />
                <span>{isMatch ? 'Passwords match' : 'Passwords do not match'}</span>
              </p>
            )}
          </div>

          {/* Action Buttons */}
          <div className="pt-3 flex items-center justify-end space-x-3">
            <button
              type="button"
              onClick={onBack}
              disabled={loading}
              className="px-4 py-2.5 rounded-xl bg-syndae-800 hover:bg-syndae-700 text-slate-300 hover:text-white text-xs font-medium border border-syndae-700 transition-all"
            >
              Cancel
            </button>
            <button
              type="submit"
              disabled={!canSubmit}
              className="px-5 py-2.5 rounded-xl bg-amber-600 hover:bg-amber-500 disabled:bg-syndae-800 disabled:text-slate-500 disabled:border-syndae-700 text-white font-medium text-xs flex items-center space-x-2 shadow-sm transition-all active:scale-95 disabled:active:scale-100"
            >
              {loading ? (
                <>
                  <RefreshCw className="w-3.5 h-3.5 animate-spin" />
                  <span>Updating Password...</span>
                </>
              ) : (
                <>
                  <Lock className="w-3.5 h-3.5" />
                  <span>Save New Master Password</span>
                </>
              )}
            </button>
          </div>
        </form>
      </div>
    </div>
  );
};
