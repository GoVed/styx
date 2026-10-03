import { describe, it, expect, vi } from 'vitest';
import { render, screen, fireEvent, waitFor } from '@testing-library/react';
import React from 'react';
import { AccessGate } from '../components/AccessGate';

describe('AccessGate Component', () => {
  it('renders First Boot setup form when uninitialized', () => {
    const onAuth = vi.fn();
    render(
      <AccessGate
        status={{
          initialized: false,
          onboarded: false,
          operator_name: null,
          requires_auth: true,
        }}
        onAuthenticated={onAuth}
      />
    );

    expect(screen.getByText(/FIRST BOOT \/\/ DEVICE SETUP/i)).toBeInTheDocument();
    expect(screen.getByText(/CREATE DEVICE ACCESS KEY/i)).toBeInTheDocument();
    expect(screen.getByText(/CONFIRM ACCESS KEY/i)).toBeInTheDocument();
    expect(screen.getByRole('button', { name: /INITIALIZE & LAUNCH/i })).toBeInTheDocument();
  });

  it('renders Device Locked screen when initialized', () => {
    const onAuth = vi.fn();
    render(
      <AccessGate
        status={{
          initialized: true,
          onboarded: true,
          operator_name: 'Alex',
          requires_auth: true,
        }}
        onAuthenticated={onAuth}
      />
    );

    expect(screen.getByText(/ENCLAVE GUARD \/\/ DEVICE LOCKED/i)).toBeInTheDocument();
    expect(screen.getByText(/Alex/i)).toBeInTheDocument();
    expect(screen.getByRole('button', { name: /UNLOCK SYNDAE OS/i })).toBeInTheDocument();
    expect(screen.queryByText(/CONFIRM ACCESS KEY/i)).not.toBeInTheDocument();
  });

  it('shows validation error when access keys do not match on first boot', async () => {
    const onAuth = vi.fn();
    render(
      <AccessGate
        status={{
          initialized: false,
          onboarded: false,
          operator_name: null,
          requires_auth: true,
        }}
        onAuthenticated={onAuth}
      />
    );

    const inputs = screen.getAllByPlaceholderText('••••••••••••');
    fireEvent.change(inputs[0], { target: { value: 'secret123' } });
    fireEvent.change(inputs[1], { target: { value: 'different123' } });

    const submitBtn = screen.getByRole('button', { name: /INITIALIZE & LAUNCH/i });
    fireEvent.click(submitBtn);

    await waitFor(() => {
      expect(screen.getByText(/Access Keys do not match/i)).toBeInTheDocument();
    });
    expect(onAuth).not.toHaveBeenCalled();
  });
});
