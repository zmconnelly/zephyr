import { invoke } from "@tauri-apps/api/core";

export const getSearchSuggestions = async (query: string, limit: number = 8) => {
    if (query.length === 0) return [];

    const suggestions = await invoke<string[]>("get_search_suggestions", { query });
    return suggestions.slice(0, limit);
};

export const getAvailableBangs = async () => {
    const availableBangs = await invoke<[string, string][]>("get_available_bangs");
    return availableBangs;
};

export const executeSearch = async (query: string) => {
    if (query.length === 0) return;
    await invoke("search", { query });
};
