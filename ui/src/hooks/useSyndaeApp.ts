import { useState, useEffect, useCallback } from 'react';
import {
  SystemTelemetry,
  ApprovalTicket,
  ChatSession,
  AuthStatus,
} from '../types';
import { useAuxiliaryData } from './useAuxiliaryData';
import { useSyndaeWebSocket } from './useSyndaeWebSocket';
import { useChatState } from './useChatState';
import {
  NotificationPayload,
  playNotificationChime,
  sendBrowserNotification,
} from '../utils/notifications';

export const useSyndaeApp = () => {
  // Authentication & Device Security
  const [authStatus, setAuthStatus] = useState<AuthStatus | null>(null);
  const [isAuthenticated, setIsAuthenticated] = useState<boolean>(false);
  const [authChecking, setAuthChecking] = useState<boolean>(true);

  const [activeTab, setActiveTab] = useState<'chat' | 'settings' | 'memory' | 'models' | 'tools'>('chat');
  const [showTelemetryStrip, setShowTelemetryStrip] = useState<boolean>(false);
  const [telemetry, setTelemetry] = useState<SystemTelemetry | null>(null);
  const [pendingTickets, setPendingTickets] = useState<ApprovalTicket[]>([]);
  const [auditOpen, setAuditOpen] = useState(false);

  // Auxiliary data management (memory, containers, models, tools, audit)
  const aux = useAuxiliaryData();

  // Chat sessions, messages, streaming & tool execution state
  const chat = useChatState({
    refreshAuditData: aux.refreshAuditData,
    setAuthStatus,
    setPendingTickets,
  });

  const fetchInitialData = useCallback(async () => {
    try {
      const sRes = await fetch('/api/chat/sessions');
      const sData = await sRes.json();
      if (sData.success) {
        chat.setSessions(sData.sessions);
        if (sData.sessions.length > 0) {
          chat.setActiveSession(sData.sessions[0]);
        } else {
          chat.handleCreateSession('chat');
        }
      }

      aux.refreshMemoryFiles();
      aux.refreshModelData();
      aux.refreshToolsData();
      aux.refreshAuditData();
    } catch (e) {
      console.error('Initial data fetch error', e);
    }
  }, [chat.handleCreateSession, chat.setSessions, chat.setActiveSession, aux]);

  const [activeNotification, setActiveNotification] = useState<NotificationPayload | null>(null);

  const { connectWebSocket, closeWebSocket } = useSyndaeWebSocket({
    onTelemetry: setTelemetry,
    onConnected: payload => {
      if (payload?.pending_tickets) {
        setPendingTickets(payload.pending_tickets);
      }
    },
    onApprovalTicket: ticket => {
      setPendingTickets(prev => [...prev.filter(t => t.ticket_id !== ticket.ticket_id), ticket]);
      playNotificationChime();
      sendBrowserNotification('Action Approval Required', {
        body: `Tool '${ticket.tool_name}' requires your authorization to proceed.`,
      });
      setActiveNotification({
        id: `ticket-${ticket.ticket_id}`,
        title: 'Action Approval Required',
        message: `Tool '${ticket.tool_name}' requires your authorization to proceed.`,
        urgency: 'action_required',
        timestamp: new Date().toISOString(),
      });
    },
    onApprovalResolved: ticketId => {
      setPendingTickets(prev => prev.filter(t => t.ticket_id !== ticketId));
      setActiveNotification(prev => (prev?.id === `ticket-${ticketId}` ? null : prev));
    },
    onChatEvent: chat.handleIncomingChatEvent,
    onNotification: notif => {
      // Only chime and trigger OS push/vibrate for actionable or urgent notifications
      if (notif.urgency === 'alert' || notif.urgency === 'action_required') {
        playNotificationChime();
        sendBrowserNotification(notif.title, {
          body: notif.message,
          urgency: notif.urgency,
          sessionId: notif.sessionId,
          onClick: () => {
            if (notif.sessionId) {
              chat.setActiveSession({ id: notif.sessionId } as any);
              setActiveTab('chat');
            }
          },
        });
      }
      setActiveNotification(notif);
    },
    onAuthEvent: () => {
      setAuthStatus(prev => (prev ? { ...prev, onboarded: true } : null));
    },
  });

  const checkAuth = useCallback(async () => {
    try {
      const res = await fetch('/api/auth/status');
      const data: AuthStatus = await res.json();
      setAuthStatus(data);

      if (!data.initialized) {
        setIsAuthenticated(false);
      } else {
        const storedKey = localStorage.getItem('syndae_access_key');
        if (!storedKey) {
          setIsAuthenticated(false);
        } else {
          const verifyRes = await fetch('/api/auth/verify', { method: 'POST' });
          if (verifyRes.ok) {
            setIsAuthenticated(true);
            fetchInitialData();
            connectWebSocket();
          } else {
            setIsAuthenticated(false);
            localStorage.removeItem('syndae_access_key');
          }
        }
      }
    } catch (e) {
      console.error('Auth status check failed', e);
      setIsAuthenticated(true);
      fetchInitialData();
      connectWebSocket();
    } finally {
      setAuthChecking(false);
    }
  }, [connectWebSocket, fetchInitialData]);

  useEffect(() => {
    checkAuth();
    return () => {
      closeWebSocket();
    };
  }, []);

  const handleStartOnboarding = async () => {
    try {
      chat.setIsStreaming(true);
      chat.setChatError(null);
      setActiveTab('chat');

      const res = await fetch('/api/auth/onboarding/start', { method: 'POST' });
      const data = await res.json();
      if (data.success && data.session_id) {
        const onboardingSession: ChatSession = {
          id: data.session_id,
          title: 'Welcome to Syndae',
          mode: 'chat',
          created_at: new Date().toISOString(),
          updated_at: new Date().toISOString(),
        };

        const sRes = await fetch('/api/chat/sessions');
        const sData = await sRes.json();
        if (sData.success && Array.isArray(sData.sessions)) {
          chat.setSessions(sData.sessions);
          const found = sData.sessions.find((s: any) => s.id === data.session_id);
          chat.setActiveSession(found || onboardingSession);
        } else {
          chat.setSessions([onboardingSession]);
          chat.setActiveSession(onboardingSession);
        }

        chat.loadMessages(data.session_id);
      } else {
        chat.setIsStreaming(false);
      }
    } catch (e) {
      console.error('Failed to trigger onboarding', e);
      chat.setIsStreaming(false);
    }
  };

  const handleDismissOnboarding = async () => {
    try {
      await fetch('/api/auth/onboarding/complete', { method: 'POST' });
      setAuthStatus(prev => (prev ? { ...prev, onboarded: true } : null));
    } catch (e) {
      console.error('Failed to dismiss onboarding', e);
      setAuthStatus(prev => (prev ? { ...prev, onboarded: true } : null));
    }
  };

  const handleAuthenticated = (token: string, newStatus: AuthStatus) => {
    localStorage.setItem('syndae_access_key', token);
    setAuthStatus(newStatus);
    setIsAuthenticated(true);
    fetchInitialData();
    connectWebSocket();

    if (!newStatus.onboarded) {
      setTimeout(() => {
        handleStartOnboarding();
      }, 300);
    }
  };

  const handleLock = () => {
    localStorage.removeItem('syndae_access_key');
    setIsAuthenticated(false);
    closeWebSocket();
  };

  const handleResolveTicket = async (
    ticketId: string,
    action: 'APPROVE' | 'REJECT' | 'MODIFY',
    modifiedArgs?: any,
    reason?: string
  ) => {
    try {
      await fetch('/api/hitl/decide', {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({
          ticket_id: ticketId,
          action,
          modified_arguments: modifiedArgs,
          reason,
        }),
      });
      setPendingTickets(prev => prev.filter(t => t.ticket_id !== ticketId));
      aux.refreshAuditData();
    } catch (e) {
      console.error(e);
    }
  };

  return {
    authStatus,
    isAuthenticated,
    authChecking,
    activeTab,
    setActiveTab,
    showTelemetryStrip,
    setShowTelemetryStrip,
    telemetry,
    pendingTickets,
    auditOpen,
    setAuditOpen,
    activeNotification,
    dismissNotification: () => setActiveNotification(null),
    ...chat,
    ...aux,
    handleResolveTicket,
    handleAuthenticated,
    handleLock,
    handleStartOnboarding,
    handleDismissOnboarding,
  };
};
