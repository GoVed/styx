import { describe, it, expect, vi, beforeEach } from 'vitest';
import { render, screen, fireEvent, waitFor } from '@testing-library/react';
import React from 'react';
import { ToolHub } from '../components/ToolHub';
import { McpServerRecord, McpTool } from '../types';

const mockServers: McpServerRecord[] = [
  {
    id: 'server-whatsapp',
    name: 'whatsapp',
    transport_type: 'http',
    url: 'http://host.docker.internal:8765/mcp',
    enabled: true,
    created_at: new Date().toISOString(),
  },
];

const mockTools: McpTool[] = [
  {
    name: 'find_stickers',
    description: 'Search sticker catalog by emotional context & tags',
    input_schema: { type: 'object' },
    server_id: 'server-whatsapp',
    policy: 'AUTONOMOUS',
    risk_level: 'LOW',
  },
  {
    name: 'send_message',
    description: 'Send plain text WhatsApp message',
    input_schema: { type: 'object' },
    server_id: 'server-whatsapp',
    policy: 'REQUIRE_APPROVAL',
    risk_level: 'HIGH',
  },
  {
    name: 'send_sticker',
    description: 'Dispatch a sticker to a contact',
    input_schema: { type: 'object' },
    server_id: 'server-whatsapp',
    policy: 'REQUIRE_APPROVAL',
    risk_level: 'HIGH',
  },
];

describe('ToolHub Component - Drag & Drop Support', () => {
  const onRefresh = vi.fn();
  const onUpdatePolicy = vi.fn();

  beforeEach(() => {
    vi.clearAllMocks();
    global.fetch = vi.fn((url: any) => {
      if (typeof url === 'string' && url.includes('/api/tools/discover')) {
        return Promise.resolve({
          ok: true,
          json: async () => ({
            success: true,
            tools: [
              {
                id: 'whatsapp',
                name: 'whatsapp',
                display_name: 'WhatsApp Connector',
                version: '1.0.0',
                description: 'Isolated Bi-Directional WhatsApp Connect Micro-Daemon',
                path: '/tools/syndae-whatsapp',
                is_installed: true,
                has_docker: true,
              },
            ],
          }),
        } as any);
      }

      if (typeof url === 'string' && url.includes('/api/tools/inspect')) {
        return Promise.resolve({
          ok: true,
          json: async () => ({
            success: true,
            tool: {
              name: 'whatsapp',
              display_name: 'WhatsApp Connector',
              version: '1.0.0',
              description: 'Isolated Bi-Directional WhatsApp Connect Micro-Daemon & MCP Tool for Syndae Agent OS',
              path: '/tools/syndae-whatsapp',
              has_docker: true,
              has_compose: true,
              container_name: 'syndae-whatsapp-connector',
              container_exists: true,
              container_running: true,
              container_status: 'Up 10 minutes',
              container_healthy: true,
              port: 8765,
              recommended_transport: 'http',
              recommended_url: 'http://host.docker.internal:8765/mcp',
              is_registered: true,
              skill_file: 'skills/whatsapp_tone.md',
              tools: [
                {
                  name: 'find_stickers',
                  description: 'Search sticker catalog by emotional context & tags',
                  risk_level: 'LOW',
                  policy: 'AUTONOMOUS',
                },
                {
                  name: 'send_message',
                  description: 'Send a plain text message',
                  risk_level: 'HIGH',
                  policy: 'REQUIRE_APPROVAL',
                },
              ],
            },
          }),
        } as any);
      }

      if (typeof url === 'string' && url.includes('/api/tools/install')) {
        return Promise.resolve({
          ok: true,
          json: async () => ({
            success: true,
            message: 'Successfully registered whatsapp with 2 tools',
            discovered_tools: [{ name: 'find_stickers' }, { name: 'send_message' }],
          }),
        } as any);
      }

      if (typeof url === 'string' && url.includes('/api/tools/trigger')) {
        return Promise.resolve({
          ok: true,
          json: async () => ({
            success: true,
            session_id: 'wa-session-123',
            turn_id: 'turn-456',
            status: 'running',
          }),
        } as any);
      }

      return Promise.resolve({
        ok: true,
        json: async () => ({ success: true }),
      } as any);
    });
  });

  it('renders dynamic tool bus header and drag & drop dropzone', async () => {
    render(
      <ToolHub
        servers={mockServers}
        tools={mockTools}
        onRefresh={onRefresh}
        onUpdatePolicy={onUpdatePolicy}
      />
    );

    expect(screen.getByText(/DYNAMIC TOOL BUS \(MCP 2024-11-05\)/i)).toBeDefined();
    expect(screen.getByText(/DRAG & DROP TOOL PACKAGE OR DIRECTORY/i)).toBeDefined();
    expect(screen.getByPlaceholderText('/path/to/tool')).toBeDefined();
    expect(screen.getByText(/Browse Tool Folder/i)).toBeDefined();
  });

  it('inspects directory path via input and triggers inspection on click', async () => {
    render(
      <ToolHub
        servers={[]}
        tools={[]}
        onRefresh={onRefresh}
        onUpdatePolicy={onUpdatePolicy}
      />
    );

    const input = screen.getByPlaceholderText('/path/to/tool');
    fireEvent.change(input, { target: { value: '/tools/syndae-whatsapp' } });

    const inspectBtn = screen.getByRole('button', { name: /Inspect/i });
    fireEvent.click(inspectBtn);

    await waitFor(() => {
      expect(global.fetch).toHaveBeenCalledWith(
        '/api/tools/inspect',
        expect.objectContaining({
          method: 'POST',
          body: JSON.stringify({
            path: '/tools/syndae-whatsapp',
          }),
        })
      );
    });

    // Verify inspected tool card appears with details
    await waitFor(() => {
      expect(screen.getByText(/v1.0.0/i)).toBeDefined();
      expect(screen.getByText(/syndae-whatsapp-connector/i)).toBeDefined();
      expect(screen.getByText(/Deploy & Connect MCP Daemon|Re-Deploy & Sync Tool/i)).toBeDefined();
    });
  });

  it('handles drag-and-drop of directory path text onto the dropzone', async () => {
    render(
      <ToolHub
        servers={[]}
        tools={[]}
        onRefresh={onRefresh}
        onUpdatePolicy={onUpdatePolicy}
      />
    );

    const dropzone = screen.getByText(/DRAG & DROP TOOL PACKAGE OR DIRECTORY/i).closest('div')!;

    // Test drag over
    fireEvent.dragOver(dropzone, {
      dataTransfer: { types: ['text/plain'] },
    });

    // Test drop with path text
    fireEvent.drop(dropzone, {
      dataTransfer: {
        getData: (format: string) =>
          format === 'text/plain' ? '/tools/syndae-whatsapp/' : '',
        items: [],
      },
    });

    await waitFor(() => {
      expect(global.fetch).toHaveBeenCalledWith(
        '/api/tools/inspect',
        expect.objectContaining({
          method: 'POST',
        })
      );
    });
  });

  it('deploys and registers inspected tool via 1-click install button', async () => {
    render(
      <ToolHub
        servers={[]}
        tools={[]}
        onRefresh={onRefresh}
        onUpdatePolicy={onUpdatePolicy}
      />
    );

    // Trigger inspection first
    const input = screen.getByPlaceholderText('/path/to/tool');
    fireEvent.change(input, { target: { value: '/tools/syndae-whatsapp' } });
    const inspectBtn = screen.getByRole('button', { name: /Inspect/i });
    fireEvent.click(inspectBtn);

    const deployButton = await screen.findByText(/Deploy & Connect MCP Daemon|Re-Deploy & Sync Tool/i);
    expect(deployButton).toBeDefined();

    fireEvent.click(deployButton);

    await waitFor(() => {
      expect(global.fetch).toHaveBeenCalledWith(
        '/api/tools/install',
        expect.objectContaining({
          method: 'POST',
          body: JSON.stringify({
            path: '/tools/syndae-whatsapp',
            name: 'whatsapp',
            transport_type: 'http',
            url: 'http://host.docker.internal:8765/mcp',
            auto_start_container: true,
          }),
        })
      );
      expect(onRefresh).toHaveBeenCalled();
    });
  });

  it('renders connected WhatsApp daemon and provides simulated inbound trigger action', async () => {
    render(
      <ToolHub
        servers={mockServers}
        tools={mockTools}
        onRefresh={onRefresh}
        onUpdatePolicy={onUpdatePolicy}
      />
    );

    expect(screen.getByText('CONNECTED MCP DAEMONS (1)')).toBeDefined();
    expect(screen.getByText('whatsapp')).toBeDefined();
    expect(screen.getByText('Simulate Inbound Event')).toBeDefined();

    const triggerBtn = screen.getByText('Simulate Inbound Event');
    fireEvent.click(triggerBtn);

    await waitFor(() => {
      expect(global.fetch).toHaveBeenCalledWith(
        '/api/tools/trigger',
        expect.objectContaining({
          method: 'POST',
        })
      );
    });
  });

  it('renders tool policy matrix with Auto, Ask, and Block controls', () => {
    render(
      <ToolHub
        servers={mockServers}
        tools={mockTools}
        onRefresh={onRefresh}
        onUpdatePolicy={onUpdatePolicy}
      />
    );

    expect(screen.getByText('find_stickers')).toBeDefined();
    expect(screen.getByText('send_message')).toBeDefined();
    expect(screen.getByText('send_sticker')).toBeDefined();

    const autoButtons = screen.getAllByText('Auto');
    expect(autoButtons.length).toBeGreaterThan(0);
    fireEvent.click(autoButtons[0]);
    expect(onUpdatePolicy).toHaveBeenCalled();
  });
});
