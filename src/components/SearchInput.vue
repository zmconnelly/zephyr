<script setup lang="ts">
  import { ref, computed } from 'vue';

  const props = defineProps<{
    modelValue: string;
  }>();

  const emit = defineEmits<{
    (e: 'update:modelValue', value: string): void;
    (e: 'search'): void;
    (e: 'keydown', event: KeyboardEvent): void;
    (e: 'blur'): void;
    (e: 'toggleInfo'): void;
    (e: 'toggleSettings'): void;
  }>();

  const searchInput = ref<HTMLInputElement | null>(null);
  const inputContainer = ref<HTMLDivElement | null>(null);

  // Format the input text with styled spans
  const formattedText = computed(() => {
    if (!props.modelValue) return '';
    
    // Replace spaces with non-breaking spaces to preserve them in HTML
    const text = props.modelValue.replace(/ /g, '&nbsp;');
    
    // Split by non-breaking spaces to process each word
    const words = text.split('&nbsp;');
    const result = words.map((word, index) => {
      // Skip empty words but preserve the space
      if (!word) return index < words.length - 1 ? '&nbsp;' : '';
      
      // Color words starting with ! in blue
      if (word.startsWith('!')) {
        return `<span class="text-blue-500">${word}</span>${index < words.length - 1 ? '&nbsp;' : ''}`;
      }
      
      return `<span>${word}</span>${index < words.length - 1 ? '&nbsp;' : ''}`;
    }).join('');
    
    return result;
  });

  function updateValue(event: Event) {
    const target = event.target as HTMLInputElement;
    emit('update:modelValue', target.value);
  }

  function handleKeyDown(event: KeyboardEvent) {
    emit('keydown', event);
  }

  function handleBlur() {
    setTimeout(() => {
      emit('blur');
    }, 150);
  }

  function toggleInfoPanel() {
    emit('toggleInfo');
  }

  function toggleSettingsPanel() {
    emit('toggleSettings');
  }

  function focus() {
    searchInput.value?.focus();
  }

  function setValue(value: string) {
    if (searchInput.value) {
      searchInput.value.value = value;
      emit('update:modelValue', value);
    }
  }

  defineExpose({
    focus,
    setValue,
  });
</script>

<template>
  <form @submit.prevent="$emit('search')" class="w-full">
    <div class="relative">
      <button
        type="submit"
        class="absolute left-2 top-1/2 transform -translate-y-1/2 text-gray-500 dark:text-gray-400 hover:text-blue-500 dark:hover:text-blue-400 z-10"
      >
        <svg
          class="h-6 w-6"
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
      </button>

      <div ref="inputContainer" class="relative">
        <input
          ref="searchInput"
          :value="modelValue"
          @input="updateValue"
          type="text"
          placeholder="Search or type a URL"
          class="w-full px-4 py-3 pl-10 pr-16 rounded-lg border border-gray-300 dark:border-gray-700 focus:outline-none focus:ring-2 focus:ring-blue-500 bg-white dark:bg-gray-800 text-transparent caret-gray-900 dark:caret-white shadow-lg"
          @keydown="handleKeyDown"
          @blur="handleBlur"
        />
        <div 
          class="absolute left-0 top-0 w-full h-full pointer-events-none px-4 py-3 pl-10 pr-16 text-gray-900 dark:text-white flex items-center whitespace-pre overflow-hidden"
          v-html="formattedText || (modelValue ? '' : 'Search or type a URL')"
        ></div>
      </div>

      <div class="absolute right-2 top-1/2 transform -translate-y-1/2 flex space-x-2 z-10">
        <button
          type="button"
          @click.prevent="toggleSettingsPanel"
          class="text-gray-500 dark:text-gray-400 hover:text-blue-500 dark:hover:text-blue-400 focus:outline-none"
          title="Settings"
        >
          <svg
            class="h-5 w-5"
            fill="none"
            viewBox="0 0 24 24"
            stroke="currentColor"
          >
            <path
              stroke-linecap="round"
              stroke-linejoin="round"
              stroke-width="2"
              d="M10.325 4.317c.426-1.756 2.924-1.756 3.35 0a1.724 1.724 0 002.573 1.066c1.543-.94 3.31.826 2.37 2.37a1.724 1.724 0 001.065 2.572c1.756.426 1.756 2.924 0 3.35a1.724 1.724 0 00-1.066 2.573c.94 1.543-.826 3.31-2.37 2.37a1.724 1.724 0 00-2.572 1.065c-.426 1.756-2.924 1.756-3.35 0a1.724 1.724 0 00-2.573-1.066c-1.543.94-3.31-.826-2.37-2.37a1.724 1.724 0 00-1.065-2.572c-1.756-.426-1.756-2.924 0-3.35a1.724 1.724 0 001.066-2.573c-.94-1.543.826-3.31 2.37-2.37.996.608 2.296.07 2.572-1.065z"
            />
            <path
              stroke-linecap="round"
              stroke-linejoin="round"
              stroke-width="2"
              d="M15 12a3 3 0 11-6 0 3 3 0 016 0z"
            />
          </svg>
        </button>
        <button
          type="button"
          @click.prevent="toggleInfoPanel"
          class="text-gray-500 dark:text-gray-400 hover:text-blue-500 dark:hover:text-blue-400 focus:outline-none"
          title="Learn more"
        >
          <svg
            class="h-5 w-5"
            fill="none"
            viewBox="0 0 24 24"
            stroke="currentColor"
          >
            <path
              stroke-linecap="round"
              stroke-linejoin="round"
              stroke-width="2"
              d="M13 16h-1v-4h-1m1-4h.01M21 12a9 9 0 11-18 0 9 9 0 0118 0z"
            />
          </svg>
        </button>
      </div>
    </div>
  </form>
</template>
