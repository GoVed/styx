import { useRef, useCallback, useEffect } from 'react';

export interface WebSocketHandlers {
  onTelemetry: (telemetry: any) => void;
  onConnected: (payload: any) => void;
  onApprovalTicket: (ticket: any) => void;
  onApprovalResolved: (ticketId: string) => void;
  onChatEvent: (event: any, sessionId?: string) => void;
  onNotification?: (notification: any) => void;
  onAuthEvent: () => void;
}

export const useStyxWebSocket = (handlers: WebSocketHandlers) => {
  const wsRef = useRef<WebSocket | null>(null);
  const handlersRef = useRef<WebSocketHandlers>(handlers);
  handlersRef.current = handlers;

  const connectWebSocket = useCallback(() => {
    const protocol = window.location.protocol === 'https:' ? 'wss:' : 'ws:';
    const host = window.location.host;
    const token = localStorage.getItem('styx_access_key') || '';
    const wsUrl = `${protocol}//${host}/ws${token ? `?token=${encodeURIComponent(token)}` : ''}`;

    const ws = new WebSocket(wsUrl);
    wsRef.current = ws;

    ws.onmessage = event => {
      try {
        const data = JSON.parse(event.data);
        const { topic, payload } = data;

        if (topic === 'telemetry') {
          handlersRef.current.onTelemetry(payload);
        } else if (topic === 'connected') {
          handlersRef.current.onConnected(payload);
        } else if (topic === 'approval_ticket') {
          handlersRef.current.onApprovalTicket(payload);
        } else if (topic === 'approval_resolved') {
          handlersRef.current.onApprovalResolved(payload.ticket_id);
        } else if (topic === 'chat_event') {
          handlersRef.current.onChatEvent(data.event, data.session_id);
          if (data.event?.type === 'notification') {
            handlersRef.current.onNotification?.({
              id: `notif-${Date.now()}`,
              title: data.event.title || 'Styx Notification',
              message: data.event.message || '',
              urgency: data.event.urgency || 'action_required',
              sessionId: data.session_id,
              timestamp: new Date().toISOString(),
            });
          }
        } else if (topic === 'notification') {
          handlersRef.current.onNotification?.(payload || data);
        } else if (topic === 'auth_event' || topic === 'onboarding_completed') {
          handlersRef.current.onAuthEvent();
        }
      } catch (e) {
        console.error('WS parse error', e);
      }
    };

    ws.onclose = () => {
      if (localStorage.getItem('styx_access_key')) {
        setTimeout(connectWebSocket, 2000);
      }
    };
  }, []);

  const closeWebSocket = useCallback(() => {
    wsRef.current?.close();
  }, []);

  useEffect(() => {
    return () => {
      wsRef.current?.close();
    };
  }, []);

  return {
    connectWebSocket,
    closeWebSocket,
  };
};
