import { useState, useEffect, useRef, useCallback } from 'react';
import {
  ApprovalTicket,
  ChatSession,
  ChatMessage,
  AuthStatus,
} from '../types';

interface UseChatStateProps {
  refreshAuditData: () => void;
  setAuthStatus: React.Dispatch<React.SetStateAction<AuthStatus | null>>;
  setPendingTickets: React.Dispatch<React.SetStateAction<ApprovalTicket[]>>;
}

export const useChatState = ({
  refreshAuditData,
  setAuthStatus,
  setPendingTickets,
}: UseChatStateProps) => {
  const [sessions, setSessions] = useState<ChatSession[]>([]);
  const [activeSession, setActiveSession] = useState<ChatSession | null>(null);
  const [messages, setMessages] = useState<ChatMessage[]>([]);
  const [liveThought, setLiveThought] = useState('');
  const [liveStreamingText, setLiveStreamingText] = useState('');
  const [liveToolExecutions, setLiveToolExecutions] = useState<any[]>([]);
  const [isStreaming, setIsStreaming] = useState(false);
  const [chatError, setChatError] = useState<string | null>(null);

  const activeSessionRef = useRef<ChatSession | null>(null);
  activeSessionRef.current = activeSession;

  const loadMessages = useCallback(async (sessionId: string) => {
    try {
      const res = await fetch(`/api/chat/sessions/${sessionId}/messages`);
      const data = await res.json();
      if (data.success) {
        setMessages(data.messages);
      }
    } catch (e) {
      console.error('Failed to load messages', e);
    }
  }, []);

  useEffect(() => {
    setIsStreaming(false);
    setLiveStreamingText('');
    setLiveThought('');
    setLiveToolExecutions([]);
    setChatError(null);
    if (activeSession?.id) {
      loadMessages(activeSession.id);
    } else {
      setMessages([]);
    }
  }, [activeSession?.id, loadMessages]);

  // Streaming watchdog: polls messages if turn finishes via background without socket done
  useEffect(() => {
    if (!isStreaming || !activeSession?.id) return;

    const interval = setInterval(async () => {
      try {
        const res = await fetch(`/api/chat/sessions/${activeSession.id}/messages`);
        const data = await res.json();
        if (data.success && Array.isArray(data.messages) && data.messages.length > 0) {
          const lastMsg = data.messages[data.messages.length - 1];
          if (lastMsg && lastMsg.role === 'assistant') {
            setMessages(data.messages);
            setIsStreaming(false);
            setLiveStreamingText('');
            setLiveThought('');
            setLiveToolExecutions([]);
          }
        }
      } catch (err) {
        console.error('Watchdog polling error', err);
      }
    }, 1000);

    return () => clearInterval(interval);
  }, [isStreaming, activeSession?.id]);

  const handleCreateSession = useCallback(async (mode: 'chat' | 'mission') => {
    try {
      const res = await fetch('/api/chat/sessions', {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({ mode }),
      });
      const data = await res.json();
      if (data.success) {
        setSessions(prev => [data.session, ...prev]);
        setActiveSession(data.session);
      }
    } catch (e) {
      console.error(e);
    }
  }, []);

  const handleDeleteSession = async (id: string) => {
    try {
      await fetch(`/api/chat/sessions/${id}`, { method: 'DELETE' });
      setSessions(prev => prev.filter(s => s.id !== id));
      if (activeSession?.id === id) {
        setActiveSession(sessions.find(s => s.id !== id) || null);
      }
    } catch (e) {
      console.error(e);
    }
  };

  const handleSendPrompt = async (sessionId: string, prompt: string, mode: string) => {
    const optimisticMsg: ChatMessage = {
      id: `temp-${Date.now()}`,
      session_id: sessionId,
      role: 'user',
      content: prompt,
      thought: null,
      created_at: new Date().toISOString(),
    };
    setMessages(prev => [...prev, optimisticMsg]);
    setIsStreaming(true);
    setChatError(null);
    setLiveStreamingText('');
    setLiveThought('');
    setLiveToolExecutions([]);

    try {
      const res = await fetch('/api/chat/send', {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({ session_id: sessionId, prompt, mode }),
      });
      const data = await res.json();
      if (!data.success) {
        setIsStreaming(false);
        setChatError(data.error || 'Failed to dispatch prompt');
        loadMessages(sessionId);
      }
    } catch (e) {
      setIsStreaming(false);
      setChatError(e instanceof Error ? e.message : 'Network communication error');
      console.error(e);
      loadMessages(sessionId);
    }
  };

  const handleIncomingChatEvent = useCallback((evt: any, eventSessionId?: string) => {
    const isForActiveSession = !eventSessionId || eventSessionId === activeSessionRef.current?.id;

    if (evt.type === 'onboarding_completed') {
      setAuthStatus(prev => (prev ? { ...prev, onboarded: true } : null));
      return;
    }

    if (evt.type === 'tool_pending_approval') {
      setPendingTickets(prev => [
        ...prev.filter(t => t.ticket_id !== evt.ticket.ticket_id),
        evt.ticket,
      ]);
      if (isForActiveSession) {
        setLiveToolExecutions(prev =>
          prev.map(item =>
            item.tool_name === evt.ticket.tool_name ? { ...item, status: 'pending_approval' } : item
          )
        );
      }
      return;
    } else if (evt.type === 'tool_approved') {
      if (isForActiveSession) {
        setLiveToolExecutions(prev =>
          prev.map(item =>
            item.id === evt.ticket_id ? { ...item, status: 'approved' } : item
          )
        );
      }
      return;
    } else if (evt.type === 'tool_rejected') {
      if (isForActiveSession) {
        setLiveToolExecutions(prev =>
          prev.map(item =>
            item.id === evt.ticket_id ? { ...item, status: 'rejected' } : item
          )
        );
      }
      return;
    }

    if (!isForActiveSession) {
      if (evt.type === 'queued' || evt.type === 'queue_started' || evt.type === 'done') {
        fetch('/api/chat/sessions')
          .then(r => r.json())
          .then(sData => {
            if (sData.success && Array.isArray(sData.sessions)) {
              setSessions(sData.sessions);
            }
          })
          .catch(() => {});
      }
      if (evt.type === 'tool_completed') {
        refreshAuditData();
      }
      return;
    }

    const targetSessionId = activeSessionRef.current?.id;

    if (evt.type === 'queued') {
      setIsStreaming(true);
      setLiveStreamingText(`⏳ Queued in GPU scheduler (Position ${evt.queue_position}/${evt.total_queued}). Waiting for inference slot...`);
    } else if (evt.type === 'queue_started') {
      setIsStreaming(true);
      setLiveStreamingText('');
      setLiveThought('');
      if (targetSessionId) {
        loadMessages(targetSessionId);
      }
    } else if (evt.type === 'thought') {
      setIsStreaming(true);
      setLiveThought(prev => prev + (evt.content || ''));
    } else if (evt.type === 'token') {
      setIsStreaming(true);
      setLiveStreamingText(prev => prev + (evt.content || ''));
    } else if (evt.type === 'tool_call_started') {
      setLiveToolExecutions(prev => [
        ...prev,
        {
          id: evt.id,
          tool_name: evt.tool_name,
          arguments: evt.arguments,
          status: 'running',
        },
      ]);
    } else if (evt.type === 'tool_completed') {
      setLiveToolExecutions(prev =>
        prev.map(item =>
          item.id === evt.id
            ? { ...item, status: 'completed', output: evt.output, success: evt.success }
            : item
        )
      );
      refreshAuditData();
    } else if (evt.type === 'done') {
      if (targetSessionId) {
        fetch(`/api/chat/sessions/${targetSessionId}/messages`)
          .then(r => r.json())
          .then(d => {
            if (d.success && Array.isArray(d.messages)) {
              setMessages(d.messages);
            }
          })
          .catch(err => {
            console.error('Failed to reload messages on done:', err);
          })
          .finally(() => {
            setIsStreaming(false);
            setLiveStreamingText('');
            setLiveThought('');
            setLiveToolExecutions([]);
            fetch('/api/auth/status')
              .then(r => r.json())
              .then(s => setAuthStatus(s))
              .catch(() => {});
            fetch('/api/chat/sessions')
              .then(r => r.json())
              .then(sData => {
                if (sData.success && Array.isArray(sData.sessions)) {
                  setSessions(sData.sessions);
                }
              })
              .catch(() => {});
          });
      } else {
        setIsStreaming(false);
        setLiveStreamingText('');
        setLiveThought('');
        setLiveToolExecutions([]);
      }
    } else if (evt.type === 'error') {
      setIsStreaming(false);
      setLiveStreamingText('');
      setLiveThought('');
      setLiveToolExecutions([]);
      setChatError(evt.message || 'Inference engine connection error. The model may still be initializing.');
      if (targetSessionId) {
        loadMessages(targetSessionId);
      }
    }
  }, [loadMessages, refreshAuditData, setAuthStatus, setPendingTickets]);

  return {
    sessions,
    setSessions,
    activeSession,
    setActiveSession,
    messages,
    setMessages,
    liveThought,
    liveStreamingText,
    liveToolExecutions,
    isStreaming,
    setIsStreaming,
    chatError,
    setChatError,
    loadMessages,
    handleCreateSession,
    handleDeleteSession,
    handleSendPrompt,
    handleIncomingChatEvent,
  };
};
