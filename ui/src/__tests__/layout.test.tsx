import { describe, it, expect, vi } from 'vitest';
import { render, screen, act } from '@testing-library/react';
import React from 'react';
import { App } from '../App';
import { TelemetryStrip } from '../components/TelemetryStrip';
import { SystemTelemetry } from '../types';

// Mock global fetch for initial data calls
global.fetch = vi.fn().mockImplementation((url: string) => {
  if (url === '/api/auth/status') {
    return Promise.resolve({
      ok: true,
      json: () => Promise.resolve({ initialized: true, onboarded: true, operator_name: 'Test Operator', requires_auth: false }),
    });
  }
  if (url === '/api/auth/verify') {
    return Promise.resolve({
      ok: true,
      json: () => Promise.resolve({ valid: true, onboarded: true, operator_name: 'Test Operator' }),
    });
  }
  if (url === '/api/chat/sessions') {
    return Promise.resolve({
      json: () => Promise.resolve({ success: true, sessions: [{ id: 's1', title: 'Test Session', mode: 'chat', created_at: '', updated_at: '' }] }),
    });
  }
  if (url === '/api/memory/tree') {
    return Promise.resolve({
      json: () => Promise.resolve({ success: true, files: [] }),
    });
  }
  if (url === '/api/models/containers') {
    return Promise.resolve({
      json: () => Promise.resolve({ success: true, containers: [] }),
    });
  }
  if (url === '/api/models/presets') {
    return Promise.resolve({
      json: () => Promise.resolve({ success: true, presets: [] }),
    });
  }
  if (url === '/api/models/configs') {
    return Promise.resolve({
      json: () => Promise.resolve({ success: true, configs: [] }),
    });
  }
  if (url === '/api/tools/servers') {
    return Promise.resolve({
      json: () => Promise.resolve({ success: true, servers: [] }),
    });
  }
  if (url === '/api/tools/list') {
    return Promise.resolve({
      json: () => Promise.resolve({ success: true, tools: [] }),
    });
  }
  if (url.startsWith('/api/audit/events')) {
    return Promise.resolve({
      json: () => Promise.resolve({ success: true, events: [] }),
    });
  }
  if (url.includes('/messages')) {
    return Promise.resolve({
      json: () => Promise.resolve({ success: true, messages: [] }),
    });
  }
  return Promise.resolve({
    json: () => Promise.resolve({ success: true }),
  });
});

// Mock WebSocket
class MockWebSocket {
  onopen: any = null;
  onmessage: any = null;
  onclose: any = null;
  close = vi.fn();
  send = vi.fn();
}
(global as any).WebSocket = MockWebSocket;

describe('Layout Structure & Overflow Prevention Tests', () => {
  it('App container has h-screen w-screen overflow-hidden to prevent document-level scrolling', async () => {
    let container!: HTMLElement;
    await act(async () => {
      const res = render(<App />);
      container = res.container;
    });
    const rootDiv = container.firstChild as HTMLElement;
    expect(rootDiv).toBeInTheDocument();
    expect(rootDiv.className).toContain('h-screen');
    expect(rootDiv.className).toContain('w-screen');
    expect(rootDiv.className).toContain('overflow-hidden');
    expect(rootDiv.className).toContain('flex-col');
  });

  it('Navigation bar is flex-shrink-0 with solid background to prevent overlap with header during scroll', async () => {
    await act(async () => {
      render(<App />);
    });
    const nav = screen.getByRole('navigation');
    expect(nav).toBeInTheDocument();
    expect(nav.className).toContain('flex-shrink-0');
    expect(nav.className).toContain('bg-syndae-900');
    expect(nav.className).toContain('border-b');
  });

  it('Main workspace container has flex-1 min-h-0 overflow-hidden', async () => {
    let container!: HTMLElement;
    await act(async () => {
      const res = render(<App />);
      container = res.container;
    });
    const main = container.querySelector('main');
    expect(main).toBeInTheDocument();
    expect(main?.className).toContain('flex-1');
    expect(main?.className).toContain('min-h-0');
    expect(main?.className).toContain('overflow-hidden');
  });

  it('TelemetryStrip does not use sticky or flex-wrap to prevent colliding with tabs', () => {
    const mockTelemetry: SystemTelemetry = {
      host_cpu_pct: 15.2,
      memory_used_mb: 4096,
      memory_total_mb: 16384,
      memory_pct: 25.0,
      disk_used_gb: 120,
      disk_total_gb: 512,
      disk_pct: 23.4,
      tokens_per_second: 32.5,
      active_model: 'Local Ollama Host: fredrezones55/Qwen3.6-35B-A3B-Uncensored-HauhauCS-Aggressive:IQ2_M',
      running_containers: 2,
      active_mcp_servers: 3,
    };

    const { container } = render(
      <TelemetryStrip
        telemetry={mockTelemetry}
        pendingApprovalsCount={0}
        onOpenApprovals={() => {}}
        onToggleAudit={() => {}}
        auditOpen={false}
      />
    );

    const header = container.querySelector('header');
    expect(header).toBeInTheDocument();
    expect(header?.className).toContain('flex-shrink-0');
    expect(header?.className).not.toContain('sticky');
    expect(header?.className).toContain('bg-syndae-900');

    // Inner flex container should not wrap
    const innerFlex = header?.firstElementChild as HTMLElement;
    expect(innerFlex.className).not.toContain('flex-wrap');
    expect(innerFlex.className).toContain('overflow-x-auto');
  });

  it('TelemetryStrip truncates long model names and sets title attribute to prevent width blowout', () => {
    const veryLongModelName =
      'Local Ollama Host: fredrezones55/Qwen3.6-35B-A3B-Uncensored-HauhauCS-Aggressive:IQ2_M';

    const mockTelemetry: SystemTelemetry = {
      host_cpu_pct: 20.0,
      memory_used_mb: 8192,
      memory_total_mb: 16384,
      memory_pct: 50.0,
      disk_used_gb: 100,
      disk_total_gb: 500,
      disk_pct: 20.0,
      tokens_per_second: 0,
      active_model: veryLongModelName,
      running_containers: 1,
      active_mcp_servers: 1,
    };

    render(
      <TelemetryStrip
        telemetry={mockTelemetry}
        pendingApprovalsCount={0}
        onOpenApprovals={() => {}}
        onToggleAudit={() => {}}
        auditOpen={false}
      />
    );

    const modelSpan = screen.getByTitle(veryLongModelName);
    expect(modelSpan).toBeInTheDocument();
    expect(modelSpan.className).toContain('truncate');
    expect(modelSpan.className).toContain('max-w-');
  });

  it('TelemetryStrip vitals have flex-shrink-0 so indicators never collapse or crush', () => {
    const mockTelemetry: SystemTelemetry = {
      host_cpu_pct: 45.0,
      memory_used_mb: 8192,
      memory_total_mb: 16384,
      memory_pct: 50.0,
      disk_used_gb: 100,
      disk_total_gb: 500,
      disk_pct: 20.0,
      tokens_per_second: 12.0,
      active_model: 'vllm:qwen',
      running_containers: 1,
      active_mcp_servers: 2,
    };

    const { container } = render(
      <TelemetryStrip
        telemetry={mockTelemetry}
        pendingApprovalsCount={0}
        onOpenApprovals={() => {}}
        onToggleAudit={() => {}}
        auditOpen={false}
      />
    );

    // CPU badge container
    const cpuText = screen.getByText('CPU');
    const cpuBadge = cpuText.closest('div');
    expect(cpuBadge?.className).toContain('flex-shrink-0');

    // RAM badge container
    const ramText = screen.getByText('RAM');
    const ramBadge = ramText.closest('div');
    expect(ramBadge?.className).toContain('flex-shrink-0');
  });
});
