import { describe, it, expect, vi } from 'vitest';
import { render, screen, fireEvent } from '@testing-library/react';
import React from 'react';
import { SettingsHub } from '../components/SettingsHub';
import { SystemTelemetry, AuthStatus } from '../types';

const mockTelemetry: SystemTelemetry = {
  host_cpu_pct: 18.5,
  memory_used_mb: 8192,
  memory_total_mb: 16384,
  memory_pct: 50.0,
  disk_used_gb: 120,
  disk_total_gb: 500,
  disk_pct: 24.0,
  tokens_per_second: 34.2,
  active_model: 'QuantTrio/Qwen3.5-9B-AWQ:latest',
  running_containers: 1,
  active_mcp_servers: 2,
  gpu: {
    name: 'NVIDIA RTX 4090',
    vram_used_mb: 14000,
    vram_total_mb: 24576,
    vram_pct: 57.0,
    temperature_c: 54,
  },
  concurrency: {
    active_requests: 1,
    max_concurrent: 1,
    queued_requests: 0,
    queue_wait_time_estimate_sec: 0,
  },
};

const mockAuthStatus: AuthStatus = {
  initialized: true,
  onboarded: true,
  operator_name: 'Lead Architect',
  requires_auth: true,
};

describe('SettingsHub Android-style System Menu Tests', () => {
  it('renders Operator Profile Card with operator name and enclave status', () => {
    render(
      <SettingsHub
        telemetry={mockTelemetry}
        authStatus={mockAuthStatus}
        modelConfigs={[]}
        containers={[]}
        mcpTools={[]}
        pendingTicketsCount={0}
        onNavigate={() => {}}
        onLock={() => {}}
        onStartOnboarding={() => {}}
        onOpenAudit={() => {}}
        onBackToChat={() => {}}
      />
    );

    expect(screen.getByText('Lead Architect')).toBeInTheDocument();
    expect(screen.getByText(/Private Personal Assistant/i)).toBeInTheDocument();
  });

  it('renders all core settings categories', () => {
    render(
      <SettingsHub
        telemetry={mockTelemetry}
        authStatus={mockAuthStatus}
        modelConfigs={[]}
        containers={[]}
        mcpTools={[]}
        pendingTicketsCount={0}
        onNavigate={() => {}}
        onLock={() => {}}
        onStartOnboarding={() => {}}
        onOpenAudit={() => {}}
        onBackToChat={() => {}}
      />
    );

    expect(screen.getByText('AI Voice & Intelligence')).toBeInTheDocument();
    expect(screen.getByText('Memory & What I Know About You')).toBeInTheDocument();
    expect(screen.getByText('Connected Apps & Assistants')).toBeInTheDocument();
    expect(screen.getByText('Device Health & Performance')).toBeInTheDocument();
    expect(screen.getByText('Privacy & Safety Approvals')).toBeInTheDocument();
    expect(screen.getByText('Master Access Key & Password')).toBeInTheDocument();
    expect(screen.getByText('Personal Setup & Preferences')).toBeInTheDocument();
  });

  it('clicking a menu card invokes onNavigate with the correct target', () => {
    const handleNavigate = vi.fn();
    render(
      <SettingsHub
        telemetry={mockTelemetry}
        authStatus={mockAuthStatus}
        modelConfigs={[]}
        containers={[]}
        mcpTools={[]}
        pendingTicketsCount={0}
        onNavigate={handleNavigate}
        onLock={() => {}}
        onStartOnboarding={() => {}}
        onOpenAudit={() => {}}
        onBackToChat={() => {}}
      />
    );

    const modelCard = screen.getByText('AI Voice & Intelligence').closest('div[class*="cursor-pointer"]');
    expect(modelCard).toBeInTheDocument();
    fireEvent.click(modelCard!);
    expect(handleNavigate).toHaveBeenCalledWith('models');

    const memoryCard = screen.getByText('Memory & What I Know About You').closest('div[class*="cursor-pointer"]');
    expect(memoryCard).toBeInTheDocument();
    fireEvent.click(memoryCard!);
    expect(handleNavigate).toHaveBeenCalledWith('memory');
  });

  it('clicking System Diagnostics opens hardware vitals sub-view', () => {
    render(
      <SettingsHub
        telemetry={mockTelemetry}
        authStatus={mockAuthStatus}
        modelConfigs={[]}
        containers={[]}
        mcpTools={[]}
        pendingTicketsCount={0}
        onNavigate={() => {}}
        onLock={() => {}}
        onStartOnboarding={() => {}}
        onOpenAudit={() => {}}
        onBackToChat={() => {}}
      />
    );

    const diagCard = screen.getByText('Device Health & Performance').closest('div[class*="cursor-pointer"]');
    fireEvent.click(diagCard!);

    expect(screen.getByText('Device Performance & Health')).toBeInTheDocument();
    expect(screen.getByText(/NVIDIA RTX 4090/i)).toBeInTheDocument();
    expect(screen.getByText(/TYPING SPEED/i)).toBeInTheDocument();
    expect(screen.getByText('34.2')).toBeInTheDocument();
    expect(screen.getByText('words/sec')).toBeInTheDocument();

    // Click back to main settings
    const backBtn = screen.getByTitle('Back');
    fireEvent.click(backBtn);
    expect(screen.getByText('Assistant Settings')).toBeInTheDocument();
  });

  it('search filter narrows settings categories', () => {
    render(
      <SettingsHub
        telemetry={mockTelemetry}
        authStatus={mockAuthStatus}
        modelConfigs={[]}
        containers={[]}
        mcpTools={[]}
        pendingTicketsCount={0}
        onNavigate={() => {}}
        onLock={() => {}}
        onStartOnboarding={() => {}}
        onOpenAudit={() => {}}
        onBackToChat={() => {}}
      />
    );

    const searchInput = screen.getByPlaceholderText(/Search settings/i);
    fireEvent.change(searchInput, { target: { value: 'apps' } });

    expect(screen.getByText('Connected Apps & Assistants')).toBeInTheDocument();
    expect(screen.queryByText('Memory & What I Know About You')).not.toBeInTheDocument();
  });

  it('clicking Master Access Key & Password opens ChangePasswordView with form controls', () => {
    const handlePasswordChanged = vi.fn();
    render(
      <SettingsHub
        telemetry={mockTelemetry}
        authStatus={mockAuthStatus}
        modelConfigs={[]}
        containers={[]}
        mcpTools={[]}
        pendingTicketsCount={0}
        onNavigate={() => {}}
        onLock={() => {}}
        onStartOnboarding={() => {}}
        onOpenAudit={() => {}}
        onBackToChat={() => {}}
        onPasswordChanged={handlePasswordChanged}
      />
    );

    expect(screen.getByText('Master Access Key & Password')).toBeInTheDocument();
    const passCard = screen.getByText('Master Access Key & Password').closest('div[class*="cursor-pointer"]');
    fireEvent.click(passCard!);

    expect(screen.getByText('Change Master Password')).toBeInTheDocument();
    expect(screen.getByPlaceholderText('Enter current master password')).toBeInTheDocument();
    expect(screen.getByPlaceholderText('Enter new password (min. 8 characters)')).toBeInTheDocument();
    expect(screen.getByPlaceholderText('Re-enter new password')).toBeInTheDocument();
    expect(screen.getByText('Save New Master Password')).toBeInTheDocument();

    // Click back to main settings
    const backBtn = screen.getByText('Back to Settings Menu');
    fireEvent.click(backBtn);
    expect(screen.getByText('Assistant Settings')).toBeInTheDocument();
  });
});
