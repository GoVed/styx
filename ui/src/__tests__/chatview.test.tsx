import { describe, it, expect, vi } from 'vitest';
import { render, screen, act } from '@testing-library/react';
import {
  ChatView,
  extractThoughtAndContent,
  extractOptionsAndContent,
  parseToolEvent,
} from '../components/ChatView';

describe('ChatView Layout & HITL Deterministic Gate Tests', () => {
  const mockSession: ChatSession = {
    id: 'test-session-1',
    title: 'Autonomous Triage & Maintenance',
    mode: 'mission',
    created_at: new Date().toISOString(),
    updated_at: new Date().toISOString(),
  };

  const mockPendingTicket: ApprovalTicket = {
    ticket_id: 'ticket-999',
    session_id: 'test-session-1',
    tool_name: 'write_memory',
    arguments: { path: 'scratchpad/daily_log.md', content: 'Styx is deployed.' },
    risk_level: 'MUTATING',
    status: 'pending',
    created_at: new Date().toISOString(),
  };

  it('ChatView has h-full w-full overflow-hidden and message stream has flex-1 min-h-0', async () => {
    let container!: HTMLElement;
    await act(async () => {
      const res = render(
        <ChatView
          sessions={[mockSession]}
          activeSession={mockSession}
          onSelectSession={() => {}}
          onCreateSession={() => {}}
          onDeleteSession={() => {}}
          pendingTickets={[]}
          onResolveTicket={() => {}}
          memoryFiles={[]}
          onSendPrompt={() => {}}
          liveThought=""
          liveStreamingText=""
          liveToolExecutions={[]}
          isStreaming={false}
        />
      );
      container = res.container;
    });

    const root = container.firstChild as HTMLElement;
    expect(root.className).toContain('h-full');
    expect(root.className).toContain('w-full');
    expect(root.className).toContain('overflow-hidden');

    const mainStream = container.querySelector('main');
    expect(mainStream?.className).toContain('flex-1');
    expect(mainStream?.className).toContain('min-h-0');
    expect(mainStream?.className).toContain('overflow-hidden');
  });

  it('Sidebar session drawer is flex-shrink-0 and active skills selector is flex-shrink-0', async () => {
    let container!: HTMLElement;
    await act(async () => {
      const res = render(
        <ChatView
          sessions={[mockSession]}
          activeSession={mockSession}
          onSelectSession={() => {}}
          onCreateSession={() => {}}
          onDeleteSession={() => {}}
          pendingTickets={[]}
          onResolveTicket={() => {}}
          memoryFiles={[]}
          onSendPrompt={() => {}}
          liveThought=""
          liveStreamingText=""
          liveToolExecutions={[]}
          isStreaming={false}
        />
      );
      container = res.container;
    });

    const aside = container.querySelector('aside');
    expect(aside).toBeInTheDocument();
    expect(aside?.className).toContain('flex-shrink-0');
    expect(aside?.className).toContain('h-full');
    expect(aside?.className).toContain('overflow-hidden');

    const availableSkillsets = screen.getByText('AVAILABLE SKILLSETS');
    const skillContainer = availableSkillsets.closest('div')?.parentElement;
    expect(skillContainer?.className).toContain('flex-shrink-0');
  });

  it('Input bar is flex-shrink-0 so prompt textarea is never pushed off bottom', async () => {
    await act(async () => {
      render(
        <ChatView
          sessions={[mockSession]}
          activeSession={mockSession}
          onSelectSession={() => {}}
          onCreateSession={() => {}}
          onDeleteSession={() => {}}
          pendingTickets={[]}
          onResolveTicket={() => {}}
          memoryFiles={[]}
          onSendPrompt={() => {}}
          liveThought=""
          liveStreamingText=""
          liveToolExecutions={[]}
          isStreaming={false}
        />
      );
    });

    const textarea = screen.getByPlaceholderText(/Define autonomous mission goal/i);
    expect(textarea).toBeInTheDocument();

    const inputBar = textarea.closest('main')?.querySelector('.flex-shrink-0:last-child');
    expect(inputBar).toBeInTheDocument();
    expect(inputBar?.className).toContain('flex-shrink-0');
    expect(inputBar?.className).toContain('bg-styx-900');
  });

  it('Renders inline HITL approval banner with Approve & Continue, Reject, and Edit Arguments buttons', async () => {
    await act(async () => {
      render(
        <ChatView
          sessions={[mockSession]}
          activeSession={mockSession}
          onSelectSession={() => {}}
          onCreateSession={() => {}}
          onDeleteSession={() => {}}
          pendingTickets={[mockPendingTicket]}
          onResolveTicket={() => {}}
          memoryFiles={[]}
          onSendPrompt={() => {}}
          liveThought=""
          liveStreamingText=""
          liveToolExecutions={[]}
          isStreaming={false}
        />
      );
    });

    expect(screen.getByText(/DETERMINISTIC GATE: MUTATING ACTION HALTED/i)).toBeInTheDocument();
    expect(screen.getByText('RISK: MUTATING')).toBeInTheDocument();
    expect(screen.getByRole('button', { name: /Approve & Continue/i })).toBeInTheDocument();
    expect(screen.getByRole('button', { name: /Reject/i })).toBeInTheDocument();
    expect(screen.getByRole('button', { name: /Edit Arguments/i })).toBeInTheDocument();
  });

  it('extractThoughtAndContent extracts <think> tags correctly into thought and cleans content', () => {
    const raw = '<think>\nReview memory index.\nIdentify user intent.\n</think>\n\nAll systems operational.';
    const res = extractThoughtAndContent(raw);
    expect(res.thought).toBe('Review memory index.\nIdentify user intent.');
    expect(res.content).toBe('All systems operational.');
  });

  it('extractThoughtAndContent extracts untagged inner monologue into thought and leaves response', () => {
    const raw = 'The user is asking "so what\'s up?" which is a casual follow-up to my initial status confirmation. I need to respond in my established communication style.\n\nStatus: Idle / Ready\nNo active tasks queued.';
    const res = extractThoughtAndContent(raw);
    expect(res.thought).toContain('The user is asking');
    expect(res.content).toBe('Status: Idle / Ready\nNo active tasks queued.');
  });

  it('Renders reasoning trace block and clean content for assistant message', async () => {
    const assistantMessage: ChatMessage = {
      id: 'msg-assist-1',
      session_id: 'test-session-1',
      role: 'assistant',
      content: '<think>Operator wants status update.</think>Operating at peak efficiency.',
      thought: null,
      created_at: new Date().toISOString(),
    };

    await act(async () => {
      render(
        <ChatView
          sessions={[mockSession]}
          activeSession={mockSession}
          onSelectSession={() => {}}
          onCreateSession={() => {}}
          onDeleteSession={() => {}}
          pendingTickets={[]}
          onResolveTicket={() => {}}
          memoryFiles={[]}
          onSendPrompt={() => {}}
          liveThought=""
          liveStreamingText=""
          liveToolExecutions={[]}
          isStreaming={false}
          messages={[assistantMessage]}
        />
      );
    });

    expect(screen.getByText(/Reasoning Trace/i)).toBeInTheDocument();
    expect(screen.getByText('Operator wants status update.')).toBeInTheDocument();
    expect(screen.getByText('Operating at peak efficiency.')).toBeInTheDocument();
  });

  it('Renders interactive option chips and clicking fires onSendPrompt', async () => {
    const handleSendPrompt = vi.fn();
    const assistantMessageWithOptions: ChatMessage = {
      id: 'msg-opts-1',
      session_id: 'test-session-1',
      role: 'assistant',
      content: `Please select your focus:
<options>
<option>Software Engineer</option>
<option>AI Researcher</option>
<option other="true">Other...</option>
</options>`,
      thought: null,
      created_at: new Date().toISOString(),
    };

    await act(async () => {
      render(
        <ChatView
          sessions={[mockSession]}
          activeSession={mockSession}
          onSelectSession={() => {}}
          onCreateSession={() => {}}
          onDeleteSession={() => {}}
          pendingTickets={[]}
          onResolveTicket={() => {}}
          memoryFiles={[]}
          onSendPrompt={handleSendPrompt}
          liveThought=""
          liveStreamingText=""
          liveToolExecutions={[]}
          isStreaming={false}
          messages={[assistantMessageWithOptions]}
        />
      );
    });

    expect(screen.getByText('CHOOSE AN OPTION OR SPECIFY DETAILS:')).toBeInTheDocument();
    const optBtn = screen.getByText('Software Engineer');
    expect(optBtn).toBeInTheDocument();

    await act(async () => {
      optBtn.click();
    });

    expect(handleSendPrompt).toHaveBeenCalledWith('test-session-1', 'Software Engineer', 'mission');
  });

  it('renders incoming external WhatsApp message as contact card aligned on the left, not OPERATOR', async () => {
    const incomingMsg: ChatMessage = {
      id: 'msg-whatsapp-1',
      session_id: 'test-session-1',
      role: 'user',
      content: `[EXTERNAL WhatsApp MESSAGE RECEIVED]
• Sender: Sarah (155500011122233@lid)
• Recipient / Channel: Direct 1-on-1 Chat with Operator
• Message Content: "wahh!"

OPERATING DIRECTIVES:
1. SENDER IDENTITY: Incoming message sent by "Sarah" to Operator.`,
      thought: null,
      created_at: new Date().toISOString(),
    };

    let container!: HTMLElement;
    await act(async () => {
      const res = render(
        <ChatView
          sessions={[mockSession]}
          activeSession={mockSession}
          onSelectSession={() => {}}
          onCreateSession={() => {}}
          onDeleteSession={() => {}}
          pendingTickets={[]}
          onResolveTicket={() => {}}
          memoryFiles={[]}
          onSendPrompt={() => {}}
          liveThought=""
          liveStreamingText=""
          liveToolExecutions={[]}
          isStreaming={false}
          messages={[incomingMsg]}
        />
      );
      container = res.container;
    });

    // Should NOT have OPERATOR header for this message
    expect(screen.queryByText('OPERATOR')).not.toBeInTheDocument();

    // Should display sender and protocol
    expect(screen.getByText('Sarah (155500011122233@lid)')).toBeInTheDocument();
    expect(screen.getByText('WhatsApp')).toBeInTheDocument();
    expect(screen.getByText('Incoming WhatsApp Message')).toBeInTheDocument();
    expect(screen.getByText('"wahh!"')).toBeInTheDocument();
    expect(screen.getByText('Direct 1-on-1 Chat with Operator')).toBeInTheDocument();

    // Alignment should be items-start (left), not items-end (right)
    const msgWrapper = screen.getByText('Incoming WhatsApp Message').closest('.flex.flex-col');
    expect(msgWrapper?.className).toContain('items-start');
    expect(msgWrapper?.className).not.toContain('items-end');
  });
});
