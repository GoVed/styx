import { ChatSession, ChatMessage, ApprovalTicket, MemoryFileNode } from '../../types';

export interface InteractiveOption {
  id: string;
  label: string;
  isOther?: boolean;
}

export interface ParsedToolEvent {
  protocol: string;
  senderName: string;
  channel: string;
  messageText: string;
  isGroup: boolean;
  replyTo?: string;
  rawContent: string;
}

export interface ToolExecutionCard {
  id: string;
  tool_name: string;
  arguments: any;
  output?: string;
  success?: boolean;
  status: 'running' | 'pending_approval' | 'approved' | 'rejected' | 'completed';
  ticket?: ApprovalTicket;
}

export interface ChatViewProps {
  sessions: ChatSession[];
  activeSession: ChatSession | null;
  onSelectSession: (session: ChatSession) => void;
  onCreateSession: (mode: 'chat' | 'mission') => void;
  onDeleteSession: (id: string) => void;
  pendingTickets: ApprovalTicket[];
  onResolveTicket: (
    ticketId: string,
    action: 'APPROVE' | 'REJECT' | 'MODIFY',
    modifiedArgs?: any,
    reason?: string
  ) => void;
  memoryFiles: MemoryFileNode[];
  onSendPrompt: (sessionId: string, prompt: string, mode: string) => void;
  liveThought: string;
  liveStreamingText: string;
  liveToolExecutions: ToolExecutionCard[];
  isStreaming: boolean;
  chatError?: string | null;
  onClearChatError?: () => void;
  messages?: ChatMessage[];
  onReloadMessages?: () => void;
  onNavigateToSettings?: () => void;
}
