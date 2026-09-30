import { describe, it, expect } from 'vitest';
import { render } from '@testing-library/react';
import React from 'react';
import { renderMarkdown } from '../components/ChatView';

describe('Markdown Parsing & Typography Rendering Tests', () => {
  it('converts # and ## headers into styled <h1> and <h2> HTML elements', () => {
    const input = '# WhatsApp Communication Protocol\n\n## Style Guidelines';
    const html = renderMarkdown(input);

    const { container } = render(<div dangerouslySetInnerHTML={{ __html: html }} />);
    const h1 = container.querySelector('h1');
    const h2 = container.querySelector('h2');

    expect(h1).toBeInTheDocument();
    expect(h1?.textContent).toBe('WhatsApp Communication Protocol');
    expect(h2).toBeInTheDocument();
    expect(h2?.textContent).toBe('Style Guidelines');
  });

  it('converts bulleted and numbered lists into <ul> and <ol> elements', () => {
    const input = '* Item 1\n* Item 2\n\n1. Numbered 1\n2. Numbered 2';
    const html = renderMarkdown(input);

    const { container } = render(<div dangerouslySetInnerHTML={{ __html: html }} />);
    const ul = container.querySelector('ul');
    const ol = container.querySelector('ol');
    const lis = container.querySelectorAll('li');

    expect(ul).toBeInTheDocument();
    expect(ol).toBeInTheDocument();
    expect(lis.length).toBe(4);
    expect(lis[0].textContent).toBe('Item 1');
    expect(lis[2].textContent).toBe('Numbered 1');
  });

  it('converts bold text (**text**) into <strong> elements', () => {
    const input = '1. **Message Structure:** Start messages with a direct summary.';
    const html = renderMarkdown(input);

    const { container } = render(<div dangerouslySetInnerHTML={{ __html: html }} />);
    const strong = container.querySelector('strong');

    expect(strong).toBeInTheDocument();
    expect(strong?.textContent).toBe('Message Structure:');
  });

  it('converts inline backticks into <code> elements and fenced blocks into <pre><code>', () => {
    const input = 'Call `read_memory(path)` first.\n\n```json\n{"key": "value"}\n```';
    const html = renderMarkdown(input);

    const { container } = render(<div dangerouslySetInnerHTML={{ __html: html }} />);
    const pre = container.querySelector('pre');
    const codeElements = container.querySelectorAll('code');

    expect(pre).toBeInTheDocument();
    expect(codeElements.length).toBe(2);
    expect(codeElements[0].textContent).toBe('read_memory(path)');
    expect(pre?.querySelector('code')?.textContent?.trim()).toBe('{"key": "value"}');
  });

  it('renders fenced code blocks without stripping formatting', () => {
    const input = '```markdown\n# WhatsApp Communication Protocol\n\n## Style Guidelines\n1. **Message Structure:** Start messages with a direct summary\n```';
    const html = renderMarkdown(input);

    const { container } = render(<div dangerouslySetInnerHTML={{ __html: html }} />);
    const pre = container.querySelector('pre');
    const code = container.querySelector('code');

    expect(pre).toBeInTheDocument();
    expect(code).toBeInTheDocument();
    expect(code?.textContent).toContain('# WhatsApp Communication Protocol');
    expect(code?.textContent).toContain('## Style Guidelines');
    expect(code?.textContent).toContain('**Message Structure:**');
  });

  it('handles empty, null, or undefined strings without crashing', () => {
    expect(renderMarkdown('')).toBe('');
    expect(renderMarkdown(null as any)).toBe('');
    expect(renderMarkdown(undefined as any)).toBe('');
  });
});
