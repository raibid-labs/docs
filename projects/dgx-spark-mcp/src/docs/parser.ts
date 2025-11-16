/**
 * Markdown Parser
 * Parses markdown files and extracts structured content
 * DGX-Spark MCP Server - Workstream 4
 */

import * as fs from 'fs/promises';
import * as path from 'path';
import { ParsedDocument, DocumentHeading, CodeBlock, DocumentLink } from '../types/docs.js';
import { extract as extractFrontmatter } from './frontmatter.js';

/**
 * Parse markdown file and extract all structured data
 */
export async function parseMarkdown(filePath: string): Promise<ParsedDocument> {
  const rawContent = await fs.readFile(filePath, 'utf-8');
  const id = generateDocId(filePath);

  const { metadata, content } = extractFrontmatter(rawContent);
  const headings = extractHeadings(content);
  const codeBlocks = extractCodeBlocks(content);
  const links = extractLinks(content);

  return {
    id,
    filePath,
    metadata,
    content,
    rawContent,
    headings,
    codeBlocks,
    links,
  };
}

/**
 * Generate unique document ID from file path
 */
function generateDocId(filePath: string): string {
  // Use relative path from docs directory as ID
  const normalized = filePath.replace(/\\/g, '/');
  const parts = normalized.split('/docs/');
  if (parts.length > 1 && parts[1]) {
    return parts[1].replace(/\.md$/, '');
  }
  return path.basename(filePath, '.md');
}

/**
 * Extract headings with hierarchy
 */
export function extractHeadings(markdown: string): DocumentHeading[] {
  const headings: DocumentHeading[] = [];
  const lines = markdown.split('\n');
  let lineNumber = 0;

  for (const line of lines) {
    lineNumber++;

    // Match ATX-style headings (# Heading)
    const atxMatch = line.match(/^(#{1,6})\s+(.+)$/);
    if (atxMatch && atxMatch[1] && atxMatch[2]) {
      const level = atxMatch[1].length;
      const text = atxMatch[2].trim();
      const id = generateHeadingId(text);

      headings.push({
        level,
        text,
        id,
      });
      continue;
    }

    // Match Setext-style headings (Heading\n====)
    if (lineNumber < lines.length) {
      const nextLine = lines[lineNumber];
      if (nextLine && /^=+$/.test(nextLine)) {
        headings.push({
          level: 1,
          text: line.trim(),
          id: generateHeadingId(line.trim()),
        });
      } else if (nextLine && /^-+$/.test(nextLine)) {
        headings.push({
          level: 2,
          text: line.trim(),
          id: generateHeadingId(line.trim()),
        });
      }
    }
  }

  // Build hierarchy
  return buildHeadingHierarchy(headings);
}

/**
 * Generate heading ID for anchor links
 */
function generateHeadingId(text: string): string {
  return text
    .toLowerCase()
    .replace(/[^\w\s-]/g, '')
    .replace(/\s+/g, '-')
    .replace(/-+/g, '-')
    .replace(/^-|-$/g, '');
}

/**
 * Build hierarchical heading structure
 */
function buildHeadingHierarchy(flatHeadings: DocumentHeading[]): DocumentHeading[] {
  const root: DocumentHeading[] = [];
  const stack: DocumentHeading[] = [];

  for (const heading of flatHeadings) {
    // Find parent heading
    let stackTop = stack[stack.length - 1];
    while (stack.length > 0 && stackTop && stackTop.level >= heading.level) {
      stack.pop();
      stackTop = stack[stack.length - 1];
    }

    if (stack.length === 0) {
      // Top-level heading
      root.push(heading);
    } else {
      // Child heading
      const parent = stack[stack.length - 1];
      if (parent) {
        if (!parent.children) {
          parent.children = [];
        }
        parent.children.push(heading);
      }
    }

    stack.push(heading);
  }

  return root;
}

/**
 * Extract code blocks from markdown
 */
export function extractCodeBlocks(markdown: string): CodeBlock[] {
  const codeBlocks: CodeBlock[] = [];
  const lines = markdown.split('\n');
  let inCodeBlock = false;
  let currentBlock: Partial<CodeBlock> = {};
  let blockLines: string[] = [];

  for (let i = 0; i < lines.length; i++) {
    const line = lines[i];
    if (!line) continue;

    // Check for fenced code block
    const fenceMatch = line.match(/^```(\w+)?/);
    if (fenceMatch) {
      if (!inCodeBlock) {
        // Start of code block
        inCodeBlock = true;
        currentBlock = {
          language: fenceMatch[1] || 'text',
          lineStart: i + 1,
        };
        blockLines = [];
      } else {
        // End of code block
        inCodeBlock = false;
        currentBlock.code = blockLines.join('\n');
        currentBlock.lineEnd = i;
        codeBlocks.push(currentBlock as CodeBlock);
        currentBlock = {};
        blockLines = [];
      }
      continue;
    }

    if (inCodeBlock) {
      blockLines.push(line);
    }
  }

  return codeBlocks;
}

/**
 * Extract links from markdown
 */
export function extractLinks(markdown: string): DocumentLink[] {
  const links: DocumentLink[] = [];

  // Match markdown links: [text](url)
  const linkRegex = /\[([^\]]+)\]\(([^)]+)\)/g;
  let match;

  while ((match = linkRegex.exec(markdown)) !== null) {
    const text = match[1];
    const url = match[2];
    if (!text || !url) continue;
    const isExternal = /^https?:\/\//.test(url);

    links.push({
      text,
      url,
      isExternal,
    });
  }

  return links;
}

/**
 * Strip markdown formatting to get plain text
 */
export function stripMarkdown(markdown: string): string {
  let text = markdown;

  // Remove code blocks
  text = text.replace(/```[\s\S]*?```/g, '');

  // Remove inline code
  text = text.replace(/`[^`]+`/g, '');

  // Remove images
  text = text.replace(/!\[([^\]]*)\]\([^)]+\)/g, '$1');

  // Remove links but keep text
  text = text.replace(/\[([^\]]+)\]\([^)]+\)/g, '$1');

  // Remove headings markers
  text = text.replace(/^#{1,6}\s+/gm, '');

  // Remove bold and italic
  text = text.replace(/(\*\*|__)(.*?)\1/g, '$2');
  text = text.replace(/(\*|_)(.*?)\1/g, '$2');

  // Remove blockquotes
  text = text.replace(/^>\s+/gm, '');

  // Remove horizontal rules
  text = text.replace(/^(-{3,}|_{3,}|\*{3,})$/gm, '');

  // Remove list markers
  text = text.replace(/^[\s]*[-*+]\s+/gm, '');
  text = text.replace(/^[\s]*\d+\.\s+/gm, '');

  // Collapse multiple newlines
  text = text.replace(/\n{3,}/g, '\n\n');

  return text.trim();
}

/**
 * Extract excerpt from markdown content
 */
export function extractExcerpt(markdown: string, maxLength: number = 200): string {
  const plainText = stripMarkdown(markdown);

  if (plainText.length <= maxLength) {
    return plainText;
  }

  // Find natural break point (sentence end)
  const truncated = plainText.substring(0, maxLength);
  const lastPeriod = truncated.lastIndexOf('.');
  const lastExclamation = truncated.lastIndexOf('!');
  const lastQuestion = truncated.lastIndexOf('?');

  const lastSentenceEnd = Math.max(lastPeriod, lastExclamation, lastQuestion);

  if (lastSentenceEnd > maxLength * 0.5) {
    return truncated.substring(0, lastSentenceEnd + 1);
  }

  // Fall back to word boundary
  const lastSpace = truncated.lastIndexOf(' ');
  if (lastSpace > 0) {
    return truncated.substring(0, lastSpace) + '...';
  }

  return truncated + '...';
}
