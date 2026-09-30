import '@testing-library/jest-dom';
import { vi } from 'vitest';

// Polyfill scrollIntoView for jsdom
window.HTMLElement.prototype.scrollIntoView = vi.fn();
Element.prototype.scrollIntoView = vi.fn();

// Pre-seed mock auth key in localStorage for tests
localStorage.setItem('styx_access_key', 'test-key');

// Global fetch mock fallback for relative URLs in jsdom
global.fetch = vi.fn().mockImplementation((url: string) => {
  return Promise.resolve({
    ok: true,
    status: 200,
    json: () =>
      Promise.resolve({
        success: true,
        initialized: true,
        onboarded: true,
        operator_name: 'Test Operator',
        requires_auth: false,
        valid: true,
        messages: [],
        sessions: [
          {
            id: 'default-session',
            title: 'Default Test Session',
            mode: 'chat',
            created_at: new Date().toISOString(),
            updated_at: new Date().toISOString(),
          },
        ],
        files: [],
        containers: [],
        presets: [],
        configs: [],
        servers: [],
        tools: [],
        events: [],
      }),
  });
});
