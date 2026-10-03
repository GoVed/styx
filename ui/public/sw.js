// Syndae Local Device Service Worker (Zero Cloud, 100% Local)
self.addEventListener('install', (event) => {
  self.skipWaiting();
});

self.addEventListener('activate', (event) => {
  event.waitUntil(self.clients.claim());
});

// Handle incoming local notification dispatch from client page
self.addEventListener('message', (event) => {
  if (event.data && event.data.type === 'SHOW_NOTIFICATION') {
    const { title, options } = event.data;
    event.waitUntil(
      self.registration.showNotification(title, {
        body: options.body || '',
        tag: options.tag || 'syndae-alert',
        icon: options.icon || '/icon-192.png',
        badge: options.badge || '/icon-192.png',
        vibrate: options.vibrate || [200, 100, 200, 100, 300],
        renotify: true,
        data: options.data || { url: '/' },
      })
    );
  }
});

// Handle clicking on notification from device lock screen or notification drawer
self.addEventListener('notificationclick', (event) => {
  event.notification.close();
  const data = event.notification.data || {};
  const targetUrl = data.url || '/';

  event.waitUntil(
    self.clients.matchAll({ type: 'window', includeUncontrolled: true }).then((clientList) => {
      for (const client of clientList) {
        if (client.url.includes(self.location.origin) && 'focus' in client) {
          client.focus();
          if (data.sessionId) {
            client.postMessage({
              type: 'NAVIGATE_SESSION',
              sessionId: data.sessionId,
            });
          }
          return;
        }
      }
      if (self.clients.openWindow) {
        return self.clients.openWindow(targetUrl);
      }
    })
  );
});
