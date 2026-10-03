export interface NotificationPayload {
  id: string;
  title: string;
  message: string;
  urgency: 'info' | 'action_required' | 'alert';
  sessionId?: string;
  timestamp: string;
}

/**
 * Synthesize a gentle, pleasant notification chime using Web Audio API.
 * No external sound files or network requests required.
 */
export function playNotificationChime(): void {
  try {
    const AudioContextClass = window.AudioContext || (window as any).webkitAudioContext;
    if (!AudioContextClass) return;
    const ctx = new AudioContextClass();

    const now = ctx.currentTime;
    // Tone 1: 523.25 Hz (C5)
    const osc1 = ctx.createOscillator();
    const gain1 = ctx.createGain();
    osc1.type = 'sine';
    osc1.frequency.setValueAtTime(523.25, now);
    gain1.gain.setValueAtTime(0, now);
    gain1.gain.linearRampToValueAtTime(0.18, now + 0.02);
    gain1.gain.exponentialRampToValueAtTime(0.001, now + 0.18);
    osc1.connect(gain1);
    gain1.connect(ctx.destination);
    osc1.start(now);
    osc1.stop(now + 0.2);

    // Tone 2: 659.25 Hz (E5) slightly delayed
    const osc2 = ctx.createOscillator();
    const gain2 = ctx.createGain();
    osc2.type = 'sine';
    osc2.frequency.setValueAtTime(659.25, now + 0.08);
    gain2.gain.setValueAtTime(0, now + 0.08);
    gain2.gain.linearRampToValueAtTime(0.22, now + 0.1);
    gain2.gain.exponentialRampToValueAtTime(0.001, now + 0.32);
    osc2.connect(gain2);
    gain2.connect(ctx.destination);
    osc2.start(now + 0.08);
    osc2.stop(now + 0.35);
  } catch {
    // AudioContext may be blocked before first user gesture
  }
}

export function isNotificationSupported(): boolean {
  return typeof window !== 'undefined' && 'Notification' in window;
}

export function getNotificationPermission(): NotificationPermission {
  if (!isNotificationSupported()) return 'denied';
  return Notification.permission;
}

export async function requestNotificationPermission(): Promise<NotificationPermission> {
  if (!isNotificationSupported()) return 'denied';
  try {
    return await Notification.requestPermission();
  } catch {
    return 'denied';
  }
}

export function sendBrowserNotification(
  title: string,
  options?: {
    body?: string;
    tag?: string;
    sessionId?: string;
    urgency?: 'info' | 'action_required' | 'alert';
    onClick?: () => void;
  }
): boolean {
  // Trigger physical device vibration only for alert or action_required urgency
  if (options?.urgency && options.urgency !== 'info') {
    if (typeof navigator !== 'undefined' && 'vibrate' in navigator) {
      try {
        navigator.vibrate([200, 100, 200, 100, 300]);
      } catch {
        // Ignore vibration errors if blocked by OS policy
      }
    }
  }

  if (!isNotificationSupported() || Notification.permission !== 'granted') {
    return false;
  }

  // 1. Prefer ServiceWorkerRegistration.showNotification (native Android & iOS PWA lockscreen)
  if (typeof navigator !== 'undefined' && 'serviceWorker' in navigator) {
    navigator.serviceWorker.ready
      .then((reg) => {
        if (reg && typeof reg.showNotification === 'function') {
          return reg.showNotification(title, {
            body: options?.body,
            tag: options?.tag || 'syndae-alert',
            icon: '/icon-192.png',
            badge: '/icon-192.png',
            vibrate: [200, 100, 200, 100, 300],
            renotify: true,
            data: {
              url: options?.sessionId ? `/?session=${options.sessionId}` : '/',
              sessionId: options?.sessionId,
            },
          });
        }
      })
      .catch(() => {
        // Fall back to window Notification below if SW is unavailable
      });
  }

  // 2. Standard window Notification fallback (desktop browsers)
  try {
    const notif = new Notification(title, {
      body: options?.body,
      tag: options?.tag || 'syndae-alert',
      icon: '/icon-192.png',
    });

    if (options?.onClick) {
      notif.onclick = () => {
        window.focus();
        options.onClick?.();
        notif.close();
      };
    }
    return true;
  } catch {
    return false;
  }
}
