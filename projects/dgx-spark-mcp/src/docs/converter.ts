/**
 * HTML to Markdown Converter
 * Converts HTML documentation to markdown format
 * DGX-Spark MCP Server - Workstream 4
 */

/**
 * Convert HTML to Markdown
 * Simple implementation without external dependencies
 */
export function htmlToMarkdown(html: string): string {
  let markdown = html;

  // Remove script and style tags
  markdown = markdown.replace(/<script\b[^<]*(?:(?!<\/script>)<[^<]*)*<\/script>/gi, '');
  markdown = markdown.replace(/<style\b[^<]*(?:(?!<\/style>)<[^<]*)*<\/style>/gi, '');

  // Convert headings
  markdown = markdown.replace(/<h1[^>]*>(.*?)<\/h1>/gi, '# $1\n\n');
  markdown = markdown.replace(/<h2[^>]*>(.*?)<\/h2>/gi, '## $1\n\n');
  markdown = markdown.replace(/<h3[^>]*>(.*?)<\/h3>/gi, '### $1\n\n');
  markdown = markdown.replace(/<h4[^>]*>(.*?)<\/h4>/gi, '#### $1\n\n');
  markdown = markdown.replace(/<h5[^>]*>(.*?)<\/h5>/gi, '##### $1\n\n');
  markdown = markdown.replace(/<h6[^>]*>(.*?)<\/h6>/gi, '###### $1\n\n');

  // Convert bold and italic
  markdown = markdown.replace(/<strong[^>]*>(.*?)<\/strong>/gi, '**$1**');
  markdown = markdown.replace(/<b[^>]*>(.*?)<\/b>/gi, '**$1**');
  markdown = markdown.replace(/<em[^>]*>(.*?)<\/em>/gi, '*$1*');
  markdown = markdown.replace(/<i[^>]*>(.*?)<\/i>/gi, '*$1*');

  // Convert links
  markdown = markdown.replace(/<a\s+href=["']([^"']+)["'][^>]*>(.*?)<\/a>/gi, '[$2]($1)');

  // Convert images
  markdown = markdown.replace(/<img\s+src=["']([^"']+)["'][^>]*alt=["']([^"']+)["'][^>]*>/gi, '![$2]($1)');
  markdown = markdown.replace(/<img\s+alt=["']([^"']+)["'][^>]*src=["']([^"']+)["'][^>]*>/gi, '![$1]($2)');
  markdown = markdown.replace(/<img\s+src=["']([^"']+)["'][^>]*>/gi, '![]($1)');

  // Convert code blocks
  markdown = markdown.replace(/<pre[^>]*><code[^>]*class=["']language-(\w+)["'][^>]*>(.*?)<\/code><\/pre>/gis, '```$1\n$2\n```\n\n');
  markdown = markdown.replace(/<pre[^>]*><code[^>]*>(.*?)<\/code><\/pre>/gis, '```\n$1\n```\n\n');
  markdown = markdown.replace(/<code[^>]*>(.*?)<\/code>/gi, '`$1`');

  // Convert lists
  markdown = markdown.replace(/<ul[^>]*>(.*?)<\/ul>/gis, (match, content) => {
    const items = content.match(/<li[^>]*>(.*?)<\/li>/gi);
    if (items) {
      return items.map((item: string) => {
        const text = item.replace(/<li[^>]*>(.*?)<\/li>/i, '$1').trim();
        return `- ${text}`;
      }).join('\n') + '\n\n';
    }
    return match;
  });

  markdown = markdown.replace(/<ol[^>]*>(.*?)<\/ol>/gis, (match, content) => {
    const items = content.match(/<li[^>]*>(.*?)<\/li>/gi);
    if (items) {
      return items.map((item: string, index: number) => {
        const text = item.replace(/<li[^>]*>(.*?)<\/li>/i, '$1').trim();
        return `${index + 1}. ${text}`;
      }).join('\n') + '\n\n';
    }
    return match;
  });

  // Convert blockquotes
  markdown = markdown.replace(/<blockquote[^>]*>(.*?)<\/blockquote>/gis, (_match, content) => {
    const lines = content.trim().split('\n');
    return lines.map((line: string) => `> ${line.trim()}`).join('\n') + '\n\n';
  });

  // Convert horizontal rules
  markdown = markdown.replace(/<hr[^>]*>/gi, '\n---\n\n');

  // Convert paragraphs
  markdown = markdown.replace(/<p[^>]*>(.*?)<\/p>/gi, '$1\n\n');

  // Convert line breaks
  markdown = markdown.replace(/<br\s*\/?>/gi, '\n');

  // Remove remaining HTML tags
  markdown = markdown.replace(/<[^>]+>/g, '');

  // Decode HTML entities
  markdown = decodeHtmlEntities(markdown);

  // Clean up extra whitespace
  markdown = markdown.replace(/\n{3,}/g, '\n\n');
  markdown = markdown.trim();

  return markdown;
}

/**
 * Decode HTML entities
 */
function decodeHtmlEntities(text: string): string {
  const entities: Record<string, string> = {
    '&amp;': '&',
    '&lt;': '<',
    '&gt;': '>',
    '&quot;': '"',
    '&#39;': "'",
    '&nbsp;': ' ',
    '&copy;': '©',
    '&reg;': '®',
    '&trade;': '™',
  };

  let decoded = text;
  for (const [entity, char] of Object.entries(entities)) {
    decoded = decoded.replace(new RegExp(entity, 'g'), char);
  }

  // Decode numeric entities
  decoded = decoded.replace(/&#(\d+);/g, (_match, dec) => {
    return String.fromCharCode(parseInt(dec, 10));
  });

  decoded = decoded.replace(/&#x([0-9a-f]+);/gi, (_match, hex) => {
    return String.fromCharCode(parseInt(hex, 16));
  });

  return decoded;
}

/**
 * Extract text content from HTML
 */
export function extractTextFromHtml(html: string): string {
  let text = html;

  // Remove script and style tags
  text = text.replace(/<script\b[^<]*(?:(?!<\/script>)<[^<]*)*<\/script>/gi, '');
  text = text.replace(/<style\b[^<]*(?:(?!<\/style>)<[^<]*)*<\/style>/gi, '');

  // Remove all HTML tags
  text = text.replace(/<[^>]+>/g, ' ');

  // Decode entities
  text = decodeHtmlEntities(text);

  // Clean up whitespace
  text = text.replace(/\s+/g, ' ').trim();

  return text;
}

/**
 * Clean markdown content
 */
export function cleanMarkdown(markdown: string): string {
  let cleaned = markdown;

  // Remove excessive blank lines
  cleaned = cleaned.replace(/\n{4,}/g, '\n\n\n');

  // Ensure proper spacing around headings
  cleaned = cleaned.replace(/([^\n])\n(#{1,6}\s)/g, '$1\n\n$2');
  cleaned = cleaned.replace(/(#{1,6}\s[^\n]+)\n([^\n])/g, '$1\n\n$2');

  // Ensure proper spacing around code blocks
  cleaned = cleaned.replace(/([^\n])\n```/g, '$1\n\n```');
  cleaned = cleaned.replace(/```\n([^\n])/g, '```\n\n$1');

  // Trim whitespace from lines
  cleaned = cleaned.split('\n').map(line => line.trimEnd()).join('\n');

  return cleaned.trim();
}
