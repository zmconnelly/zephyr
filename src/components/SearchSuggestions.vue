<script setup lang="ts">
  import { ref, defineProps, defineEmits, watch } from 'vue';

  const props = defineProps<{
    suggestions: string[];
    showSuggestions: boolean;
  }>();

  const emit = defineEmits<{
    (e: 'select', suggestion: string): void;
    (e: 'highlight', suggestion: string): void;
  }>();

  const selectedIndex = ref(-1);

  // Reset selection when suggestions change
  watch(
    () => props.suggestions,
    () => {
      selectedIndex.value = -1;
    }
  );

  function selectSuggestion(suggestion: string) {
    emit('select', suggestion);
  }

  function emitHighlight() {
    if (selectedIndex.value >= 0 && selectedIndex.value < props.suggestions.length) {
      emit('highlight', props.suggestions[selectedIndex.value]);
    }
  }

  function handleMouseEnter(index: number) {
    selectedIndex.value = index;
  }

  function handleClick(suggestion: string) {
    selectSuggestion(suggestion);
  }

  // Methods for parent component to control selection
  function getSelectedIndex(): number {
    return selectedIndex.value;
  }

  function setSelectedIndex(index: number): void {
    selectedIndex.value = index;
    emitHighlight();
  }

  function clearSelection(): void {
    selectedIndex.value = -1;
  }

  // Explicitly expose methods for parent components
  defineExpose({
    getSelectedIndex,
    setSelectedIndex,
    clearSelection,
    getSelectedSuggestion: () =>
      selectedIndex.value >= 0 ? props.suggestions[selectedIndex.value] : null,
  });
</script>

<template>
  <div
    v-if="showSuggestions && suggestions.length > 0"
    class="absolute inset-x-0 mt-1 bg-white dark:bg-gray-800 border border-gray-300 dark:border-gray-700 rounded-lg shadow-lg overflow-hidden z-50"
  >
    <ul class="overflow-y-auto max-h-[400px]">
      <li
        v-for="(suggestion, index) in suggestions"
        :key="index"
        :class="[
          'px-4 py-2 cursor-pointer hover:bg-gray-100 dark:hover:bg-gray-700 transition-colors duration-150 text-gray-900 dark:text-white',
          {
            'bg-blue-100 dark:bg-blue-900 text-blue-800 dark:text-blue-200 font-medium border-l-4 border-blue-500 dark:border-blue-400':
              index === selectedIndex,
            'border-l-4 border-transparent': index !== selectedIndex,
          },
        ]"
        @mouseenter="handleMouseEnter(index)"
        @click="handleClick(suggestion)"
      >
        <div class="flex items-center">
          <svg
            class="h-4 w-4 mr-2 text-gray-500 flex-shrink-0"
            xmlns="http://www.w3.org/2000/svg"
            fill="none"
            viewBox="0 0 24 24"
            stroke="currentColor"
          >
            <path
              stroke-linecap="round"
              stroke-linejoin="round"
              stroke-width="2"
              d="M21 21l-6-6m2-5a7 7 0 11-14 0 7 7 0 0114 0z"
            />
          </svg>
          <span class="truncate">{{ suggestion }}</span>
        </div>
      </li>
    </ul>
  </div>
</template>
