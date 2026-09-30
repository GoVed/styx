import { describe, it, expect, vi, beforeEach } from 'vitest';
import React from 'react';
import { render, screen, fireEvent, act } from '@testing-library/react';
import { NotificationToast } from '../components/navigation/NotificationToast';
import { NotificationPayload } from '../utils/notifications';

describe('Notification System & NotificationToast Component', () => {
  beforeEach(() => {
    vi.useFakeTimers();
  });

  afterEach(() => {
    vi.useRealTimers();
  });

  it('renders nothing when notification is null', () => {
    const { container } = render(
      <NotificationToast notification={null} onDismiss={vi.fn()} />
    );
    expect(container.firstChild).toBeNull();
  });

  it('renders notification title, message, and action_required urgency badge', () => {
    const notif: NotificationPayload = {
      id: 'notif-1',
      title: 'Clarification Needed',
      message: 'Should I reply to Alice with the waving cat sticker?',
      urgency: 'action_required',
      timestamp: new Date().toISOString(),
    };

    const handleDismiss = vi.fn();

    render(
      <NotificationToast notification={notif} onDismiss={handleDismiss} />
    );

    expect(screen.getByText('Clarification Needed')).toBeInTheDocument();
    expect(
      screen.getByText('Should I reply to Alice with the waving cat sticker?')
    ).toBeInTheDocument();
    expect(screen.getByText('Action Needed')).toBeInTheDocument();
  });

  it('renders alert urgency badge with appropriate style for urgent notifications', () => {
    const notif: NotificationPayload = {
      id: 'notif-2',
      title: 'Container Offline',
      message: 'The inference container stopped unexpectedly.',
      urgency: 'alert',
      timestamp: new Date().toISOString(),
    };

    render(
      <NotificationToast notification={notif} onDismiss={vi.fn()} />
    );

    expect(screen.getByText('Container Offline')).toBeInTheDocument();
    expect(screen.getByText('Urgent Alert')).toBeInTheDocument();
  });

  it('renders Open Active Chat link when sessionId is provided and handles click', () => {
    const notif: NotificationPayload = {
      id: 'notif-3',
      title: 'New External Message',
      message: 'Incoming message from Alice',
      urgency: 'action_required',
      sessionId: 'sess-123',
      timestamp: new Date().toISOString(),
    };

    const handleOpenSession = vi.fn();
    const handleDismiss = vi.fn();

    render(
      <NotificationToast
        notification={notif}
        onDismiss={handleDismiss}
        onOpenSession={handleOpenSession}
      />
    );

    const openBtn = screen.getByText('Open Active Chat');
    expect(openBtn).toBeInTheDocument();
    fireEvent.click(openBtn);

    expect(handleOpenSession).toHaveBeenCalledWith('sess-123');
    expect(handleDismiss).toHaveBeenCalled();
  });

  it('dismisses notification when dismiss button is clicked', () => {
    const notif: NotificationPayload = {
      id: 'notif-4',
      title: 'Task Finished',
      message: 'Background crawl complete.',
      urgency: 'info',
      timestamp: new Date().toISOString(),
    };

    const handleDismiss = vi.fn();

    render(
      <NotificationToast notification={notif} onDismiss={handleDismiss} />
    );

    const dismissBtn = screen.getByTitle('Dismiss notification');
    fireEvent.click(dismissBtn);

    expect(handleDismiss).toHaveBeenCalledTimes(1);
  });

  it('auto-dismisses notification after 8 seconds', () => {
    const notif: NotificationPayload = {
      id: 'notif-5',
      title: 'Auto-dismiss test',
      message: 'This will disappear in 8 seconds.',
      urgency: 'info',
      timestamp: new Date().toISOString(),
    };

    const handleDismiss = vi.fn();

    render(
      <NotificationToast notification={notif} onDismiss={handleDismiss} />
    );

    expect(handleDismiss).not.toHaveBeenCalled();

    act(() => {
      vi.advanceTimersByTime(8000);
    });

    expect(handleDismiss).toHaveBeenCalledTimes(1);
  });
});
