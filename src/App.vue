<script setup lang="ts">
  import { ref, watch, onMounted, computed } from 'vue';
  import { WebviewWindow } from '@tauri-apps/api/webviewWindow';
  import { scaleWindow, setIntialPosition } from './utils/windowUtils';
  import { executeSearch, getAvailableBangs, getSearchSuggestions } from './utils/api';
  import { checkUpdate } from '@tauri-apps/api/updater';

  // Components
  import SearchInput from './components/SearchInput.vue';
  import SearchSuggestions from './components/SearchSuggestions.vue';
  import InfoPanel from './components/InfoPanel.vue';
  import SettingsPanel from './components/SettingsPanel.vue';

  // Type definitions
  type SearchSuggestionsInstance = InstanceType<typeof SearchSuggestions> & {
    getSelectedIndex: () => number;
    setSelectedIndex: (index: number) => void;
    clearSelection: () => void;
    getSelectedSuggestion: () => string | null;
  };

  // Constants
  const MAX_DISPLAYED_SUGGESTIONS = 8;

  // UI state
  const searchQuery = ref('');
  const searchSuggestions = ref<string[]>([]);
  const showSuggestions = ref(false);
  const showInfoPanel = ref(false);
  const showSettingsPanel = ref(false);
  const isProgrammaticUpdate = ref(false);

  // Data
  const bangs = ref<[string, string][]>([]);

  // Component refs
  const searchInputRef = ref<InstanceType<typeof SearchInput> | null>(null);
  const suggestionsRef = ref<SearchSuggestionsInstance | null>(null);

  // Window sizing logic
  const windowSizeState = computed(() => {
    if (showSuggestions.value && searchSuggestions.value.length > 0) {
      return { type: 'suggestions', count: searchSuggestions.value.length };
    } else if (showInfoPanel.value) {
      return { type: 'infoPanel', count: 9 };
    } else if (showSettingsPanel.value) {
      return { type: 'settingsPanel', count: 5 };
    } else {
      return { type: 'default', count: 0 };
    }
  });

  watch(
    windowSizeState,
    (state) => {
      scaleWindow(state.count);
    },
    { immediate: true }
  );

  // Lifecycle hooks
  onMounted(async () => {
    await setIntialPosition();
    bangs.value = await getAvailableBangs();
    
    // Check for updates
    checkForUpdates();
  });

  // Window event listeners
  WebviewWindow.getCurrent().listen('tauri://focus', () => {
    searchQuery.value = '';
    searchSuggestions.value = [];
    searchInputRef.value?.focus();
  });

  WebviewWindow.getCurrent().listen('tauri://blur', () => {
    searchQuery.value = '';
    searchSuggestions.value = [];
  });

  // Search query handling
  watch(searchQuery, async (query) => {
    if (isProgrammaticUpdate.value) {
      isProgrammaticUpdate.value = false;
      return;
    }

    if (query.trim().length > 0) {
      searchSuggestions.value = await getSearchSuggestions(query.trim(), MAX_DISPLAYED_SUGGESTIONS);
      showSuggestions.value = true;
    } else {
      searchSuggestions.value = [];
      showSuggestions.value = false;
    }
  });

  // Event handlers
  function handleKeyDown(event: KeyboardEvent) {
    if (event.key === 'Escape') {
      WebviewWindow.getCurrent().hide();
      event.preventDefault();
      return;
    }

    if (handleSuggestionNavigation(event)) {
      event.preventDefault();
      return;
    }

    if (event.key === 'Enter') {
      handleEnterKey();
      event.preventDefault();
    }
  }

  function handleSuggestionNavigation(event: KeyboardEvent): boolean {
    if (!showSuggestions.value || searchSuggestions.value.length === 0) {
      return false;
    }

    if (event.key !== 'Tab' && event.key !== 'ArrowDown' && event.key !== 'ArrowUp') {
      return false;
    }

    const currentIndex = suggestionsRef.value?.getSelectedIndex() ?? -1;
    let newIndex = currentIndex;

    if ((event.key === 'Tab' && !event.shiftKey) || event.key === 'ArrowDown') {
      newIndex = getNextIndex(currentIndex, searchSuggestions.value.length);
    } else if ((event.key === 'Tab' && event.shiftKey) || event.key === 'ArrowUp') {
      newIndex = getPreviousIndex(currentIndex, searchSuggestions.value.length);
    }

    if (newIndex !== currentIndex) {
      updateSelectedSuggestion(newIndex);
    }

    return true;
  }

  function getNextIndex(currentIndex: number, totalItems: number): number {
    return (currentIndex + 1) % totalItems;
  }

  function getPreviousIndex(currentIndex: number, totalItems: number): number {
    return (currentIndex - 1 + totalItems) % totalItems;
  }

  function updateSelectedSuggestion(index: number) {
    suggestionsRef.value?.setSelectedIndex(index);
    if (index >= 0 && index < searchSuggestions.value.length) {
      isProgrammaticUpdate.value = true;
      searchInputRef.value?.setValue(searchSuggestions.value[index]);
    }
  }

  function handleEnterKey() {
    const selectedIndex = suggestionsRef.value?.getSelectedIndex() ?? -1;
    if (
      showSuggestions.value &&
      selectedIndex >= 0 &&
      selectedIndex < searchSuggestions.value.length
    ) {
      handleSuggestionSelect(searchSuggestions.value[selectedIndex]);
    } else {
      performSearch();
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
    showSettingsPanel.value = false;
  }

  function toggleSettingsPanel() {
    showSettingsPanel.value = !showSettingsPanel.value;
    showSuggestions.value = false;
    showInfoPanel.value = false;
  }

  async function checkForUpdates() {
    try {
      const { shouldUpdate, manifest } = await checkUpdate();
      if (shouldUpdate) {
        console.log(`Update available: ${manifest?.version}`);
        // The dialog will be shown automatically since dialog: true in config
      }
    } catch (error) {
      console.error('Error checking for updates:', error);
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
        @toggleInfo="toggleInfoPanel"
        @toggleSettings="toggleSettingsPanel"
      />

      <SearchSuggestions
        ref="suggestionsRef"
        :suggestions="searchSuggestions"
        :show-suggestions="showSuggestions"
        @select="handleSuggestionSelect"
        @highlight="handleSuggestionHighlight"
      />

      <InfoPanel :show="showInfoPanel" :bangs-count="bangs.length" :available-bangs="bangs" />
      
      <SettingsPanel 
        :show="showSettingsPanel" 
        @close="showSettingsPanel = false" 
      />
    </div>
  </main>
</template>
