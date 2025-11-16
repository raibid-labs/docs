/**
 * Documentation search tool
 * Searches documentation by query
 */

import { handleSearchTool } from '../docs/loader.js';
import type { SearchDocumentationArgs, ToolCallResponse } from '../types/tools.js';

/**
 * Search documentation
 */
export async function searchDocumentation(args: SearchDocumentationArgs): Promise<ToolCallResponse> {
  try {
    const { query, limit = 10, topics } = args;

    // Use the existing search handler from WS4
    const results = await handleSearchTool(query, {
      limit,
      topics,
    });

    if (!results || results.length === 0) {
      return {
        content: [{
          type: 'text',
          text: JSON.stringify({
            query,
            totalResults: 0,
            results: [],
            message: 'No documentation found matching your query. Try different keywords or check available topics.',
          }, null, 2),
        }],
      };
    }

    // Format results for consistent output
    const formattedResults = results.map((result: any) => ({
      topic: result.topic || result.document?.topic,
      title: result.title || result.document?.title,
      description: result.description || result.document?.description,
      relevanceScore: result.score || result.relevance,
      excerpt: result.excerpt || result.context,
      uri: result.uri || `dgx://docs/spark/${result.topic || result.document?.topic}`,
    }));

    const response = {
      query,
      totalResults: results.length,
      results: formattedResults,
      suggestions: generateSearchSuggestions(query, formattedResults),
    };

    return {
      content: [{
        type: 'text',
        text: JSON.stringify(response, null, 2),
      }],
    };
  } catch (error) {
    return {
      content: [{
        type: 'text',
        text: JSON.stringify({
          error: 'Failed to search documentation',
          message: error instanceof Error ? error.message : 'Unknown error',
        }, null, 2),
      }],
      isError: true,
    };
  }
}

/**
 * Generate search suggestions based on results
 */
function generateSearchSuggestions(_query: string, results: any[]): string[] {
  const suggestions: string[] = [];

  if (results.length === 0) {
    suggestions.push('Try using more general terms');
    suggestions.push('Check the spelling of your query');
    suggestions.push('Browse available topics instead');
  } else if (results.length > 0) {
    // Extract common topics from results
    const topics = new Set(results.map(r => r.topic).filter(t => t));
    if (topics.size > 1) {
      suggestions.push(`Related topics: ${Array.from(topics).slice(0, 3).join(', ')}`);
    }

    // Suggest reading top result
    if (results[0]) {
      suggestions.push(`Top result: "${results[0].title}" - ${results[0].description || 'No description'}`);
    }
  }

  return suggestions;
}
