import React, { useEffect } from 'react';
import { Bell, X, ExternalLink, AlertCircle, Info, Zap } from 'lucide-react';
import { NotificationPayload } from '../../utils/notifications';

export interface NotificationToastProps {
  notification: NotificationPayload | null;
  onDismiss: () => void;
  onOpenSession?: (sessionId: string) => void;
}

export const NotificationToast: React.FC<NotificationToastProps> = ({
  notification,
  onDismiss,
  onOpenSession,
}) => {
  useEffect(() => {
    if (!notification) return;
    const timer = setTimeout(() => {
      onDismiss();
    }, 8000);
    return () => clearTimeout(timer);
  }, [notification, onDismiss]);

  if (!notification) return null;

  const isAlert = notification.urgency === 'alert';
  const isAction = notification.urgency === 'action_required';

  const borderClass = isAlert
    ? 'border-rose-500/60 bg-gradient-to-r from-rose-950/90 via-syndae-900/95 to-syndae-900/95 shadow-[0_0_20px_rgba(244,63,94,0.3)]'
    : isAction
    ? 'border-amber-500/60 bg-gradient-to-r from-amber-950/90 via-syndae-900/95 to-syndae-900/95 shadow-[0_0_20px_rgba(245,158,11,0.25)]'
    : 'border-emerald-500/60 bg-gradient-to-r from-emerald-950/90 via-syndae-900/95 to-syndae-900/95 shadow-[0_0_20px_rgba(16,185,129,0.25)]';

  const badgeClass = isAlert
    ? 'bg-rose-950 text-rose-300 border-rose-800'
    : isAction
    ? 'bg-amber-950 text-amber-300 border-amber-800'
    : 'bg-emerald-950 text-emerald-300 border-emerald-800';

  const IconComponent = isAlert ? AlertCircle : isAction ? Zap : Info;

  return (
    <aside
      aria-label="Agent Alert"
      className="fixed top-12 sm:top-14 left-0 right-0 z-50 px-3 sm:px-4 flex justify-center pointer-events-none"
    >
      <div
        className={`w-full max-w-lg pointer-events-auto rounded-xl border p-3 sm:p-3.5 backdrop-blur-md font-sans text-xs shadow-2xl transition-all animate-in fade-in slide-in-from-top-2 duration-300 ${borderClass}`}
      >
        <div className="flex items-start justify-between gap-2.5">
          <div className="flex items-start gap-2.5 min-w-0 flex-1">
            <div className="p-1.5 rounded-lg bg-syndae-950/80 border border-slate-700/60 shrink-0 mt-0.5">
              <IconComponent className="w-4 h-4 text-emerald-400" />
            </div>

            <div className="min-w-0 flex-1 space-y-1">
              <div className="flex items-center gap-2 flex-wrap">
                <span className="font-semibold text-slate-100 text-xs truncate">
                  {notification.title}
                </span>
                <span
                  className={`text-[9px] font-mono px-1.5 py-0.5 rounded border uppercase tracking-wider ${badgeClass}`}
                >
                  {isAlert ? 'Urgent Alert' : isAction ? 'Action Needed' : 'Notice'}
                </span>
              </div>

              <p className="text-slate-300 text-[11px] leading-relaxed break-words">
                {notification.message}
              </p>

              {notification.sessionId && onOpenSession && (
                <button
                  type="button"
                  onClick={() => {
                    onOpenSession(notification.sessionId!);
                    onDismiss();
                  }}
                  className="mt-1 inline-flex items-center gap-1 text-[11px] font-mono text-emerald-400 hover:text-emerald-300 font-medium transition-colors"
                >
                  <span>Open Active Chat</span>
                  <ExternalLink className="w-3 h-3" />
                </button>
              )}
            </div>
          </div>

          <button
            type="button"
            onClick={onDismiss}
            className="p-1 rounded hover:bg-syndae-800/80 text-slate-400 hover:text-slate-200 transition-colors shrink-0"
            title="Dismiss notification"
          >
            <X className="w-4 h-4" />
          </button>
        </div>
      </div>
    </aside>
  );
};
