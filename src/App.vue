<script setup lang="ts">
import { ref, watch, onMounted, onUnmounted, computed } from "vue";
import { invoke } from "@tauri-apps/api/core";
import { WebviewWindow } from "@tauri-apps/api/webviewWindow";
import { PhysicalSize, PhysicalPosition } from "@tauri-apps/api/window";

const searchQuery = ref("");
const searchInput = ref<HTMLInputElement | null>(null);
const suggestions = ref<string[]>([]);
const showSuggestions = ref(false);
const selectedIndex = ref(-1);
const appWindow = WebviewWindow.getCurrent();
// Flag to track if the search query was changed programmatically
const isProgrammaticChange = ref(false);
// Flag to prevent key event double-triggering
const keyHandled = ref(false);
// Store available bangs
const availableBangs = ref<[string, string][]>([]);
// Flag to track if bangs are being refreshed
const isRefreshingBangs = ref(false);
// Flag to show bang info
const showBangInfo = ref(false);

// Common search terms for offline suggestions (as fallback)
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

// Compute the window height based on suggestions
const windowHeight = computed(() => {
  // Base height for the search input and padding
  const baseHeight = 150;
  
  if (!showSuggestions.value || suggestions.value.length === 0) {
    return baseHeight; // Default height matching tauri.conf.json
  }
  
  // Calculate height based on number of suggestions (each item is about 50px)
  // Limit to a maximum of 8 suggestions visible at once
  const suggestionsHeight = Math.min(suggestions.value.length, 8) * 50;

  return baseHeight + suggestionsHeight;
});

// Watch for changes in window height and resize the window
watch(windowHeight, async (newHeight) => {
  try {
    // Use PhysicalSize for proper window resizing
    await appWindow.setSize(new PhysicalSize(750, newHeight));
  } catch (error) {
    console.error('Failed to resize window:', error);
  }
});

onMounted(async () => {
  // Focus the search input when the component is mounted
  searchInput.value?.focus();
  
  // Ensure the window has the correct initial size
  try {
    await appWindow.setSize(new PhysicalSize(900, windowHeight.value));
    
    // Position the window 2/3 up from the bottom of the screen
    const screenHeight = window.screen.height;
    const winHeight = 150; // Base height
    const yPosition = Math.round(screenHeight * (1 - 2/3) - winHeight / 2);
    
    // Set new position (centered horizontally, 2/3 up from bottom)
    await appWindow.center();
    const position = await appWindow.innerPosition();
    console.log(`Setting position to ${position.x}, ${yPosition}`);
    await appWindow.setPosition(new PhysicalPosition(position.x, yPosition));

  } catch (error) {
    console.error('Failed to set initial window size or position:', error);
  }
  
  // Load available bangs
  await loadBangs();
});

// Clean up event listener when component is unmounted
onUnmounted(() => {
  // No global event listeners to clean up
});

appWindow.listen("tauri://focus", () => {
  searchQuery.value = "";
  searchInput.value?.focus();
  suggestions.value = [];
  showSuggestions.value = false;
});

// Function to load bangs
async function loadBangs() {
  try {
    availableBangs.value = await invoke<[string, string][]>("get_available_bangs");
    console.log(`Loaded ${availableBangs.value.length} bangs`);
  } catch (error) {
    console.error('Error loading bangs:', error);
  }
}

// Function to refresh bangs from DuckDuckGo
async function refreshBangs() {
  if (isRefreshingBangs.value) return;
  
  isRefreshingBangs.value = true;
  try {
    await invoke("refresh_bangs");
    await loadBangs();
    // Show a temporary success message
    showBangInfo.value = true;
    setTimeout(() => {
      showBangInfo.value = false;
    }, 3000);
  } catch (error) {
    console.error('Error refreshing bangs:', error);
  } finally {
    isRefreshingBangs.value = false;
  }
}

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

// Function to fetch Google search suggestions
async function fetchGoogleSuggestions(query: string): Promise<string[]> {
  if (!query.trim()) return [];
  
  try {
    // Use our Rust backend function to fetch suggestions
    const suggestions = await invoke<string[]>("get_search_suggestions", { query });
    return suggestions;
  } catch (error) {
    console.error('Error fetching Google suggestions:', error);
    return [];
  }
}

// Generate suggestions based on input
const generateSuggestions = debounce(async (query: string) => {
  if (!query.trim()) {
    suggestions.value = [];
    showSuggestions.value = false;
    return;
  }

  try {
    // Check if the query ends with a bang pattern
    const bangMatch = query.match(/!(\w*)$/);
    if (bangMatch) {
      // If we have a partial bang, suggest matching bangs
      const partialBang = bangMatch[1].toLowerCase();
      
      // If the bang is empty (just "!"), show all bangs
      let matchingBangs;
      if (partialBang === "") {
        matchingBangs = availableBangs.value
          .slice(0, 8)
          .map(([key, name]) => `${query}${key} (${name})`);
      } else {
        matchingBangs = availableBangs.value
          .filter(([key, name]) => 
            key.toLowerCase().includes(partialBang) || 
            name.toLowerCase().includes(partialBang)
          )
          .map(([key, name]) => `${query.substring(0, bangMatch.index)}!${key} (${name})`);
      }
      
      if (matchingBangs.length > 0) {
        suggestions.value = matchingBangs.slice(0, 8);
        showSuggestions.value = true;
        return;
      }
    }
    
    // Try to fetch real suggestions from Google via our Rust backend
    const googleSuggestions = await fetchGoogleSuggestions(query);
    
    if (googleSuggestions.length > 0) {
      // If we got suggestions from Google, use those
      suggestions.value = googleSuggestions.slice(0, 8);
    } else {
      // Fallback to local filtering if Google API fails
      const matchingTerms = commonSearchTerms.filter(term => 
        term.toLowerCase().includes(query.toLowerCase())
      );
      
      // Generate variations of the query as fallback
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
      
      // Limit to 8 suggestions
      suggestions.value = allSuggestions.slice(0, 8);
    }
    
    showSuggestions.value = suggestions.value.length > 0;
  } catch (error) {
    console.error('Error generating suggestions:', error);
  }
}, 100);

// Watch for changes in the search query
watch(searchQuery, (newQuery) => {
  // Only regenerate suggestions if the change wasn't programmatic
  if (!isProgrammaticChange.value) {
    selectedIndex.value = -1;
    generateSuggestions(newQuery);
  } else {
    // Reset the flag after handling the programmatic change
    isProgrammaticChange.value = false;
  }
});

function selectSuggestion(suggestion: string) {
  isProgrammaticChange.value = true;
  searchQuery.value = suggestion;
  showSuggestions.value = false;
  performSearch();
}

function handleKeyDown(event: KeyboardEvent) {
  // Handle suggestions navigation
  if (showSuggestions.value && suggestions.value.length > 0) {
    // Tab key - cycle through suggestions
    if (event.key === 'Tab') {
      event.preventDefault();
      
      // Prevent double-triggering
      if (keyHandled.value) return;
      keyHandled.value = true;
      
      // Move to next suggestion (or first if at end)
      if (event.shiftKey) {
        // Shift+Tab goes backward
        selectedIndex.value = selectedIndex.value <= 0 ? suggestions.value.length - 1 : selectedIndex.value - 1;
      } else {
        // Tab goes forward
        selectedIndex.value = (selectedIndex.value + 1) % suggestions.value.length;
      }
      
      // Update search query with the selected suggestion
      isProgrammaticChange.value = true;
      searchQuery.value = suggestions.value[selectedIndex.value];
      
      // Reset key handled flag after a short delay
      setTimeout(() => {
        keyHandled.value = false;
      }, 20);
      
      return;
    }
    
    // Arrow down
    if (event.key === 'ArrowDown') {
      event.preventDefault();
      
      // Prevent double-triggering
      if (keyHandled.value) return;
      keyHandled.value = true;
      
      selectedIndex.value = (selectedIndex.value + 1) % suggestions.value.length;
      
      // Reset key handled flag after a short delay
      setTimeout(() => {
        keyHandled.value = false;
      }, 20);
      
      return;
    }
    
    // Arrow up
    if (event.key === 'ArrowUp') {
      event.preventDefault();
      
      // Prevent double-triggering
      if (keyHandled.value) return;
      keyHandled.value = true;
      
      selectedIndex.value = selectedIndex.value <= 0 ? suggestions.value.length - 1 : selectedIndex.value - 1;
      
      // Reset key handled flag after a short delay
      setTimeout(() => {
        keyHandled.value = false;
      }, 20);
      
      return;
    }
    
    // Enter key
    if (event.key === 'Enter' && selectedIndex.value >= 0) {
      event.preventDefault();
      selectSuggestion(suggestions.value[selectedIndex.value]);
    }
  }
  
  // Escape key - hide the window
  if (event.key === 'Escape') {
    appWindow.hide();
  }
  
  // Ctrl+R to refresh bangs
  if (event.key === 'r' && (event.ctrlKey || event.metaKey)) {
    event.preventDefault();
    refreshBangs();
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
  <main class="flex flex-col items-center h-screen w-screen bg-transparent overflow-hidden p-4">
    <div class="w-full max-w-[700px] relative">
      <form @submit.prevent="performSearch" class="w-full">
        <div class="relative">
          <button
            type="submit"
            class="absolute left-2 top-1/2 transform -translate-y-1/2 text-gray-500 dark:text-gray-400 hover:text-blue-500 dark:hover:text-blue-400"
          >
            <svg xmlns="http://www.w3.org/2000/svg" class="h-6 w-6" fill="none" viewBox="0 0 24 24" stroke="currentColor">
              <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M21 21l-6-6m2-5a7 7 0 11-14 0 7 7 0 0114 0z" />
            </svg>
          </button>
          <input
            ref="searchInput"
            v-model="searchQuery"
            type="text"
            placeholder="Search... (Type ! for bangs)"
            class="w-full px-4 py-3 pl-10 rounded-lg border border-gray-300 dark:border-gray-700 focus:outline-none focus:ring-2 focus:ring-blue-500 bg-white dark:bg-gray-800 text-gray-900 dark:text-white shadow-lg"
            @keydown="handleKeyDown"
            @blur="handleBlur"
          />
        </div>
      </form>
      
      <!-- Bang info message -->
      <div 
        v-if="showBangInfo" 
        class="mt-2 px-4 py-2 bg-green-100 dark:bg-green-900 text-green-800 dark:text-green-200 rounded-lg shadow-md transition-opacity duration-300"
      >
        <div class="flex items-center">
          <svg class="h-5 w-5 mr-2 text-green-500 flex-shrink-0" xmlns="http://www.w3.org/2000/svg" fill="none" viewBox="0 0 24 24" stroke="currentColor">
            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M5 13l4 4L19 7" />
          </svg>
          <span>Bangs refreshed! {{ availableBangs.length }} bangs available.</span>
        </div>
      </div>
      
      <!-- Suggestions dropdown (positioned relative to the search container) -->
      <div 
        v-if="showSuggestions" 
        class="absolute inset-x-0 mt-1 bg-white dark:bg-gray-800 border border-gray-300 dark:border-gray-700 rounded-lg shadow-lg overflow-hidden z-50"
        style="width: calc(100% - 0px);"
      >
        <ul class="overflow-y-auto max-h-[400px]">
          <li
            v-for="(suggestion, index) in suggestions" 
            :key="index"
            :class="[
              'px-4 py-2 cursor-pointer hover:bg-gray-100 dark:hover:bg-gray-700 transition-colors duration-150 text-gray-900 dark:text-white',
              { 
                'bg-blue-100 dark:bg-blue-900 text-blue-800 dark:text-blue-200 font-medium border-l-4 border-blue-500 dark:border-blue-400': index === selectedIndex,
                'border-l-4 border-transparent': index !== selectedIndex
              }
            ]"
            @mousedown="selectSuggestion(suggestion)"
            @mouseover="selectedIndex = index"
          >
            <div class="flex items-center">
              <svg class="h-4 w-4 mr-2 text-gray-500 flex-shrink-0" xmlns="http://www.w3.org/2000/svg" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M21 21l-6-6m2-5a7 7 0 11-14 0 7 7 0 0114 0z" />
              </svg>
              <span class="truncate">{{ suggestion }}</span>
            </div>
          </li>
        </ul>
      </div>
    </div>
  </main>
</template>
