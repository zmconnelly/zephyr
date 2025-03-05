<script setup lang="ts">
import { ref, watch, onMounted } from "vue";
import { invoke } from "@tauri-apps/api/core";
import { WebviewWindow } from "@tauri-apps/api/webviewWindow";

// Define a type for the window with dynamic properties
interface ExtendedWindow extends Window {
  [key: string]: any;
}

// Cast window to our extended type
const globalWindow = window as ExtendedWindow;

const searchQuery = ref("");
const searchInput = ref<HTMLInputElement | null>(null);
const suggestions = ref<string[]>([]);
const showSuggestions = ref(false);
const isLoading = ref(false);
const selectedIndex = ref(-1);
const appWindow = WebviewWindow.getCurrent();

// Common search terms for offline suggestions
const commonSearchTerms = [
  "weather",
  "news",
  "maps",
  "translate",
  "youtube",
  "facebook",
  "twitter",
  "instagram",
  "amazon",
  "netflix",
  "spotify",
  "gmail",
  "outlook",
  "linkedin",
  "github",
  "stackoverflow",
  "reddit",
  "wikipedia",
  "dictionary",
  "thesaurus"
];

onMounted(() => {
  // Focus the search input when the component is mounted
  searchInput.value?.focus();
});

appWindow.listen("tauri://focus", () => {
  searchQuery.value = "";
  searchInput.value?.focus();
  suggestions.value = [];
  showSuggestions.value = false;
});

// Debounce function to limit suggestion updates
function debounce<T extends (...args: any[]) => any>(
  fn: T,
  delay: number
): (...args: Parameters<T>) => void {
  let timeout: number | null = null;
  return (...args: Parameters<T>) => {
    if (timeout !== null) {
      clearTimeout(timeout);
    }
    timeout = setTimeout(() => fn(...args), delay) as unknown as number;
  };
}

// Generate suggestions based on input
const generateSuggestions = debounce((query: string) => {
  if (!query.trim()) {
    suggestions.value = [];
    showSuggestions.value = false;
    return;
  }

  isLoading.value = true;
  
  try {
    // Filter common search terms that include the query
    const matchingTerms = commonSearchTerms.filter(term => 
      term.toLowerCase().includes(query.toLowerCase())
    );
    
    // Generate variations of the query
    const variations = [
      query,
      `${query} how to`,
      `${query} meaning`,
      `${query} definition`,
      `${query} near me`,
      `best ${query}`,
      `${query} online`,
      `${query} tutorial`
    ];
    
    // Combine and deduplicate
    const allSuggestions = [...new Set([...matchingTerms, ...variations])];
    
    // Sort by relevance (exact matches first, then by length)
    allSuggestions.sort((a, b) => {
      const aStartsWithQuery = a.toLowerCase().startsWith(query.toLowerCase());
      const bStartsWithQuery = b.toLowerCase().startsWith(query.toLowerCase());
      
      if (aStartsWithQuery && !bStartsWithQuery) return -1;
      if (!aStartsWithQuery && bStartsWithQuery) return 1;
      
      return a.length - b.length;
    });
    
    // Limit to 10 suggestions
    suggestions.value = allSuggestions.slice(0, 10);
    showSuggestions.value = suggestions.value.length > 0;
  } catch (error) {
    console.error('Error generating suggestions:', error);
  } finally {
    isLoading.value = false;
  }
}, 300);

// Watch for changes in the search query
watch(searchQuery, (newQuery) => {
  selectedIndex.value = -1;
  generateSuggestions(newQuery);
});

function selectSuggestion(suggestion: string) {
  searchQuery.value = suggestion;
  showSuggestions.value = false;
  performSearch();
}

function handleKeyDown(event: KeyboardEvent) {
  if (!showSuggestions.value || suggestions.value.length === 0) return;
  
  // Arrow down
  if (event.key === 'ArrowDown') {
    event.preventDefault();
    selectedIndex.value = (selectedIndex.value + 1) % suggestions.value.length;
  }
  // Arrow up
  else if (event.key === 'ArrowUp') {
    event.preventDefault();
    selectedIndex.value = selectedIndex.value <= 0 ? suggestions.value.length - 1 : selectedIndex.value - 1;
  }
  // Enter key
  else if (event.key === 'Enter' && selectedIndex.value >= 0) {
    event.preventDefault();
    selectSuggestion(suggestions.value[selectedIndex.value]);
  }
  // Escape key
  else if (event.key === 'Escape') {
    showSuggestions.value = false;
  }
}

// Function to handle input blur
function handleBlur() {
  setTimeout(() => {
    showSuggestions.value = false;
  }, 200);
}

async function performSearch() {
  if (!searchQuery.value.trim()) {
    return;
  }
  
  await invoke("search", { query: searchQuery.value.trim() });
  showSuggestions.value = false;
}
</script>

<template>
  <main class="min-h-screen flex flex-col items-center justify-center bg-gray-100 dark:bg-gray-900 p-4">
    <div class="w-full max-w-md">
      <form @submit.prevent="performSearch" class="w-full">
        <div class="relative">
          <input
            ref="searchInput"
            v-model="searchQuery"
            type="text"
            placeholder="Search..."
            class="w-full px-4 py-3 pr-10 rounded-lg border border-gray-300 dark:border-gray-700 focus:outline-none focus:ring-2 focus:ring-blue-500 dark:bg-gray-800 dark:text-white"
            @keydown="handleKeyDown"
            @blur="handleBlur"
          />
          <button
            type="submit"
            class="absolute right-2 top-1/2 transform -translate-y-1/2 text-gray-500 dark:text-gray-400 hover:text-blue-500 dark:hover:text-blue-400"
          >
            <svg xmlns="http://www.w3.org/2000/svg" class="h-6 w-6" fill="none" viewBox="0 0 24 24" stroke="currentColor">
              <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M21 21l-6-6m2-5a7 7 0 11-14 0 7 7 0 0114 0z" />
            </svg>
          </button>
          
          <!-- Loading indicator -->
          <div v-if="isLoading" class="absolute right-10 top-1/2 transform -translate-y-1/2">
            <svg class="animate-spin h-4 w-4 text-gray-500" xmlns="http://www.w3.org/2000/svg" fill="none" viewBox="0 0 24 24">
              <circle class="opacity-25" cx="12" cy="12" r="10" stroke="currentColor" stroke-width="4"></circle>
              <path class="opacity-75" fill="currentColor" d="M4 12a8 8 0 018-8V0C5.373 0 0 5.373 0 12h4zm2 5.291A7.962 7.962 0 014 12H0c0 3.042 1.135 5.824 3 7.938l3-2.647z"></path>
            </svg>
          </div>
          
          <!-- Suggestions dropdown -->
          <div 
            v-if="showSuggestions" 
            class="absolute z-10 w-full mt-1 bg-white dark:bg-gray-800 border border-gray-300 dark:border-gray-700 rounded-lg shadow-lg overflow-hidden"
          >
            <ul>
              <li 
                v-for="(suggestion, index) in suggestions" 
                :key="index"
                :class="[
                  'px-4 py-2 cursor-pointer hover:bg-gray-100 dark:hover:bg-gray-700',
                  { 'bg-gray-100 dark:bg-gray-700': index === selectedIndex }
                ]"
                @mousedown="selectSuggestion(suggestion)"
                @mouseover="selectedIndex = index"
              >
                <div class="flex items-center">
                  <svg class="h-4 w-4 mr-2 text-gray-500" xmlns="http://www.w3.org/2000/svg" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                    <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M21 21l-6-6m2-5a7 7 0 11-14 0 7 7 0 0114 0z" />
                  </svg>
                  <span>{{ suggestion }}</span>
                </div>
              </li>
            </ul>
          </div>
        </div>
      </form>
    </div>
  </main>
</template>
