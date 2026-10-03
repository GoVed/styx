import React from 'react';
import { Sparkles, Shield, Key, User, ArrowRight, AlertCircle } from 'lucide-react';

interface FirstBootScreenProps {
  operatorName: string;
  setOperatorName: (val: string) => void;
  role: string;
  setRole: (val: string) => void;
  accessKey: string;
  setAccessKey: (val: string) => void;
  confirmKey: string;
  setConfirmKey: (val: string) => void;
  error: string | null;
  loading: boolean;
  onSubmit: (e: React.FormEvent) => void;
}

export const FirstBootScreen: React.FC<FirstBootScreenProps> = ({
  operatorName,
  setOperatorName,
  role,
  setRole,
  accessKey,
  setAccessKey,
  confirmKey,
  setConfirmKey,
  error,
  loading,
  onSubmit,
}) => {
  return (
    <div className="fixed inset-0 z-50 flex items-center justify-center bg-zinc-950 px-4 py-6 font-mono select-none overflow-y-auto">
      {/* Background Ambience */}
      <div className="absolute inset-0 bg-[radial-gradient(ellipse_80%_80%_at_50%_-20%,rgba(16,185,129,0.12),rgba(255,255,255,0))]" />
      <div className="absolute inset-0 bg-[linear-gradient(to_right,#18181b_1px,transparent_1px),linear-gradient(to_bottom,#18181b_1px,transparent_1px)] bg-[size:4rem_4rem] [mask-image:radial-gradient(ellipse_60%_50%_at_50%_50%,#000_70%,transparent_100%)] opacity-20 pointer-events-none" />

      <div className="relative w-full max-w-4xl my-auto">
        <div className="grid grid-cols-1 md:grid-cols-12 gap-6 items-stretch">
          
          {/* Pre-Setup Guidance Card */}
          <div className="md:col-span-5 rounded-2xl border border-emerald-500/20 bg-zinc-900/80 p-6 sm:p-7 shadow-2xl backdrop-blur-xl flex flex-col justify-between">
            <div>
              <div className="flex items-center gap-3 mb-4">
                <div className="flex h-11 w-11 items-center justify-center rounded-xl border border-emerald-500/30 bg-emerald-950/40 text-emerald-400 shadow-[0_0_15px_rgba(16,185,129,0.2)]">
                  <Sparkles className="h-5 w-5" />
                </div>
                <div>
                  <h2 className="text-base font-bold text-zinc-100 uppercase tracking-wider font-sans">
                    Welcome to Syndae
                  </h2>
                  <p className="text-[11px] text-emerald-400 font-mono">
                    Getting Started Guide
                  </p>
                </div>
              </div>

              <p className="text-xs text-zinc-300 font-sans leading-relaxed mb-5">
                Syndae is your 100% private, on-device AI companion. It runs directly on this machine with no cloud accounts, tracking, or external subscriptions.
              </p>

              {/* 3 Step Walkthrough */}
              <div className="space-y-3.5 text-xs font-sans">
                <div className="flex items-start gap-3 rounded-lg border border-zinc-800/80 bg-zinc-950/40 p-3">
                  <div className="flex h-6 w-6 shrink-0 items-center justify-center rounded-full bg-emerald-500/20 text-emerald-400 font-mono text-[11px] font-bold">
                    1
                  </div>
                  <div>
                    <h3 className="font-semibold text-zinc-200 text-xs">Set Your Name</h3>
                    <p className="text-[11px] text-zinc-400 leading-snug mt-0.5">
                      Tell Syndae who it's assisting so your interactions feel natural and personal.
                    </p>
                  </div>
                </div>

                <div className="flex items-start gap-3 rounded-lg border border-zinc-800/80 bg-zinc-950/40 p-3">
                  <div className="flex h-6 w-6 shrink-0 items-center justify-center rounded-full bg-emerald-500/20 text-emerald-400 font-mono text-[11px] font-bold">
                    2
                  </div>
                  <div>
                    <h3 className="font-semibold text-zinc-200 text-xs">Create Private Passcode</h3>
                    <p className="text-[11px] text-zinc-400 leading-snug mt-0.5">
                      Choose a PIN or key (4+ characters) to lock your assistant. Since Syndae is local, this protects your data on this machine.
                    </p>
                  </div>
                </div>

                <div className="flex items-start gap-3 rounded-lg border border-zinc-800/80 bg-zinc-950/40 p-3">
                  <div className="flex h-6 w-6 shrink-0 items-center justify-center rounded-full bg-emerald-500/20 text-emerald-400 font-mono text-[11px] font-bold">
                    3
                  </div>
                  <div>
                    <h3 className="font-semibold text-zinc-200 text-xs">Interactive Onboarding</h3>
                    <p className="text-[11px] text-zinc-400 leading-snug mt-0.5">
                      When you click launch, Syndae greets you with friendly choices to personalize your tasks, habits, and tone.
                    </p>
                  </div>
                </div>
              </div>
            </div>

            {/* Local Readiness Status */}
            <div className="mt-6 pt-4 border-t border-zinc-800/70 font-mono text-[10px] space-y-1.5 text-zinc-400">
              <div className="flex items-center justify-between">
                <span className="flex items-center gap-1.5">
                  <span className="h-2 w-2 rounded-full bg-emerald-400 animate-pulse" />
                  Local AI Engine:
                </span>
                <span className="text-emerald-400 font-medium">Ready & Connected</span>
              </div>
              <div className="flex items-center justify-between">
                <span className="flex items-center gap-1.5">
                  <span className="h-1.5 w-1.5 rounded-full bg-emerald-400" />
                  Privacy Protection:
                </span>
                <span className="text-zinc-300">100% On-Device</span>
              </div>
              <div className="flex items-center justify-between">
                <span className="flex items-center gap-1.5">
                  <span className="h-1.5 w-1.5 rounded-full bg-emerald-400" />
                  Local Memory Vault:
                </span>
                <span className="text-zinc-300">Encrypted Storage</span>
              </div>
            </div>
          </div>

          {/* Setup Form Card */}
          <div className="md:col-span-7 rounded-2xl border border-zinc-800 bg-zinc-900/90 p-6 sm:p-8 shadow-2xl backdrop-blur-xl flex flex-col justify-between">
            <div>
              {/* Enclave Status Header */}
              <div className="mb-6 flex flex-col items-center text-center">
                <div className="mb-3 flex h-14 w-14 items-center justify-center rounded-2xl border border-emerald-500/30 bg-emerald-950/40 text-emerald-400 shadow-[0_0_20px_rgba(16,185,129,0.2)]">
                  <Sparkles className="h-7 w-7" />
                </div>

                <div className="flex items-center gap-2">
                  <span className="h-2 w-2 rounded-full bg-emerald-400 animate-pulse" />
                  <span className="text-[10px] tracking-widest text-emerald-400 uppercase font-semibold">
                    First Boot // Device Setup
                  </span>
                </div>

                <h1 className="mt-2 text-xl font-bold tracking-tight text-zinc-100 uppercase">
                  Syndae Personal AI
                </h1>

                <p className="mt-1 text-xs text-zinc-400">
                  Welcome! Create your private passcode or key to protect your assistant.
                </p>
              </div>

              {error && (
                <div className="mb-4 flex items-center gap-2 rounded-lg border border-rose-500/30 bg-rose-950/30 p-3 text-xs text-rose-300">
                  <AlertCircle className="h-4 w-4 shrink-0 text-rose-400" />
                  <span>{error}</span>
                </div>
              )}

              <form onSubmit={onSubmit} className="space-y-4">
                <div>
                  <label className="mb-1 block text-[11px] font-medium text-zinc-400 uppercase tracking-wider">
                    Your Name or Nickname
                  </label>
                  <div className="relative">
                    <div className="pointer-events-none absolute inset-y-0 left-0 flex items-center pl-3 text-zinc-500">
                      <User className="h-4 w-4" />
                    </div>
                    <input
                      type="text"
                      value={operatorName}
                      onChange={(e) => setOperatorName(e.target.value)}
                      placeholder="What should Syndae call you?"
                      className="w-full rounded-lg border border-zinc-800 bg-zinc-950/60 py-2.5 pl-9 pr-3 text-sm text-zinc-100 placeholder-zinc-600 transition-colors focus:border-emerald-500/50 focus:outline-none focus:ring-1 focus:ring-emerald-500/50"
                    />
                  </div>
                </div>

                <div>
                  <label className="mb-1 block text-[11px] font-medium text-zinc-400 uppercase tracking-wider">
                    What do you do? (Optional)
                  </label>
                  <input
                    type="text"
                    value={role}
                    onChange={(e) => setRole(e.target.value)}
                    placeholder="e.g. Student, Designer, Business Owner, Writer..."
                    className="w-full rounded-lg border border-zinc-800 bg-zinc-950/60 py-2.5 px-3 text-sm text-zinc-100 placeholder-zinc-600 transition-colors focus:border-emerald-500/50 focus:outline-none focus:ring-1 focus:ring-emerald-500/50"
                  />
                </div>

                <div>
                  <label className="mb-1 block text-[11px] font-medium text-zinc-400 uppercase tracking-wider">
                    Create Device Access Key
                  </label>
                  <div className="relative">
                    <div className="pointer-events-none absolute inset-y-0 left-0 flex items-center pl-3 text-zinc-500">
                      <Key className="h-4 w-4" />
                    </div>
                    <input
                      type="password"
                      value={accessKey}
                      onChange={(e) => setAccessKey(e.target.value)}
                      placeholder="••••••••••••"
                      autoFocus
                      className="w-full rounded-lg border border-zinc-800 bg-zinc-950/60 py-2.5 pl-9 pr-3 text-sm text-zinc-100 placeholder-zinc-600 transition-colors focus:border-emerald-500/50 focus:outline-none focus:ring-1 focus:ring-emerald-500/50"
                    />
                  </div>
                </div>

                <div>
                  <label className="mb-1 block text-[11px] font-medium text-zinc-400 uppercase tracking-wider">
                    Confirm Access Key
                  </label>
                  <div className="relative">
                    <div className="pointer-events-none absolute inset-y-0 left-0 flex items-center pl-3 text-zinc-500">
                      <Shield className="h-4 w-4" />
                    </div>
                    <input
                      type="password"
                      value={confirmKey}
                      onChange={(e) => setConfirmKey(e.target.value)}
                      placeholder="••••••••••••"
                      className="w-full rounded-lg border border-zinc-800 bg-zinc-950/60 py-2.5 pl-9 pr-3 text-sm text-zinc-100 placeholder-zinc-600 transition-colors focus:border-emerald-500/50 focus:outline-none focus:ring-1 focus:ring-emerald-500/50"
                    />
                  </div>
                </div>

                <button
                  type="submit"
                  disabled={loading}
                  className="mt-6 flex w-full items-center justify-center gap-2 rounded-lg bg-emerald-500 py-2.5 text-xs font-semibold text-zinc-950 uppercase tracking-wider transition-all hover:bg-emerald-400 focus:outline-none focus:ring-2 focus:ring-emerald-500/50 disabled:opacity-50"
                >
                  {loading ? (
                    <span>Unlocking your assistant...</span>
                  ) : (
                    <>
                      <span>Initialize & Launch</span>
                      <ArrowRight className="h-4 w-4" />
                    </>
                  )}
                </button>
              </form>
            </div>

            <div className="mt-6 border-t border-zinc-800/80 pt-4 text-center">
              <p className="text-[10px] text-zinc-500">
                Private Personal AI • 100% on your device • No tracking or ads
              </p>
            </div>
          </div>

        </div>
      </div>
    </div>
  );
};
