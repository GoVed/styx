import { marked } from 'marked';
import { InteractiveOption, ParsedToolEvent } from './types';

marked.setOptions({
  breaks: true,
  gfm: true,
});

marked.use({
  renderer: {
    link({ href, title, text }: { href: string; title?: string | null; text: string }) {
      if (href && href.startsWith('file://')) {
        const cleanPath = href.replace(/^file:\/\//, '');
        return `<code class="px-1.5 py-0.5 rounded text-[11px] font-mono bg-styx-900 text-emerald-400 border border-styx-700 select-all inline-block" title="Local file: ${cleanPath}">${text || cleanPath}</code>`;
      }
      const titleAttr = title ? ` title="${title}"` : '';
      return `<a href="${href}" target="_blank" rel="noopener noreferrer"${titleAttr} class="text-emerald-400 hover:underline">${text}</a>`;
    },
  },
});

export const renderMarkdown = (content: string): string => {
  if (!content) return '';
  try {
    return marked.parse(content) as string;
  } catch (e) {
    return content;
  }
};

export const extractThoughtAndContent = (
  rawContent: string,
  existingThought?: string | null
): { thought: string | null; content: string } => {
  let thought = existingThought && existingThought.trim().length > 0 ? existingThought.trim() : null;
  let content = rawContent || '';

  // 1. Check for <think>...</think> or <thought>...</thought> tags
  const thinkTagRegex = /<(?:think|thought)>([\s\S]*?)<\/(?:think|thought)>/i;
  const match = content.match(thinkTagRegex);
  if (match) {
    const extracted = match[1].trim();
    if (!thought) {
      thought = extracted;
    } else if (!thought.includes(extracted)) {
      thought = `${thought}\n\n${extracted}`;
    }
    content = content.replace(thinkTagRegex, '').trim();
  } else {
    // 1b. Check for orphan closing </think> or </thought> tag (where opening tag was already stripped)
    const orphanCloseMatch = content.match(/^([\s\S]*?)<\/(?:think|thought)>/i);
    if (orphanCloseMatch) {
      const extracted = orphanCloseMatch[1].trim();
      if (!thought) {
        thought = extracted;
      } else if (!thought.includes(extracted)) {
        thought = `${thought}\n\n${extracted}`;
      }
      content = content.replace(/^[\s\S]*?<\/(?:think|thought)>\s*/i, '').trim();
    } else {
      // Check if starts with unclosed <think> or <thought>
      const openThinkMatch = content.match(/^<(?:think|thought)>([\s\S]*)$/i);
      if (openThinkMatch) {
        if (!thought) {
          thought = openThinkMatch[1].trim();
        }
        content = '';
      }
    }
  }

  // 2. Check for "Thinking Process: ... "
  const processRegex = /^(?:Thinking Process|Reasoning Process):\s*([\s\S]*?)(?:\n\n(?=[A-Z0-9#*`])|<\/think>|$)/i;
  const processMatch = content.match(processRegex);
  if (processMatch) {
    const extracted = processMatch[1].trim();
    if (!thought) {
      thought = extracted;
    } else if (!thought.includes(extracted)) {
      thought = `${thought}\n\n${extracted}`;
    }
    content = content.replace(processRegex, '').replace(/<\/think>/g, '').trim();
  }

  // 2b. Check for bracketed thought / reasoning blocks like [Memory Check] ... or [Plan] ...
  const bracketRegex = /^\[(?:Memory Check|Reasoning|Plan|Thought|Internal|Context)\][\s\S]*?(?:\n\n(?=[A-Z0-9#*`]|Hello|Welcome|<options>)|$)/i;
  const bracketMatch = content.match(bracketRegex);
  if (bracketMatch) {
    const extracted = bracketMatch[0].trim();
    if (!thought) {
      thought = extracted;
    } else if (!thought.includes(extracted)) {
      thought = `${thought}\n\n${extracted}`;
    }
    content = content.replace(bracketRegex, '').trim();
  }

  // 3. Extract leading untagged meta-reasoning paragraphs
  const metaReasoningRegex =
    /^(?:\[(?:Memory Check|Reasoning|Plan|Thought|Internal|Context)\]|Okay[,.]|Alright[,.]|Looking at|The user|Since (?:I|there)|I should|I shouldn't|I need to|I'll|I will|Let me|Based on|From (?:my |the )?(?:memory|scratchpad)|Thinking Process|Reasoning Process|The daily log|As (?:Styx|an AI)|However, I (?:don't|do not)|I don't have access|The memory only contains|Given (?:the |this )?|Considering (?:the |this )?|In this (?:conversation|context)|To respond to the user)/i;

  const isMetaPara = (p: string): boolean => {
    const pTrim = p.trim();
    const lower = pTrim.toLowerCase();
    return (
      metaReasoningRegex.test(pTrim) ||
      ['is asking ', 'has chosen ', 'has asked ', 'wants me to ', 'i should ', 'i need to ', "i'll ", 'i will ', 'system instructions', 'onboarding mode', '<options> block', 'wrap my thinking'].some(s => lower.includes(s)) ||
      pTrim.startsWith('- ') || pTrim.startsWith('* ') || /^\d+\.\s/.test(pTrim)
    );
  };

  const trimmedContent = content.trim();
  const paragraphs = trimmedContent.split(/\n\n+/);
  if (paragraphs.length > 1 && isMetaPara(paragraphs[0])) {
    const thoughtParas: string[] = [];
    const contentParas: string[] = [];
    let inLeadingThought = true;

    for (const para of paragraphs) {
      const pTrim = para.trim();
      if (!pTrim) continue;

      if (inLeadingThought) {
        if (isMetaPara(pTrim)) {
          thoughtParas.push(pTrim);
        } else {
          inLeadingThought = false;
          contentParas.push(pTrim);
        }
      } else {
        contentParas.push(pTrim);
      }
    }

    if (thoughtParas.length > 0 && contentParas.length > 0) {
      const leadingThought = thoughtParas.join('\n\n');
      if (!thought) {
        thought = leadingThought;
      } else if (!thought.includes(leadingThought)) {
        thought = `${thought}\n\n${leadingThought}`;
      }
      content = contentParas.join('\n\n');
    }
  }

  if (thought) {
    thought = thought.replace(/^(?:tags[.:\s]*\n*)+/i, '').trim();
    if (thought.length === 0) thought = null;
  }

  return { thought, content };
};

export const extractOptionsAndContent = (
  rawContent: string
): { content: string; options: InteractiveOption[] } => {
  const options: InteractiveOption[] = [];
  let content = rawContent || '';

  // 1. Parse complete <options>...</options>
  const optionsRegex = /<options>([\s\S]*?)<\/options>/i;
  const match = content.match(optionsRegex);
  if (match) {
    const rawOptionsBlock = match[1];
    content = content.replace(optionsRegex, '').trim();

    // Check for <option ...>...</option> tags
    const optionTagRegex = /<option(?:\s+[^>]*)?>([\s\S]*?)<\/option>/gi;
    let tagMatch;
    let idx = 0;
    while ((tagMatch = optionTagRegex.exec(rawOptionsBlock)) !== null) {
      const fullTag = tagMatch[0];
      const optText = tagMatch[1].trim();
      const isOther =
        fullTag.toLowerCase().includes('other="true"') ||
        optText.toLowerCase().startsWith('other') ||
        optText.toLowerCase().includes('other:');
      if (optText.length > 0) {
        options.push({
          id: `opt-${idx++}`,
          label: cleanOptionLabel(optText, isOther),
          isOther,
        });
      }
    }

    // Fallback: if no <option> tags were found, parse line by line (e.g. - Option 1)
    if (options.length === 0) {
      const lines = rawOptionsBlock.split('\n');
      for (const line of lines) {
        const cleaned = line.replace(/^[\s*\-•\d.]+\s*/, '').trim();
        if (cleaned.length > 0) {
          const isOther =
            cleaned.toLowerCase().startsWith('other') ||
            cleaned.toLowerCase().includes('other:');
          options.push({
            id: `opt-${idx++}`,
            label: cleanOptionLabel(cleaned, isOther),
            isOther,
          });
        }
      }
    }
  } else {
    // 2. Unclosed / partial <options> tag at the tail (e.g. while streaming)
    const lastOpenIdx = content.lastIndexOf('<options>');
    if (lastOpenIdx !== -1) {
      const tailBlock = content.slice(lastOpenIdx);
      const optionTagRegex = /<option(?:\s+[^>]*)?>([\s\S]*?)<\/option>/gi;
      let tagMatch;
      let idx = 0;
      while ((tagMatch = optionTagRegex.exec(tailBlock)) !== null) {
        const fullTag = tagMatch[0];
        const optText = tagMatch[1].trim();
        const isOther =
          fullTag.toLowerCase().includes('other="true"') ||
          optText.toLowerCase().startsWith('other') ||
          optText.toLowerCase().includes('other:');
        if (optText.length > 0) {
          options.push({
            id: `opt-${idx++}`,
            label: cleanOptionLabel(optText, isOther),
            isOther,
          });
        }
      }
      content = content.slice(0, lastOpenIdx).trim();
    }
  }

  // Also strip any dangling unclosed <option... or < tag at the very end of content
  content = content.replace(/<(?:option\b(?:\s+[^>]*)?(?:>[^<]*)?)?$/i, '').trim();

  // If options were found but none was marked 'Other', automatically append an Other option
  if (options.length > 0 && !options.some(o => o.isOther)) {
    options.push({
      id: `opt-${options.length}`,
      label: 'Other (specify details)...',
      isOther: true,
    });
  }

  return { content, options };
};

export function cleanOptionLabel(optText: string, isOther: boolean): string {
  if (isOther) return optText;
  const parenMatch = optText.match(/\(([^()]{5,})\)\s*$/);
  if (parenMatch && /[a-zA-Z]{3,}/.test(parenMatch[1])) {
    return `Translate and send: "${parenMatch[1].trim()}"`;
  }
  if (/^(?:Playful|Jokena|Casual|Joke|Meme|Friendly)?\s*(?:Gujlish|Hinglish|Dialect):\s*/i.test(optText)) {
    return optText.replace(/^(?:Playful|Jokena|Casual|Joke|Meme|Friendly)?\s*(?:Gujlish|Hinglish|Dialect):\s*/i, 'Translate and send: ');
  }
  return optText;
}

function formatProtocol(p: string): string {
  const clean = p.split('/')[0].trim().toLowerCase();
  if (clean === 'whatsapp') return 'WhatsApp';
  if (clean === 'imessage') return 'iMessage';
  return clean.charAt(0).toUpperCase() + clean.slice(1);
}

export function parseToolEvent(content: string): ParsedToolEvent | null {
  if (
    !content.startsWith('[EXTERNAL') &&
    !content.startsWith('[INCOMING TOOL EVENT') &&
    !content.startsWith('[NOTIFICATION: INCOMING') &&
    !content.startsWith('🔔')
  ) {
    return null;
  }

  let protocol = 'External';
  const protoMatch =
    content.match(/\[INCOMING TOOL EVENT:\s*([^\]]+)\]/i) ||
    content.match(/Incoming\s+([A-Za-z0-9_-]+)\s+(?:Group\s+)?(?:Message|Event)/i) ||
    content.match(/\[EXTERNAL\s+([A-Za-z0-9_-]+)/i) ||
    content.match(/\[NOTIFICATION:\s*INCOMING\s+([A-Za-z0-9_-]+)/i);
  if (protoMatch) {
    protocol = formatProtocol(protoMatch[1]);
  }

  let senderName = '';
  let channel = '';
  let messageText = '';
  const isGroup = /group/i.test(content);

  const senderMatch =
    content.match(/(?:•\s*Sender:|\bSender:|\bFrom:)\s*([^\n]+)/i) ||
    content.match(/from\s+'([^']+)'/i) ||
    content.match(/Source:\s*([^\n]+)/i);
  if (senderMatch) senderName = senderMatch[1].trim();

  const channelMatch =
    content.match(/(?:•\s*Recipient \/ Channel:|•\s*Channel:|\bChannel:)\s*([^\n]+)/i) ||
    content.match(/in\s+'([^']+)'/i) ||
    content.match(/\(ID:\s*([^\)]+)\)/i);
  if (channelMatch) channel = channelMatch[1].trim();

  const msgContentMatch =
    content.match(/(?:•\s*Message Content:|\bMessage Content:|\bMessage:)\s*"([^"]+)"/i) ||
    content.match(/(?:•\s*Message Content:|\bMessage Content:|\bMessage:)\s*([^\n]+)/i) ||
    content.match(/\n"([^"]+)"\n/);
  if (msgContentMatch) {
    messageText = msgContentMatch[1].trim();
  } else {
    const jsonMsgMatch = content.match(/"message":\s*"([^"]+)"/);
    if (jsonMsgMatch) messageText = jsonMsgMatch[1].trim();
  }

  let replyTo = '';
  const replyMatch =
    content.match(/(?:•\s*Replying To:|•\s*Replying to:|\bReplying To:|\bReplying to:)\s*"([^"]+)"(?:\s*\((?:by\s+)?([^)]+)\))?/i) ||
    content.match(/(?:•\s*Replying To:|•\s*Replying to:|\bReplying To:|\bReplying to:)\s*([^\n]+)/i);
  if (replyMatch) {
    if (replyMatch[2]) {
      replyTo = `"${replyMatch[1].trim()}" (${replyMatch[2].trim()})`;
    } else {
      replyTo = replyMatch[1].trim().replace(/^"|"$/g, '');
    }
  }

  return {
    protocol,
    senderName: senderName || 'External Contact',
    channel: channel || (isGroup ? 'Group Chat' : 'Direct Chat'),
    messageText: messageText || '(Event or media received)',
    isGroup,
    replyTo: replyTo || undefined,
    rawContent: content,
  };
}
