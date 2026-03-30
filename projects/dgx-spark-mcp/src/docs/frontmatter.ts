/**
 * Frontmatter Parser
 * Extracts YAML frontmatter from markdown files
 * DGX-Spark MCP Server - Workstream 4
 */

import { DocumentMetadata } from '../types/docs.js';

/**
 * Extract frontmatter and content from markdown
 */
export function extract(markdown: string): { metadata: DocumentMetadata; content: string } {
  const frontmatterRegex = /^---\s*\n([\s\S]*?)\n---\s*\n([\s\S]*)$/;
  const match = markdown.match(frontmatterRegex);

  if (!match) {
    return {
      metadata: { title: 'Untitled' },
      content: markdown,
    };
  }

  const yamlContent = match[1] || '';
  const markdownContent = match[2] || markdown;

  const metadata = parseYaml(yamlContent);

  return {
    metadata,
    content: markdownContent,
  };
}

/**
 * Simple YAML parser for frontmatter
 * Supports basic key-value pairs, arrays, and multiline strings
 */
function parseYaml(yaml: string): DocumentMetadata {
  const metadata: DocumentMetadata = { title: 'Untitled' };
  const lines = yaml.split('\n');
  let currentKey = '';
  let isMultiline = false;
  let multilineValue = '';

  for (let i = 0; i < lines.length; i++) {
    const line = lines[i];
    if (!line) continue;
    const trimmedLine = line.trim();

    // Skip empty lines and comments
    if (!trimmedLine || trimmedLine.startsWith('#')) {
      continue;
    }

    // Handle multiline strings
    if (isMultiline) {
      if (trimmedLine.match(/^[a-zA-Z_]/)) {
        // New key found, end multiline
        setMetadataValue(metadata, currentKey, multilineValue.trim());
        isMultiline = false;
        multilineValue = '';
      } else {
        multilineValue += ' ' + trimmedLine;
        continue;
      }
    }

    // Parse key-value pairs
    const colonIndex = line.indexOf(':');
    if (colonIndex === -1) {
      continue;
    }

    const key = line.substring(0, colonIndex).trim();
    let value = line.substring(colonIndex + 1).trim();

    // Handle arrays
    if (value === '') {
      // Check if next lines are array items
      const arrayItems: string[] = [];
      let j = i + 1;
      while (j < lines.length) {
        const nextLine = lines[j];
        if (!nextLine) {
          j++;
          continue;
        }
        const trimmedNextLine = nextLine.trim();
        if (trimmedNextLine.startsWith('-')) {
          arrayItems.push(trimmedNextLine.substring(1).trim());
          i = j;
          j++;
        } else if (trimmedNextLine === '' || trimmedNextLine.startsWith('#')) {
          j++;
        } else {
          break;
        }
      }
      if (arrayItems.length > 0) {
        setMetadataValue(metadata, key, arrayItems);
        continue;
      }
    }

    // Handle inline arrays
    if (value.startsWith('[') && value.endsWith(']')) {
      const arrayValue = value
        .substring(1, value.length - 1)
        .split(',')
        .map(item => item.trim().replace(/^["']|["']$/g, ''));
      setMetadataValue(metadata, key, arrayValue);
      continue;
    }

    // Handle quoted strings
    if ((value.startsWith('"') && value.endsWith('"')) ||
        (value.startsWith("'") && value.endsWith("'"))) {
      value = value.substring(1, value.length - 1);
    }

    // Handle multiline indicator
    if (value === '|' || value === '>') {
      isMultiline = true;
      currentKey = key;
      multilineValue = '';
      continue;
    }

    setMetadataValue(metadata, key, value);
  }

  // Handle any remaining multiline value
  if (isMultiline && multilineValue) {
    setMetadataValue(metadata, currentKey, multilineValue.trim());
  }

  return metadata;
}

/**
 * Set metadata value with type safety
 */
function setMetadataValue(metadata: DocumentMetadata, key: string, value: string | string[]): void {
  switch (key) {
    case 'title':
      metadata.title = Array.isArray(value) ? (value[0] || 'Untitled') : value;
      break;
    case 'description':
      metadata.description = Array.isArray(value) ? value.join(' ') : value;
      break;
    case 'tags':
      metadata.tags = Array.isArray(value) ? value : [value];
      break;
    case 'category':
      metadata.category = Array.isArray(value) ? value[0] : value;
      break;
    case 'author':
      metadata.author = Array.isArray(value) ? value[0] : value;
      break;
    case 'dateCreated':
    case 'date_created':
      metadata.dateCreated = Array.isArray(value) ? value[0] : value;
      break;
    case 'dateModified':
    case 'date_modified':
      metadata.dateModified = Array.isArray(value) ? value[0] : value;
      break;
    case 'version':
      metadata.version = Array.isArray(value) ? value[0] : value;
      break;
    case 'relatedDocs':
    case 'related_docs':
      metadata.relatedDocs = Array.isArray(value) ? value : [value];
      break;
  }
}

/**
 * Serialize metadata to YAML frontmatter
 */
export function serialize(metadata: DocumentMetadata): string {
  const lines: string[] = ['---'];

  if (metadata.title) {
    lines.push(`title: ${metadata.title}`);
  }
  if (metadata.description) {
    lines.push(`description: ${metadata.description}`);
  }
  if (metadata.category) {
    lines.push(`category: ${metadata.category}`);
  }
  if (metadata.tags && metadata.tags.length > 0) {
    lines.push(`tags: [${metadata.tags.join(', ')}]`);
  }
  if (metadata.author) {
    lines.push(`author: ${metadata.author}`);
  }
  if (metadata.dateCreated) {
    lines.push(`dateCreated: ${metadata.dateCreated}`);
  }
  if (metadata.dateModified) {
    lines.push(`dateModified: ${metadata.dateModified}`);
  }
  if (metadata.version) {
    lines.push(`version: ${metadata.version}`);
  }
  if (metadata.relatedDocs && metadata.relatedDocs.length > 0) {
    lines.push(`relatedDocs: [${metadata.relatedDocs.join(', ')}]`);
  }

  lines.push('---');
  lines.push('');

  return lines.join('\n');
}
