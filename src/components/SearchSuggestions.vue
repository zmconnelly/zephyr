<script setup lang="ts">
import { ref, defineProps, defineEmits, watch } from 'vue';
import { invoke } from "@tauri-apps/api/core";

const props = defineProps<{
  query: string;
  availableBangs: [string, string][];
  isProgrammaticChange?: boolean;
}>();

const emit = defineEmits<{
  (e: 'select', suggestion: string): void;
  (e: 'hover', index: number): void;
  (e: 'update:isProgrammaticChange', value: boolean): void;
}>();

const suggestions = ref<string[]>([]);
const showSuggestions = ref(false);
const selectedIndex = ref(-1);
// Flag to prevent key event double-triggering
const keyHandled = ref(false);

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
        matchingBangs = props.availableBangs
          .slice(0, 8)
          .map(([key, name]) => `${query}${key} (${name})`);
      } else {
        matchingBangs = props.availableBangs
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
    }
    
    showSuggestions.value = suggestions.value.length > 0;
  } catch (error) {
    console.error('Error generating suggestions:', error);
  }
}, 75);

// Watch for changes in the query prop
watch(() => props.query, (newQuery) => {
  // Only regenerate suggestions if the change wasn't programmatic
  if (!props.isProgrammaticChange) {
    selectedIndex.value = -1;
    generateSuggestions(newQuery);
  } else {
    // Reset the flag after handling the programmatic change
    emit('update:isProgrammaticChange', false);
  }
});

function selectSuggestion(suggestion: string) {
  emit('select', suggestion);
}

function handleMouseOver(index: number) {
  selectedIndex.value = index;
  emit('hover', index);
}

function handleKeyNavigation(event: KeyboardEvent) {
  if (showSuggestions.value && suggestions.value.length > 0) {
    // Tab key - cycle through suggestions
    if (event.key === 'Tab') {
      event.preventDefault();
      
      // Prevent double-triggering
      if (keyHandled.value) return false;
      keyHandled.value = true;
      
      // Move to next suggestion (or first if at end)
      if (event.shiftKey) {
        // Shift+Tab goes backward
        selectedIndex.value = selectedIndex.value <= 0 ? suggestions.value.length - 1 : selectedIndex.value - 1;
      } else {
        // Tab goes forward
        selectedIndex.value = (selectedIndex.value + 1) % suggestions.value.length;
      }
      
      // Reset key handled flag after a short delay
      setTimeout(() => {
        keyHandled.value = false;
      }, 20);
      
      return true;
    }
    
    // Arrow down
    if (event.key === 'ArrowDown') {
      event.preventDefault();
      
      // Prevent double-triggering
      if (keyHandled.value) return false;
      keyHandled.value = true;
      
      selectedIndex.value = (selectedIndex.value + 1) % suggestions.value.length;
      
      // Reset key handled flag after a short delay
      setTimeout(() => {
        keyHandled.value = false;
      }, 20);
      
      return true;
    }
    
    // Arrow up
    if (event.key === 'ArrowUp') {
      event.preventDefault();
      
      // Prevent double-triggering
      if (keyHandled.value) return false;
      keyHandled.value = true;
      
      selectedIndex.value = selectedIndex.value <= 0 ? suggestions.value.length - 1 : selectedIndex.value - 1;
      
      // Reset key handled flag after a short delay
      setTimeout(() => {
        keyHandled.value = false;
      }, 20);
      
      return true;
    }
    
    // Enter key
    if (event.key === 'Enter' && selectedIndex.value >= 0) {
      event.preventDefault();
      selectSuggestion(suggestions.value[selectedIndex.value]);
      return true;
    }
  }
  
  return false;
}

defineExpose({
  suggestions,
  showSuggestions,
  selectedIndex,
  handleKeyNavigation,
  getSelectedSuggestion: () => selectedIndex.value >= 0 ? suggestions.value[selectedIndex.value] : null
});
</script>

<template>
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
        @mouseover="handleMouseOver(index)"
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
</template>
