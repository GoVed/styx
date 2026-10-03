import { describe, it, expect } from 'vitest';
import {
  extractOptionsAndContent,
  parseToolEvent,
} from '../components/ChatView';

describe('ChatView Parsing Utilities', () => {
  it('extractOptionsAndContent correctly parses tags, flags Other, and auto-appends Other if missing', () => {
    // Case 1: XML tags with explicit other
    const raw1 = `Welcome to Styx!
<options>
<option>Software Engineer</option>
<option>DevOps / Cloud</option>
<option other="true">Other (custom role)...</option>
</options>`;

    const parsed1 = extractOptionsAndContent(raw1);
    expect(parsed1.content).toBe('Welcome to Styx!');
    expect(parsed1.options.length).toBe(3);
    expect(parsed1.options[0].label).toBe('Software Engineer');
    expect(parsed1.options[0].isOther).toBeFalsy();
    expect(parsed1.options[2].label).toBe('Other (custom role)...');
    expect(parsed1.options[2].isOther).toBe(true);

    // Case 2: Markdown lines inside options and auto-appends Other
    const raw2 = `Select a tone:
<options>
- Concise & Technical
- Conversational
</options>`;

    const parsed2 = extractOptionsAndContent(raw2);
    expect(parsed2.content).toBe('Select a tone:');
    expect(parsed2.options.length).toBe(3); // 2 + auto-appended Other
    expect(parsed2.options[0].label).toBe('Concise & Technical');
    expect(parsed2.options[2].isOther).toBe(true);
  });

  it('extractOptionsAndContent does not get stuck at "keeping the" when <options> is mentioned in text', () => {
    const raw = `I should introduce myself warmly, keeping the <options> block consistent with the system instructions.

I'm Styx, your personal assistant! I'm here to help with tasks and notes.

<options>
<option>Help me set a reminder</option>
<option>Jot down a quick note</option>
<`;
    const res = extractOptionsAndContent(raw);
    expect(res.content).toContain("I'm Styx, your personal assistant!");
    expect(res.options.length).toBeGreaterThanOrEqual(2);
    expect(res.options[0].label).toBe('Help me set a reminder');
    expect(res.options[1].label).toBe('Jot down a quick note');
  });

  it('parseToolEvent parses standard, generic protocol, and legacy formats correctly', () => {
    // 1. Standard protocol-agnostic Styx format
    const stdFormat = `[INCOMING TOOL EVENT: WHATSAPP]
Sender: Sarah
Channel: Direct Chat (155500011122233@lid)
Message: "Hey Operator, how are you doing?"`;
    const parsedStd = parseToolEvent(stdFormat);
    expect(parsedStd).not.toBeNull();
    expect(parsedStd?.protocol).toBe('WhatsApp');
    expect(parsedStd?.senderName).toBe('Sarah');
    expect(parsedStd?.channel).toBe('Direct Chat (155500011122233@lid)');
    expect(parsedStd?.messageText).toBe('Hey Operator, how are you doing?');
    expect(parsedStd?.isGroup).toBe(false);
    expect(parsedStd?.replyTo).toBeUndefined();

    // 1b. Standard format with Replying To quoted context
    const stdReplyFormat = `[INCOMING TOOL EVENT: WHATSAPP]
Sender: Sarah
Channel: Direct Chat (155500011122233@lid)
Replying To: "Want to grab coffee later?" (by Operator)
Message: "Yeah sure, let's meet at 4!"`;
    const parsedReply = parseToolEvent(stdReplyFormat);
    expect(parsedReply).not.toBeNull();
    expect(parsedReply?.protocol).toBe('WhatsApp');
    expect(parsedReply?.senderName).toBe('Sarah');
    expect(parsedReply?.messageText).toBe("Yeah sure, let's meet at 4!");
    expect(parsedReply?.replyTo).toBe('"Want to grab coffee later?" (Operator)');

    // 2. Generic dynamically-registered tool protocol (e.g. Slack)
    const slackFormat = `[INCOMING TOOL EVENT: SLACK]
Sender: Alice
Channel: Group: #engineering
Message: "Build deployed successfully"`;
    const parsedSlack = parseToolEvent(slackFormat);
    expect(parsedSlack).not.toBeNull();
    expect(parsedSlack?.protocol).toBe('Slack');
    expect(parsedSlack?.senderName).toBe('Alice');
    expect(parsedSlack?.channel).toBe('Group: #engineering');
    expect(parsedSlack?.messageText).toBe('Build deployed successfully');
    expect(parsedSlack?.isGroup).toBe(true);

    // 3. Bell legacy format
    const bellFormat = `🔔 **Incoming WhatsApp Message from 'Tech News' (ID: 120363000000000002@newsletter):**
"Release v2.0 is now live!"

Raw Metadata:
\`\`\`json
{"from": "120363000000000002@newsletter"}
\`\`\``;
    const parsedBell = parseToolEvent(bellFormat);
    expect(parsedBell).not.toBeNull();
    expect(parsedBell?.protocol).toBe('WhatsApp');
    expect(parsedBell?.senderName).toBe('Tech News');
    expect(parsedBell?.channel).toBe('120363000000000002@newsletter');
    expect(parsedBell?.messageText).toBe('Release v2.0 is now live!');

    // 4. Historical format
    const histFormat = `[INCOMING TOOL EVENT: whatsapp/new_message]
Source: Sarah (155500011122233@lid)
Payload:
{
  "sender": "155500011122233@lid",
  "pushName": "Sarah",
  "message": "hello styx!"
}`;
    const parsedHist = parseToolEvent(histFormat);
    expect(parsedHist).not.toBeNull();
    expect(parsedHist?.protocol).toBe('WhatsApp');
    expect(parsedHist?.senderName).toBe('Sarah (155500011122233@lid)');
    expect(parsedHist?.messageText).toBe('hello styx!');
  });
});
