<script setup lang="ts">
import { ref, watch, onMounted, computed } from "vue";
import { invoke } from "@tauri-apps/api/core";
import { WebviewWindow } from "@tauri-apps/api/webviewWindow";
import { PhysicalSize, PhysicalPosition } from "@tauri-apps/api/window";
import SearchInput from "./components/SearchInput.vue";
import SearchSuggestions from "./components/SearchSuggestions.vue";
import BangInfoMessage from "./components/BangInfoMessage.vue";

const DEFAULT_WINDOW_HEIGHT = 150;
const DEFAULT_WINDOW_WIDTH = 900;

const searchQuery = ref("");
const searchInputRef = ref<InstanceType<typeof SearchInput> | null>(null);
const suggestionsRef = ref<InstanceType<typeof SearchSuggestions> | null>(null);
const availableBangs = ref<[string, string][]>([]);
  
const appWindow = WebviewWindow.getCurrent();
  
const isProgrammaticChange = ref(false);
const showBangInfo = ref(false);

const windowHeight = computed(() => {
  const baseHeight = DEFAULT_WINDOW_HEIGHT;
  
  const bangInfoHeight = showBangInfo.value ? 450 : 0;
  
  if (!suggestionsRef.value?.showSuggestions || suggestionsRef.value?.suggestions.length === 0) {
    return baseHeight + bangInfoHeight;
  }
  
  // Calculate height based on number of suggestions (each item is about 50px)
  // Limit to a maximum of 8 suggestions visible at once
  const suggestionsHeight = Math.min(suggestionsRef.value?.suggestions.length || 0, 8) * 50;

  return baseHeight + suggestionsHeight + bangInfoHeight;
});

// Watch for changes in window height and resize the window
watch(windowHeight, async (newHeight) => {
  try {
    const currentSize = await appWindow.size();
    await appWindow.setSize(new PhysicalSize(currentSize.width, newHeight));
  } catch (error) {
    console.error('Failed to resize window:', error);
  }
});

watch(searchQuery, (newVal) => {
  if (newVal.length > 0) {
    showBangInfo.value = false;
  }
});

onMounted(async () => {
  searchInputRef.value?.focus();
  
  try {
    await appWindow.setSize(new PhysicalSize(DEFAULT_WINDOW_WIDTH, windowHeight.value));
    
    // Position the window in the center, 2/3 up from the bottom of the screen
    const screenHeight = window.screen.height;
    const winHeight = DEFAULT_WINDOW_HEIGHT;
    const yPosition = Math.round(screenHeight * (1 - 2/3) - winHeight / 2);
    const position = await appWindow.innerPosition();

    console.log(`Setting position to ${position.x}, ${yPosition}`);
    await appWindow.setPosition(new PhysicalPosition(position.x, yPosition));
  } catch (error) {
    console.error('Failed to set initial window size or position:', error);
  }
  
  await loadBangs();
});

appWindow.listen("tauri://focus", () => {
  searchQuery.value = "";
  searchInputRef.value?.focus();
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

function toggleBangInfo() {
  showBangInfo.value = !showBangInfo.value;
}

function selectSuggestion(suggestion: string) {
  isProgrammaticChange.value = true;
  searchQuery.value = suggestion;
  performSearch();
}

function handleKeyDown(event: KeyboardEvent) {
  // Try to handle navigation in the suggestions component first
  if (suggestionsRef.value?.handleKeyNavigation(event)) {
    // If the suggestions component handled the key event, update the search query
    // if a suggestion is selected
    const selectedSuggestion = suggestionsRef.value?.getSelectedSuggestion();
    if (selectedSuggestion) {
      isProgrammaticChange.value = true;
      searchQuery.value = selectedSuggestion;
    }
    return;
  }
  
  if (event.key === 'Escape') {
    appWindow.hide();
  }
}

function handleBlur() {
  setTimeout(() => {
    if (suggestionsRef.value) {
      suggestionsRef.value.showSuggestions = false;
    }
  }, 50);
}

async function performSearch() {
  if (!searchQuery.value.trim()) {
    return;
  }
  
  await invoke("search", { query: searchQuery.value.trim() });
  if (suggestionsRef.value) {
    suggestionsRef.value.showSuggestions = false;
  }
}
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
        @toggleInfo="toggleBangInfo"
      />
      
      <!-- Bang info message -->
      <BangInfoMessage
        :show-bang-info="showBangInfo"
        :bangs-count="availableBangs.length"
        :available-bangs="availableBangs"
      />
      
      <!-- Suggestions dropdown -->
      <SearchSuggestions
        ref="suggestionsRef"
        :query="searchQuery"
        :available-bangs="availableBangs"
        :is-programmatic-change="isProgrammaticChange"
        @update:is-programmatic-change="isProgrammaticChange = $event"
        @select="selectSuggestion"
      />
    </div>
  </main>
</template>
