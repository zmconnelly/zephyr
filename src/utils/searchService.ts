import { invoke } from '@tauri-apps/api/core';

export class SearchService {
  private bangs: [string, string][] = [];
  private currentSuggestions: string[] = [];

  public async getSearchResults(query: string): Promise<string[]> {
    if (query.length === 0) return [];
    return await invoke('search', { query });
  }

  public async getBangs(): Promise<[string, string][]> {
    if (!this.bangs) {
      this.bangs = await invoke<[string, string][]>('get_available_bangs');
    }
    return this.bangs;
  }

  public async getSearchSuggestions(query: string, limit: number = 8): Promise<string[]> {
    if (query.length === 0) return [];

    // If no bang, fetch suggestions for full query
    if (!query.includes('!')) {
      this.currentSuggestions = await invoke<string[]>('get_search_suggestions', { query });
      return this.currentSuggestions.slice(0, limit);
    }

    const bangs = await this.getBangs();

    // If starts with bang, show suggestions from available bangs
    if (query.startsWith('!')) {
      const bang = query.slice(1);

      // Get unique bangs only - keep only the first occurrence of each [0] value
      const uniqueBangs: [string, string][] = [];
      const seenPrefixes = new Set<string>();

      for (const b of bangs) {
        if (!seenPrefixes.has(b[0])) {
          uniqueBangs.push(b);
          seenPrefixes.add(b[0]);
        }
      }

      const possibleBangs = uniqueBangs.filter((b) => b[0].startsWith(bang));
      const sortedBangs = possibleBangs.sort((a, b) => a[0].length - b[0].length);

      const suggestions = sortedBangs.map((b) => b[1]).slice(0, limit);
      const suggestionStrings = suggestions.map((s) => `!${bang} (${s})`);
      return suggestionStrings;
    }

    // If contains a bang, but not at the start, just return current suggestions
    return this.currentSuggestions.slice(0, limit);
  }
}
