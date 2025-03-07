<script setup lang="ts">
import { ref, watch, onMounted, computed } from "vue";
import { WebviewWindow } from "@tauri-apps/api/webviewWindow";
import SearchInput from "./components/SearchInput.vue";
import SearchSuggestions from "./components/SearchSuggestions.vue";
import InfoPanel from "./components/InfoPanel.vue";
import { scaleWindow, setIntialPosition } from "./utils/windowUtils";
import { executeSearch, getAvailableBangs, getSearchSuggestions } from "./utils/api";

const MAX_DISPLAYED_SUGGESTIONS = 8;

const searchQuery = ref("");
const searchSuggestions = ref<string[]>([]);
const bangs = ref<[string, string][]>([]);

const showInfoPanel = ref(false);
const showSuggestions = ref(false);

const searchInputRef = ref<InstanceType<typeof SearchInput> | null>(null);
const suggestionsRef = ref<SearchSuggestionsInstance | null>(null);

const isProgrammaticUpdate = ref(false);

const windowSizeState = computed(() => {
  if (showSuggestions.value && searchSuggestions.value.length > 0) {
    return { type: 'suggestions', count: searchSuggestions.value.length };
  } else if (showInfoPanel.value) {
    return { type: 'infoPanel', count: 9 }; // Magic number for info panel height
  } else {
    return { type: 'default', count: 0 };
  }
});

watch(windowSizeState, (state) => {
  scaleWindow(state.count);
}, { immediate: true });

onMounted(async () => {
  await setIntialPosition();
  bangs.value = await getAvailableBangs();
});

// Reset state each time window is opened / closed
WebviewWindow.getCurrent().listen("tauri://focus", () => {
  console.info("Focused");
  searchQuery.value = "";
  searchSuggestions.value = [];
  searchInputRef.value?.focus();
});

WebviewWindow.getCurrent().listen("tauri://blur", () => {
  console.info("Hidden");
  searchQuery.value = "";
  searchSuggestions.value = [];
});

watch(searchQuery, async (query) => {
  // Only fetch suggestions if this is a user-initiated change
  if (!isProgrammaticUpdate.value) {
    if (query.trim().length > 0) {
      searchSuggestions.value = await getSearchSuggestions(query.trim(), MAX_DISPLAYED_SUGGESTIONS);
      showSuggestions.value = true;
    } else {
      searchSuggestions.value = [];
      showSuggestions.value = false;
    }
  } else {
    // Reset the flag after handling the programmatic update
    isProgrammaticUpdate.value = false;
  }
});

function handleKeyDown(event: KeyboardEvent) {
  if (event.key === 'Escape') {
    WebviewWindow.getCurrent().hide();
    event.preventDefault();
    return;
  }

  // Handle Tab/ArrowDown and Shift+Tab/ArrowUp for suggestion navigation
  if ((event.key === 'Tab' || event.key === 'ArrowDown' || event.key === 'ArrowUp') && 
      showSuggestions.value && searchSuggestions.value.length > 0) {
    
    const currentIndex = suggestionsRef.value?.getSelectedIndex() ?? -1;
    let newIndex = currentIndex;
    
    // Calculate the new index based on key pressed
    if ((event.key === 'Tab' && !event.shiftKey) || event.key === 'ArrowDown') {
      // Move down or wrap to top
      if (currentIndex < searchSuggestions.value.length - 1) {
        newIndex = currentIndex + 1;
      } else if (currentIndex === searchSuggestions.value.length - 1) {
        newIndex = 0; // Wrap to top
      } else if (currentIndex === -1) {
        newIndex = 0; // Select first item if nothing selected
      }
    } else if ((event.key === 'Tab' && event.shiftKey) || event.key === 'ArrowUp') {
      // Move up or wrap to bottom
      if (currentIndex > 0) {
        newIndex = currentIndex - 1;
      } else if (currentIndex === 0) {
        newIndex = searchSuggestions.value.length - 1; // Wrap to bottom
      } else if (currentIndex === -1) {
        newIndex = searchSuggestions.value.length - 1; // Select last item if nothing selected
      }
    }
    
    // Update selection and highlight
    if (newIndex !== currentIndex) {
      suggestionsRef.value?.setSelectedIndex(newIndex);
      if (newIndex >= 0 && newIndex < searchSuggestions.value.length) {
        isProgrammaticUpdate.value = true;
        searchInputRef.value?.setValue(searchSuggestions.value[newIndex]);
      }
    }
    
    event.preventDefault();
    return;
  }

  // Handle Enter to select current suggestion or perform search
  if (event.key === 'Enter') {
    const selectedIndex = suggestionsRef.value?.getSelectedIndex() ?? -1;
    if (showSuggestions.value && selectedIndex >= 0 && selectedIndex < searchSuggestions.value.length) {
      // Select the current suggestion
      handleSuggestionSelect(searchSuggestions.value[selectedIndex]);
    } else {
      // Perform search with current query
      performSearch();
    }
    event.preventDefault();
    return;
  }
}

function handleSuggestionSelect(suggestion: string) {
  isProgrammaticUpdate.value = true;
  searchQuery.value = suggestion;
  performSearch();
}

function handleSuggestionHighlight(suggestion: string) {
  isProgrammaticUpdate.value = true;
  searchInputRef.value?.setValue(suggestion);
}

function handleBlur() {
  setTimeout(() => {
    showSuggestions.value = false;
  }, 200);
}

async function performSearch() {
  if (!searchQuery.value.trim()) {
    return;
  }
  
  showSuggestions.value = false;
  await executeSearch(searchQuery.value.trim());
}

function toggleInfoPanel() {
  showInfoPanel.value = !showInfoPanel.value;
  showSuggestions.value = false;
}

// Define types for component instances with exposed methods
type SearchSuggestionsInstance = InstanceType<typeof SearchSuggestions> & {
  getSelectedIndex: () => number;
  setSelectedIndex: (index: number) => void;
  clearSelection: () => void;
  getSelectedSuggestion: () => string | null;
};
</script>

<template>
  <main class="flex flex-col items-center h-screen w-screen bg-transparent overflow-hidden p-4">
    <div class="w-full max-w-[700px] relative">
      <SearchInput
        ref="searchInputRef"
        v-model="searchQuery"
        @search="performSearch"
        @keydown="handleKeyDown"
        @blur="handleBlur"
        @toggleInfo="toggleInfoPanel"
      />
      
      <!-- Suggestions dropdown -->
      <SearchSuggestions
        ref="suggestionsRef"
        :suggestions="searchSuggestions"
        :show-suggestions="showSuggestions"
        @select="handleSuggestionSelect"
        @highlight="handleSuggestionHighlight"
      />

      <!-- Info panel -->
      <InfoPanel
        :show="showInfoPanel"
        :bangs-count="bangs.length"
        :available-bangs="bangs"
      />
    </div>
  </main>
</template>
